package tracker

import (
	"database/sql"
	"fmt"
	"time"
	"zealotd/apps/item"
	"zealotd/web"
)

type TrackerEntry struct {
	TrackerID int64     `json:"tracker_id"`
	Item      item.Item `json:"item"`
	Timestamp time.Time `json:"timestamp"`
	Level     int       `json:"level"`
	Comment   string    `json:"comment"`
}

func GetForDay(day time.Time, accountID int) ([]TrackerEntry, error) {
	dayDate := time.Date(day.Year(), day.Month(), day.Day(), 0, 0, 0, 0, day.Location())
	dayEnd := dayDate.Add(24 * time.Hour)

	query := `
		select te.tracker_id, te.item_id, te.timestamp, te.level, te.comment,
			i.title, i.content
		from tracker_entry te
		join item i on i.item_id = te.item_id
		join item_item_type_link l on l.item_id = i.item_id
		join item_type t on t.type_id = l.type_id
		where i.account_id = $1
			and t.name = 'Tracker'
			and (t.account_id = $1 or t.account_id is null)
			and te.timestamp >= $2
			and te.timestamp < $3
		order by te.timestamp asc
	`

	rows, err := web.Database.Query(query, accountID, dayDate, dayEnd)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	entries := make([]TrackerEntry, 0)
	for rows.Next() {
		var (
			entry TrackerEntry
			itemID int
			comment sql.NullString
		)
		if err := rows.Scan(
			&entry.TrackerID,
			&itemID,
			&entry.Timestamp,
			&entry.Level,
			&comment,
			&entry.Item.Title,
			&entry.Item.Content,
		); err != nil {
			return nil, err
		}
		entry.Item.ItemID = itemID
		entry.Comment = comment.String
		entries = append(entries, entry)
	}

	return entries, nil
}

func AddEntry(itemID int, timestamp time.Time, level int, comment string, accountID int) (int64, error) {
	if level < 1 || level > 10 {
		return 0, fmt.Errorf("level must be between 1 and 10")
	}

	query := `
		insert into tracker_entry (item_id, timestamp, level, comment)
		select i.item_id, $3, $4, $5
		from item i
		join item_item_type_link l on l.item_id = i.item_id
		join item_type t on t.type_id = l.type_id
		where i.item_id = $1
			and i.account_id = $2
			and t.name = 'Tracker'
			and (t.account_id = $2 or t.account_id is null)
		returning tracker_id;
	`

	var trackerID int64
	err := web.Database.QueryRow(query, itemID, accountID, timestamp, level, comment).Scan(&trackerID)
	if err == sql.ErrNoRows {
		return 0, fmt.Errorf("tracker item not found")
	}
	if err != nil {
		return 0, err
	}
	return trackerID, nil
}

func DeleteEntry(trackerID int64, accountID int) error {
	query := `
		delete from tracker_entry te
		using item i
		where te.item_id = i.item_id
			and te.tracker_id = $1
			and i.account_id = $2;
	`
	_, err := web.Database.Exec(query, trackerID, accountID)
	return err
}
