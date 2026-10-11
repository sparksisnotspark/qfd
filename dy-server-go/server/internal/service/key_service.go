package service

import (
	"context"
	"encoding/json"
	"time"

	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
)

type KeyService struct {
	db *database.DB
}

func NewKeyService(db *database.DB) *KeyService {
	return &KeyService{db: db}
}

func (s *KeyService) GetKeyInfo(ctx context.Context, keyCode string) (*model.KeysManage, error) {
	cacheKey := "keys:" + keyCode
	if s.db.Redis != nil {
		value, err := s.db.Redis.Get(ctx, cacheKey).Result()
		if err == nil && value != "" {
			var result model.KeysManage
			if json.Unmarshal([]byte(value), &result) == nil {
				return &result, nil
			}
		}
	}

	var result model.KeysManage
	if err := s.db.SQL.WithContext(ctx).Where("key_code = ?", keyCode).First(&result).Error; err != nil {
		return nil, err
	}

	if s.db.Redis != nil {
		if payload, err := json.Marshal(result); err == nil {
			s.db.Redis.Set(ctx, cacheKey, payload, 24*time.Hour)
		}
	}

	return &result, nil
}
