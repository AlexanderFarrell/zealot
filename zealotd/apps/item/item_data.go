package item

import (
	"database/sql"
	"fmt"
	"zealotd/apps/item/attribute"
	"zealotd/apps/item/itemtype"
	"zealotd/web"

	"github.com/lib/pq"
)

type Item struct {
	ItemID     int                 `json:"item_id"`
	Title      string              `json:"title"`
	Content    string              `json:"content"`
	Attributes map[string]any      `json:"attributes"`
	Types      []itemtype.ItemType `json:"types"`
}

const (
	ItemQuery = `select i.item_id, i.title, i.content from item i `
)

var (
	acceptableAttributeColumns = map[string]int{
		"value_int":  0,
		"value_date": 0,
		"value_text": 0,
		"value_num":  0,
		"value_item_id": 0,
	}
)

func GetItemByID(item_id int, account_id int) (*Item, error) {
	query := fmt.Sprintf(`%s
	where i.item_id = $1
	and i.account_id = $2;
	`, ItemQuery)
	rows, err := web.Database.Query(query, item_id, account_id)
	return addAttrsTypesToItem(rows, err, account_id)
}

func GetItemByTitle(title string, account_id int) (*Item, error) {
	query := fmt.Sprintf(`%s
	where i.title = $1
	and i.account_id = $2
	limit 1;
	`, ItemQuery)

	rows, err := web.Database.Query(query, title, account_id)
	return addAttrsTypesToItem(rows, err, account_id)
}

func GetItemsByAttribute(key string, value interface{}, valueCol string, accountID int) ([]Item, error) {
	if _, ok := acceptableAttributeColumns[valueCol]; !ok {
		return nil, fmt.Errorf("invalid column passed")
	}

	query := fmt.Sprintf(`%s
	where i.account_id=$1
	and (
		exists (
			select 1
			from attribute a
			where a.item_id = i.item_id
			  and a.key = $2
			  and a.%s  = $3
		)
		or exists (
			select 1
			from attribute_list_value alv
			where alv.item_id = i.item_id
			and alv.key = $2
			and alv.%s = $3
		)
	);
	`, ItemQuery, key, valueCol)

	rows, err := web.Database.Query(query, accountID, key, value)
	return addAttrsTypesToItems(rows, err, accountID)
}

// func GetItemsByAttributes(keys []string, values []interface{}, valueCol string, accountID int) ([]Item, error) {
// 	if _, ok := acceptableAttributeColumns[valueCol]; !ok {
// 		return nil, fmt.Errorf("invalid column passed")
// 	}

// }

func addAttrsTypesToItem(rows *sql.Rows, err error, accountID int) (*Item, error) {
	item, err := ScanRow(rows, err)
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

	if item.Types == nil {
		item.Types = make([]itemtype.ItemType, 0)
	}

	return item, nil
}

func addAttrsTypesToItems(rows *sql.Rows, err error, accountID int) ([]Item, error) {
	items, err := ScanRows(rows, err)
	if err != nil {
		return nil, err
	}

	ids := make([]int, len(items))
	index := make(map[int]*Item, len(items))
	for i := range items {
		ids[i] = items[i].ItemID
		index[items[i].ItemID] = &items[i]
		items[i].Attributes = make(map[string]any)
		items[i].Types = make([]itemtype.ItemType, 0)
	}

	// Attributes
	rows, err = web.Database.Query(`
		select item_id, key, value_int, value_date, value_text, value_num
		from attribute
		where item_id = ANY($1)
	`, pq.Array(ids))

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var (
			itemID int
			key    string
			vInt   sql.NullInt64
			vDate  sql.NullTime
			vText  sql.NullString
			vNum   sql.NullFloat64
			vItem  sql.NullInt64
		)
		if err := rows.Scan(&itemID, &key, &vInt, &vDate, &vText, &vNum, &vItem); err != nil {
			return nil, err
		}

		it := index[itemID]
		if it == nil {
			continue
		}

		switch {
		case vInt.Valid:
			it.Attributes[key] = vInt.Int64
		case vDate.Valid:
			it.Attributes[key] = vDate.Time
		case vText.Valid:
			it.Attributes[key] = vText.String
		case vNum.Valid:
			it.Attributes[key] = vNum.Float64
		case vItem.Valid:
			it.Attributes[key] = vItem.Int64
		}
	}

	// Attributes (list)
	rows, err = web.Database.Query(`
	select item_id, key, value_int, value_date, value_text, value_num, value_item_id, ordinal
	from attribute_list_value
	where item_id = ANY($1)
	order by item_id, key, ordinal
	`, pq.Array(ids))

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var (
			itemID int
			key    string
			vInt   sql.NullInt64
			vDate  sql.NullTime
			vText  sql.NullString
			vNum   sql.NullFloat64
			vItem  sql.NullInt64
			ordinal int
		)
		if err := rows.Scan(&itemID, &key, &vInt, &vDate, &vText, &vNum, &vItem, &ordinal); err != nil {
			return nil, err
		}

		it := index[itemID]
		if it == nil {
			continue
		}

		var v any
		switch {
		case vInt.Valid:
			v = vInt.Int64
		case vDate.Valid:
			v = vDate.Time
		case vText.Valid:
			v = vText.String
		case vNum.Valid:
			v = vNum.Float64
		case vItem.Valid:
			v = vItem.Int64
		}

		existing, ok := it.Attributes[key]
		if !ok {
			it.Attributes[key] = []any{v}
			continue
		}

		if list, ok := existing.([]any); ok {
			it.Attributes[key] = append(list, v)
		} else {
			it.Attributes[key] = []any{existing, v}
		}
	}

	// Types
	rows, err = web.Database.Query(`
		select l.item_id, t.type_id, t.name, t.description
		from item_item_type_link l
		join item_type t on l.type_id = t.type_id
		where l.item_id = ANY($1)
		  and (t.account_id = $2 or t.account_id is null);
	`, pq.Array(ids), accountID)

	if err != nil {
		return nil, err
	}
	defer rows.Close()

	for rows.Next() {
		var itID int
		var t itemtype.ItemType
		if err := rows.Scan(&itID, &t.TypeID, &t.Name, &t.Description); err != nil {
			return nil, err
		}
		it := index[itID]
		if it == nil {
			continue
		}
		it.Types = append(it.Types, t)
	}
	return items, nil
}

func SearchByTitle(title string, account_id int) ([]Item, error) {
	query := fmt.Sprintf(`%s
	where i.title ilike '%%' || $1 || '%%'
	and i.account_id = $2
	`, ItemQuery)

	rows, err := web.Database.Query(query, title, account_id)
	return addAttrsTypesToItems(rows, err, account_id)
}

func AddItem(title string, account_id int) error {
	query := `
	insert into item (title, account_id)
	values ($1, $2);
	`

	_, err := web.Database.Exec(query, title, account_id)
	return err
}

func AddItem2(item *Item, account_id int) error {
	query := `
	insert into item(title, account_id)
	values ($1, $2)
	returning item_id;
	`
	err := web.Database.QueryRow(query, item.Title, account_id).Scan(&item.ItemID)
	if err != nil {
		return err
	}

	// Add attributes
	for attr_key, attr_value := range item.Attributes {
		if err := attribute.SetAttributeForItem(item.ItemID, account_id, attr_key, attr_value); err != nil {
			return err
		}
	}

	return nil
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

	// Verify that all required fields are there
	keys, err := attribute.GetAttributeKeysForItem(itemID, accountID)
	if err != nil {
		return fmt.Errorf("error verifying required attributes for item: %d", itemID)
	}

	for _, key := range t.RequiredAttributeKeys {
		if !keys[key] {
			return fmt.Errorf("item does not adhere to required attribute %s", key)
		}
	}

	// Assign the item type to the item
	// We already verified that both are owned or allowed by the account.
	query := `
	insert into item_item_type_link (item_id, type_id)
	values ($1, $2);
	`
	_, err = web.Database.Exec(query, itemID, t.TypeID)
	if err != nil {
		fmt.Printf("Error assigning item to item type: %v\n", err)
		return fmt.Errorf("server error assigning item type to item")
	} else {
		return nil
	}
}

func UnassignItemType(itemID int, typeName string, accountID int) error {
	query := `
	delete from item_item_type_link
	where item_id=(select item_id from item where item_id=$1 and account_id=$2)
	and type_id=(select type_id from item_type where name=$3 and (
		account_id=$2 or account_id is null
	));
	`

	_, err := web.Database.Exec(query, itemID, accountID, typeName)
	if err != nil {
		fmt.Printf("Error unassigning item type from item: %v\n", err)
		return fmt.Errorf("server error unassigning item type from item")
	} else {
		return nil
	}
}

func ScanRows(rows *sql.Rows, err error) ([]Item, error) {
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

func ScanRow(rows *sql.Rows, err error) (*Item, error) {
	items, err := ScanRows(rows, err)
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
