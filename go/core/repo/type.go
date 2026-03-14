package repo

import (
	"context"
	"zealot-core/domain"
)



type TypeRepository interface {
	List(ctx context.Context, owner *domain.Account) ([]domain.ItemType, error)
	GetByName(ctx context.Context, owner *domain.Account) (*domain.ItemType, error)
	Create(ctx context.Context, owner *domain.Account, )

	List(ctx context.Context, owner auth.AccountID) ([]domain.ItemType, error)
	GetByName(ctx context.Context, owner auth.AccountID, name string) (domain.ItemType, error)
	Create(ctx context.Context, owner auth.AccountID, t domain.ItemTypeDraft) (domain.ItemType, error)
	Update(ctx context.Context, owner auth.AccountID, t domain.ItemTypePatch) error
	Delete(ctx context.Context, owner auth.AccountID, typeID domain.TypeID) error
	SetRequiredKinds(ctx context.Context, owner auth.AccountID, typeName string, keys []string) error
}