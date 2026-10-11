package router

import (
	"path/filepath"

	"dy-server-go/server/internal/config"
	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/handler"
	"dy-server-go/server/internal/middleware"
	"dy-server-go/server/internal/service"
	"github.com/gin-gonic/gin"
)

func New(cfg config.Config, db *database.DB) *gin.Engine {
	r := gin.Default()
	r.Use(corsMiddleware())
	r.Static("/assets", filepath.Join("..", "web", "dist", "assets"))
	r.GET("/", func(c *gin.Context) {
		c.Redirect(302, "/caiji/login")
	})

	keys := service.NewKeyService(db)
	appHandler := handler.NewAppHandler()
	authHandler := handler.NewAuthHandler(db, keys)
	apiHandler := handler.NewAPIHandler(db)
	caijiHandler := handler.NewCaijiHandler(cfg, db)

	r.GET("/app/version.json", appHandler.Version)
	r.GET("/api/getRunningStatus", apiHandler.GetRunningStatus)
	r.Any("/auth/tokenExpiredAt", authHandler.TokenExpiredAt)
	r.POST("/auth/login", authHandler.Login)
	r.POST("/auth/register", authHandler.Register)
	r.POST("/auth/unbound", authHandler.Unbound)
	r.POST("/auth/recharge", authHandler.Recharge)
	r.Any("/auth/logout", authHandler.Logout)

	appKey := middleware.AppKeyAuth(keys)
	api := r.Group("/api")
	api.Use(appKey)
	{
		api.GET("/serverRooms", apiHandler.ServerRooms)
		api.POST("/addRoom", apiHandler.AddRoom)
		api.Any("/checkToken", apiHandler.CheckToken)
		api.Any("/fetchCookie", apiHandler.FetchCookie)
		api.Any("/caijiStatus", apiHandler.CaijiStatus)
		api.Any("/updateCookie", apiHandler.UpdateCookie)
		api.Any("/fetchRoom", apiHandler.FetchRoom)
		api.Any("/fetchOneRoom", apiHandler.FetchOneRoom)
		api.Any("/consumeRoom", apiHandler.ConsumeRoom)
		api.Any("/fetchDailyLog", apiHandler.FetchDailyLog)
		api.Any("/updateDailyLog", apiHandler.UpdateDailyLog)
		api.Any("/redisLoadKey", apiHandler.RedisLoadKey)
	}

	caijiHandler.Register(r)
	return r
}

func corsMiddleware() gin.HandlerFunc {
	return func(c *gin.Context) {
		c.Header("Access-Control-Allow-Origin", "*")
		c.Header("Access-Control-Allow-Methods", "GET, POST, PUT, PATCH, DELETE, OPTIONS")
		c.Header("Access-Control-Allow-Headers", "Origin, Content-Type, Content-Length, Accept, Accept-Encoding, Authorization, X-Requested-With, app-key")
		c.Header("Access-Control-Expose-Headers", "Content-Length, Content-Type")
		c.Header("Access-Control-Max-Age", "86400")
		if c.Request.Method == "OPTIONS" {
			c.AbortWithStatus(204)
			return
		}
		c.Next()
	}
}
