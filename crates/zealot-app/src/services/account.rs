use std::sync::Arc;

use zealot_domain::{
    account::{APIKey, Account, AccountError, LoginBasicDto, RegisterBasicDto},
    common::id::Id,
};

use crate::{repos::account::AccountRepo, services::common::ServiceError};

#[derive(Debug, Clone)]
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    pub fn new(repo: &Arc<dyn AccountRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn generate_api_key(&self, account_id: &Id) -> Result<APIKey, ServiceError<AccountError>> {
        todo!()
    }

    pub fn revoke_api_key(&self, account_id: &Id) -> Result<(), ServiceError<AccountError>> {
        todo!()
    }

    pub fn has_api_key(&self, account_id: &Id) -> Result<bool, ServiceError<AccountError>> {
        todo!()
    }
}
