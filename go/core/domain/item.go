package domain

type ItemID int64

type Item struct {
	ItemID ItemID  `json:"item_id"`
	Title string `json:"title"`
	Content string `json:"content"`
	Attributes map[string]Attribute `json:"attributes"`
	Types []ItemType `json:"types"`
}

type NewItem struct {
	Title string `json:"title"`
	Content *string `json:"content"`
	Attributes map[string]Attribute `json:"attributes"`
	Types []ItemType `json:"types"`	
}

type UpdateItem struct {
	ItemID ItemID  `json:"item_id"`
	Title *string `json:"title"`
	Content *string `json:"content"`
	Attributes map[string]Attribute `json:"attributes"`
	Types []ItemType `json:"types"`
}