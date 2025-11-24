package account

import (
	"github.com/gofiber/fiber/v2"
)

func RequireLoginMiddleware(c *fiber.Ctx) error {
	if IsLoggedIn(c) {
		return c.Next()
	} else {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
}
