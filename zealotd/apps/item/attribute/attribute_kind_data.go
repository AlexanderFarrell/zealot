package attribute

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"zealotd/web"
)

type AttributeKind struct {
	KindID      int             `json:"kind_id"`
	Key         string          `json:"key"`
	Description string          `json:"description"`
	BaseType    string          `json:"base_type"`
	Config      json.RawMessage `json:"config"`
	IsSystem    bool            `json:"is_system"`
}

var updatableFieldsAttrKind = map[string]int{
	"key":         0,
	"description": 0,
	"base_type":   0,
	"config":      0,
}

func GetAttributeKind(key string, accountID int) (*AttributeKind, error) {
	query := `
	select kind_id, key, description, base_type, config, is_system
	from attribute_kind
	where key=$1
	and (account_id = $2 or is_system)
	limit 1;
	`
	rows, err := web.Database.Query(query, key, accountID)
	if err != nil {
		return nil, nil
	}

	kinds, err := scanAttributeKinds(rows, err)
	if err != nil {
		return nil, err
	}
	if len(kinds) == 0 {
		return nil, nil
	}

	return &kinds[0], nil
}

func GetAllAttributeKinds(accountID int) ([]AttributeKind, error) {
	query := `
	select kind_id, key, description, base_type, config, is_system
	from attribute_kind
	where (account_id=$1 or is_system);
	`

	rows, err := web.Database.Query(query, accountID)
	return scanAttributeKinds(rows, err)
}

func AddAttributeKind(kind *AttributeKind, accountID int) error {
	query := `
	insert into attribute_kind (account_id, key, description, base_type, config, is_system)
	values ($1, $2, $3, $4, $5, false);
	`

	_, err := web.Database.Exec(query, accountID, kind.Key, kind.Description, kind.BaseType,
		kind.Config)

	if err != nil {
		return fmt.Errorf("error adding attribute kind: %w", err)
	}
	return nil
}

func DeleteAttributeKind(kindID int, accountID int) error {
	query := `
	delete from attribute_kind
	where account_id = $1
	and kind_id = $2
	and is_system=false;
	`

	_, err := web.Database.Exec(query, accountID, kindID)
	if err != nil {
		return fmt.Errorf("error removing attribute kind: %w", err)
	}
	return nil
}

func scanAttributeKinds(rows *sql.Rows, err error) ([]AttributeKind, error) {
	if err != nil {
		return nil, err
	}
	var attributeKinds []AttributeKind = make([]AttributeKind, 0)
	for rows.Next() {
		var kind AttributeKind
		if err := rows.Scan(&kind.KindID, &kind.Key, &kind.Description, &kind.BaseType,
			&kind.Config, &kind.IsSystem); err != nil {
			return nil, err
		}
		attributeKinds = append(attributeKinds, kind)
	}
	return attributeKinds, nil
}

