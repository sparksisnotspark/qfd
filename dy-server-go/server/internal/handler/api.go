package handler

import (
	"context"
	"encoding/json"
	"strconv"
	"time"

	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
	"dy-server-go/server/internal/response"
	"dy-server-go/server/internal/service"
	"github.com/gin-gonic/gin"
	"gorm.io/gorm"
)

type APIHandler struct {
	db        *database.DB
	roomQueue *service.RoomQueueService
}

func NewAPIHandler(db *database.DB) *APIHandler {
	return &APIHandler{
		db:        db,
		roomQueue: service.NewRoomQueueService(db),
	}
}

func (h *APIHandler) GetRunningStatus(c *gin.Context) {
	var option model.DictOption
	if err := h.db.SQL.Where("name = ?", "caiji_status").First(&option).Error; err != nil {
		c.JSON(200, gin.H{"code": 0, "msg": "ok", "data": gin.H{}})
		return
	}

	var items []map[string]any
	_ = json.Unmarshal([]byte(option.Value), &items)
	result := gin.H{}
	for _, item := range items {
		name := stringsValue(item["name"])
		value := item["value"]
		if name == "" {
			continue
		}
		if name == "running" {
			result[name] = stringsValue(value) == "1"
		} else {
			result[name] = value
		}
	}
	c.JSON(200, gin.H{"code": 0, "msg": "ok", "data": result})
}

func (h *APIHandler) ServerRooms(c *gin.Context) {
	id := intValue(c, "id", 0)
	tp := stringValue(c, "tp")
	if tp == "" {
		tp = "hb"
	}
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)

	var list []model.RoomURL
	h.db.SQL.Where("id > ? AND key_id = ? AND tp = ?", id, keyInfo.ID, tp).
		Order("id asc").
		Limit(100).
		Find(&list)
	response.Success(c, list)
}

func (h *APIHandler) AddRoom(c *gin.Context) {
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)
	room := service.RoomAddPayload{
		RoomURL:   stringValue(c, "room_url"),
		KeyID:     keyInfo.ID,
		RoomID:    stringValue(c, "room_id"),
		WebRID:    stringValue(c, "web_rid"),
		Title:     stringValue(c, "title"),
		TP:        stringValue(c, "tp"),
		SendTime:  int64(intValue(c, "send_time", 0)),
		UsedTimes: intValue(c, "used_times", 0),
		Delay:     intValue(c, "delay", 0),
		ExpiredAt: int64(intValue(c, "expired_at", 0)),
	}
	if room.TP == "" {
		room.TP = stringValue(c, "type")
	}
	if room.SendTime == 0 {
		room.SendTime = time.Now().Unix()
	}
	if room.Delay <= 0 {
		room.Delay = 300
	}
	if room.ExpiredAt == 0 && room.Delay > 0 {
		room.ExpiredAt = room.SendTime + int64(room.Delay)
	}
	if room.RoomURL != "" || room.RoomID != "" || room.WebRID != "" {
		if err := h.roomQueue.Enqueue(c.Request.Context(), room); err != nil {
			response.Failure(c, 500, "加入房间队列失败", []any{})
			return
		}
	}
	response.Success(c, []any{})
}

func (h *APIHandler) CheckToken(c *gin.Context) {
	response.Success(c, []any{})
}

func (h *APIHandler) FetchCookie(c *gin.Context) {
	cjName := stringValue(c, "cj_name")

	var cookies []model.DyCookies
	h.db.SQL.Table("dy_cookies").
		Select("dy_cookies.*").
		Joins("left join dy_cj_ck on dy_cj_ck.ck_id = dy_cookies.id").
		Joins("left join dy_caiji on dy_caiji.id = dy_cj_ck.cj_id").
		Where("dy_cookies.can_use = 1 AND dy_cookies.in_used = 0 AND dy_caiji.cj_name = ?", cjName).
		Find(&cookies)
	response.Success(c, cookies)
}

func (h *APIHandler) CaijiStatus(c *gin.Context) {
	if c.Request.Method == "GET" {
		var option model.DictOption
		if err := h.db.SQL.Where("name = ?", "caiji_status").First(&option).Error; err != nil {
			response.Success(c, []any{})
			return
		}
		response.Success(c, option.Value)
		return
	}

	running := stringValue(c, "running")
	if running != "" {
		var option model.DictOption
		err := h.db.SQL.Where("name = ?", "caiji_status").First(&option).Error
		if err != nil && err != gorm.ErrRecordNotFound {
			response.Failure(c, 500, "保存失败", []any{})
			return
		}
		option.Name = "caiji_status"
		option.Value = `[{"name":"running","value":"` + running + `"}]`
		h.db.SQL.Save(&option)
	}
	response.Success(c, []any{})
}

func (h *APIHandler) UpdateCookie(c *gin.Context) {
	id := intValue(c, "cookie_id", 0)
	if id > 0 {
		var cookie model.DyCookies
		if err := h.db.SQL.First(&cookie, id).Error; err == nil {
			if value := stringValue(c, "can_used"); value != "" {
				v, _ := strconv.Atoi(value)
				cookie.CanUse = v
			}
			if value := stringValue(c, "in_used"); value != "" {
				v, _ := strconv.Atoi(value)
				cookie.InUsed = v
			}
			if value := stringValue(c, "used_info"); value != "" {
				cookie.UsedInfo = &value
			}
			if value := stringValue(c, "other"); value != "" {
				cookie.TodyUsed = &value
			}
			h.db.SQL.Save(&cookie)
		}
	}
	response.Success(c, []any{})
}

func (h *APIHandler) FetchRoom(c *gin.Context) {
	limit, _ := strconv.Atoi(c.DefaultQuery("limit", "1"))
	if limit <= 0 {
		limit = 1
	}
	tp := stringValue(c, "type")
	canUsedTimes := intValue(c, "can_used_times", 100)
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)

	query := h.db.SQL.Where("tp = ? AND key_id = ?", tp, keyInfo.ID).Where("used_times < ?", canUsedTimes)
	if tp != "hb" {
		query = query.Where("expired_at > ?", time.Now().Unix())
	}

	var list []model.RoomURL
	query.Order("used_times asc").Order("expired_at asc").Limit(limit).Find(&list)
	for _, item := range list {
		h.db.SQL.Model(&model.RoomURL{}).Where("id = ?", item.ID).Update("used_times", item.UsedTimes+1)
	}
	response.Success(c, list)
}

func (h *APIHandler) ConsumeRoom(c *gin.Context) {
	response.Success(c, requestMap(c))
}

func (h *APIHandler) FetchDailyLog(c *gin.Context) {
	date := stringValue(c, "date")
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)
	var list []model.DyDailyLog
	h.db.SQL.Where("key_id = ? AND created_at = ?", keyInfo.ID, date).Find(&list)
	if len(list) == 0 {
		response.Success(c, nil)
		return
	}
	response.Success(c, list)
}

func (h *APIHandler) FetchOneRoom(c *gin.Context) {
	tp := stringValue(c, "type")
	if tp == "" {
		tp = "hb"
	}
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)

	var item model.RoomURL
	if err := h.db.SQL.Where("key_id = ? AND tp = ?", keyInfo.ID, tp).Order("used_times asc").First(&item).Error; err != nil {
		response.Success(c, nil)
		return
	}
	response.Success(c, item)
}

func (h *APIHandler) UpdateDailyLog(c *gin.Context) {
	keyInfo := c.MustGet("keyInfo").(*model.KeysManage)
	nickname := stringValue(c, "nickname")
	dyNo := stringValue(c, "dy_no")
	tp := stringValue(c, "type")
	ssZS := intValue(c, "ss_zs", 0)
	createdAt := time.Now().Format("2006-01-02")

	var item model.DyDailyLog
	err := h.db.SQL.Where("key_id = ? AND dy_no = ? AND created_at = ?", keyInfo.ID, dyNo, createdAt).First(&item).Error
	if err != nil {
		item = model.DyDailyLog{
			Nickname:  nickname,
			KeyID:     ptrInt(int(keyInfo.ID)),
			DYNo:      dyNo,
			CreatedAt: createdAt,
			SSZS:      ssZS,
		}
		h.db.SQL.Create(&item)
	}

	if tp == "hb" {
		item.HBCS++
		if intValue(c, "hb_z_zs", 0) != 0 {
			item.HBZCS++
		}
		item.HBXHZS += intValue(c, "hb_xh_zs", 0)
		item.HBHDZS += intValue(c, "hb_hd_zs", 0)
	} else if tp == "fd" {
		item.FDCS++
		if intValue(c, "fd_z_cs", 0) != 0 {
			item.FDZCS++
		}
		if intValue(c, "fd_hd_zs", 0) != 0 {
			item.FDHDZS++
		}
		if intValue(c, "fd_xh_zs", 0) != 0 {
			item.FDXHZS++
		}
	}
	if ssZS != 0 {
		item.SSZS = ssZS
	}

	h.db.SQL.Save(&item)
	response.Success(c, nil)
}

func (h *APIHandler) RedisLoadKey(c *gin.Context) {
	var items []model.KeysManage
	h.db.SQL.Where("is_expired = 0").Find(&items)
	for _, item := range items {
		payload, _ := json.Marshal(item)
		h.db.Redis.Set(context.Background(), "keys:"+item.KeyCode, payload, 0)
	}
	response.Success(c, []any{})
}

func ptrUint(v uint) *uint {
	return &v
}

func ptrInt(v int) *int {
	return &v
}

func stringsValue(v any) string {
	switch value := v.(type) {
	case string:
		return value
	case float64:
		return strconv.FormatInt(int64(value), 10)
	default:
		return ""
	}
}
