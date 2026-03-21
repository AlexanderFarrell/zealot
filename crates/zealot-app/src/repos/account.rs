use zealot_domain::{account::{APIKey, Account, AccountError, LoginBasicDto, RegisterBasicDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait AccountRepo {
    async fn is_logged_in(&self) -> Result<bool, RepoError<AccountError>>;
    async fn generate_api_key(&self, account_id: &Id) -> Result<APIKey, RepoError<AccountError>>;
    async fn revoke_api_key(&self, account_id: &Id) -> Result<(), RepoError<AccountError>>;
    async fn has_api_key(&self, account_id: &Id) -> Result<bool, RepoError<AccountError>>;
    async fn get_account_from_api_key(&self, api_key: &APIKey) -> Result<Option<Account>, RepoError<AccountError>>;
    async fn register_account(&self, dto: &RegisterBasicDto) -> Result<Option<Account>, RepoError<AccountError>>;
    async fn login_account(&self, dto: &LoginBasicDto) -> Result<Option<Account>, RepoError<AccountError>>;
    async fn username_exists(&self, username: String) -> Result<bool, RepoError<AccountError>>;
}



