package item

type ItemType struct {
	TypeID int `json:"type_id"`
	Name string `json:"name"`
	Description string `json:"description"`
	Instructions ItemTypeInstructions `json:"instructions"`
}

type ItemTypeInstructions struct {
	RequiredFields []string `json:"required_fields"`
}

func item_type_add() {
	
}

func item_type_remove() {

}

func get_item_types() {
	
}

func get_item_types_for_item(item_id int, account_id int) {

}