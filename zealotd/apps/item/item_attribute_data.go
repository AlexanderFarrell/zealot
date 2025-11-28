package item

import (
	"database/sql"
	"fmt"
	"zealotd/web"
)

type AttributeMeta struct {
	AttributeMetaID int `json:"attribute_meta_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	Kind string `json:"kind"`
}

var updatableFieldsAttributeMeta = map[string]int {
	"name": 0,
	"description": 0,
	"kind": 0,
}

func AttributeMetaAdd(attributeMeta AttributeMeta, accountID int) error {
	query := `
	insert into attribute_meta (name, description, kind, account_id)
	values ($1, $2, $3, $4);
	`
	_, err := web.Database.Exec(query, attributeMeta.Name, attributeMeta.Description, 
		attributeMeta.Kind, accountID)
	return err
}

func AttributeMetaRemove(attributeMetaID int, accountID int) error {
	query := `
	delete from attribute_meta
	where attribute_meta_id =
		(select attribute_meta_id from attribute_meta 
		where attribute_meta_id=$1
		and account_id=$2);
	`

	_, err := web.Database.Exec(query, attributeMetaID, accountID)
	return err
}

func GetAttributeMetas(accountID int) ([]AttributeMeta, error) {
	query := `
	select attribute_meta_id, name, description, kind
	from attribute_meta
	where account_id=$1;
	`

	rows, err := web.Database.Query(query, accountID)
	return scanAttributeMetas(rows, err)
}

func scanAttributeMetas(rows *sql.Rows, err error) ([]AttributeMeta, error) {
	if err != nil {
		return nil, err
	}
	attributeMetas := make([]AttributeMeta, 0)

	for (rows.Next()) {
		meta := AttributeMeta{}
		err := rows.Scan(&meta.AttributeMetaID, &meta.Name,
			&meta.Description, &meta.Kind)
		if err != nil {
			return nil, fmt.Errorf("failed to scan row: %v", err)
		}
		attributeMetas = append(attributeMetas, meta)
	}
	return attributeMetas, nil
}