package repo

import (
	"context"
	"zealot-core/domain"
)

type AccountRepository interface {
	// Auth
	Register(context context.Context, registration *domain.AccountRegistration) (*domain.Account, error)
	Login(context context.Context, login *domain.AccountLogin) (*domain.Account, error)
	IsLoggedIn(context context.Context) (bool, error) 
	Exists(context context.Context, username string) (bool, error)

	// Account Management
	UpdateSettings(context context.Context, accountID domain.AccountID, settings domain.AccountSettings) error
	ChangeUsername(context context.Context, accountID domain.AccountID, password string, newUsername string) error
	ChangePassword(context context.Context, accountID domain.AccountID, oldPassword string, newPassword string) error
	ChangeName(context context.Context, accountID domain.AccountID, newName string) error

	// API Keys
	IssueAPIKey(context context.Context, owner *domain.Account)
	RevokeAPIKey(context context.Context, owner *domain.Account)
	HasAPIKey(context context.Context, owner *domain.Account)
	GetAccountFromAPIKey(context context.Context, apiKey domain.APIKey) (*domain.Account, error)
}