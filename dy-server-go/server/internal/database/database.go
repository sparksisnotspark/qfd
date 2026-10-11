package database

import (
	"database/sql"
	"fmt"
	"time"

	"dy-server-go/server/internal/config"
	"github.com/redis/go-redis/v9"
	"gorm.io/driver/mysql"
	"gorm.io/gorm"
)

type DB struct {
	SQL   *gorm.DB
	Redis *redis.Client
}

func New(cfg config.Config) (*DB, error) {
	sqlDB, err := gorm.Open(mysql.Open(cfg.MySQLDSN), &gorm.Config{})
	if err != nil {
		return nil, fmt.Errorf("open mysql: %w", err)
	}

	rawDB, err := sqlDB.DB()
	if err != nil {
		return nil, fmt.Errorf("get raw mysql db: %w", err)
	}
	configureMySQLPool(rawDB)
	if err := rawDB.Ping(); err != nil {
		return nil, fmt.Errorf("ping mysql: %w", err)
	}

	rdb := redis.NewClient(&redis.Options{
		Addr:     cfg.RedisAddr,
		Password: cfg.RedisPassword,
		DB:       cfg.RedisDB,
	})

	return &DB{
		SQL:   sqlDB,
		Redis: rdb,
	}, nil
}

func configureMySQLPool(db *sql.DB) {
	db.SetMaxOpenConns(25)
	db.SetMaxIdleConns(10)
	db.SetConnMaxLifetime(5 * time.Minute)
	db.SetConnMaxIdleTime(2 * time.Minute)
}
