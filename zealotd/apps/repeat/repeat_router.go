package repeat

import (
	"fmt"
	"strconv"
	"time"
	"zealotd/apps/account"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/repeat")
	router.Use(account.RequireLoginMiddleware)

	router.Get("/day/:date", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		dateStr := c.Params("date")
		day, err := time.Parse(time.DateOnly, dateStr)
		if err != nil {
			fmt.Printf("Error parsing date: %v\n", err)
			return c.Status(fiber.StatusBadRequest).SendString("Please send a date in format YYYY-MM-DD")
		}

		results, err := GetForDay(day, accountID)
		return web.SendJSONOrError(c, results, err, "getting repeats for day")
	})

	router.Patch("/:item_id/day/:date", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		itemID, err := strconv.Atoi(c.Params("item_id"))
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
		}
		dateStr := c.Params("date")
		day, err := time.Parse(time.DateOnly, dateStr)
		if err != nil {
			fmt.Printf("Error parsing date: %v\n", err)
			return c.Status(fiber.StatusBadRequest).SendString("Please send a date in format YYYY-MM-DD")
		}

		payload := struct {
			Status  string `json:"status"`
			Comment string `json:"comment"`
		}{}
		if err := c.BodyParser(&payload); err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}
		if payload.Status == "" {
			return c.Status(fiber.StatusBadRequest).SendString("status is required")
		}

		err = SetRepeatStatus(itemID, day, accountID, payload.Status, payload.Comment)
		return web.SendOkOrError(c, err, "setting repeat status")
	})

	return router
}
