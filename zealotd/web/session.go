package web

import (
	"fmt"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/session"
)

var (
	sessionStore = session.New()
)

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
