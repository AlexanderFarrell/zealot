package repo

import (
	"context"
	"zealot-core/domain"
	// "github.com/alexanderfarrell/zealot/go/core/auth"
	// "github.com/alexanderfarrell/zealot/go/core/event"
	// "github.com/alexanderfarrell/zealot/go/core/item/domain"
)

type ItemRepository interface {
	Create(ctx context.Context, owner *domain.Account, draft domain.NewItem) (domain.Item, error)
	Update(ctx context.Context, owner *domain.Account, cmd domain.UpdateItem) (domain.Item, error)
	Delete(ctx context.Context, owner *domain.Account, id domain.ItemID) error
	Get(ctx context.Context, owner *domain.Account, id domain.ItemID) (domain.Item, error)
	GetByTitle(ctx context.Context, owner *domain.Account, title string) (domain.Item, error)
	Search(ctx context.Context, owner *domain.Account, q domain.SearchQuery) ([]domain.Item, error)
	ListByFilter(ctx context.Context, owner *domain.Account, filter []domain.AttributeFilter) ([]domain.Item, error)
	SetAttributes(ctx context.Context, owner *domain.Account, 
		id domain.ItemID, attrs []domain.AttributePair) error
	RenameAttribute(ctx context.Context, owner *domain.Account, id domain.ItemID, 
		oldKey string, newKey string) error
	DeleteAttribute(ctx context.Context, owner *domain.Account, id domain.ItemID, key string) error
}



