package main

import (
	"context"
	"log"
	"time"

	"dy-server-go/server/internal/config"
	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/router"
	"dy-server-go/server/internal/service"
)

func main() {
	cfg := config.Load()
	db, err := database.New(cfg)
	if err != nil {
		log.Fatalf("init db failed: %v", err)
	}

	ctx := context.Background()
	service.NewExpiryService(db).Start(ctx, 20*time.Second)
	service.NewRoomQueueService(db).Start(ctx)

	r := router.New(cfg, db)
	if err := r.Run(cfg.HTTPAddr()); err != nil {
		log.Fatalf("start server failed: %v", err)
	}
}
