package domain

import (
	"encoding/json"
	"time"
)

type AccountID int64
type APIKeyID int64
type AccountSettings json.RawMessage

type Account struct {
	AccountID AccountID `json:"account_id"`
	Username string `json:"username"`
	Email string `json:"email"`
	Name string `json:"name"`
	Settings AccountSettings `json:"settings"`	
}

type AccountRegistration struct {
	Username string `json:"username"`
	Password string `json:"password"`
	Email string `json:"email"`
	Name string `json:"name"`
}

type AccountLogin struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type APIKey string

// Do not add json fields, we do not need to serialize for security
// reasons.
type APIKeyEntry struct {
	APIKeyID APIKeyID
	AccountID AccountID
	KeyHash string
	CreatedAt time.Time
}