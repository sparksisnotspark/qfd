package handler

import (
	"net/http"
	"strings"
	"time"

	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
	"dy-server-go/server/internal/response"
	"dy-server-go/server/internal/service"
	"github.com/gin-gonic/gin"
	"golang.org/x/crypto/bcrypt"
)

type AuthHandler struct {
	db   *database.DB
	keys *service.KeyService
}

func NewAuthHandler(db *database.DB, keys *service.KeyService) *AuthHandler {
	return &AuthHandler{db: db, keys: keys}
}

func (h *AuthHandler) TokenExpiredAt(c *gin.Context) {
	keyCode := strings.TrimSpace(c.GetHeader("app-key"))
	if keyCode == "" {
		keyCode = stringValue(c, "keycode")
	}
	if keyCode == "" {
		response.Failure(c, 400, "keycode不能为空", []any{})
		return
	}

	keyInfo, err := h.keys.GetKeyInfo(c.Request.Context(), keyCode)
	if err != nil || keyInfo == nil {
		response.Failure(c, 404, "keycode不存在", []any{})
		return
	}

	expiredAt := ""
	isExpired := 1
	remainingSeconds := int64(0)
	if keyInfo.ExpiredAt != nil {
		expiredAt = keyInfo.ExpiredAt.Format("2006-01-02 15:04:05")
		if keyInfo.ExpiredAt.After(time.Now()) {
			isExpired = 0
			remainingSeconds = int64(time.Until(*keyInfo.ExpiredAt).Seconds())
		}
	}

	response.Success(c, gin.H{
		"key_code":          keyCode,
		"expired_at":        expiredAt,
		"is_expired":        isExpired,
		"remaining_seconds": max(0, remainingSeconds),
	})
}

func (h *AuthHandler) Login(c *gin.Context) {
	username := stringValue(c, "username")
	password := stringValue(c, "password")
	keycode := stringValue(c, "keycode")
	macAddress := stringValue(c, "mac_address")
	if len(username) < 6 || len(password) < 6 || len(keycode) < 6 || len(macAddress) < 6 {
		response.Failure(c, http.StatusBadRequest, "参数不合法", []any{})
		return
	}

	var user model.User
	if err := h.db.SQL.Where("username = ?", username).First(&user).Error; err != nil {
		response.Failure(c, 500, "用户名或密码不正确", []any{})
		return
	}
	if bcrypt.CompareHashAndPassword([]byte(user.Password), []byte(password)) != nil {
		response.Failure(c, 500, "用户名或密码不正确", []any{})
		return
	}
	if user.MachineID != nil && *user.MachineID != "" && *user.MachineID != macAddress {
		response.Failure(c, 500, "请重新绑定用户", []any{})
		return
	}
	if user.MachineID == nil || *user.MachineID == "" {
		user.MachineID = &macAddress
		h.db.SQL.Save(&user)
	}

	var keyInfo model.KeysManage
	if err := h.db.SQL.Where("key_code = ?", keycode).First(&keyInfo).Error; err != nil {
		response.Failure(c, 500, "后台数据码不正确，请重新输入", []any{})
		return
	}
	if keyInfo.ExpiredAt != nil && keyInfo.ExpiredAt.Before(time.Now()) {
		response.Failure(c, 500, "已经过期的数据码", []any{})
		return
	}
	if keyInfo.UserID != nil && *keyInfo.UserID != user.ID {
		response.Failure(c, 500, "请先解绑再登陆！", []any{})
		return
	}
	if keyInfo.UserID == nil {
		keyInfo.UserID = &user.ID
		h.db.SQL.Save(&keyInfo)
	}

	expiredAt := ""
	if keyInfo.ExpiredAt != nil {
		expiredAt = keyInfo.ExpiredAt.Format("2006-01-02 15:04:05")
	}
	response.SuccessWithMsg(c, "登陆成功！", gin.H{"expired_at": expiredAt})
}

func (h *AuthHandler) Register(c *gin.Context) {
	username := stringValue(c, "username")
	password := stringValue(c, "password")
	macAddress := stringValue(c, "mac_address")
	if len(username) < 8 || len(password) < 8 || len(macAddress) < 8 {
		response.Failure(c, 500, "参数不合法", []any{})
		return
	}

	var count int64
	h.db.SQL.Model(&model.User{}).Where("username = ?", username).Count(&count)
	if count > 0 {
		response.Failure(c, 500, "用户名已经注册", []any{})
		return
	}

	hash, _ := bcrypt.GenerateFromPassword([]byte(password), bcrypt.DefaultCost)
	user := model.User{Username: username, Password: string(hash), MachineID: &macAddress}
	h.db.SQL.Create(&user)
	response.SuccessWithMsg(c, "注册成功!", []any{})
}

func (h *AuthHandler) Unbound(c *gin.Context) {
	username := stringValue(c, "username")
	if len(username) < 8 {
		response.Failure(c, 500, "参数不合法", []any{})
		return
	}

	var user model.User
	if err := h.db.SQL.Where("username = ?", username).First(&user).Error; err == nil {
		user.MachineID = nil
		h.db.SQL.Save(&user)
	}
	response.SuccessWithMsg(c, "解绑成功!", []any{})
}

func (h *AuthHandler) Logout(c *gin.Context) {
	response.Success(c, nil)
}

func (h *AuthHandler) Recharge(c *gin.Context) {
	username := stringValue(c, "username")
	keycode := stringValue(c, "keycode")
	if len(username) < 8 || len(keycode) < 8 {
		response.Failure(c, 500, "参数不合法", []any{})
		return
	}

	var user model.User
	if err := h.db.SQL.Where("username = ?", username).First(&user).Error; err != nil {
		response.Failure(c, 500, "用户不存在!", []any{})
		return
	}

	var key model.KeyStore
	if err := h.db.SQL.Where("key_code = ?", keycode).First(&key).Error; err != nil {
		response.Failure(c, 500, "不存在的KEY", []any{})
		return
	}
	if key.IsUsed != 0 {
		response.Failure(c, 500, "已经使用过的KEY， 不能再次使用", []any{})
		return
	}

	var keyManage model.KeysManage
	if err := h.db.SQL.Where("user_id = ? AND is_expired = 0", user.ID).First(&keyManage).Error; err != nil {
		response.Failure(c, 500, "用户不存在可续费的数据码", []any{})
		return
	}

	base := time.Now()
	if keyManage.ExpiredAt != nil {
		base = *keyManage.ExpiredAt
	}
	switch key.ExpirationDateUnit {
	case "m":
		expired := base.AddDate(0, key.ExpirationDate, 0)
		keyManage.ExpiredAt = &expired
	case "d":
		expired := base.AddDate(0, 0, key.ExpirationDate)
		keyManage.ExpiredAt = &expired
	case "y":
		expired := base.AddDate(key.ExpirationDate, 0, 0)
		keyManage.ExpiredAt = &expired
	}

	h.db.SQL.Model(&model.KeyStore{}).Where("key_code = ?", key.KeyCode).UpdateColumn("is_used", 1)
	h.db.SQL.Model(&model.KeysManage{}).Where("id = ?", keyManage.ID).Update("expired_at", keyManage.ExpiredAt)
	response.SuccessWithMsg(c, "卡密更新成功", []any{})
}

func max(a, b int64) int64 {
	if a > b {
		return a
	}
	return b
}
