package settings

import (
	"encoding/json"
	"zealotd/apps/account"
)

type Settings struct {
}

func SettingsHandler(accountID int, raw json.RawMessage) error {
	var s Settings
	if err := json.Unmarshal(raw, &s); err != nil {
		return err
	}

	// Validate

	return account.SaveUserSettings(accountID, raw)
}