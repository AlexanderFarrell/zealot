package web

import (
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/helmet"
	"github.com/gofiber/fiber/v2/middleware/limiter"
	"github.com/gofiber/fiber/v2/middleware/logger"
	"log"
	"os"
	"time"
)

func InitServer() *fiber.App {
	app := fiber.New(fiber.Config{})
	app.Use(cors.New(cors.Config{
		AllowOrigins: "",
		AllowHeaders: "Open, Content-Type, Accept",
	}))
	app.Use(helmet.New(helmet.Config{
		CrossOriginResourcePolicy: "cross-origin",
		CrossOriginEmbedderPolicy: "cross-origin",
	}))
	app.Use(limiter.New(limiter.Config{
		Max:        100,
		Expiration: 30 * time.Second,
	}))
	app.Use(logger.New())
	app.Get("/health", func(c *fiber.Ctx) error {
		return c.SendString("{\"status\": \"ok\"}")
	})

	return app
}

func RunServer(app *fiber.App) {
	app.Use(notFoundHandler)

	log.Fatal(app.Listen("0.0.0.0:" + GetPort()))
}

func notFoundHandler(c *fiber.Ctx) error {
	return c.SendStatus(404)
}

func GetPort() string {
	port := os.Getenv("PORT")
	if os.Getenv("PORT") == "" {
		port = "3000"
	}
	return port
}
