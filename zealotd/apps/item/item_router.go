package item

import "github.com/gofiber/fiber/v2"

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/item", getRootItems)

	router.Get("/", getRootItems)

	return router
}

func getRootItems(c *fiber.Ctx) error {
	return c.SendStatus(200)
}