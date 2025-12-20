package attribute

import (
	"database/sql"
	"fmt"
	"time"
	"zealotd/web"
)

func GetAttributesForItem(item_id int, account_id int) (map[string]any, error) {
	scalars, err := getScalarAttributesForItem(item_id, account_id)
	if err != nil {
		return nil, err
	}

	lists, err := getListAttributesForItem(item_id, account_id)
	if err != nil {
		return nil, err
	}

	for k, v := range lists {
		scalars[k] = v
	}

	return scalars, nil
}

func GetAttributeForItem(item_id int, account_id int, key string) (any, error) {
	kind, err := GetAttributeKind(key, account_id)
	if err != nil {
		return nil, err
	}

	if kind != nil && kind.BaseType == "list" {
		return getListAttributeForItem(item_id, account_id, key)
	}

	val, err := getScalarAttributeForItem(item_id, account_id, key)
	if err != nil {
		return val, err
	}

	// Allow some fallback to lists
	return getListAttributeForItem(item_id, account_id, key)
}

func GetAttributeKeysForItem(itemID int, accountID int) (map[string]bool, error) {
	query := `
		select key
		from attribute
		where item_id = (select item_id from item where item_id=$1 and account_id=$2)
		union
		select key
		from attribute_list_value
		where item_id = (select item_id from item where item_id=$1 and account_id=$2);
	`
	rows, err := web.Database.Query(query, itemID, accountID)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	keys := make(map[string]bool)
	for rows.Next() {
		var key string
		if err := rows.Scan(&key); err != nil {
			return nil, err
		}
		keys[key] = true
	}
	return keys, nil
}


func getScalarAttributesForItem(item_id int, account_id int) (map[string]any, error) {
	query := `
	select key, value_text, value_num, value_int, value_date, value_item_id
	from attribute
	where item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2);
	`
	rows, err := web.Database.Query(query, item_id, account_id)
	return scanAttributes(rows, err)
}

func getScalarAttributeForItem(item_id int, account_id int, key string) (any, error) {
	query := `
	select key, value_text, value_num, value_int, value_date, value_item_id
	from attribute
	where item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2)
	and key=$3;
	`

	rows, err := web.Database.Query(query, item_id, account_id, key)
	attrs, err := scanAttributes(rows, err)
	if err != nil {
		return nil, err
	}

	for _, v := range attrs {
		return v, nil
	}
	return nil, fmt.Errorf("not found")
}

func getListAttributesForItem(item_id int, account_id int) (map[string][]any, error) {
	query := `
	select key, ordinal, value_text, value_num, value_int, value_date, value_item_id
	from attribute_list_value
	where item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2)
	order by key, ordinal
	`

	rows, err := web.Database.Query(query, item_id, account_id)
	return scanListAttributeValues(rows, err)
}

func getListAttributeForItem(item_id int, account_id int, key string) ([]any, error) {
	query := `
	select key, ordinal, value_text, value_num, value_int, value_date, value_item_id
	from attribute_list_value
	where item_id = (select item_id from item i where i.item_id=$1 and i.account_id=$2)
	order by ordinal;
	`

	rows, err := web.Database.Query(query, item_id, account_id, key)
	values, err := scanListAttributeValues(rows, err)
	if err != nil {
		return nil, err
	}
	if len(values) == 0 {
		return nil, fmt.Errorf("not found")
	}
	return values[key], nil
}

func SetAttributeForItem(itemID int, accountID int, key string, value interface{}) error {
	// Check if there is an attribute kind defined.
	kind, err := GetAttributeKind(key, accountID)
	if err != nil {
		return err
	}
	
	if kind != nil && kind.BaseType == "list" {
		return setListAttributeForItem(itemID, accountID, key, kind, value)
	}

	return setScalarAttributeForItem(itemID, accountID, key, value, kind)
}

func setScalarAttributeForItem(itemID int, accountID int, key string, value interface{}, kind *AttributeKind) error {
	var column string
	var preparedValue interface{}
	var err error

	if kind != nil {
		if kind.BaseType == "list" {
			column, preparedValue, err = prepareListItemValue(kind, value)
		} else {
			column, preparedValue, err = prepareAttrValueFromKind(kind, value)
		}
		if err != nil {
			return err
		}
	} else {
		column, preparedValue = inferColumnFromValue(value)
	}

	query := fmt.Sprintf(`
	insert into attribute (item_id, key, %s)
	values (
		(select item_id from item i where i.item_id=$1 and i.account_id=$2),
		$3,
		$4
	)
		on conflict (item_id, key)
		do update 
		set %s = $4;
	`, column, column)

	_, err = web.Database.Exec(query, itemID, accountID, key, preparedValue)
	return err
}

func setListAttributeForItem(itemID int, accountID int, key string, kind *AttributeKind, value interface{}) error {
	listValues, err := toListAny(value)
	if err != nil {
		return err
	}

	tx, err := web.Database.Begin()
	if err != nil {
		return err
	}

	rollback := func(err error) error {
		_ = tx.Rollback()
		return err
	}

	deleteListQuery := `
		delete from attribute_list_value
		where item_id = (select item_id from item where item_id=$1 and account_id=$2)
		and key = $3;
	`

	if _, err := tx.Exec(deleteListQuery, itemID, accountID, key); err != nil {
		return rollback(err)
	}

	deleteScalarQuery := `
		delete from attribute
		where item_id = (select item_id from item where item_id=$1 and account_id=$2)
		and key=$3;
	`

	if _, err := tx.Exec(deleteScalarQuery, itemID, accountID, key); err != nil {
		return rollback(err)
	}

	for ordinal, raw := range listValues {
		column, preparedValue, err := prepareListItemValue(kind, raw)
		if err != nil {
			return rollback(err)
		}

		insertQuery := fmt.Sprintf(`
		insert into attribute_list_value (item_id, key, ordinal, %s)
		values (
			(select item_id from item i where i.item_id=$1 and i.account_id=$2),
			$3,
			$4,
			$5
		);
		`, column)

		if _, err := tx.Exec(insertQuery, itemID, accountID, key, ordinal, preparedValue); err != nil {
			return rollback(err)
		}
	}

	if err := tx.Commit(); err != nil {
		return err
	}
	return nil
}

func RenameAttribute(item_id int, account_id int, old_key string, new_key string) error {
	query := `
	UPDATE attribute
	SET key=$1
	WHERE key=$2
	AND item_id = (select item_id from item where item_id=$3 and account_id=$4);
	`

	if _, err := web.Database.Exec(query, new_key, old_key, item_id, account_id); err != nil {
		return err
	}

	listQuery := `
	UPDATE attribute_list_value
	SET key=$1
	WHERE key=$2
	AND item_id = (select item_id from item where item_id=$3 and account_id=$4);
	`

	_, err := web.Database.Exec(listQuery, new_key, old_key, item_id, account_id)
	return err
}

func DeleteAttribute(item_id int, account_id int, key string) error {
	query := `
	delete from attribute
	where item_id = (select item_id from item where item_id=$1 and account_id=$2)
	and key = $3;
	`

	if _, err := web.Database.Exec(query, item_id, account_id, key); err != nil {
		return err
	}

	listQuery := `
	delete from attribute_list_value
	where item_id = (select item_id from item where item_id=$1 and account_id=$2)
	and key = $3;
	`

	_, err := web.Database.Exec(listQuery, item_id, account_id, key)
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
			key       string
			valueText *string
			valueNum  *float64
			valueInt  *int64
			valueDate *time.Time
			valueItemID *int64
		)

		if err := rows.Scan(&key, &valueText, &valueNum, &valueInt, &valueDate, &valueItemID); err != nil {
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
		case valueItemID != nil:
			attrs[key] = *valueItemID
		}
	}

	return attrs, nil
}

func scanListAttributeValues(rows *sql.Rows, err error) (map[string][]any, error) {
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	attrs := make(map[string][]any)

	for rows.Next() {
		var (
			key       string
			ordinal int
			valueText *string
			valueNum  *float64
			valueInt  *int64
			valueDate *time.Time
			valueItemID *int64
		)

		if err := rows.Scan(&key, &ordinal, &valueText, &valueNum, &valueInt, &valueDate, &valueItemID); err != nil {
			return nil, err
		}

		var v any
		switch {
		case valueText != nil:
			v = *valueText
		case valueNum != nil:
			v = *valueNum
		case valueInt != nil:
			v = *valueInt
		case valueDate != nil:
			v = valueDate.Format(time.RFC3339)
		case valueItemID != nil:
			v = *valueItemID
		}
		attrs[key] = append(attrs[key], v)
	}
	return attrs, nil
}

// Converters

