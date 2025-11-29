package item

type AttributeMeta struct {
	AttributeMetaID int `json:"attribute_meta_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	Kind string `json:"kind"`
}

type ItemType struct {
	TypeID int `json:"type_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	Instructions ItemTypeInstructions `json:"instructions"`
}

type ItemTypeInstructions struct {
	RequiredFields []string `json:"required_fields"`
}
