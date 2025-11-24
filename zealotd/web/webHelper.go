package web

import (
	"fmt"
	"net/url"

	"github.com/gofiber/fiber/v2"
)

func HandleUpdateRoute(c *fiber.Ctx, tableName string, identifierName string, allowedFields map[string]int) error {
	account_id := GetKeyFromSessionInt(c, "account_id")
	identifier := c.Params(identifierName)
	identifier, err := url.QueryUnescape(identifier)
	if err != nil {
		fmt.Printf("Error unescaping %s\n", identifierName)
		return c.SendStatus(400)
	}

	updates := make(map[string]interface{})
	if err := c.BodyParser(&updates); err != nil {
		fmt.Printf("Error parsing updates: %v\n", err)
		return c.Status(400).SendString("Incorrect format")
	}

	if len(updates) == 0 {
		return c.Status(400).SendString("Please specify fields to update")
	}

	err = UpdateRow(account_id, tableName, identifier, identifierName, updates, allowedFields)
	if err != nil {
		fmt.Printf("Error updating %s: %v\n", tableName, err)
		return c.Status(500).SendString("Server error updating " + tableName)
	}
	return c.SendStatus(200)
}
