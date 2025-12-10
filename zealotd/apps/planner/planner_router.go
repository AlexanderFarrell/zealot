package planner

import (
	"fmt"
	"strconv"
	"time"
	"zealotd/apps/item"
	"zealotd/apps/item/attribute"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) {
	router := app.Group("/planner")

	// Get on day
	router.Get("/day/:date", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		dateStr := c.Params("date")
		t, err := time.Parse(time.DateOnly, dateStr)
		if err != nil {
			fmt.Printf("Error parsing date: %v\n", err)
			return c.Status(fiber.StatusBadRequest).SendString("Please send a date in format YYYY-MM-DD")
		}
		items, err := item.GetItemsByAttribute("Date", t, "value_date", accountID)
		return web.SendJSONOrError(c, items, err, "getting items on date")
	})

	router.Get("/week/:week", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		week, err := attribute.ToWeekCode(c.Params("week"))
		if err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}
		items, err := item.GetItemsByAttribute("Week", week, "value_int", accountID)
		return web.SendJSONOrError(c, items, err, "getting items on week")
	})

	router.Get("/month/:month/year/:year", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		month, err := strconv.Atoi(c.Params("month"))
		if err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}
		year, err := strconv.Atoi(c.Params("year"))
		if err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		// TODO: This is a hack, filter both month and year here.
		items, err := item.GetItemsByAttribute("Year", year, "value_int", accountID)
		if err != nil {
			fmt.Printf("Error getting month items: %v\n", err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		fmt.Printf("Items from year: %d\n", len(items))
		itemsSend := make([]item.Item, 0)
		for _, item := range items {
			if v, ok := item.Attributes["Month"].(int64); ok {
				if v == int64(month) {
					itemsSend = append(itemsSend, item)
				}
			}
		}

		return c.JSON(itemsSend)
	})

	router.Get("/year/:year", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		year, err := strconv.Atoi(c.Params("year"))
		if err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		items, err := item.GetItemsByAttribute("Year", year, "value_int", accountID)
		return web.SendJSONOrError(c, items, err, "getting year items")
	})
}
