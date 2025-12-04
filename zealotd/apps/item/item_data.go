package item

import (
	"database/sql"
	"fmt"
	"time"
	"zealotd/web"
)

type Item struct {
	ItemID int `json:"item_id"`
	Title string `json:"title"`
	Content string `json:"content"`
	Attributes map[string]any `json:"attributes"`
}

func GetItemByID(item_id int, account_id int) (*Item, error) {
	attrs, err := GetAttributesForItem(item_id, account_id)
	if err != nil {
		return nil, err
	}
	
	query := `
	select i.item_id, i.title, i.content
	from item i
	where i.item_id = $1
	and i.account_id = $2;
	`
	rows, err := web.Database.Query(query, item_id, account_id)
	item, err := scanRow(rows, err)
	if err != nil || item == nil {
		return nil, err
	}
	item.Attributes = attrs
	return item, nil
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
	item, err := scanRow(rows, err)
	if err != nil || item == nil {
		return nil, err
	}

	attrs, err := GetAttributesForItem(item.ItemID, account_id)
	if err != nil {
		return nil, err
	}
	item.Attributes = attrs
	return item, nil
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

func AddItemByUsername(title string, username string) error {
	query := `
	insert into item (title, account_id)
	values ($1, (select account_id from account where username=$2));
	`

	_, err := web.Database.Exec(query, title, username)
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


// -- Attribute and Type Engine

// Attributes

// func SetAttributeForItem(item_id int, account_id int, key string, value interface{}) error {
// 	column := "value_text"
// 	t := reflect.TypeOf(value)
// 	switch t.Kind() {
// 	case reflect.Int:
// 		column = "value_int"
// 	case reflect.String:
// 		column = "value_text"
// 	case reflect.Float64:
// 		column = "value_num"
// 	case reflect.Bool:
// 		column = "value_bool"
// 	}

// 	// No chance of SQL injection, just be sure to ensure column value is controlled.
// 	query := fmt.Sprintf(`
// 	insert into attribute (item_id, key, %s)
// 	values (
// 	(select item_id from item i where i.item_id=$1 and i.account_id=$2),
// 	$3,
// 	$4)
// 	on conflict (item_id, key)
// 	do update set
// 		%s = $4;
// 	`, column, column)

// 	_, err := web.Database.Exec(query, item_id, account_id, key, value)

// 	return err
// }

func GetAttributesForItem(item_id int, account_id int) (map[string]any, error) {
	query := `
	SELECT key, value_text, value_num, value_int, value_date
	FROM attribute
	WHERE item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2);
	`
	rows, err := web.Database.Query(query, item_id, account_id)
	return scanAttributes(rows, err)
}

func GetAttributeForItem(item_id int, account_id int, key string) (any, error) {
	query := `
	SELECT key, value_text, value_num, value_int, value_date
	FROM attribute
	WHERE item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2)
	AND key=$3;
	`

	rows, err := web.Database.Query(query, item_id, account_id)
	attrs, err := scanAttributes(rows, err)
	if err != nil {
		return nil, err
	}

	for _, v := range attrs {
		return v, nil
	}
	return nil, fmt.Errorf("not found")
}

func DeleteAttribute(item_id int, account_id int, key string) error {
	query := `
	delete from attribute
	where item_id = (select item_id from item where item_id=$1 and account_id=$2)
	and key = $3;
	`

	_, err := web.Database.Exec(query, item_id, account_id, key)
	return err
}

func RenameAttribute(item_id int, account_id int, old_key string, new_key string) error {
	query := `
	UPDATE attribute
	SET key=$1
	WHERE key=$2
	AND item_id = (select item_id from item where item_id=$3 and account_id=$4);
	`

	_, err := web.Database.Exec(query, new_key, old_key, item_id, account_id)
	return err
}

func scanAttributes(rows *sql.Rows, err error) (map[string]any, error) {
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	attrs := make(map[string]any)

	for rows.Next() {
		var (
			key string
			valueText *string
			valueNum *float64
			valueInt *int64
			valueDate *time.Time
		)

		if err := rows.Scan(&key, &valueText, &valueNum, &valueInt, &valueDate); err != nil {
			return nil, err
		}		
		switch {
		case valueText != nil:
			attrs[key] = *valueText
		case valueNum != nil:
			attrs[key] = *valueNum
		case valueInt != nil:
			attrs[key] = *valueInt
		case valueDate != nil:
			attrs[key] = valueDate.Format(time.RFC3339)
		}
	}

	return attrs, nil
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