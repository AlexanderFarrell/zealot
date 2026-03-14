package domain

type SearchQuery struct {
	Term string `json:"term"`
	Page int `json:"page"`
}

type Day string