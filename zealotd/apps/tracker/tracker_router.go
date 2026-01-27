package tracker

import (
	"fmt"
	"strconv"
	"time"
	"zealotd/apps/account"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/tracker")
	router.Use(account.RequireLoginMiddleware)

	router.Get("/day/:date", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		dateStr := c.Params("date")
		day, err := time.Parse(time.DateOnly, dateStr)
		if err != nil {
			fmt.Printf("Error parsing date: %v\n", err)
			return c.Status(fiber.StatusBadRequest).SendString("Please send a date in format YYYY-MM-DD")
		}

		entries, err := GetForDay(day, accountID)
		return web.SendJSONOrError(c, entries, err, "getting tracker entries")
	})

	router.Post("/", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		payload := struct {
			ItemID    int    `json:"item_id"`
			Timestamp string `json:"timestamp"`
			Level     int    `json:"level"`
			Comment   string `json:"comment"`
		}{}
		if err := c.BodyParser(&payload); err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}
		if payload.ItemID == 0 {
			return c.Status(fiber.StatusBadRequest).SendString("item_id required")
		}

		var ts time.Time
		var err error
		if payload.Timestamp == "" {
			ts = time.Now()
		} else {
			ts, err = time.Parse(time.RFC3339, payload.Timestamp)
			if err != nil {
				return c.Status(fiber.StatusBadRequest).SendString("timestamp must be RFC3339")
			}
		}

		if payload.Level == 0 {
			payload.Level = 3
		}

		_, err = AddEntry(payload.ItemID, ts, payload.Level, payload.Comment, accountID)
		return web.SendOkOrError(c, err, "adding tracker entry")
	})

	router.Delete("/:tracker_id", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		trackerID, err := strconv.ParseInt(c.Params("tracker_id"), 10, 64)
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse tracker_id")
		}
		err = DeleteEntry(trackerID, accountID)
		return web.SendOkOrError(c, err, "deleting tracker entry")
	})

	return router
}
