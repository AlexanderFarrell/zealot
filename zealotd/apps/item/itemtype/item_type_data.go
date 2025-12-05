package itemtype

import (
	"database/sql"
	"strings"
	"zealotd/web"
)

type ItemType struct {
	TypeID int `json:"type_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	RequiredAttributeKeys []string `json:"required_attribute_keys,omitempty"`
}

var updatableFields = map[string]int {
	"name": 0,
	"description": 0,
}

func GetItemTypes(accountID int) ([]ItemType, error) {
	query := `
	select i.type_id, i.name, i.description,
		(select string_agg(k.key, ';') 
		from attribute_kind k
		inner join item_type_attribute_kind_link l on l.attribute_kind_id=k.kind_id
		where l.item_type_id=i.type_id) as required_fields
	from item_type i
	where i.account_id=$1;
	`

	rows, err := web.Database.Query(query, accountID)
	return scanItemTypes(rows, err)
}

func GetItemType(itemTypeID int, accountID int) (*ItemType, error) {
	query := `
	select i.type_id, i.name, i.description,
		(select string_agg(k.key, ';') 
		from attribute_kind k
		inner join item_type_attribute_kind_link l on l.attribute_kind_id=k.kind_id
		where l.item_type_id=i.type_id) as required_fields
	from item_type i
	where i.type_id=$1
	where i.account_id=$2
	limit 1;
	`

	rows, err := web.Database.Query(query, itemTypeID, accountID)
	types, err := scanItemTypes(rows, err)
	if err != nil {
		return nil, err
	}
	if len(types) == 0 {
		return nil, nil
	}
	return &types[0], nil
}

func AddItemType(itemType ItemType, accountID int) error {
	query := `
	insert into item_type (name, description, account_id)
	values ($1, $2, $3);
	`

	_, err := web.Database.Exec(query, itemType.Name, itemType.Description, accountID)
	return err
}

func RemoveItemType(itemTypeID int, accountID int) error {
	query := `
	delete from item_type (name, description, account_id)
	where type_id=$1
	and account_id=$2;
	`

	_, err := web.Database.Exec(query, itemTypeID, accountID)
	return err
}

func scanItemTypes(rows *sql.Rows, err error) ([]ItemType, error) {
	if err != nil {
		return nil, err
	}
	var itemTypes []ItemType = make([]ItemType, 0)

	for rows.Next() {
		var itemType ItemType
		var requiredFieldsStr sql.NullString
		err := rows.Scan(&itemType.TypeID, &itemType.Name, &itemType.Description, &requiredFieldsStr)
		if requiredFieldsStr.Valid {
			itemType.RequiredAttributeKeys = strings.Split(requiredFieldsStr.String, ";")
		} else {
			itemType.RequiredAttributeKeys = make([]string, 0)
		}
		if err != nil {
			return nil, err
		}
		itemTypes = append(itemTypes, itemType)
	}
	return itemTypes, nil
}