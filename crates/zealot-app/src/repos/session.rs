use std::fmt::Debug;

use zealot_domain::{account::Account, auth::CreateSessionDto, common::id::Id};

use crate::repos::common::RepoError;

pub trait SessionRepo: Debug + Send + Sync {
    fn create_session(&self, dto: &CreateSessionDto) -> Result<(), RepoError>;
    fn get_account_by_token_hash(&self, token_hash: &str) -> Result<Option<Account>, RepoError>;
    fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), RepoError>;
    fn delete_sessions_for_account(&self, account_id: &Id) -> Result<(), RepoError>;
}
