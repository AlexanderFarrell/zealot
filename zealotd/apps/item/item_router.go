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

	router.Patch("/:item_id/attr", setAttributes)
	router.Patch("/:item_id/attr/rename", renameAttribute)
	router.Delete("/:item_id/attr/:key", deleteAttribute)

	// When creating accounts, add a Home item
	web.EventOn("on_create_account", func(args []string) {
		username := args[0]
		err := AddItemByUsername("Home", username)
		if err != nil {
			fmt.Printf("Failed to make home page for new user %s: %v\n", username, err)
		}
	})

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
		fmt.Printf("%v\n", err)
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

func deleteAttribute(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	item_id, err := strconv.Atoi(c.Params("item_id"))
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Unable to convert item_id to number")
	}
	key := c.Params("key")
	err = DeleteAttribute(item_id, account_id, key)
	if err != nil {
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}
}

func renameAttribute(c * fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	item_id, err := strconv.Atoi(c.Params("item_id"))
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Unable to convert item_id to number")
	}
	payload := struct {
		OldKey string `json:"old_key"`
		NewKey string `json:"new_key"`
	}{}

	if err := c.BodyParser(&payload); err != nil {
		fmt.Printf("Error parsing rename value: %v", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	err = RenameAttribute(item_id, account_id, payload.OldKey, payload.NewKey)
	if err != nil {
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}
}

func setAttributes(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	item_id, err := strconv.Atoi(c.Params("item_id"))
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Unable to convert item_id to number")
	}
	
	var body map[string]any

	if err := c.BodyParser(&body); err != nil {
		fmt.Printf("Error parsing body: %w\n", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	if len(body) > 10 {
		return c.Status(fiber.StatusBadRequest).SendString("Please only update 10 fields at a time")
	}

	for key, value := range body {
		err := SetAttributeForItem(item_id, account_id, key, value)
		if err != nil {
			fmt.Printf("Error setting item: %w\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}
	}

	return c.SendStatus(fiber.StatusOK)	
}