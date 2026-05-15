//! Provides services for a user. For login, registration, etc. see AuthService.

use std::sync::Arc;

use uuid::Uuid;
use zealot_domain::common::id::Id;

use crate::{repos::{account::AccountRepo, common::RepoError}, services::{auth::AuthService, common::ServiceError}};

/// Access to user specific settings.
#[derive(Debug, Clone)]
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    pub fn new(repo: &Arc<dyn AccountRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn update_settings(&self, account_id: &Id, settings: serde_json::Value) -> Result<(), RepoError> {
        self.repo.update_settings(account_id, &settings)
    }

    /// Generates a new API key, stores its hash, and returns the raw key (shown once).
    pub fn generate_api_key(&self, account_id: &Id) -> Result<String, ServiceError<AccountError>> {
        let raw = Uuid::new_v4().simple().to_string();
        let hash = AuthService::hash_token(&raw);
        self.repo
            .upsert_api_key(account_id, &hash)
            .map_err(|_| ServiceError::DomainError { err: AccountError::ServerError })?;
        Ok(raw)
    }

    pub fn revoke_api_key(&self, account_id: &Id) -> Result<(), ServiceError<AccountError>> {
        self.repo
            .delete_api_key(account_id)
            .map_err(|_| ServiceError::DomainError { err: AccountError::ServerError })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("server error")]
    ServerError,
}
