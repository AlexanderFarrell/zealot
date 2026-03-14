package repo

import (
	"context"
	"zealot-core/domain"
)

type CommentRepository interface {
	Add(ctx context.Context, owner *domain.Account, cmd domain.NewComment) (domain.Comment, error)
	Update(ctx context.Context, owner *domain.Account, cmd domain.UpdateComment) error
	Delete(ctx context.Context, owner *domain.Account, id domain.CommentID) error
	ListForItem(ctx context.Context, owner *domain.Account, itemID domain.ItemID) ([]domain.Comment, error)
	ListForDay(ctx context.Context, owner *domain.Account, day domain.Day) ([]domain.Comment, error)
}