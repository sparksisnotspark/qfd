package service

import (
	"context"
	"encoding/json"
	"log"
	"time"

	"dy-server-go/server/internal/database"
	"dy-server-go/server/internal/model"
)

const roomAddQueueKey = "room-add"

type RoomAddPayload struct {
	RoomURL   string `json:"room_url"`
	KeyID     uint   `json:"key_id"`
	RoomID    string `json:"room_id"`
	WebRID    string `json:"web_rid"`
	Title     string `json:"title"`
	TP        string `json:"tp"`
	SendTime  int64  `json:"send_time"`
	UsedTimes int    `json:"used_times"`
	Delay     int    `json:"delay"`
	ExpiredAt int64  `json:"expired_at"`
}

type RoomQueueService struct {
	db *database.DB
}

func NewRoomQueueService(db *database.DB) *RoomQueueService {
	return &RoomQueueService{db: db}
}

func (s *RoomQueueService) Enqueue(ctx context.Context, payload RoomAddPayload) error {
	body, err := json.Marshal(payload)
	if err != nil {
		return err
	}
	return s.db.Redis.LPush(ctx, roomAddQueueKey, body).Err()
}

func (s *RoomQueueService) Start(ctx context.Context) {
	go func() {
		for {
			select {
			case <-ctx.Done():
				return
			default:
			}

			if err := s.consumeBatch(ctx); err != nil {
				log.Printf("room queue consume failed: %v", err)
				time.Sleep(1 * time.Second)
			}
		}
	}()
}

func (s *RoomQueueService) consumeBatch(ctx context.Context) error {
	entries, err := s.db.Redis.BRPop(ctx, 2*time.Second, roomAddQueueKey).Result()
	if err != nil {
		if err == context.DeadlineExceeded || err == context.Canceled {
			return nil
		}
		if err.Error() == "redis: nil" {
			return nil
		}
		return err
	}
	if len(entries) < 2 {
		return nil
	}

	payloads := make([]RoomAddPayload, 0, 64)
	first, ok := s.decodePayload(entries[1])
	if ok {
		payloads = append(payloads, first)
	}

	for len(payloads) < 64 {
		raw, err := s.db.Redis.RPop(ctx, roomAddQueueKey).Result()
		if err != nil {
			if err.Error() == "redis: nil" {
				break
			}
			return err
		}
		payload, ok := s.decodePayload(raw)
		if ok {
			payloads = append(payloads, payload)
		}
	}

	if len(payloads) == 0 {
		return nil
	}
	return s.flush(payloads)
}

func (s *RoomQueueService) decodePayload(raw string) (RoomAddPayload, bool) {
	var payload RoomAddPayload
	if err := json.Unmarshal([]byte(raw), &payload); err != nil {
		log.Printf("room queue decode failed: %v", err)
		return RoomAddPayload{}, false
	}
	if payload.RoomID == "" && payload.RoomURL == "" && payload.WebRID == "" {
		return RoomAddPayload{}, false
	}
	if payload.Delay <= 0 {
		payload.Delay = 300
	}
	if payload.SendTime == 0 {
		payload.SendTime = time.Now().Unix()
	}
	if payload.ExpiredAt == 0 {
		payload.ExpiredAt = payload.SendTime + int64(payload.Delay)
	}
	return payload, true
}

func (s *RoomQueueService) flush(payloads []RoomAddPayload) error {
	roomIDs := make([]string, 0, len(payloads))
	for _, payload := range payloads {
		if payload.RoomID != "" {
			roomIDs = append(roomIDs, payload.RoomID)
		}
	}

	exists := map[string]struct{}{}
	if len(roomIDs) > 0 {
		var existing []model.RoomURL
		if err := s.db.SQL.Select("room_id").Where("room_id IN ?", roomIDs).Find(&existing).Error; err != nil {
			return err
		}
		for _, item := range existing {
			exists[item.RoomID] = struct{}{}
		}
	}

	seen := map[string]struct{}{}
	rooms := make([]model.RoomURL, 0, len(payloads))
	for _, payload := range payloads {
		if payload.RoomID != "" {
			if _, ok := exists[payload.RoomID]; ok {
				continue
			}
			if _, ok := seen[payload.RoomID]; ok {
				continue
			}
			seen[payload.RoomID] = struct{}{}
		}

		keyID := payload.KeyID
		room := model.RoomURL{
			RoomURL:   payload.RoomURL,
			KeyID:     &keyID,
			RoomID:    payload.RoomID,
			WebRID:    payload.WebRID,
			Title:     payload.Title,
			TP:        payload.TP,
			SendTime:  payload.SendTime,
			UsedTimes: 0,
			Delay:     payload.Delay,
			ExpiredAt: payload.ExpiredAt,
		}
		rooms = append(rooms, room)
	}

	if len(rooms) == 0 {
		return nil
	}
	return s.db.SQL.CreateInBatches(rooms, 100).Error
}
