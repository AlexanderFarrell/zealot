package analysis

import (
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app * fiber.App) fiber.Router {
	api := app.Group("/analysis")

	api.Get("/last", func (c * fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		entries, err := LastThirtyDays(accountID)
		return web.SendJSONOrError(c, entries, err, "getting last 30 days")
	})

	return api
}