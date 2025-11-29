package settings

import (
	"encoding/json"
	"zealotd/apps/account"
	"zealotd/apps/item"
)

type Settings struct {
	AttributeMetas []item.AttributeMeta `json:"attribute_metas"`
	ItemTypes []item.ItemType `json:"item_types"`
}

func SettingsHandler(accountID int, raw json.RawMessage) error {
	var s Settings
	if err := json.Unmarshal(raw, &s); err != nil {
		return err
	}

	// Validate

	return account.SaveUserSettings(accountID, raw)
}