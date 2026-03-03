package account

import (
	"fmt"
	"strings"

	"github.com/gofiber/fiber/v2"
)

func RequireLoginMiddleware(c *fiber.Ctx) error {
	if IsLoggedIn(c) {
		return c.Next()
	}

	auth := c.Get("Authorization")
	if strings.HasPrefix(auth, "Bearer ") {
		key := strings.TrimPrefix(auth, "Bearer ")
		accountID, username, err := GetAccountFromAPIKey(key)
		if err != nil {
			fmt.Printf("Error validating API key: %v\n", err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}
		if accountID != 0 {
			c.Locals("account_id", accountID)
			c.Locals("username", username)
			return c.Next()
		}
	}

	return c.SendStatus(fiber.StatusUnauthorized)
}
