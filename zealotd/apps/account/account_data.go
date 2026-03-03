package account

import (
	"crypto/rand"
	"crypto/sha256"
	"database/sql"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"zealotd/web"

	"github.com/gofiber/fiber/v2"
	"golang.org/x/crypto/bcrypt"
)

type AccountDetails struct {
	Username  string          `json:"username"`
	Email     string          `json:"email"`
	Name      string          `json:"name"`
	AccountID int             `json:"account_id"`
	Settings  json.RawMessage `json:"settings"`
}

func IsLoggedIn(c *fiber.Ctx) bool {
	if c.Locals("account_id") != nil {
		return true
	}
	sess := web.GetSessionStore(c)
	return sess.Get("account_id") != nil
}

const apiKeyPrefix = "zl_"

// GenerateAPIKey creates a new API key for the account, storing only its
// SHA-256 hash. Returns the plaintext key — shown once and never stored.
func GenerateAPIKey(accountID int) (string, error) {
	b := make([]byte, 24)
	if _, err := rand.Read(b); err != nil {
		return "", fmt.Errorf("generating random bytes: %w", err)
	}
	plaintext := apiKeyPrefix + hex.EncodeToString(b)

	hash := sha256.Sum256([]byte(plaintext))
	hashHex := hex.EncodeToString(hash[:])

	_, err := web.Database.Exec(`
		insert into api_key (account_id, key_hash)
		values ($1, $2)
		on conflict (account_id)
		do update set key_hash = $2, created_at = now()
	`, accountID, hashHex)
	if err != nil {
		return "", fmt.Errorf("storing api key: %w", err)
	}
	return plaintext, nil
}

// GetAccountFromAPIKey looks up the account_id and username for the given
// plaintext key. Returns zero/empty with no error when the key is not found.
func GetAccountFromAPIKey(key string) (int, string, error) {
	hash := sha256.Sum256([]byte(key))
	hashHex := hex.EncodeToString(hash[:])

	var accountID int
	var username string
	err := web.Database.QueryRow(`
		select a.account_id, a.username
		from api_key ak
		join account a on a.account_id = ak.account_id
		where ak.key_hash = $1
	`, hashHex).Scan(&accountID, &username)
	if errors.Is(err, sql.ErrNoRows) {
		return 0, "", nil
	}
	if err != nil {
		return 0, "", fmt.Errorf("looking up api key: %w", err)
	}
	return accountID, username, nil
}

// RevokeAPIKey deletes the API key for the given account.
func RevokeAPIKey(accountID int) error {
	_, err := web.Database.Exec("delete from api_key where account_id=$1", accountID)
	return err
}

// HasAPIKey reports whether the account currently has an API key.
func HasAPIKey(accountID int) (bool, error) {
	var exists bool
	err := web.Database.QueryRow(
		"select exists(select 1 from api_key where account_id=$1)", accountID,
	).Scan(&exists)
	return exists, err
}

// GetAccountDetailsByID retrieves account information by account_id.
func GetAccountDetailsByID(accountID int) (AccountDetails, error) {
	row := web.Database.QueryRow(
		"select account_id, username, email, full_name, settings from account where account_id=$1",
		accountID,
	)
	var details AccountDetails
	if err := row.Scan(&details.AccountID, &details.Username, &details.Email, &details.Name, &details.Settings); err != nil {
		fmt.Printf("Error scanning account details for account_id %d: %v\n", accountID, err)
		return AccountDetails{}, errors.New("issue getting account details")
	}
	return details, nil
}

func CreateAccount(username string, password string,
	confirm string, email string, name string) (int, error) {
	if !accountCreationEnabled {
		return 401, fmt.Errorf("account creation is currently disabled")
	}
	err := isWithin(username, usernameMin, usernameMax, "username")
	if err != nil {
		return 400, err
	}
	err = isWithin(password, passwordMin, passwordMax, "password")
	if err != nil {
		return 400, err
	}
	err = isWithin(email, emailMin, emailMax, "first name")
	if err != nil {
		return 400, err
	}
	err = isWithin(name, nameMin, nameMax, "last name")
	if err != nil {
		return 400, err
	}

	exists, err := UsernameExists(username)
	if err != nil {
		return 500, fmt.Errorf("server error creating account")
	}
	if exists {
		return 400, fmt.Errorf("account already exists")
	}

	passByte := []byte(password)

	hash, err := bcrypt.GenerateFromPassword(passByte, saltRounds)
	if err != nil {
		fmt.Printf("%v\n", err)
		return 500, errors.New("issue creating account please try again")
	}

	_, err = web.Database.Exec("insert into account(username, password, email, full_name) values ($1, $2, $3, $4);", username, hash, email, name)
	if err != nil {
		fmt.Printf("%v\n", err)
		return 500, errors.New("issue creating account please try again")
	}

	web.EventEmit("on_create_account", []string{username})

	return 200, nil
}

func UsernameExists(username string) (bool, error) {
	row := web.Database.QueryRow("select count(*) > 1 from account where username=$1", username)
	if row.Err() != nil {
		fmt.Printf("Error: %v\n", row.Err())
		return false, errors.New("issue verifying username")
	}

	var result bool
	var err = row.Scan(&result)
	if err != nil {
		fmt.Printf("Error: %v\n", row.Err())
		return false, errors.New("issue verifying username")
	}
	return result, nil
}

func GetAccountDetails(username string) (AccountDetails, error) {
	row := web.Database.QueryRow("select account_id, username, email, full_name, settings from account where username=$1", username)
	if row.Err() != nil {
		fmt.Printf("Error retrieving account details for account %s: %v\n", username, row.Err())
		return AccountDetails{}, errors.New("issue getting account details")
	}
	var details AccountDetails
	err := row.Scan(&details.AccountID, &details.Username, &details.Email, &details.Name, &details.Settings)
	if err != nil {
		fmt.Printf("Error scanning account details for account %s: %v\n", username, err)
		return AccountDetails{}, errors.New("issue getting account details")
	}
	return details, nil
}

func Login(username string, password string) (bool, error) {
	row := web.Database.QueryRow("select password from account where username=$1", username)
	if row.Err() != nil {
		fmt.Printf("Error getting data from database %v\n", row.Err())
		return false, errors.New("issue logging in")
	}

	var hash string
	err := row.Scan(&hash)
	if errors.Is(err, sql.ErrNoRows) {
		return false, nil
	} else if err != nil {
		fmt.Printf("Error getting data from database %v\n", err)
		return false, errors.New("issue logging in")
	}
	hashByte := []byte(hash)
	passByte := []byte(password)

	err = bcrypt.CompareHashAndPassword(hashByte, passByte)
	if err != nil {
		fmt.Printf("Error testing hash: %v\n", err)
		return false, nil
	} else {
		return true, nil
	}
}

func SaveUserSettings(accountID int, raw json.RawMessage) error {
	query := `
	update account
	set settings=$1
	where account_id=$2;
	`

	_, err := web.Database.Exec(query, raw, accountID)
	return err
}

func isWithin(item string, min int, max int, name string) error {
	l := len(item)
	if l < min || l > max {
		s := fmt.Sprintf("%s must be within %d and %d", name, min, max)
		return errors.New(s)
	} else {
		return nil
	}
}
