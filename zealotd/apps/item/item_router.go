package item

import "github.com/gofiber/fiber/v2"

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/item", getRootItems)

	router.Get("/")

	return router
}

func getRootItems(c *fiber.Ctx) error {
	
}