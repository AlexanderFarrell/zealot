package item

import (
	"database/sql"
	"fmt"
	"zealotd/apps/item/attribute"
	"zealotd/apps/item/itemtype"
	"zealotd/web"
)

type Item struct {
	ItemID int `json:"item_id"`
	Title string `json:"title"`
	Content string `json:"content"`
	Attributes map[string]any `json:"attributes"`
	Types []itemtype.ItemType `json:"types"`
}

func GetItemByID(item_id int, account_id int) (*Item, error) {
	attrs, err := attribute.GetAttributesForItem(item_id, account_id)
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
	return addAttrsTypesToItem(rows, err, account_id)
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
	return addAttrsTypesToItem(rows, err, account_id)
}

func addAttrsTypesToItem(rows *sql.Rows, err error, accountID int) (*Item, error) {
	item, err := scanRow(rows, err)
	if err != nil || item == nil {
		return nil, err
	}

	attrs, err := attribute.GetAttributesForItem(item.ItemID, accountID)
	if err != nil {
		return nil, err
	}

	t, err := itemtype.GetItemTypesForItem(item.ItemID, accountID)
	if err != nil {
		return nil, err
	}

	item.Attributes = attrs
	item.Types = t
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

func AssignItemType(itemID int, typeName string, accountID int) error {
	// Enforce Required Fields
	// Get the item type requested
	t, err := itemtype.GetItemTypeByName(typeName, accountID)
	if err != nil {
		fmt.Printf("error getting item type of name %s: %v", typeName, err)
		return fmt.Errorf("error getting item type")
	}

	// Get the item requested
	item, err := GetItemByID(itemID, accountID)
	if err != nil {
		fmt.Printf("Error getting item of id %d: %v\n", itemID, err)
		return fmt.Errorf("error finding item")
	}

	// Verify that all required fields are there
	for _, key := range t.RequiredAttributeKeys {
		found := false
		for attrKey, _ := range item.Attributes {
			if attrKey == key {
				found = true
				break
			}
		}
		if !found {
			return fmt.Errorf("item does not adhere to required attribute %s", key)
		}
	}

	// Assign the item type to the item
	query := `
	select 
	`
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