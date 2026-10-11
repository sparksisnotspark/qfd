package service

import (
	"context"
	"log"
	"time"

	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
)

type ExpiryService struct {
	db *database.DB
}

func NewExpiryService(db *database.DB) *ExpiryService {
	return &ExpiryService{db: db}
}

func (s *ExpiryService) Start(ctx context.Context, interval time.Duration) {
	ticker := time.NewTicker(interval)
	go func() {
		defer ticker.Stop()
		s.runOnce()
		for {
			select {
			case <-ctx.Done():
				return
			case <-ticker.C:
				s.runOnce()
			}
		}
	}()
}

func (s *ExpiryService) runOnce() {
	now := time.Now()
	if err := s.db.SQL.Model(&model.KeysManage{}).
		Where("is_expired = 0").
		Where("expired_at IS NOT NULL").
		Where("expired_at <= ?", now).
		Update("is_expired", 1).Error; err != nil {
		log.Printf("expiry service update keys failed: %v", err)
	}

	if err := s.db.SQL.Where("expired_at > 0").
		Where("expired_at <= ?", now.Unix()).
		Delete(&model.RoomURL{}).Error; err != nil {
		log.Printf("expiry service delete room_url failed: %v", err)
	}
}
