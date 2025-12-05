package attribute

import (
	"zealotd/web"
	"fmt"
	"strconv"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(parent fiber.Router) fiber.Router {
	router := parent.Group("/kind")

	// Gets all attribute kinds, system and account specific.
	router.Get("/", func (c * fiber.Ctx) error {
		account_id := web.GetKeyFromSessionInt(c, "account_id")
		kinds, err := GetAllAttributeKinds(account_id)
		return web.SendJSONOrError(c, kinds, err, "getting attribute kinds")
	})

	// Adds a new attribute kind
	router.Post("/", func (c * fiber.Ctx) error {
		account_id := web.GetKeyFromSessionInt(c, "account_id")
		var kind AttributeKind
		if err := c.BodyParser(&kind); err != nil {
			fmt.Printf("Unable to parse new kind: %v", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}
		kind.IsSystem = false

		err := AddAttributeKind(&kind, account_id)
		return web.SendOkOrError(c, err, "adding attribute kind")
	})

	// Updates fields on an attribute kind
	router.Patch("/:kind_id", func (c *fiber.Ctx) error {
		return web.HandleUpdateRoute(c, "attribute_kind", "kind_id", updatableFieldsAttrKind)
	})

	// Deletes a non-system attribute kind
	router.Delete("/:kind_id", func (c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		kindID, err := strconv.Atoi(c.Params("kind_id"))
		if err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		err = DeleteAttributeKind(kindID, accountID)
		return web.SendOkOrError(c, err, "delete attribute kind")
	})
	return router
}