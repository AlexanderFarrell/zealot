package account

import (
	"fmt"
	"github.com/gofiber/fiber/v2"
	"os"
	"zealotd/web"
	"strconv"
)

var (
	accountCreationEnabled = false
	saltRounds             = 10
	usernameMin            = 1
	usernameMax            = 20
	passwordMin            = 6
	passwordMax            = 70
	firstNameMin           = 1
	firstNameMax           = 50
	lastNameMin            = 1
	lastNameMax            = 50
)

// Provides username and password authentication via API responses
func InitRouter(app *fiber.App) fiber.Router {
	initEnvVariables()

	api := app.Group("/account")
	api.Post("/register", createAccount)
	api.Post("/login", login)
	api.Get("/exists", usernameExists)
	api.Get("/is_logged_in", isLoggedIn)
	api.Get("/logout", logout)

	return api
}

func isLoggedIn(c *fiber.Ctx) error {
	if IsLoggedIn(c) {
		return c.SendStatus(fiber.StatusOK)
	} else {
		return c.SendStatus(fiber.StatusUnauthorized)
	}
}

func createAccount(c *fiber.Ctx) error {
	if !accountCreationEnabled {
		fmt.Printf("Account creation denied")
		return c.Status(fiber.StatusForbidden).SendString("Account registration disabled")
	}

	payload := struct {
		Username  string `json:"username"`
		Password  string `json:"password"`
		Confirm   string `json:"confirm"`
		FirstName string `json:"first_name"`
		LastName  string `json:"last_name"`
	}{}

	err := c.BodyParser(&payload)
	if err != nil {
		return c.Status(fiber.StatusBadRequest).SendString("Invalid Request")
	}

	status, err := CreateAccount(payload.Username, payload.Password,
		payload.Confirm, payload.FirstName, payload.LastName)
	if err != nil {
		fmt.Printf("Error creating account: %v", err)
		return c.Status(status).SendString(err.Error())
	} else {
		fmt.Printf("User created: %s", payload.Username)
		sess := web.GetSessionStore(c)
		sess.Set("username", payload.Username)
		web.SaveSession(sess)
		return c.SendStatus(fiber.StatusCreated)
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
		sess := web.GetSessionStore(c)
		sess.Set("username", payload.Username)
		web.SaveSession(sess)
		return c.SendStatus(fiber.StatusAccepted)
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
	username := web.GetKeyFromSession(c, "username")
	fmt.Printf("User %s logged out\n", username)

	sess := web.GetSessionStore(c)
	sess.Delete("username")
	web.SaveSession(sess)
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
