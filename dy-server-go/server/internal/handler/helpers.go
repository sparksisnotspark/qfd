package handler

import (
	"fmt"
	"net/http"
	"strconv"
	"strings"

	"github.com/gin-gonic/gin"
)

func requestMap(c *gin.Context) map[string]any {
	if cached, ok := c.Get("_request_map"); ok {
		if data, yes := cached.(map[string]any); yes {
			return data
		}
	}

	result := map[string]any{}
	contentType := c.GetHeader("Content-Type")
	if strings.Contains(contentType, "application/json") {
		var body map[string]any
		if err := c.ShouldBindJSON(&body); err == nil {
			for k, v := range body {
				result[k] = v
			}
		}
	} else if c.Request.Method == http.MethodPost || c.Request.Method == http.MethodPut || c.Request.Method == http.MethodPatch || c.Request.Method == http.MethodDelete {
		_ = c.Request.ParseForm()
		for key, values := range c.Request.Form {
			if len(values) == 1 {
				result[key] = values[0]
			} else if len(values) > 1 {
				result[key] = values
			}
		}
	}

	for key, values := range c.Request.URL.Query() {
		if len(values) == 1 {
			result[key] = values[0]
		} else if len(values) > 1 {
			result[key] = values
		}
	}

	c.Set("_request_map", result)
	return result
}

func stringValue(c *gin.Context, key string) string {
	data := requestMap(c)
	value, ok := data[key]
	if !ok || value == nil {
		return ""
	}
	switch v := value.(type) {
	case string:
		return strings.TrimSpace(v)
	case float64:
		return strconv.FormatInt(int64(v), 10)
	case int:
		return strconv.Itoa(v)
	case []string:
		if len(v) > 0 {
			return strings.TrimSpace(v[0])
		}
	default:
		return strings.TrimSpace(fmt.Sprint(v))
	}
	return ""
}

func intValue(c *gin.Context, key string, fallback int) int {
	value := stringValue(c, key)
	if value == "" {
		if data := requestMap(c); data[key] != nil {
			if f, ok := data[key].(float64); ok {
				return int(f)
			}
		}
		return fallback
	}
	if parsed, err := strconv.Atoi(value); err == nil {
		return parsed
	}
	return fallback
}

func anyValue(c *gin.Context, key string) any {
	return requestMap(c)[key]
}
