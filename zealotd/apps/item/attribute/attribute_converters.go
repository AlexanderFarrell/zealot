package attribute

import (
	"encoding/json"
	"fmt"
	"reflect"
	"strconv"
	"strings"
	"time"
	// "encoding/json"
)



func prepareAttrValueFromKind(kind *AttributeKind, raw interface{}) (column string, dbVal interface{}, err error) {
	switch kind.BaseType {
	case "integer":
		column = "value_int"
		v, err := toInt64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid integer value for %s: %v", kind.Key, &err)
		}
		return column, v, nil
	case "decimal":
		column = "value_num"
		v, err := toFloat64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid decimal value for %s: %v", kind.Key, &err)
		}
		return column, v, nil
	case "text":
		column = "value_text"
		v, err := toString(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid text value for %s: %v", kind.Key, err)
		}
		return column, v, nil
	case "date":
		column = "value_date"
		v, err := toTime(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid date value for %s: %v", kind.Key, err)
		}
		return column, v, nil
	case "week":
		column = "value_int"
		weekCode, err := ToWeekCode(raw)
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
		if err := validateDropdownValue(kind, v); err != nil {
			return "", nil, err
		}
		return column, v, nil
	case "boolean":
		column = "value_int"
		v, err := toBoolInt(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid boolean value for %s: %w", kind.Key, err)
		}
		return column, v, nil
	case "item":
		column = "value_item_id"
		v, err := toInt64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid item id value for %s: %v", kind.Key, err)
		}
		return column, v, nil
	default:
		return "", nil, fmt.Errorf("unsupported base type %q for attribute %s", kind.BaseType, kind.Key)
	}
}

func prepareListItemValue(kind *AttributeKind, raw interface{}) (string, interface{}, error) {
	var cfg AttributeListConfig
	if len(kind.Config) > 0 {
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return "", nil, fmt.Errorf("invalid list config for %s: %w", kind.Key, err)
		}
	}

	switch cfg.ListType {
	case "integer":
		v, err := toInt64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid integer value for %s: %v", kind.Key, &err)
		}
		return "value_int", v, nil
	case "decimal":
		v, err := toFloat64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid decimal value for %s: %v", kind.Key, &err)
		}
		return "value_num", v, nil
	case "text":
		v, err := toString(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid text value for %s: %v", kind.Key, err)
		}
		return "value_text", v, nil
	case "date":
		v, err := toTime(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid date value for %s: %v", kind.Key, err)
		}
		return "value_date", v, nil
	case "week":
		weekCode, err := ToWeekCode(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid week value for %s: %w", kind.Key, err)
		}
		return "value_int", weekCode, nil
	case "boolean":
		v, err := toBoolInt(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid boolean value for %s: %w", kind.Key, err)
		}
		return "value_int", v, nil
	case "item":
		v, err := toInt64(raw)
		if err != nil {
			return "", nil, fmt.Errorf("invalid item id value for %s: %v", kind.Key, err)
		}
		return "value_item_id", v, nil
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
		column = "value_int"
		if b, ok := value.(bool); ok {
			if b {
				value = int64(1)
			} else {
				value = int64(0)
			}
		}
	}
	return column, value
}

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
		if vv {
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

func ToWeekCode(v interface{}) (int, error) {
	switch vv := v.(type) {
	case int:
		// year := vv / 100
		// week := vv % 100
		return vv, nil
	case string:
		if len(vv) != 8 {
			break
		}
		year, err := strconv.Atoi(vv[0:4])
		if err != nil {
			return 0, fmt.Errorf("error parsing year: %w", err)
		}
		week, err := strconv.Atoi(vv[6:8])
		if err != nil {
			return 0, fmt.Errorf("error parsing week: %w", err)
		}
		return year*100 + week, nil
	default:
		return 0, fmt.Errorf("cannot convert %T to week", v)
	}
	return 0, fmt.Errorf("incorrect format, please specify YYYY-W##")
}

func toListAny(raw interface{}) ([]any, error) {
	switch vv := raw.(type) {
	case []any:
		return vv, nil
	case []int:
		out := make([]any, 0, len(vv))
		for _, v := range vv {
			out = append(out, v)
		}
		return out, nil
	case []int64:
		out := make([]any, 0, len(vv))
		for _, v := range vv {
			out = append(out, v)
		}
		return out, nil
	case []string:
		out := make([]any, 0, len(vv))
		for _, v := range vv {
			out = append(out, v)
		}
		return out, nil
	case nil:
		return []any{}, nil
	default:
		// For scalars, do array of 1 element
		return []any{vv}, nil
	}
}

func ResolveColumnAndValue(kind *AttributeKind, raw interface{}) (string, interface{}, error) {
	if kind == nil {
		column, val := inferColumnFromValue(raw)
		return column, val, nil
	}
	if kind.BaseType == "list" {
		return prepareListItemValue(kind, raw)
	}
	return prepareAttrValueFromKind(kind, raw)
}