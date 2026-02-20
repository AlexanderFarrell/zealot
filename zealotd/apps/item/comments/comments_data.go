package comments

import (
	"database/sql"
	"time"
	"zealotd/apps/item"
	"zealotd/web"
)

type CommentEntry struct {
	CommentID int64     `json:"comment_id"`
	Item      item.Item `json:"item"`
	Timestamp time.Time `json:"timestamp"`
	Content   string    `json:"content"`
}

func GetForDay(day time.Time, accountID int) ([]CommentEntry, error) {
	dayDate := time.Date(day.Year(), day.Month(), day.Day(), 0, 0, 0, 0, day.Location())
	dayEnd := dayDate.Add(24 * time.Hour)

	query := `
		select c.comment_id, c.item_id, c.time, c.content,
			i.title, i.content
		from comment c
		join item i on i.item_id = c.item_id
		where i.account_id = $1
			and c.time >= $2
			and c.time < $3
		order by c.time asc
	`

	rows, err := web.Database.Query(query, accountID, dayDate, dayEnd)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	entries := make([]CommentEntry, 0)
	for rows.Next() {
		var (
			entry        CommentEntry
			itemID       int
			commentText  sql.NullString
			itemContent  sql.NullString
		)
		if err := rows.Scan(
			&entry.CommentID,
			&itemID,
			&entry.Timestamp,
			&commentText,
			&entry.Item.Title,
			&itemContent,
		); err != nil {
			return nil, err
		}
		entry.Item.ItemID = itemID
		entry.Content = commentText.String
		entry.Item.Content = itemContent.String
		entries = append(entries, entry)
	}

	return entries, nil
}

func GetForItem(itemID int, accountID int) ([]CommentEntry, error) {
	query := `
		select c.comment_id, c.item_id, c.time, c.content,
			i.title, i.content
		from comment c
		join item i on i.item_id = c.item_id
		where i.account_id = $1
			and c.item_id = $2
		order by c.time asc
	`

	rows, err := web.Database.Query(query, accountID, itemID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	entries := make([]CommentEntry, 0)
	for rows.Next() {
		var (
			entry       CommentEntry
			itemIDValue int
			commentText sql.NullString
			itemContent sql.NullString
		)
		if err := rows.Scan(
			&entry.CommentID,
			&itemIDValue,
			&entry.Timestamp,
			&commentText,
			&entry.Item.Title,
			&itemContent,
		); err != nil {
			return nil, err
		}
		entry.Item.ItemID = itemIDValue
		entry.Content = commentText.String
		entry.Item.Content = itemContent.String
		entries = append(entries, entry)
	}

	return entries, nil
}

func AddEntry(itemID int, timestamp time.Time, content string, accountID int) (int64, error) {
	query := `
		insert into comment (item_id, time, content)
		select i.item_id, $3, $4
		from item i
		where i.item_id = $1
			and i.account_id = $2
		returning comment_id;
	`

	var commentID int64
	err := web.Database.QueryRow(query, itemID, accountID, timestamp, content).Scan(&commentID)
	if err == sql.ErrNoRows {
		return 0, sql.ErrNoRows
	}
	if err != nil {
		return 0, err
	}
	return commentID, nil
}

func UpdateEntry(commentID int64, content string, timestamp *time.Time, accountID int) error {
	if timestamp == nil {
		query := `
			update comment c
			set content = $2,
				last_updated = now()
			from item i
			where c.comment_id = $1
				and c.item_id = i.item_id
				and i.account_id = $3;
		`
		_, err := web.Database.Exec(query, commentID, content, accountID)
		return err
	}

	query := `
		update comment c
		set content = $2,
			time = $3,
			last_updated = now()
		from item i
		where c.comment_id = $1
			and c.item_id = i.item_id
			and i.account_id = $4;
	`
	_, err := web.Database.Exec(query, commentID, content, *timestamp, accountID)
	return err
}

func DeleteEntry(commentID int64, accountID int) error {
	query := `
		delete from comment c
		using item i
		where c.item_id = i.item_id
			and c.comment_id = $1
			and i.account_id = $2;
	`
	_, err := web.Database.Exec(query, commentID, accountID)
	return err
}
