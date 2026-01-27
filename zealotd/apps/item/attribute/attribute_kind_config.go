package attribute

import (
	"encoding/json"
	"fmt"
	"regexp"
)

type NumberConfig struct {
	Min *float64 `json:"min"`
	Max *float64 `json:"max"`
}

type IntConfig struct {
	Min *int64 `json:"min"`
	Max *int64 `json:"max"`
}

type TextConfig struct {
	MinLen *int `json:"min_len"`
	MaxLen *int `json:"max_len"`
	Pattern *string `json:"pattern"`
}

type DropdownConfig struct {
	Values []string `json:"values"`
}

type AttributeListConfig struct {
	ListType string `json:"list_type"`
}

func validateNumberBounds(kind AttributeKind, v float64) error {
	if len(kind.Config) == 0 {
		return nil
	}
	var cfg NumberConfig
	if err := json.Unmarshal(kind.Config, &cfg); err != nil {
		return fmt.Errorf("invalid number config for %s: %w", kind.Key, err)
	}
	if cfg.Min != nil && v < *cfg.Min {
		return fmt.Errorf("%s below min", kind.Key)
	}
	if cfg.Max != nil && v > *cfg.Max {
		return fmt.Errorf("%s above max", kind.Key)
	}
	return nil
}

func validateIntBounds(kind AttributeKind, v int64) error {
	if len(kind.Config) == 0 {
		return nil
	}
	var cfg IntConfig
	if err := json.Unmarshal(kind.Config, &cfg); err != nil {
		return fmt.Errorf("invalid int config for %s: %w", kind.Key, err)
	}
	if cfg.Min != nil && v < *cfg.Min {
		return fmt.Errorf("%s below min", kind.Key)
	}
	if cfg.Max != nil && v > *cfg.Max {
		return fmt.Errorf("%s above max", kind.Key)
	}
	return nil
}

func validateTextConfig(kind *AttributeKind, v string) error {
	if len(kind.Config) == 0 {
		return nil
	}
	var cfg TextConfig
	if err := json.Unmarshal(kind.Config, &cfg); err != nil {
		return fmt.Errorf("invalid text config for %s: %w", kind.Key, err)
	}
	if cfg.MinLen != nil && len(v) < *cfg.MinLen {
		return fmt.Errorf("%s below min length", kind.Key)
	}
	if cfg.MaxLen != nil && len(v) > *cfg.MaxLen {
		return fmt.Errorf("%s above max length", kind.Key)
	}
	if cfg.Pattern != nil {
		re, err := regexp.Compile(fullMatchPattern(*cfg.Pattern))
		if err != nil {
			return fmt.Errorf("invalid text pattern for %s: %w", kind.Key, err)
		}
		if !re.MatchString(v) {
			return fmt.Errorf("%s does not match pattern", kind.Key)
		}
	}
	return nil
}

func validateDropdownValue(kind* AttributeKind, v string) error {
	if len(kind.Config) == 0 {
		return nil
	}
	var cfg DropdownConfig
	if err := json.Unmarshal(kind.Config, &cfg); err != nil {
		return fmt.Errorf("invalid dropdown config for %s: %w", kind.Key, err)
	}
	for _, check := range cfg.Values {
		if v == check {
			return nil
		}
	}
	return fmt.Errorf("not within one of the required values for %s", kind.Key)
}

func validateAttributeKindConfig(kind *AttributeKind) error {
	switch kind.BaseType {
	case "integer":
		var cfg IntConfig
		if len(kind.Config) == 0 {
			return nil
		}
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return err
		}
		if cfg.Min != nil && cfg.Max != nil && *cfg.Min > *cfg.Max {
			return fmt.Errorf("min greater than max")
		}
	case "decimal":
		var cfg NumberConfig
		if len(kind.Config) == 0 {
			return nil
		}
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return err
		}
		if cfg.Min != nil && cfg.Max != nil && *cfg.Min > *cfg.Max {
			return fmt.Errorf("min greater than max")
		}
	case "text":
		var cfg TextConfig
		if len(kind.Config) == 0 {
			return nil
		}
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return err
		}
		if cfg.MinLen != nil && cfg.MaxLen != nil && *cfg.MinLen > *cfg.MaxLen {
			return fmt.Errorf("min_len greater than max_len")
		}
		if cfg.Pattern != nil {
			if _, err := regexp.Compile(fullMatchPattern(*cfg.Pattern)); err != nil {
				return fmt.Errorf("regex does not compile: %v", err)
			}
		}
	case "dropdown":
		var cfg DropdownConfig
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return err
		}
		if len(cfg.Values) == 0 {
			return fmt.Errorf("dropdown requires values")
		}
	case "list":
		var cfg AttributeListConfig
		if err := json.Unmarshal(kind.Config, &cfg); err != nil {
			return err
		}
		if cfg.ListType == "" {
			return fmt.Errorf("list requires list_type")
		}
	}
	return nil
}

func fullMatchPattern(pattern string) string {
	return fmt.Sprintf("^(?:%s)$", pattern)
}
