package planner

import (
	"fmt"
	"time"
	"zealotd/apps/item"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) {
	router := app.Group("/planner")

	// Get on day
	router.Get("/day/:date", func (c * fiber.Ctx) error {
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

	router.Get("/week/:week", func (c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		week := c.Params("week")
		items, err := item.GetItemsByAttribute("Week", week, "value_text", accountID)
		return web.SendJSONOrError(c, items, err, "getting items on week")
	})

	router.Get("/month/:month/year/:year", func (c *fiber.Ctx) error {
		return c.SendStatus(fiber.StatusNotImplemented)
	})

	router.Get("/year/:year", func (c *fiber.Ctx) error {
		return c.SendStatus(fiber.StatusNotImplemented)
	})

	// router.Get("/today", func (c *fiber.Ctx) error {
		// return c.Redirect()
	// })
}