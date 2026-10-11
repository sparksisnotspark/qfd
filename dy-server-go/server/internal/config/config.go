package config

import "os"

type Config struct {
	AppEnv             string
	Host               string
	Port               string
	MySQLDSN           string
	RedisAddr          string
	RedisPassword      string
	RedisDB            int
	CaijiProxyURL      string
	CaijiAdminUser     string
	CaijiAdminPassword string
	CaijiAuthCookie    string
	CaijiAuthToken     string
}

func Load() Config {
	return Config{
		AppEnv:             getenv("APP_ENV", "development"),
		Host:               getenv("APP_HOST", "0.0.0.0"),
		Port:               getenv("APP_PORT", "8080"),
		MySQLDSN:           getenv("MYSQL_DSN", "root:root@tcp(127.0.0.1:3306)/dy_server?charset=utf8mb4&parseTime=True&loc=Local"),
		RedisAddr:          getenv("REDIS_ADDR", "127.0.0.1:6379"),
		RedisPassword:      os.Getenv("REDIS_PASSWORD"),
		RedisDB:            0,
		CaijiProxyURL:      getenv("CAIJI_PROXY_URL", "http://127.0.0.1:28888"),
		CaijiAdminUser:     getenv("CAIJI_ADMIN_USER", "312790696@qq.com"),
		CaijiAdminPassword: getenv("CAIJI_ADMIN_PASSWORD", "zhuxueyi"),
		CaijiAuthCookie:    getenv("CAIJI_AUTH_COOKIE", "caiji_auth"),
		CaijiAuthToken:     getenv("CAIJI_AUTH_TOKEN", "caiji_auth_ok"),
	}
}

func (c Config) HTTPAddr() string {
	return c.Host + ":" + c.Port
}

func getenv(key, fallback string) string {
	if value := os.Getenv(key); value != "" {
		return value
	}
	return fallback
}
