package repo

import (
	"context"
	"zealot-core/domain"
)

type AttributeKindRepository interface {
	List(ctx context.Context, owner *domain.Account) ([]domain.AttributeKind, error)
	GetByKey(ctx context.Context, owner *domain.Account, key string) (*domain.AttributeKind, error)
	Create(ctx context.Context, owner *domain.Account, kind domain.NewAttributeKind) (*domain.AttributeKind, error)
	UpdateConfig(ctx context.Context, owner *domain.Account, kindID domain.AttributeKindID, cfg domain.AttributeKindConfig) error
	Delete(ctx context.Context, owner *domain.AccountID, kindID domain.AttributeKindID) error
}