package web

import (
	"fmt"
	"os"
	"strconv"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/session"
	"github.com/gofiber/storage/redis"
)

var (
	sessionStore *session.Store
)

func InitSessions() {
	cfg := session.Config{
		CookieHTTPOnly: GetEnvVar("SESSION_COOKIE_HTTPONLY", "true") == "true",
		CookieSecure:   GetEnvVar("SESSION_COOKIE_SECURE", "false") == "true",
		CookieSameSite: GetEnvVar("SESSION_COOKIE_SAMESITE", "Lax"),
	}
	if os.Getenv("SESSION_STORE") == "redis" {
		port, err := strconv.Atoi(GetEnvVar("REDIS_PORT", "6379"))
		if err != nil {
			panic("Redis port is not a number, please set to a number")
		}
		database, err := strconv.Atoi(GetEnvVar("REDIS_DATABASE", "0"))
		if err != nil {
			panic("Redis database must be a number")
		}
		redisStore := redis.New(redis.Config{
			Host:     GetEnvVar("REDIS_HOST", "127.0.0.1"),
			Port:     port,
			Password: GetEnvVar("REDIS_PASSWORD", ""),
			Database: database,
			Reset:    GetEnvVar("REDIS_RESET", "false") == "true",
		})
		cfg.Storage = redisStore
	}
	sessionStore = session.New(cfg)
}

func GetSessionStore(c *fiber.Ctx) *session.Session {
	sess, err := sessionStore.Get(c)
	if err != nil {
		panic(err)
	}
	return sess
}

func GetKeyFromSession(c *fiber.Ctx, key string) string {
	sess := GetSessionStore(c)
	return fmt.Sprintf("%v", sess.Get(key))
}

func GetKeyFromSessionInt(c *fiber.Ctx, key string) int {
	sess := GetSessionStore(c)
	v := sess.Get(key)

	switch val := v.(type) {
	case int:
		return val
	case int64:
		return int(val)
	case float64:
		return int(val)
	case string:
	default:
		panic(fmt.Errorf("cannot convert key %q (type %T) to int", key, v))
	}
	return 0
}

func SaveSession(s *session.Session) {
	if err := s.Save(); err != nil {
		panic(err)
	}
}

func DestroySession(s *session.Session) {
	if err := s.Destroy(); err != nil {
		panic(err)
	}
}
