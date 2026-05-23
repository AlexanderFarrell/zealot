use std::fmt::Debug;

use zealot_domain::{
    account::{Account, ApiKeyRecord, CreateAccountDto},
    common::id::Id,
};

use crate::repos::common::RepoError;

pub trait AccountRepo: Debug + Send + Sync {
    fn get_password_hash_by_username(&self, username: &str) -> Result<Option<String>, RepoError>;
    fn get_account_by_id(&self, id: &Id) -> Result<Option<Account>, RepoError>;
    fn get_account_by_username(&self, username: &str) -> Result<Option<Account>, RepoError>;
    fn get_account_by_api_key(&self, key: &str) -> Result<Option<Account>, RepoError>;
    fn add_account(&self, account: &CreateAccountDto) -> Result<Account, RepoError>;
    fn delete_account(&self, account_id: &Id) -> Result<(), RepoError>;
    fn insert_api_key(&self, account_id: &Id, key_hash: &str, label: &str) -> Result<ApiKeyRecord, RepoError>;
    fn list_api_keys(&self, account_id: &Id) -> Result<Vec<ApiKeyRecord>, RepoError>;
    fn delete_api_key_by_id(&self, api_key_id: &Id, account_id: &Id) -> Result<(), RepoError>;
    fn update_settings(&self, account_id: &Id, settings: &serde_json::Value) -> Result<(), RepoError>;
}
