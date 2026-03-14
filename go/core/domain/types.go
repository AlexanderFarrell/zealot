package domain

type TypeID int64

type ItemType struct {
	TypeID TypeID `json:"type_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	IsSystem bool `json:"is_system"`
	RequiredAttributes []string `json:"required_attributes"`
}

type NewItemType struct {
	Name string `json:"name"`
	Description string `json:"description"`
	RequiredAttributes []string `json:"required_attributes"`
}