package core

import (
	"./repo"
)

type Service struct {
	Attributes repo.AttributeKindRepository
	Comments repo.CommentRepository
	Item repo.ItemRepository
	Media repo.MediaRepository
	Repeat repo.RepeatRepository
	Rules repo.RulesRepository
	Types repo.TypeRepository
}

