package attribute

import (
	"database/sql"
	"fmt"
	"time"
	"zealotd/web"
)

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

func SetAttributeForItem(itemID int, accountID int, key string, value interface{}) error {
	// Check if there is an attribute kind defined.
	kind, err := GetAttributeKind(key, accountID)
	if err != nil {
		return err
	}

	var column string
	var preparedValue interface{}

	if kind != nil {
		column, preparedValue, err = prepareAttrValueFromKind(*kind, value)
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

func DeleteAttribute(item_id int, account_id int, key string) error {
	query := `
	delete from attribute
	where item_id = (select item_id from item where item_id=$1 and account_id=$2)
	and key = $3;
	`

	_, err := web.Database.Exec(query, item_id, account_id, key)
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

// Converters

