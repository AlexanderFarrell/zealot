package analysis

import (
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app * fiber.App) fiber.Router {
	api := app.Group("/analysis")

	api.Get("/last", func (c * fiber.Ctx) error {
		accountID, authErr := web.GetAccountID(c)
		if authErr != nil {
			return c.SendStatus(fiber.StatusUnauthorized)
		}
		entries, err := LastThirtyDays(accountID)
		return web.SendJSONOrError(c, entries, err, "getting last 30 days")
	})

	return api
}