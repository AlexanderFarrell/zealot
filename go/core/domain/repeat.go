package domain

import "time"

type RepeatStatus string
const (
	RepeatStatusNotComplete RepeatStatus = "Not Complete"
	RepeatStatusComplete RepeatStatus = "Complete"
	RepeatStatusNote RepeatStatus = "Note"
	RepeatStatusSkipped RepeatStatus = "Skipped"
)

type RepeatEntry struct {
	Status RepeatStatus `json:"status"`
	Item Item `json:"item"`
	Date time.Time `json:"date"`
	Comment string `json:"comment"`
}

type UpdateRepeatEntry struct {
	// Required
	Date time.Time `json:"date"`

	Status *RepeatStatus `json:"status"`
	ItemID *ItemID `json:"item_id"`
	Comment *string `json:"comment"`	
}