package item

import (
	"database/sql"
	"fmt"
	"zealotd/web"
)

type Item struct {
	ItemID int `json:"item_id"`
	Title string `json:"title"`
	Content string `json:"content"`
}

func GetItemByID(item_id int, account_id int) (*Item, error) {
	query := `
	select i.item_id, i.title, i.content
	from item i
	where i.item_id = $1
	and i.account_id = $2;
	`
	rows, err := web.Database.Query(query, item_id, account_id)
	return scanRow(rows, err)
}

func GetItemByTitle(title string, account_id int) (*Item, error) {
	query := `
	select i.item_id, i.title, i.content
	from item i
	where i.title = $1
	and i.account_id = $2
	limit 1;
	`

	rows, err := web.Database.Query(query, title, account_id)
	return scanRow(rows, err)
}

func SearchByTitle(title string, account_id int) ([]Item, error) {
	query := `
	select i.item_id, i.title, i.content
	from item i
	where i.title ilike '%' || $1 || '%'
	and i.account_id = $2
	`

	rows, err := web.Database.Query(query, title, account_id)
	return scanRows(rows, err)
}

func AddItem(title string, account_id int) error {
	query := `
	insert into item (title, account_id)
	values ($1, $2);
	`

	_, err := web.Database.Exec(query, title, account_id);
	return err
}

func DeleteItem(item_id int, account_id int) error {
	query := `
	delete from item 
	where item_id = $1
	and account_id = $2;
	`

	_, err := web.Database.Exec(query, item_id, account_id)
	return err
}

func scanRows(rows *sql.Rows, err error) ([]Item, error) {
	if err != nil {
		return nil, err
	}
	items := make([]Item, 0)

	for rows.Next() {
		item := Item{}
		err = rows.Scan(&item.ItemID, &item.Title, &item.Content)
		if err != nil {
			fmt.Printf("Error scanning item %v\n", err)
			continue
		}
		items = append(items, item)
	}
	return items, nil
}

func scanRow(rows *sql.Rows, err error) (*Item, error) {
	items, err := scanRows(rows, err)
	if err != nil {
		return nil, nil
	}
	if len(items) > 1 {
		return nil, fmt.Errorf("SQL query returned more than one item")
	} else if len(items) == 1 {
		// Make a copy
		first := items[0]
		return &first, nil
	}
	return nil, nil
}