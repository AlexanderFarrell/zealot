package web

import (
	"fmt"
	"net/url"
	"os"
	"strconv"

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

// func HandleAddRoute[T any](c *fiber.Ctx, tableName string, identifierName string) error {
// 	account_id := GetKeyFromSessionInt(c, "account_id")
// 	identifier := c.Params(identifierName)
// 	identifier, err := url.QueryUnescape(identifier)
// 	if err != nil {
// 		fmt.Printf("Error unescaping %s\n", identifierName)
// 		return c.SendStatus(fiber.StatusBadRequest)
// 	}
// 	identiferInt, err := strconv.Atoi(identifier)
// 	if err != nil {
// 		return c.Status(fiber.StatusBadRequest).SendString("Please send a number for " + identifierName)
// 	}

// }

func HandleDeleteRoute(c *fiber.Ctx, tableName string, identifierName string) error {
	account_id := GetKeyFromSessionInt(c, "account_id")
	identifier := c.Params(identifierName)
	identifier, err := url.QueryUnescape(identifier)
	if err != nil {
		fmt.Printf("Error unescaping %s\n", identifierName)
		return c.SendStatus(400)
	}
	identifierInt, err := strconv.Atoi(identifier)
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Please send a number for " + identifierName)
	}

	err = DeleteRow(account_id, tableName, identifierInt, identifierName)
	if err != nil {
		fmt.Printf("Error deleting item from table %s: %v", tableName, err)
		return c.Status(fiber.StatusInternalServerError).SendString("Server error updating " + tableName)
	}
	return c.SendStatus(fiber.StatusOK)
}

func SendJSONOrError[T any](c *fiber.Ctx, data T, err error, errorMessage string) error {
	if err != nil {
		fmt.Printf("Error: %s: %v\n", errorMessage, err)
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.JSON(data)
	}
}

func SendOkOrError(c *fiber.Ctx, err error, errorMessage string) error {
	if err != nil {
		fmt.Printf("Error: %s: %v\n", errorMessage, err)
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.SendStatus(fiber.StatusOK)
	}
}

func GetEnvVar(name string, defaultVal string) string {
	val := os.Getenv(name)
	if val == "" {
		val = defaultVal
	}
	return val
}
