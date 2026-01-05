package web

import (
	"log"
	"os"
	"time"

	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/csrf"
	"github.com/gofiber/fiber/v2/middleware/helmet"
	"github.com/gofiber/fiber/v2/middleware/limiter"
	"github.com/gofiber/fiber/v2/middleware/logger"
)

func InitServer() *fiber.App {
	app := fiber.New(fiber.Config{})
	allowOrigins := GetEnvVar("CORS_ALLOW_ORIGINS", "http://localhost:3000")
	app.Use(cors.New(cors.Config{
		AllowOrigins:     allowOrigins,
		AllowHeaders:     "Open, Origin, Content-Type, Accept, X-Csrf-Token",
		AllowMethods:     "GET, POST, PUT, PATCH, DELETE, OPTIONS",
		AllowCredentials: true,
	}))
	app.Use(helmet.New(helmet.Config{
		CrossOriginResourcePolicy: "cross-origin",
		CrossOriginEmbedderPolicy: "cross-origin",
	}))
	app.Use(csrf.New(csrf.Config{
		KeyLookup:     "header:X-Csrf-Token",
		CookieSameSite: GetEnvVar("CSRF_COOKIE_SAMESITE", "Lax"),
		CookieSecure:   GetEnvVar("CSRF_COOKIE_SECURE", "false") == "true",
		CookieHTTPOnly: false,
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
