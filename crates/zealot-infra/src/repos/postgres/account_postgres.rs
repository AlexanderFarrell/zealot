use sqlx::PgPool;
use zealot_app::repos::{account::AccountRepo, common::RepoError};
use zealot_domain::{account::{Account, CreateAccountDto}, common::id::Id};

#[derive(Debug)]
pub struct AccountPostgresRepo {
    pool: PgPool,
}

impl AccountPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AccountRepo for AccountPostgresRepo {
    fn get_password_hash_by_username(&self, _username: &str) -> Result<Option<String>, RepoError> {
        todo!()
    }

    fn get_account_by_id(&self, _id: &Id) -> Result<Option<Account>, RepoError> {
        todo!()
    }

    fn get_account_by_username(&self, _username: &str) -> Result<Option<Account>, RepoError> {
        todo!()
    }

    fn get_account_by_api_key(&self, _key: &str) -> Result<Option<Account>, RepoError> {
        todo!()
    }

    fn add_account(&self, _account: &CreateAccountDto) -> Result<Account, RepoError> {
        todo!()
    }

    fn delete_account(&self, _account_id: &Id) -> Result<(), RepoError> {
        todo!()
    }

    fn upsert_api_key(&self, _account_id: &Id, _key: &str) -> Result<(), RepoError> {
        todo!()
    }

    fn delete_api_key(&self, _account_id: &Id) -> Result<(), RepoError> {
        todo!()
    }
}
