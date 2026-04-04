//! Provides services for a user. For login, registration, etc. see AuthService.

use std::sync::Arc;

use zealot_domain::{
    account::{APIKey, Account, AccountError, LoginBasicDto, RegisterBasicDto},
    common::id::Id,
};

use crate::{repos::account::AccountRepo, services::common::ServiceError};

/// Access to user specific settings. 
#[derive(Debug, Clone)]
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    pub fn new(repo: &Arc<dyn AccountRepo>) -> Self {
        Self { repo: repo.clone() }
    }
}
