package handler

import (
	"encoding/json"
	"os"
	"path/filepath"

	"github.com/gin-gonic/gin"
)

type AppHandler struct{}

func NewAppHandler() *AppHandler {
	return &AppHandler{}
}

func (h *AppHandler) Version(c *gin.Context) {
	path := filepath.Join("..", "web", "public", "version.json")
	data, err := os.ReadFile(path)
	if err != nil {
		c.JSON(200, gin.H{})
		return
	}

	var body any
	if json.Unmarshal(data, &body) != nil {
		c.JSON(200, gin.H{})
		return
	}
	c.JSON(200, body)
}
