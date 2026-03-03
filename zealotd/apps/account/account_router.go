package account

import (
	"encoding/json"
	"fmt"
	"os"
	"strconv"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
)

var (
	accountCreationEnabled = false
	saltRounds             = 10
	usernameMin            = 1
	usernameMax            = 20
	passwordMin            = 6
	passwordMax            = 70
	emailMin               = 1
	emailMax               = 50
	nameMin                = 1
	nameMax                = 50
)

type SettingsHandler func(accountID int, raw json.RawMessage) error

// Provides username and password authentication via API responses
func InitRouter(app *fiber.App, settingsHandler SettingsHandler) fiber.Router {
	initEnvVariables()

	api := app.Group("/account")
	api.Post("/register", createAccount)
	api.Post("/login", login)
	api.Get("/exists", usernameExists)
	api.Get("/is_logged_in", isLoggedIn)
	api.Patch("/settings", func(c *fiber.Ctx) error {
		return updateSettings(c, settingsHandler)
	})
	api.Get("/details", getAccountDetails)
	api.Get("/logout", logout)
	api.Post("/api-key", generateAPIKey)
	api.Delete("/api-key", revokeAPIKey)
	api.Get("/api-key", getAPIKeyStatus)

	return api
}

func isLoggedIn(c *fiber.Ctx) error {
	if IsLoggedIn(c) {
		return c.SendStatus(fiber.StatusOK)
	} else {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
}

func getAccountDetails(c *fiber.Ctx) error {
	accountID, err := web.GetAccountID(c)
	if err != nil {
		return c.SendStatus(fiber.StatusUnauthorized)
	}

	details, err := GetAccountDetailsByID(accountID)
	if err != nil {
		fmt.Printf("Error getting account details: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.JSON(details)
}

func createAccount(c *fiber.Ctx) error {
	if !accountCreationEnabled {
		fmt.Printf("Account creation denied")
		return c.Status(fiber.StatusForbidden).SendString("Account registration disabled")
	}

	payload := struct {
		Username string `json:"username"`
		Password string `json:"password"`
		Confirm  string `json:"confirm"`
		Email    string `json:"email"`
		Name     string `json:"name"`
	}{}

	err := c.BodyParser(&payload)
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Invalid Request")
	}

	status, err := CreateAccount(payload.Username, payload.Password,
		payload.Confirm, payload.Email, payload.Name)
	if err != nil {
		fmt.Printf("Error creating account: %v", err)
		return c.Status(status).SendString(err.Error())
	} else {
		fmt.Printf("User %s created\n", payload.Username)

		details, err := GetAccountDetails(payload.Username)
		if err != nil {
			return c.SendStatus(fiber.StatusInternalServerError)
		}
		sess := web.GetSessionStore(c)
		sess.Set("account_id", details.AccountID)
		sess.Set("username", details.Username)
		web.SaveSession(sess)

		return c.Status(fiber.StatusCreated).JSON(details)
	}
}

func login(c *fiber.Ctx) error {
	payload := struct {
		Username string `json:"username"`
		Password string `json:"password"`
	}{}
	err := c.BodyParser(&payload)
	if err != nil {
		return c.SendStatus(fiber.StatusBadRequest)
	}

	success, err := Login(payload.Username, payload.Password)

	if err != nil {
		fmt.Printf("Error logging in: %v", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	} else if success {
		fmt.Printf("User %s logged in\n", payload.Username)

		details, err := GetAccountDetails(payload.Username)
		if err != nil {
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		sess := web.GetSessionStore(c)
		sess.Set("account_id", details.AccountID)
		sess.Set("username", details.Username)
		web.SaveSession(sess)
		return c.Status(fiber.StatusOK).JSON(details)
	} else {
		fmt.Printf("Failed login on user %s\n", payload.Username)
		return c.SendStatus(fiber.StatusUnauthorized)
	}
}

func usernameExists(c *fiber.Ctx) error {
	// If we cannot create accounts, no one needs to know
	// if a username is taken.
	if !accountCreationEnabled {
		return c.SendStatus(fiber.StatusForbidden)
	}

	username := c.FormValue("username")
	exists, err := UsernameExists(username)
	if err != nil {
		fmt.Printf("Error determining if username exists: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	} else {
		return c.JSON(fiber.Map{
			"exists": exists,
		})
	}
}

func logout(c *fiber.Ctx) error {
	if !IsLoggedIn(c) {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
	username, _ := web.GetUsername(c)
	fmt.Printf("User %s logged out\n", username)

	// Only destroy the session when using session-based auth.
	if c.Locals("account_id") == nil {
		sess := web.GetSessionStore(c)
		sess.Delete("account_id")
		web.SaveSession(sess)
	}
	return c.SendStatus(fiber.StatusOK)
}

func changeUsername(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusNotImplemented)
}

func changePassword(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusNotImplemented)
}

func changeAccountDetails(c *fiber.Ctx) error {
	return c.SendStatus(fiber.StatusNotImplemented)
}

func updateSettings(c *fiber.Ctx, handler SettingsHandler) error {
	accountID, err := web.GetAccountID(c)
	if err != nil {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
	raw := json.RawMessage(c.Body())

	if handler == nil {
		return c.SendStatus(fiber.StatusNotImplemented)
	}

	if err := handler(accountID, raw); err != nil {
		fmt.Printf("Error parsing data: %v", err)
		return c.SendStatus(fiber.StatusBadRequest)
	}

	return c.SendStatus(fiber.StatusOK)
}

func generateAPIKey(c *fiber.Ctx) error {
	accountID, err := web.GetAccountID(c)
	if err != nil {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
	key, err := GenerateAPIKey(accountID)
	if err != nil {
		fmt.Printf("Error generating API key: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.JSON(fiber.Map{"key": key})
}

func revokeAPIKey(c *fiber.Ctx) error {
	accountID, err := web.GetAccountID(c)
	if err != nil {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
	if err := RevokeAPIKey(accountID); err != nil {
		fmt.Printf("Error revoking API key: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.SendStatus(fiber.StatusOK)
}

func getAPIKeyStatus(c *fiber.Ctx) error {
	accountID, err := web.GetAccountID(c)
	if err != nil {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
	exists, err := HasAPIKey(accountID)
	if err != nil {
		fmt.Printf("Error checking API key status: %v\n", err)
		return c.SendStatus(fiber.StatusInternalServerError)
	}
	return c.JSON(fiber.Map{"exists": exists})
}

func initEnvVariables() {
	creationEnabled := os.Getenv("ACCOUNT_CREATION_ENABLED")
	if creationEnabled == "true" {
		accountCreationEnabled = true
	}

	saltRoundsEnv := os.Getenv("ACCOUNT_SALT_ROUNDS")
	if saltRoundsEnv != "" {
		saltRoundsInt, err := strconv.Atoi(saltRoundsEnv)
		if err == nil {
			saltRounds = saltRoundsInt
		} else {
			fmt.Printf("Error converting Salt Rounds to string%v\n", err)
		}
	}
}
