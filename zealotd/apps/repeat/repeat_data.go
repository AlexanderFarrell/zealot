package repeat

import (
	"database/sql"
	"fmt"
	"time"
	"zealotd/apps/item/attribute"
	"zealotd/apps/item/itemtype"
	"zealotd/apps/item"
	"zealotd/web"

	"github.com/lib/pq"
)


// Represents the status of a repeat goal on a specified 
// date or week, or month or year. Maps to 
type RepeatStatusDate struct {
	Status  string    `json:"status"`
	Item    item.Item `json:"item"`
	Date    time.Time `json:"date"`
	Comment string    `json:"comment"`
}

func GetForDay(day time.Time, accountID int) ([]RepeatStatusDate, error) {
	// Figure out the weekday
	weekday := day.Weekday()
	weekdayIndex := int(weekday)
	weekdayPos := weekdayIndex + 1
	dayDate := time.Date(day.Year(), day.Month(), day.Day(), 0, 0, 0, 0, day.Location())

	// Does a database query for the account for
	// any item which has a Schedule attribute kind
	// and which is 1 for the weekday. The text is a 10
	// digit number where the 1st digit is Sunday, second
	// is Monday and so forth. If said digit is 1, then 
	// the repeat is active on that day. 
	query := `
		select i.item_id, i.title, i.content
		from item i
		join item_item_type_link l on l.item_id = i.item_id
		join item_type t on t.type_id = l.type_id
		join attribute sched on sched.item_id = i.item_id
			and sched.key = 'Schedule'
		left join attribute end_date on end_date.item_id = i.item_id
			and end_date.key = 'End Date'
		where i.account_id = $1
			and t.name = 'Repeat'
			and (t.account_id = $1 or t.account_id is null)
			and substring(sched.value_text, $2, 1) = '1'
			and (end_date.value_date is null or end_date.value_date >= $3)
	`

	rows, err := web.Database.Query(query, accountID, weekdayPos, dayDate)
	if err != nil {
		return nil, err
	}
	items, err := item.ScanRows(rows, nil)
	_ = rows.Close()
	if err != nil {
		return nil, err
	}

	for i := range items {
		attrs, err := attribute.GetAttributesForItem(items[i].ItemID, accountID)
		if err != nil {
			return nil, err
		}
		types, err := itemtype.GetItemTypesForItem(items[i].ItemID, accountID)
		if err != nil {
			return nil, err
		}
		items[i].Attributes = attrs
		items[i].Types = types
		if items[i].Types == nil {
			items[i].Types = make([]itemtype.ItemType, 0)
		}
	}

	if len(items) == 0 {
		return []RepeatStatusDate{}, nil
	}

	ids := make([]int, len(items))
	for i := range items {
		ids[i] = items[i].ItemID
	}

	entryRows, err := web.Database.Query(`
		select item_id, status, comment
		from repeat_entry
		where item_id = ANY($1)
			and date = $2
	`, pq.Array(ids), dayDate)
	if err != nil {
		return nil, err
	}
	defer entryRows.Close()

	type entryInfo struct {
		status  string
		comment string
	}
	entries := make(map[int]entryInfo, len(items))

	for entryRows.Next() {
		var (
			itemID int
			status string
			comment sql.NullString
		)
		if err := entryRows.Scan(&itemID, &status, &comment); err != nil {
			return nil, err
		}
		entries[itemID] = entryInfo{
			status:  status,
			comment: comment.String,
		}
	}

	results := make([]RepeatStatusDate, 0, len(items))
	for _, it := range items {
		status := "Not Completed"
		comment := ""
		if entry, ok := entries[it.ItemID]; ok {
			status = entry.status
			comment = entry.comment
		}
		results = append(results, RepeatStatusDate{
			Status:  status,
			Item:    it,
			Date:    dayDate,
			Comment: comment,
		})
	}

	return results, nil
}

func SetRepeatStatus(itemID int, day time.Time, accountID int, status string, comment string) error {
	dayDate := time.Date(day.Year(), day.Month(), day.Day(), 0, 0, 0, 0, day.Location())

	tx, err := web.Database.Begin()
	if err != nil {
		return err
	}

	rollback := func(err error) error {
		_ = tx.Rollback()
		return err
	}

	deleteQuery := `
		delete from repeat_entry re
		using item i
		join item_item_type_link l on l.item_id = i.item_id
		join item_type t on t.type_id = l.type_id
		where re.item_id = i.item_id
			and i.item_id = $1
			and i.account_id = $2
			and t.name = 'Repeat'
			and (t.account_id = $2 or t.account_id is null)
			and re.date = $3
	`

	if _, err := tx.Exec(deleteQuery, itemID, accountID, dayDate); err != nil {
		return rollback(err)
	}

	if status == "Not Completed" {
		return tx.Commit()
	}

	insertQuery := `
		insert into repeat_entry (item_id, date, status, comment)
		select i.item_id, $3, $4, $5
		from item i
		join item_item_type_link l on l.item_id = i.item_id
		join item_type t on t.type_id = l.type_id
		where i.item_id = $1
			and i.account_id = $2
			and t.name = 'Repeat'
			and (t.account_id = $2 or t.account_id is null)
		limit 1
	`

	res, err := tx.Exec(insertQuery, itemID, accountID, dayDate, status, comment)
	if err != nil {
		return rollback(err)
	}

	rows, err := res.RowsAffected()
	if err != nil {
		return rollback(err)
	}
	if rows == 0 {
		return rollback(fmt.Errorf("repeat item not found"))
	}

	return tx.Commit()
}

// func GetForWeek(accountID int) {

// }

// func GetForMonth(accountID int) {

// }

// func GetForYear(accountID int) {

// }
