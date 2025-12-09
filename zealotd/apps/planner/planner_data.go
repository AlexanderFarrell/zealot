package planner

import (
	"time"
	"zealotd/apps/item"
)

func GetItemsOnDate(date time.Time, accountID int) ([]item.Item, error) {
	return item.GetItemsByAttribute("Date", date, "value_date", accountID)
}

func GetItemsOnWeek(week string, accountID int) ([]item.Item, error) {
	return item.GetItemsByAttribute("Week", week, "value_text", accountID)
}