use std::fmt::Debug;

use zealot_domain::{account::{Account, InsertAccountDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait AccountRepo: Debug + Send + Sync {
     fn get_hash_by_username(&self, username: &str) -> Result<Option<String>, RepoError>;
     fn get_account_by_id(&self, id: &Id) -> Result<Option<Account>, RepoError>;
     fn get_account_by_title(&self, title: &str) -> Result<Option<Account>, RepoError>;
     fn add_account(&self, account: &InsertAccountDto) -> Result<Option<Account>, RepoError>;
     fn delete_account(&self, account_id: &Id) -> Result<(), RepoError>;
     fn upsert_api_key(&self, account_id: &Id, key: &str) -> Result<(), RepoError>;
     fn delete_api_key(&self, account_id: &Id) -> Result<(), RepoError>;
}
