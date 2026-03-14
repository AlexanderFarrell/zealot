package domain

import (
	"time"
)

type CommentID int64

type Comment struct {
	CommentID CommentID `json:"comment_id"`
	Item Item `json:"item"`
	Timestamp time.Time `json:"timestamp"`
	Content string `json:"content"`
}

type NewComment struct {
	Item Item `json:"item"`
	Content string `json:"content"`
}

type UpdateComment struct {
	CommentID CommentID `json:"comment_id"`
	ItemID *ItemID `json:"item_id"`
	Content *string `json:"string"`
}