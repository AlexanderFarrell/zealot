package item

import (
	"fmt"
	"strconv"
	"zealotd/apps/account"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

var updatableFields = map[string]int {
	"title": 0,
	"content": 0,
}

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/item")
	router.Use(account.RequireLoginMiddleware)

	router.Get("/", getRootItems)
	router.Get("/title/:title", getByTitle)
	router.Get("/search", searchTitle)
	router.Post("/", addItem)
	router.Patch("/:item_id", updateItem)
	router.Delete("/:item_id", deleteItem)

	return router
}

func getRootItems(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusNotImplemented)
}

func getByTitle(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	title := c.Params("title")
	item, err := GetItemByTitle(title, account_id)
	if err != nil {
		fmt.Printf("%v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.JSON(item)
}

func searchTitle(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	title := c.Query("term")
	items, err := SearchByTitle(title, account_id)
	if err != nil {
		fmt.Printf("%v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.JSON(items)
}

func addItem(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	// Payload
	payload := struct {
		Title string `json:"title"`
	}{}

	if err := c.BodyParser(&payload); err != nil {
		fmt.Printf("Error parsing updates: %v", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	err := AddItem(payload.Title, account_id)
	if err != nil {
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}
} 

func updateItem(c * fiber.Ctx) error {
	return web.HandleUpdateRoute(c, "item", "item_id", updatableFields)
}

func deleteItem(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	item_id, err := strconv.Atoi(c.Params("item_id"))

	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Unable to convert item_id to number")
	}

	err = DeleteItem(item_id, account_id)
	if err != nil {
		fmt.Printf("Error deleting item of id %d for account %d: %v\n", item_id, account_id, err)
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}

}