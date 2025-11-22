package item

import (
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/item", getRootItems)

	router.Get("/", getRootItems)

	return router
}

func getRootItems(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusNotImplemented)
}

func addItem(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	// Payload

	err := AddItem()
}