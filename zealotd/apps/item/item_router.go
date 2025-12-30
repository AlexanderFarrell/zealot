package item

import (
	"fmt"
	"net/url"
	"strconv"
	"zealotd/apps/account"
	"zealotd/apps/item/attribute"
	"zealotd/apps/item/itemtype"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

var updatableFields = map[string]int{
	"title":   0,
	"content": 0,
}

var updatableFieldsAttrKind = map[string]int{
	"key":         0,
	"description": 0,
	"base_type":   0,
	"config":      0,
}

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/item")
	router.Use(account.RequireLoginMiddleware)

	// Protects sub routers
	itemtype.InitRouter(router)
	attribute.InitRouter(router)

	router.Get("/", getRootItems)
	router.Get("/title/:title", getByTitle)
	router.Get("/id/:item_id", func (c *fiber.Ctx) error {
		account_id := web.GetKeyFromSessionInt(c, "account_id")
		itemID, err := strconv.Atoi(c.Params("item_id"))
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
		}
		// title := c.Params("title")
		// title, err := url.QueryUnescape(title)
		// if err != nil {
			// fmt.Printf("Failed to query unescape input: %v\n", err)
			// return c.SendStatus(fiber.StatusInternalServerError)
		// }
		item, err := GetItemByID(itemID, account_id)
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}
		return c.JSON(item)
	})
	router.Get("/search", searchTitle)
	router.Get("/children/:item_id", getChildren)

	router.Post("/", addItem)
	router.Patch("/:item_id", updateItem)
	router.Delete("/:item_id", deleteItem)

	router.Patch("/:item_id/attr", setAttributes)
	router.Patch("/:item_id/attr/rename", renameAttribute)
	router.Delete("/:item_id/attr/:key", deleteAttribute)

	router.Post("/:item_id/assign_type/:type_name", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		itemID, err := strconv.Atoi(c.Params("item_id"))
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
		}
		typeName := c.Params("type_name")
		err = AssignItemType(itemID, typeName, accountID)
		return web.SendOkOrError(c, err, "assigning item type")
	})

	router.Delete("/:item_id/assign_type/:type_name", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		itemID, err := strconv.Atoi(c.Params("item_id"))
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
		}
		typeName := c.Params("type_name")
		err = UnassignItemType(itemID, typeName, accountID)
		return web.SendOkOrError(c, err, "unassigning item type")
	})

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
	accountID := web.GetKeyFromSessionInt(c, "account_id");
	
	// See if type is passed, if so do that:
	kind := c.Query("type", "")
	var items []Item
	var err error

	if kind != "" {
		items, err = GetItemsByType(kind, accountID)
	} else {
		items, err = GetItemsByAttribute("Root", 1, "value_int", accountID)

	}

	return web.SendJSONOrError(c, items, err, "getting root items")
}

func getChildren(c *fiber.Ctx) error {
	accountID := web.GetKeyFromSessionInt(c, "account_id")		
	itemID, err := strconv.Atoi(c.Params("item_id"))
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
	}
	// title := c.Params("title")
	// title, err := url.QueryUnescape(title)
	// if err != nil {
		// fmt.Printf("Failed to query unescape input: %v\n", err)
		// return c.SendStatus(fiber.StatusInternalServerError)
	// }
	item, err := GetItemByID(itemID, accountID)
	if err != nil || item == nil {
		fmt.Printf("Failed to get item for children: %v\n", err)
		if err == nil {
			return c.SendStatus(fiber.StatusNotFound)
		} else {
			return c.SendStatus(fiber.StatusInternalServerError)
		}
	}
	
	items, err := GetItemsByAttribute("Parent", item.Title, "value_text", accountID)
	return web.SendJSONOrError(c, items, err, "getting children")
}

func getByTitle(c *fiber.Ctx) error {
	account_id := web.GetKeyFromSessionInt(c, "account_id")
	title := c.Params("title")
	title, err := url.QueryUnescape(title)
	if err != nil {
		fmt.Printf("Failed to query unescape input: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	item, err := GetItemByTitle(title, account_id)
	if err != nil {
		fmt.Printf("%v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	if item == nil {
		fmt.Printf("%v\n", err)
		return c.SendStatus(fiber.StatusNotFound)
	}

	return c.Redirect("/api/item/id/" + strconv.Itoa(item.ItemID))
	// if err != nil {
	// 	fmt.Printf("%v\n", err)
	// 	return c.SendStatus(fiber.StatusInternalServerError)
	// }
	// return c.JSON(item)
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
	var payload Item

	if err := c.BodyParser(&payload); err != nil {
		fmt.Printf("Error parsing updates: %v", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	err := AddItem2(&payload, account_id)
	return web.SendJSONOrError(c, payload, err, "adding item")
}

func updateItem(c *fiber.Ctx) error {
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
	err = attribute.DeleteAttribute(item_id, account_id, key)
	if err != nil {
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}
}

func renameAttribute(c *fiber.Ctx) error {
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

	err = attribute.RenameAttribute(item_id, account_id, payload.OldKey, payload.NewKey)
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
		fmt.Printf("Error parsing body: %v\n", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	if len(body) > 10 {
		return c.Status(fiber.StatusBadRequest).SendString("Please only update 10 fields at a time")
	}

	for key, value := range body {
		err := attribute.SetAttributeForItem(item_id, account_id, key, value)
		if err != nil {
			fmt.Printf("Error setting item: %v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}
	}

	return c.SendStatus(fiber.StatusOK)
}
