package item

import (
	"context"

	"github.com/alexanderfarrell/zealot/go/core/auth"
	"github.com/alexanderfarrell/zealot/go/core/event"
	"github.com/alexanderfarrell/zealot/go/core/item/domain"
)

type Repository interface {
	Create(ctx context.Context, owner auth.AccountID, draft domain.NewItem) (domain.Item, error)
	Update(ctx context.Context, owner auth.AccountID, cmd domain.UpdateItem) (domain.Item, error)
	Delete(ctx context.Context, owner auth.AccountID, id domain.ItemID) error
	Get(ctx context.Context, owner auth.AccountID, id domain.ItemID) (domain.Item, error)
	GetByTitle(ctx context.Context, owner auth.AccountID, title string) (domain.Item, error)
	Search(ctx context.Context, owner auth.AccountID, q domain.SearchQuery) ([]domain.Item, error)
	ListByFilter(ctx context.Context, owner auth.AccountID, filter domain.FilterSet) ([]domain.Item, error)
	SetAttributes(ctx context.Context, owner auth.AccountID, 
		id domain.ItemID, attrs []domain.AttributeAssignment) error
	RenameAttribute(ctx context.Context, owner auth.AccountID, id domain.ItemID, 
		oldKey string, newKey string) error
	DeleteAttribute(ctx context.Context, owner auth.AccountID, id domain.ItemID, key string) error
}

type TypeRepository interface {
	List(ctx context.Context, owner auth.AccountID) ([]domain.ItemType, error)
	GetByName(ctx context.Context, owner auth.AccountID, name string) (domain.ItemType, error)
	Create(ctx context.Context, owner auth.AccountID, t domain.ItemTypeDraft) (domain.ItemType, error)
	Update(ctx context.Context, owner auth.AccountID, t domain.ItemTypePatch) error
	Delete(ctx context.Context, owner auth.AccountID, typeID domain.TypeID) error
	SetRequiredKinds(ctx context.Context, owner auth.AccountID, typeName string, keys []string) error
}

type AttributeKindRepository interface {
	List(ctx context.Context, owner auth.AccountID) ([]domain.AttributeKind, error)
	GetByKey(ctx context.Context, owner auth.AccountID, key string) (domain.AttributeKind, error)
	Create(ctx context.Context, owner auth.AccountID, kind domain.AttributeKindDraft) (domain.AttributeKind, error)
	UpdateConfig(ctx context.Context, owner auth.AccountID, kindID, domain.KindID, cfg []byte) error
	Delete(ctx context.Context, owner auth.AccountID, kindID, domain.KindID) error
}

type CommentRepository interface {
	Add(ctx context.Context, owner auth.AccountID, cmd domain.AddComment) (domain.Comment, error)
	Update(ctx context.Context, owner auth.AccountID, cmd domain.UpdateComment) error
	Delete(ctx context.Context, owner auth.AccountID, id domain.CommentID) error
	ListForItem(ctx context.Context, owner auth.AccountID, itemID domain.ItemID) ([]domain.Comment, error)
	ListForDay(ctx context.Context, owner auth.AccountID, day domain.Day) ([]domain.Comment, error)
}

type UnitOfWork interface {
	WithinTx(ctx context.Context, fn func(context.Context) error) error
}

type Publisher interface {
	Publish(ctx context.Context, events ...event.Envelope) error
}