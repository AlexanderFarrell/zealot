package domain

import "encoding/json"

type AttributeKindID int64
type Attribute any
type ListMode string
const (
	ListModeAny  ListMode = "any"
	ListModeAll  ListMode = "all"
	ListModeNone ListMode = "none"
)
type AttributeKindConfig json.RawMessage

type AttributeKind struct {
	KindID AttributeKindID `json:"kind_id"`
	Key string `json:"key"`
	Description string `json:"description"`
	BaseType string `json:"base_type"`
	Config AttributeKindConfig `json:"config"`
	IsSystem bool `json:"is_system"`
}

type NewAttributeKind struct {
	Key string `json:"key"`
	Description string `json:"description"`
	BaseType string `json:"base_type"`
	Config *AttributeKindConfig `json:"config"`
}

type AttributeFilter struct {
	Key string `json:"key"`
	Op string `json:"op"`
	Value interface{} `json:"value"`
	ListMode ListMode `json:"list_mode"` // any|all|none
}

type AttributePair struct {
	Key string `json:"key"`
	Value Attribute `json:"value"`
}