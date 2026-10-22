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

    /// Merges `patch` into the account's settings, one top-level key at a time.
    ///
    /// Keys absent from `patch` are kept, so a client that only knows some settings (the
    /// `timezone` key, for one, is read by the server and never sent by the UI) cannot erase the
    /// rest. A non-object `patch` is rejected.
    pub fn update_settings(
        &self,
        account_id: &Id,
        patch: serde_json::Value,
    ) -> Result<(), RepoError> {
        let serde_json::Value::Object(patch) = patch else {
            return Err(RepoError::DatabaseError {
                err: "settings must be a JSON object".to_string(),
            });
        };
        let current = self
            .repo
            .get_account_by_id(account_id)?
            .ok_or(RepoError::NotFound)?
            .settings;
        let merged = merge_settings(current, patch);
        self.repo.update_settings(account_id, &merged)
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
            .map_err(|e| {
                tracing::error!(account_id = ?account_id, %e, "failed to insert api key");
                ServiceError::DomainError {
                    err: AccountError::ServerError,
                }
            })?;
        tracing::info!(account_id = ?account_id, api_key_id = ?record.api_key_id, %label, "api key created");
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
            .map_err(|e| {
                tracing::error!(account_id = ?account_id, ?api_key_id, %e, "failed to revoke api key");
                ServiceError::DomainError {
                    err: AccountError::ServerError,
                }
            })?;
        tracing::info!(account_id = ?account_id, ?api_key_id, "api key revoked");
        Ok(())
    }
}

fn merge_settings(
    current: serde_json::Value,
    patch: serde_json::Map<String, serde_json::Value>,
) -> serde_json::Value {
    let mut merged = match current {
        serde_json::Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    merged.extend(patch);
    serde_json::Value::Object(merged)
}

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("server error")]
    ServerError,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::merge_settings;

    fn patch(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
        value.as_object().cloned().unwrap()
    }

    #[test]
    fn merge_keeps_keys_the_patch_does_not_mention() {
        let merged = merge_settings(
            json!({"timezone": "Europe/London", "theme": "light"}),
            patch(json!({"theme": "dark", "item_actions": "menu"})),
        );
        assert_eq!(
            merged,
            json!({"timezone": "Europe/London", "theme": "dark", "item_actions": "menu"})
        );
    }

    #[test]
    fn merge_replaces_a_non_object_current_value() {
        let merged = merge_settings(json!(null), patch(json!({"theme": "dark"})));
        assert_eq!(merged, json!({"theme": "dark"}));
    }
}
