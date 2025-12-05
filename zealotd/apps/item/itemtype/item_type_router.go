package itemtype

import (
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(parent fiber.Router) fiber.Router {
	router := parent.Group("/type")
	
	// Gets all item types
	router.Get("/", func (c * fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		types, err := GetItemTypes(accountID)
		return web.SendJSONOrError(c, types, err, "retrieving attribute types")
	})

	// Adds a new item type
	router.Post("/", func (c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		var itemType ItemType
		if err := c.BodyParser(&itemType); err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}
		err := AddItemType(itemType, accountID)
		return web.SendOkOrError(c, err, "adding item type")
	})

	// Updates an items fields
	router.Patch("/:type_id", func (c *fiber.Ctx) error {
		return web.HandleUpdateRoute(c, "item_type", "type_id", updatableFields)
	})

	// Deletes an item
	router.Delete("/:type_id", func (c *fiber.Ctx) error {
		return web.HandleDeleteRoute(c, "item_type", "type_id")
	})

	return router
}