package main

import (
	"encoding/json"
	"fmt"
	"strings"

	"github.com/mark3labs/mcp-go/mcp"
)

func decodeArg[T any](raw any) (T, error) {
	var value T

	data, err := json.Marshal(raw)
	if err != nil {
		return value, err
	}
	if err := json.Unmarshal(data, &value); err != nil {
		return value, err
	}
	return value, nil
}

func requiredArg[T any](req mcp.CallToolRequest, key string) (T, error) {
	raw, ok := req.Params.Arguments[key]
	if !ok {
		var zero T
		return zero, fmt.Errorf("%s is required", key)
	}
	return decodeArg[T](raw)
}

func optionalArg[T any](req mcp.CallToolRequest, key string) (T, bool, error) {
	raw, ok := req.Params.Arguments[key]
	if !ok {
		var zero T
		return zero, false, nil
	}

	value, err := decodeArg[T](raw)
	return value, true, err
}

func requiredNonEmptyString(req mcp.CallToolRequest, key string) (string, error) {
	value, err := requiredArg[string](req, key)
	if err != nil {
		return "", err
	}
	if strings.TrimSpace(value) == "" {
		return "", fmt.Errorf("%s must not be empty", key)
	}
	return value, nil
}

func collectArgMap(req mcp.CallToolRequest, keys ...string) (map[string]any, error) {
	updates := make(map[string]any)
	for _, key := range keys {
		if raw, ok := req.Params.Arguments[key]; ok {
			updates[key] = raw
		}
	}

	if len(updates) == 0 {
		return nil, fmt.Errorf("at least one of %s is required", strings.Join(keys, ", "))
	}
	return updates, nil
}

// parseFilters converts the raw MCP argument ([]interface{} from JSON) into []AttributeFilter.
func parseFilters(raw any) ([]AttributeFilter, error) {
	filters, err := decodeArg[[]AttributeFilter](raw)
	if err != nil {
		return nil, err
	}
	if len(filters) == 0 {
		return nil, fmt.Errorf("at least one filter is required")
	}
	return filters, nil
}
