package itemtype

import (
	"database/sql"
	"fmt"
	"strings"
	"zealotd/web"

	"github.com/lib/pq"
)

const (
	itemTypeQuery = `select i.type_id, i.name, i.description, i.account_id is null as is_system,
		(select string_agg(k.key, ';') 
		from attribute_kind k
		inner join item_type_attribute_kind_link l on l.attribute_kind_id=k.kind_id
		where l.item_type_id=i.type_id) as required_fields
	from item_type i`
)

type ItemType struct {
	TypeID int `json:"type_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	IsSystem bool `json:"is_system"`
	RequiredAttributeKeys []string `json:"required_attribute_keys"`
}

var updatableFields = map[string]int {
	"name": 0,
	"description": 0,
}

func GetItemTypes(accountID int) ([]ItemType, error) {
	query := fmt.Sprintf(`%s
	where (i.account_id=$1 or i.account_id is null);
	`, itemTypeQuery)

	rows, err := web.Database.Query(query, accountID)
	return scanItemTypes(rows, err)
}

func GetItemType(itemTypeID int, accountID int) (*ItemType, error) {
	query := fmt.Sprintf(`%s
	where i.type_id=$1
	and (i.account_id=$2 or i.account_id is null)
	limit 1;
	`, itemTypeQuery)

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
func GetItemTypeByName(typeName string, accountID int) (*ItemType, error) {
	query := fmt.Sprintf(`%s
	where i.name=$1
	and (i.account_id=$2 or i.account_id is null)
	limit 1;
	`, itemTypeQuery)

	rows, err := web.Database.Query(query, typeName, accountID)
	types, err := scanItemTypes(rows, err)
	if err != nil {
		return nil, err
	}
	if len(types) == 0 {
		return nil, nil
	}
	return &types[0], nil
}

// Item to Item Type

func GetItemTypesForItem(itemID int, accountID int) ([]ItemType, error) {
	query := fmt.Sprintf(`%s
	inner join item_item_type_link l on
		l.type_id = i.type_id
	inner join item t on
		t.item_id = l.item_id
	where (i.account_id=$1 or i.account_id is null)
	and t.account_id=$1
	and t.item_id=$2
	`, itemTypeQuery)

	rows, err := web.Database.Query(query, accountID, itemID)
	return scanItemTypes(rows, err)
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

func AddAttributeKindsToItemType(attributeKinds []string, itemTypeKey string, accountID int) error {
	if len(attributeKinds) == 0 {
		fmt.Printf("Warning, no attribute kinds given")
		return nil
	}
	query := `
		insert into item_type_attribute_kind_link (attribute_kind_id, item_type_id)
		select ak.kind_id, it.type_id
		from attribute_kind ak
		join item_type it
		  on it.name=$1
		 and it.account_id=$2
	    where ak.key = ANY($3)
		 on conflict (attribute_kind_id, item_type_id) do nothing;
	`
	_, err := web.Database.Exec(query, itemTypeKey, accountID, pq.Array(attributeKinds))
	return err
}

func RemoveAttributeKindsFromItemType(attributeKinds []string, itemTypeKey string, accountID int) error {
	if len(attributeKinds) == 0 {
		fmt.Printf("Warning, no attribute kinds given")
		return nil
	}
	fmt.Printf("Kinds: %v, Item Type: %s, Account ID: %d", attributeKinds, itemTypeKey, accountID)
	query := `
	delete from item_type_attribute_kind_link ia
	where ia.item_type_id = (
		select it.type_id
		from item_type it
		where it.name = $1
			and it.account_id=$2
	)
	and ia.attribute_kind_id IN (
		select ak.kind_id
		from attribute_kind ak
		where ak.key = ANY($3)
	);`

	_, err := web.Database.Exec(query, itemTypeKey, accountID, pq.Array(attributeKinds))
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
		err := rows.Scan(&itemType.TypeID, &itemType.Name, &itemType.Description, &itemType.IsSystem, &requiredFieldsStr)
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