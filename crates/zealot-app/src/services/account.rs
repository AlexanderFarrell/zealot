//! Provides services for a user. For login, registration, etc. see AuthService.

use std::sync::Arc;

use uuid::Uuid;
use zealot_domain::{account::ApiKeyRecord, common::id::Id};

use crate::{
    repos::{account::AccountRepo, common::RepoError},
    services::{auth::AuthService, common::ServiceError},
};

/// Access to user specific settings.
#[derive(Debug, Clone)]
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    pub fn new(repo: &Arc<dyn AccountRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn update_settings(
        &self,
        account_id: &Id,
        settings: serde_json::Value,
    ) -> Result<(), RepoError> {
        self.repo.update_settings(account_id, &settings)
    }

    /// Generates a new API key, stores its hash, and returns the record plus the raw key (shown once).
    pub fn generate_api_key(
        &self,
        account_id: &Id,
        label: &str,
    ) -> Result<(ApiKeyRecord, String), ServiceError<AccountError>> {
        let raw = Uuid::new_v4().simple().to_string();
        let hash = AuthService::hash_token(&raw);
        let record = self
            .repo
            .insert_api_key(account_id, &hash, label)
            .map_err(|_| ServiceError::DomainError {
                err: AccountError::ServerError,
            })?;
        Ok((record, raw))
    }

    pub fn list_api_keys(
        &self,
        account_id: &Id,
    ) -> Result<Vec<ApiKeyRecord>, ServiceError<AccountError>> {
        self.repo
            .list_api_keys(account_id)
            .map_err(|_| ServiceError::DomainError {
                err: AccountError::ServerError,
            })
    }

    pub fn revoke_api_key(
        &self,
        api_key_id: &Id,
        account_id: &Id,
    ) -> Result<(), ServiceError<AccountError>> {
        self.repo
            .delete_api_key_by_id(api_key_id, account_id)
            .map_err(|_| ServiceError::DomainError {
                err: AccountError::ServerError,
            })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("server error")]
    ServerError,
}
