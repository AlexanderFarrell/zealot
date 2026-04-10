//! Provides services for a user. For login, registration, etc. see AuthService.

use std::sync::Arc;

use zealot_domain::common::id::Id;

use crate::{repos::{account::AccountRepo, common::RepoError}, services::common::ServiceError};

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
}
