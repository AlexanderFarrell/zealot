package item

import (
	"database/sql"
	"encoding/json"
	"fmt"
	"strconv"
	"strings"
	"time"
	"zealotd/web"
	"reflect"
)

type AttributeKind struct {
	KindID int `json:"kind_id"`
	Key string `json:"key"`
	Description string `json:"description"`
	BaseType string `json:"base_type"`
	Config json.RawMessage `json:"config"`
	IsSystem bool `json:"is_system"`
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

func prepareAttrValueFromKind(kind AttributeKind, raw interface{}) (column string, dbVal interface{}, err error) {
	switch kind.BaseType {
	case "integer":
		column = "value_int"
		v, err := toInt64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid integer value for %s: %w", kind.Key, &err)
		}
		return column, v, nil
	case "decimal":
		column = "value_float"
		v, err := toFloat64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid decimal value for %s: %w", kind.Key, &err)
		}
		return column, v, nil
	case "text":
		column = "value_text"
		v, err := toString(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid text value for %s: %w", kind.Key, err)
		}
		return column, v, nil
	case "date":
		column = "value_date"
		v, err := toTime(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid date value for %s: %w", kind.Key, err)
		}
		return column, v, nil
	case "week":
		column = "value_int"
		weekCode, err := toWeekCode(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid week value for %s: %w", kind.Key, err)
		}
		return column, weekCode, nil
	case "dropdown":
		column = "value_text"
		v, err := toString(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid dropdown value for %s: %w", kind.Key, err)
		}
		return column, v, nil
	case "boolean":
		column = "value_int"
		v, err := toBoolInt(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid boolean value for %s: %w", kind.Key, err)
		}
		return column, v, nil
	default:
		return "", nil, fmt.Errorf("unsupported base type %q for attribute %s", kind.BaseType, kind.Key)
	}
}

func inferColumnFromValue(value interface{}) (string, interface{}) {
	column := "value_text"

	if value == nil {
		return column, nil
	}

	t := reflect.TypeOf(value)
	switch t.Kind() {
	case reflect.Int, reflect.Int8, reflect.Int16, reflect.Int32, reflect.Int64:
		column = "value_int"
	case reflect.String:
		column = "value_text"
	case reflect.Float64, reflect.Float32:
		column = "value_num"
	case reflect.Bool:
		column = "value_bool"
	}
	return column, value
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



// Converters

func toInt64(v interface{}) (int64, error) {
	switch vv := v.(type) {
	case int:
		return int64(vv), nil
	case int8:
		return int64(vv), nil
	case int16:
		return int64(vv), nil
	case int32:
		return int64(vv), nil
	case int64:
		return vv, nil
	case float64:
		return int64(vv), nil
	case string:
		return strconv.ParseInt(vv, 10, 64)
	default:
		return 0, fmt.Errorf("cannot convert %T to int64", v)
	}
}

func toFloat64(v interface{}) (float64, error) {
	switch vv := v.(type) {
	case float32:
		return float64(vv), nil
	case float64:
		return vv, nil
	case int:
		return float64(vv), nil
	case int8:
		return float64(vv), nil
	case int16:
		return float64(vv), nil
	case int32:
		return float64(vv), nil
	case int64:
		return float64(vv), nil
	case string:
		return strconv.ParseFloat(vv, 64)
	default:
		return 0, fmt.Errorf("cannot convert %T to float64", v)
	}
}

func toString(v interface{}) (string, error) {
	switch vv := v.(type) {
	case string:
		return vv, nil
	case []byte:
		return string(vv), nil
	default:
		return fmt.Sprintf("%v", v), nil
	}
}

func toBoolInt(v interface{}) (int, error) {
	switch vv := v.(type) {
	case bool:
		if vv == true {
			return 1, nil
		} else {
			return 0, nil
		}
	case string:
		switch strings.ToLower(strings.TrimSpace(vv)) {
		case "true", "1", "yes", "y", "on":
			return 1, nil
		case "false", "0", "no", "n", "off":
			return 0, nil
		}
		return 0, fmt.Errorf("cannot parse %q as bool", vv)
	case int:
		val, err := toInt64(vv)
		if err != nil {
			return 0, err
		}
		switch val {
		case 1:
			return 1, nil
		case 0:
			return 0, nil
		default:
			return 0, fmt.Errorf("cannot parse invalid int %d as bool: %w", val, err)
		}
	default:
		return 0, fmt.Errorf("cannot convert %T to bool", v)
	}
}

func toTime(v interface{}) (time.Time, error) {
	switch vv := v.(type) {
	case time.Time:
		return vv, nil
	case string:
		t, err := time.Parse(time.RFC3339, vv)
		if err != nil {
			t, err = time.Parse(time.DateOnly, vv)
			if err != nil {
				return t, err
			}
		}
		return t, nil
	default:
		return time.Time{}, fmt.Errorf("cannot convert %T to time.Time", v)
	}
}

func toWeekCode(v interface{}) (int, error) {
	switch vv := v.(type) {
	case string:
		if len(vv) != 8 {
			break
		}
		year, err := strconv.Atoi(vv[0:4])
		if err != nil {
			return 0, fmt.Errorf("error parsing year: %w", err)
		}
		week, err := strconv.Atoi(vv[5:7])
		if err != nil {
			return 0, fmt.Errorf("error parsing week: %w", err)
		}
		return year * 100 + week, nil
	default:
		return 0, fmt.Errorf("cannot convert %T to week", v)
	}
	return 0, fmt.Errorf("incorrect format, please specify YYYY-W##")
}