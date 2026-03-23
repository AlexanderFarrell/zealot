use std::sync::Arc;

use zealot_domain::{account::{APIKey, Account, AccountError, LoginBasicDto, RegisterBasicDto}, common::id::Id};

use crate::{repos::account::AccountRepo, services::common::ServiceError};

#[derive(Debug, Clone)]
pub struct AccountService {
    repo: Arc<dyn AccountRepo>,
}

impl AccountService {
    pub fn is_logged_in(&self) -> Result<bool, ServiceError<AccountError>> {
        todo!()
    }

    pub fn generate_api_key(&self, account_id: &Id) -> Result<APIKey, ServiceError<AccountError>> {
        todo!()
    }

    pub fn revoke_api_key(&self, account_id: &Id) -> Result<(), ServiceError<AccountError>>{
        todo!()
    }
    
    pub fn has_api_key(&self, account_id: &Id) -> Result<bool, ServiceError<AccountError>> {
        todo!()
    }

    pub fn get_account_from_api_key(&self, api_key: &APIKey) -> Result<Option<Account>, ServiceError<AccountError>> {
        todo!()
    }

    pub fn register_account(&self, dto: &RegisterBasicDto) -> Result<Option<Account>, ServiceError<AccountError>> {
        todo!()
    }
    
    pub fn login_account(&self, dto: &LoginBasicDto) -> Result<Option<Account>, ServiceError<AccountError>> {
        todo!()
    }

    pub fn username_exists(&self, username: String) -> Result<bool, ServiceError<AccountError>> {
        todo!()
    }
}