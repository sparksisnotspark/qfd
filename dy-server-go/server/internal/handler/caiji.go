package handler

import (
	"bytes"
	"database/sql"
	"encoding/json"
	"errors"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"time"

	"dy-server-go/server/internal/config"
	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
	"dy-server-go/server/internal/response"
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

type CaijiHandler struct {
	cfg    config.Config
	db     *database.DB
	client *http.Client
}

func NewCaijiHandler(cfg config.Config, db *database.DB) *CaijiHandler {
	return &CaijiHandler{
		cfg:    cfg,
		db:     db,
		client: &http.Client{Timeout: 10 * time.Second},
	}
}

func (h *CaijiHandler) Register(r *gin.Engine) {
	r.GET("/caiji/login", h.loginPage)
	r.POST("/caiji/login", h.loginSubmit)
	r.GET("/caiji/logout", h.logout)
	r.GET("/caiji/caijiSettings", h.authPage, h.caijiSettings)

	api := r.Group("/caiji")
	api.Use(h.authAPI)
	{
		api.GET("/cookieList", h.cookieList)
		api.Any("/caijiGetCookie", h.caijiGetCookie)
		api.GET("/getCaijiList", h.getCaijiList)
		api.POST("/addCaiji", h.addCaiji)
		api.GET("/addCookie", h.addCookiePage)
		api.POST("/addCookie", h.addCookie)
		api.POST("/startCaiji", h.startCaiji)
		api.POST("/stopCaiji", h.stopCaiji)
		api.POST("/reloadOptions", h.reloadOptions)
		api.POST("/syncCaijiStatus", h.syncCaijiStatus)
		api.POST("/allocateCookie", h.allocateCookie)
		api.POST("/deleteCookie", h.deleteCookie)
		api.POST("/deleteCaiji", h.deleteCaiji)
		api.GET("/adminSummary", h.adminSummary)
		api.POST("/addKeysManage", h.addKeysManage)
		api.GET("/keysManageList", h.keysManageList)
		api.POST("/updateKeysManage", h.updateKeysManage)
		api.POST("/deleteKeysManage", h.deleteKeysManage)
		api.GET("/usersList", h.usersList)
		api.POST("/updateUser", h.updateUser)
		api.GET("/roomUrlList", h.roomURLList)
		api.GET("/keyStoreList", h.keyStoreList)
		api.POST("/updateKeyStore", h.updateKeyStore)
		api.POST("/deleteKeyStore", h.deleteKeyStore)
		api.POST("/addKeyStore", h.addKeyStore)
	}
}

func (h *CaijiHandler) isAuthed(c *gin.Context) bool {
	value, err := c.Cookie(h.cfg.CaijiAuthCookie)
	return err == nil && value == h.cfg.CaijiAuthToken
}

func (h *CaijiHandler) authPage(c *gin.Context) {
	if h.isAuthed(c) {
		c.Next()
		return
	}
	c.Redirect(http.StatusFound, "/caiji/login")
	c.Abort()
}

func (h *CaijiHandler) authAPI(c *gin.Context) {
	if h.isAuthed(c) {
		c.Next()
		return
	}
	response.Failure(c, 401, "未登录", []any{})
	c.Abort()
}

func (h *CaijiHandler) loginPage(c *gin.Context) {
	if h.isAuthed(c) {
		c.Redirect(http.StatusFound, "/caiji/caijiSettings")
		return
	}
	h.serveSPA(c)
}

func (h *CaijiHandler) caijiSettings(c *gin.Context) {
	h.serveSPA(c)
}

func (h *CaijiHandler) loginSubmit(c *gin.Context) {
	username := stringValue(c, "username")
	password := stringValue(c, "password")
	if username != h.cfg.CaijiAdminUser || password != h.cfg.CaijiAdminPassword {
		response.Failure(c, 401, "用户名或密码错误", []any{})
		return
	}
	c.SetCookie(h.cfg.CaijiAuthCookie, h.cfg.CaijiAuthToken, 86400*7, "/", "", false, true)
	response.Success(c, []any{})
}

func (h *CaijiHandler) logout(c *gin.Context) {
	http.SetCookie(c.Writer, &http.Cookie{Name: h.cfg.CaijiAuthCookie, Value: "", Path: "/", MaxAge: -1})
	c.Redirect(http.StatusFound, "/caiji/login")
}

func (h *CaijiHandler) cookieList(c *gin.Context) {
	query := h.db.SQL.Model(&model.DyCookies{}).Preload("Caiji", func(db *gorm.DB) *gorm.DB {
		return db.Select("dy_caiji.id", "dy_caiji.cj_name")
	})
	if _, hasPage := c.GetQuery("page"); hasPage {
		paginate(c, query, &[]model.DyCookies{})
		return
	}
	if _, hasPageSize := c.GetQuery("page_size"); hasPageSize {
		paginate(c, query, &[]model.DyCookies{})
		return
	}

	var list []model.DyCookies
	query.Find(&list)
	response.Success(c, list)
}

func (h *CaijiHandler) caijiGetCookie(c *gin.Context) {
	name := stringValue(c, "cj_name")
	if strings.TrimSpace(name) == "" {
		response.Success(c, nil)
		return
	}

	var item model.DyCaiji
	err := h.db.SQL.Preload("Cookies", func(db *gorm.DB) *gorm.DB {
		return db.Select("dy_cookies.id", "dy_cookies.dy_name")
	}).Where("cj_name = ?", name).First(&item).Error
	if err != nil {
		if err == gorm.ErrRecordNotFound {
			response.Success(c, nil)
			return
		}
		response.Failure(c, 500, "查询失败", []any{})
		return
	}

	response.Success(c, item)
}

func (h *CaijiHandler) getCaijiList(c *gin.Context) {
	var list []model.DyCaiji
	h.db.SQL.
		Select("id", "cj_name", "cj_count", "up_count", "cj_remark", "cj_status", "created_at", "updated_at").
		Preload("Cookies", func(db *gorm.DB) *gorm.DB {
			return db.Select("dy_cookies.id", "dy_cookies.dy_name")
		}).
		Find(&list)
	response.Success(c, gin.H{
		"cj_list":   list,
		"cj_online": []any{},
	})
}

func (h *CaijiHandler) addCaiji(c *gin.Context) {
	name := stringValue(c, "cj_name")
	if name != "" {
		if err := h.db.SQL.Create(&model.DyCaiji{CJName: name}).Error; err != nil {
			response.Failure(c, 500, "新增采集器失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) addCookiePage(c *gin.Context) {
	c.Header("Content-Type", "text/html; charset=utf-8")
	c.String(200, `<html><body><form method="post"><input name="dy_name"/><textarea name="cookie"></textarea><button>submit</button></form></body></html>`)
}

func (h *CaijiHandler) addCookie(c *gin.Context) {
	item := model.DyCookies{
		DYName: stringValue(c, "dy_name"),
		Cookie: stringValue(c, "cookie"),
	}
	if err := h.db.SQL.Create(&item).Error; err != nil {
		response.Failure(c, 500, "新增Cookie失败", []any{})
		return
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) startCaiji(c *gin.Context)      { response.Success(c, h.proxyPost("START_CJ", c)) }
func (h *CaijiHandler) stopCaiji(c *gin.Context)       { response.Success(c, h.proxyPost("STOP_CJ", c)) }
func (h *CaijiHandler) reloadOptions(c *gin.Context)   { response.Success(c, h.proxyPost("RELOAD_PARAMS", c)) }
func (h *CaijiHandler) syncCaijiStatus(c *gin.Context) { response.Success(c, h.proxyPost("SYNC_CAIJI_STATUS", c)) }

func (h *CaijiHandler) allocateCookie(c *gin.Context) {
	var payload struct {
		Caiji  []uint `json:"caiji"`
		Cookie []uint `json:"cookie"`
	}
	if c.ShouldBindJSON(&payload) != nil {
		payload.Caiji = normalizeIDs(anyValue(c, "caiji"))
		payload.Cookie = normalizeIDs(anyValue(c, "cookie"))
		if len(payload.Caiji) == 0 {
			payload.Caiji = uintSliceFromForm(c.PostFormArray("caiji[]"))
		}
		if len(payload.Cookie) == 0 {
			payload.Cookie = uintSliceFromForm(c.PostFormArray("cookie[]"))
		}
	}

	for _, caijiID := range payload.Caiji {
		if err := h.db.SQL.Where("cj_id = ?", caijiID).Delete(&model.DyCJCK{}).Error; err != nil {
			response.Failure(c, 500, "分配Cookie失败", []any{})
			return
		}
		for _, cookieID := range payload.Cookie {
			if err := h.db.SQL.Create(&model.DyCJCK{CJID: caijiID, CKID: cookieID}).Error; err != nil {
				response.Failure(c, 500, "分配Cookie失败", []any{})
				return
			}
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) deleteCookie(c *gin.Context) {
	ids := collectIDs(c)
	if len(ids) > 0 {
		if err := h.db.SQL.Where("ck_id IN ?", ids).Delete(&model.DyCJCK{}).Error; err != nil {
			response.Failure(c, 500, "删除Cookie失败", []any{})
			return
		}
		if err := h.db.SQL.Where("id IN ?", ids).Delete(&model.DyCookies{}).Error; err != nil {
			response.Failure(c, 500, "删除Cookie失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) deleteCaiji(c *gin.Context) {
	ids := collectIDs(c)
	if len(ids) > 0 {
		if err := h.db.SQL.Where("cj_id IN ?", ids).Delete(&model.DyCJCK{}).Error; err != nil {
			response.Failure(c, 500, "删除采集器失败", []any{})
			return
		}
		if err := h.db.SQL.Where("id IN ?", ids).Delete(&model.DyCaiji{}).Error; err != nil {
			response.Failure(c, 500, "删除采集器失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) adminSummary(c *gin.Context) {
	var keys []model.KeysManage
	var users []model.User
	var rooms []model.RoomURL
	var store []model.KeyStore
	h.db.SQL.Order("id desc").Find(&keys)
	h.db.SQL.Order("id desc").Find(&users)
	h.db.SQL.Order("id desc").Find(&rooms)
	h.db.SQL.Order("id desc").Find(&store)
	response.Success(c, gin.H{"keys_manage": keys, "users": users, "room_url": rooms, "key_store": store})
}

func (h *CaijiHandler) addKeysManage(c *gin.Context)    { h.saveKeysManage(c, false) }
func (h *CaijiHandler) updateKeysManage(c *gin.Context) { h.saveKeysManage(c, true) }

func (h *CaijiHandler) saveKeysManage(c *gin.Context, updating bool) {
	id := intValue(c, "id", 0)
	keyCode := strings.ToUpper(stringValue(c, "key_code"))
	expiredAt := stringValue(c, "expired_at")
	isExpired := intValue(c, "is_expired", 0)
	userID := nullableUint(stringValue(c, "user_id"))

	if updating && id == 0 {
		response.Failure(c, 500, "ID不能为空", []any{})
		return
	}
	if keyCode == "" {
		response.Failure(c, 500, "KEY CODE不能为空", []any{})
		return
	}
	if expiredAt == "" {
		response.Failure(c, 500, "过期时间不能为空", []any{})
		return
	}

	var duplicate int64
	query := h.db.SQL.Model(&model.KeysManage{}).Where("key_code = ?", keyCode)
	if updating {
		query = query.Where("id <> ?", id)
	}
	query.Count(&duplicate)
	if duplicate > 0 {
		response.Failure(c, 500, "KEY CODE已存在", []any{})
		return
	}

	t, _ := time.ParseInLocation("2006-01-02 15:04:05", expiredAt, time.Local)
	record := model.KeysManage{KeyCode: keyCode, UserID: userID, ExpiredAt: &t, IsExpired: isExpired}
	if updating {
		var existing model.KeysManage
		if err := h.db.SQL.First(&existing, id).Error; err != nil {
			response.Failure(c, 500, "记录不存在", []any{})
			return
		}
		existing.KeyCode = record.KeyCode
		existing.UserID = record.UserID
		existing.ExpiredAt = record.ExpiredAt
		existing.IsExpired = record.IsExpired
		if err := h.db.SQL.Save(&existing).Error; err != nil {
			response.Failure(c, 500, "更新KEY失败", []any{})
			return
		}
	} else {
		if err := h.db.SQL.Create(&record).Error; err != nil {
			response.Failure(c, 500, "新增KEY失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) keysManageList(c *gin.Context) { paginate(c, h.db.SQL.Model(&model.KeysManage{}), &[]model.KeysManage{}) }
func (h *CaijiHandler) usersList(c *gin.Context)      { paginate(c, h.db.SQL.Model(&model.User{}), &[]model.User{}) }
func (h *CaijiHandler) roomURLList(c *gin.Context)    { paginate(c, h.db.SQL.Model(&model.RoomURL{}), &[]model.RoomURL{}) }

func (h *CaijiHandler) keyStoreList(c *gin.Context) {
	page, pageSize, offset := pagination(c)
	var total int64
	var list []model.KeyStore
	if err := queryWithRetry(func() error {
		return h.db.SQL.Model(&model.KeyStore{}).Count(&total).Error
	}); err != nil {
		response.Failure(c, 500, "查询KEY库存失败", []any{})
		return
	}
	if err := queryWithRetry(func() error {
		return h.db.SQL.Preload("KeysManage").Order("id desc").Offset(offset).Limit(pageSize).Find(&list).Error
	}); err != nil {
		response.Failure(c, 500, "查询KEY库存失败", []any{})
		return
	}
	response.Success(c, gin.H{"list": list, "total": total, "page": page, "page_size": pageSize})
}

func (h *CaijiHandler) updateUser(c *gin.Context) {
	id := intValue(c, "id", 0)
	username := stringValue(c, "username")
	machineID := stringValue(c, "machine_id")
	expiredAt := stringValue(c, "expired_at")
	if id == 0 {
		response.Failure(c, 500, "ID不能为空", []any{})
		return
	}
	if username == "" {
		response.Failure(c, 500, "用户名不能为空", []any{})
		return
	}

	var user model.User
	if err := h.db.SQL.First(&user, id).Error; err != nil {
		response.Failure(c, 500, "用户不存在", []any{})
		return
	}
	var duplicate int64
	h.db.SQL.Model(&model.User{}).Where("username = ? AND id <> ?", username, id).Count(&duplicate)
	if duplicate > 0 {
		response.Failure(c, 500, "用户名已存在", []any{})
		return
	}
	user.Username = username
	if machineID == "" {
		user.MachineID = nil
	} else {
		user.MachineID = &machineID
	}
	if expiredAt == "" {
		user.ExpiredAt = nil
	} else if t, err := time.ParseInLocation("2006-01-02 15:04:05", expiredAt, time.Local); err == nil {
		user.ExpiredAt = &t
	}
	if err := h.db.SQL.Save(&user).Error; err != nil {
		response.Failure(c, 500, "更新用户失败", []any{})
		return
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) addKeyStore(c *gin.Context)    { h.saveKeyStore(c, false) }
func (h *CaijiHandler) updateKeyStore(c *gin.Context) { h.saveKeyStore(c, true) }

func (h *CaijiHandler) saveKeyStore(c *gin.Context, updating bool) {
	id := intValue(c, "id", 0)
	keyCode := strings.ToUpper(stringValue(c, "key_code"))
	expirationDate := intValue(c, "expiration_date", 0)
	unit := stringValue(c, "expiration_date_unit")
	isUsed := intValue(c, "is_used", 0)
	remark := stringValue(c, "remark")
	createdAt := stringValue(c, "created_at")
	kid := nullableUint(stringValue(c, "kid"))

	if updating && id == 0 {
		response.Failure(c, 500, "ID不能为空", []any{})
		return
	}
	if keyCode == "" {
		response.Failure(c, 500, "软件的CODE不能为空", []any{})
		return
	}
	if expirationDate <= 0 {
		response.Failure(c, 500, "有效期不能为空", []any{})
		return
	}
	if unit == "" {
		response.Failure(c, 500, "单位不能为空", []any{})
		return
	}
	if createdAt == "" {
		response.Failure(c, 500, "创建时间和更新时间不能为空", []any{})
		return
	}

	record := model.KeyStore{
		KID:                kid,
		KeyCode:            keyCode,
		ExpirationDate:     expirationDate,
		ExpirationDateUnit: unit,
		IsUsed:             isUsed,
		Remark:             remark,
	}
	ct, _ := time.ParseInLocation("2006-01-02 15:04:05", createdAt, time.Local)
	record.CreatedAt = &ct

	var duplicate int64
	duplicateQuery := h.db.SQL.Model(&model.KeyStore{}).Where("key_code = ?", keyCode)
	if updating {
		duplicateQuery = duplicateQuery.Where("id <> ?", id)
	}
	duplicateQuery.Count(&duplicate)
	if duplicate > 0 {
		response.Failure(c, 500, "软件的CODE已存在", []any{})
		return
	}

	if updating {
		var existing model.KeyStore
		if err := h.db.SQL.First(&existing, id).Error; err != nil {
			response.Failure(c, 500, "记录不存在", []any{})
			return
		}
		existing.KID = record.KID
		existing.KeyCode = record.KeyCode
		existing.ExpirationDate = record.ExpirationDate
		existing.ExpirationDateUnit = record.ExpirationDateUnit
		existing.IsUsed = record.IsUsed
		existing.Remark = record.Remark
		existing.CreatedAt = record.CreatedAt
		if err := h.db.SQL.Model(&model.KeyStore{}).Where("id = ?", id).Updates(map[string]any{
			"kid":                  existing.KID,
			"key_code":             existing.KeyCode,
			"expiration_date":      existing.ExpirationDate,
			"expiration_date_unit": existing.ExpirationDateUnit,
			"is_used":              existing.IsUsed,
			"remark":               existing.Remark,
			"created_at":           existing.CreatedAt,
		}).Error; err != nil {
			response.Failure(c, 500, "更新升级码失败", []any{})
			return
		}
	} else {
		if err := h.db.SQL.Omit("updated_at").Create(&record).Error; err != nil {
			response.Failure(c, 500, "新增升级码失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) deleteKeysManage(c *gin.Context) {
	ids := collectIDs(c)
	if len(ids) > 0 {
		if err := h.db.SQL.Where("id IN ?", ids).Delete(&model.KeysManage{}).Error; err != nil {
			response.Failure(c, 500, "删除KEY失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) deleteKeyStore(c *gin.Context) {
	ids := collectIDs(c)
	if len(ids) > 0 {
		if err := h.db.SQL.Where("id IN ?", ids).Delete(&model.KeyStore{}).Error; err != nil {
			response.Failure(c, 500, "删除升级码失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *CaijiHandler) proxyPost(action string, c *gin.Context) any {
	body := map[string]any{}
	_ = c.ShouldBindJSON(&body)
	if len(body) == 0 {
		_ = c.Request.ParseForm()
		for key, values := range c.Request.PostForm {
			if len(values) == 1 {
				body[key] = values[0]
			} else {
				body[key] = values
			}
		}
	}
	payload, _ := json.Marshal(body)
	req, _ := http.NewRequest(http.MethodPost, h.cfg.CaijiProxyURL+"?action="+action, bytes.NewReader(payload))
	req.Header.Set("Content-Type", "application/json; charset=utf-8")
	resp, err := h.client.Do(req)
	if err != nil {
		return gin.H{"code": -1, "msg": err.Error(), "data": nil}
	}
	defer resp.Body.Close()
	raw, _ := io.ReadAll(resp.Body)
	var data any
	_ = json.Unmarshal(raw, &data)
	return gin.H{"code": resp.StatusCode, "msg": map[bool]string{true: "请求成功", false: "请求失败"}[resp.StatusCode == 200], "data": data, "raw": string(raw)}
}

func (h *CaijiHandler) proxyGet(query string) map[string]any {
	url := h.cfg.CaijiProxyURL
	if query != "" {
		url += query
	}
	resp, err := h.client.Get(url)
	if err != nil {
		return map[string]any{"data": []any{}}
	}
	defer resp.Body.Close()
	raw, _ := io.ReadAll(resp.Body)
	var data map[string]any
	_ = json.Unmarshal(raw, &data)
	return data
}

func paginate[T any](c *gin.Context, query *gorm.DB, target *[]T) {
	page, pageSize, offset := pagination(c)
	var total int64
	if err := queryWithRetry(func() error {
		return query.Count(&total).Error
	}); err != nil {
		response.Failure(c, 500, "查询列表失败", []any{})
		return
	}
	if err := queryWithRetry(func() error {
		return query.Order("id desc").Offset(offset).Limit(pageSize).Find(target).Error
	}); err != nil {
		response.Failure(c, 500, "查询列表失败", []any{})
		return
	}
	response.Success(c, gin.H{"list": *target, "total": total, "page": page, "page_size": pageSize})
}

func queryWithRetry(run func() error) error {
	err := run()
	if !isRetryableDBError(err) {
		return err
	}
	time.Sleep(150 * time.Millisecond)
	return run()
}

func isRetryableDBError(err error) bool {
	if err == nil {
		return false
	}
	if errors.Is(err, sql.ErrConnDone) {
		return true
	}
	message := strings.ToLower(err.Error())
	return strings.Contains(message, "invalid connection") ||
		strings.Contains(message, "bad connection") ||
		strings.Contains(message, "broken pipe") ||
		strings.Contains(message, "connection reset")
}

func pagination(c *gin.Context) (int, int, int) {
	page := intValue(c, "page", 1)
	if page <= 0 {
		page = 1
	}
	pageSize := intValue(c, "page_size", 10)
	if pageSize <= 0 {
		pageSize = 10
	}
	if pageSize > 100 {
		pageSize = 100
	}
	return page, pageSize, (page - 1) * pageSize
}

func collectIDs(c *gin.Context) []uint {
	var payload struct {
		ID any `json:"id"`
	}
	_ = c.ShouldBindJSON(&payload)
	if payload.ID != nil {
		return normalizeIDs(payload.ID)
	}
	if id := c.PostForm("id"); id != "" {
		return normalizeIDs(id)
	}
	if id := stringValue(c, "id"); id != "" {
		return normalizeIDs(id)
	}
	return nil
}

func normalizeIDs(v any) []uint {
	switch value := v.(type) {
	case string:
		id, _ := strconv.Atoi(value)
		if id > 0 {
			return []uint{uint(id)}
		}
	case []any:
		var ids []uint
		for _, item := range value {
			ids = append(ids, normalizeIDs(item)...)
		}
		return ids
	case []string:
		return uintSliceFromForm(value)
	case float64:
		if value > 0 {
			return []uint{uint(value)}
		}
	}
	return nil
}

func uintSliceFromForm(values []string) []uint {
	result := make([]uint, 0, len(values))
	for _, value := range values {
		id, _ := strconv.Atoi(value)
		if id > 0 {
			result = append(result, uint(id))
		}
	}
	return result
}

func nullableUint(value string) *uint {
	value = strings.TrimSpace(value)
	if value == "" {
		return nil
	}
	id, _ := strconv.Atoi(value)
	if id <= 0 {
		return nil
	}
	result := uint(id)
	return &result
}

func (h *CaijiHandler) serveSPA(c *gin.Context) {
	indexPath := filepath.Join("..", "web", "dist", "index.html")
	if _, err := os.Stat(indexPath); err == nil {
		c.File(indexPath)
		return
	}
	c.Header("Content-Type", "text/html; charset=utf-8")
	c.String(200, `<!doctype html><html><body><div id="app"></div><p>前端资源尚未构建，请先执行 web 构建。</p></body></html>`)
}
