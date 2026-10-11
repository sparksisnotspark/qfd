package middleware

import (
	"net/http"
	"strings"
	"time"

	"dy-server-go/server/internal/response"
	"dy-server-go/server/internal/service"
	"github.com/gin-gonic/gin"
)

func AppKeyAuth(keys *service.KeyService) gin.HandlerFunc {
	return func(c *gin.Context) {
		keyCode := strings.TrimSpace(c.GetHeader("app-key"))
		if keyCode == "" {
			response.Failure(c, http.StatusUnauthorized, "app-key不能为空", []any{})
			c.Abort()
			return
		}

		keyInfo, err := keys.GetKeyInfo(c.Request.Context(), keyCode)
		if err != nil || keyInfo == nil {
			response.Failure(c, http.StatusUnauthorized, "keycode不存在", []any{})
			c.Abort()
			return
		}
		if keyInfo.ExpiredAt != nil && keyInfo.ExpiredAt.Before(time.Now()) {
			response.Failure(c, http.StatusUnauthorized, "已经过期的数据码", []any{})
			c.Abort()
			return
		}

		c.Set("keyInfo", keyInfo)
		c.Next()
	}
}
