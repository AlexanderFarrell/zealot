package repo

import (
	"context"
	"time"
	"zealot-core/domain"
)

type RepeatRepository interface {
	GetOnDate(context context.Context, owner *domain.Account, day time.Time) ([]domain.RepeatEntry, error)
	SetRepeatStatus(context context.Context, owner *domain.Account, updates *domain.UpdateRepeatEntry) error

}