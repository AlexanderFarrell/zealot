package comments

import (
	"fmt"
	"strconv"
	"time"
	"zealotd/apps/account"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

func InitRouter(app *fiber.App) fiber.Router {
	router := app.Group("/comments")
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
		return web.SendJSONOrError(c, entries, err, "getting comment entries")
	})

	router.Get("/item/:item_id", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		itemID, err := strconv.Atoi(c.Params("item_id"))
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse item_id")
		}

		entries, err := GetForItem(itemID, accountID)
		return web.SendJSONOrError(c, entries, err, "getting comment entries for item")
	})

	router.Post("/", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		payload := struct {
			ItemID    int    `json:"item_id"`
			Timestamp string `json:"timestamp"`
			Content   string `json:"content"`
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
			ts, err = time.Parse(time.RFC3339Nano, payload.Timestamp)
			if err != nil {
				return c.Status(fiber.StatusBadRequest).SendString("timestamp must be RFC3339")
			}
		}

		_, err = AddEntry(payload.ItemID, ts, payload.Content, accountID)
		return web.SendOkOrError(c, err, "adding comment entry")
	})

	router.Patch("/:comment_id", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		commentID, err := strconv.ParseInt(c.Params("comment_id"), 10, 64)
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse comment_id")
		}

		payload := struct {
			Content string `json:"content"`
			Time    string `json:"time"`
		}{}
		if err := c.BodyParser(&payload); err != nil {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		var ts *time.Time
		if payload.Time != "" {
			parsed, err := time.Parse(time.RFC3339Nano, payload.Time)
			if err != nil {
				return c.Status(fiber.StatusBadRequest).SendString("time must be RFC3339")
			}
			ts = &parsed
		}

		err = UpdateEntry(commentID, payload.Content, ts, accountID)
		return web.SendOkOrError(c, err, "updating comment entry")
	})

	router.Delete("/:comment_id", func(c *fiber.Ctx) error {
		accountID := web.GetKeyFromSessionInt(c, "account_id")
		commentID, err := strconv.ParseInt(c.Params("comment_id"), 10, 64)
		if err != nil {
			return c.Status(fiber.StatusBadRequest).SendString("Unable to parse comment_id")
		}
		err = DeleteEntry(commentID, accountID)
		return web.SendOkOrError(c, err, "deleting comment entry")
	})

	return router
}
