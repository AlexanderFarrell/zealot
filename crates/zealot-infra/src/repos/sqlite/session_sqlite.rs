use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, session::SessionRepo};
use zealot_domain::{account::Account, auth::CreateSessionDto, common::id::Id};

#[derive(Debug)]
pub struct SessionSqliteRepo {
    pool: SqlitePool,
}

impl SessionSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl SessionRepo for SessionSqliteRepo {
    fn create_session(&self, _dto: &CreateSessionDto) -> Result<(), RepoError> {
        todo!()
    }

    fn get_account_by_token_hash(&self, _token_hash: &str) -> Result<Option<Account>, RepoError> {
        todo!()
    }

    fn delete_session_by_token_hash(&self, _token_hash: &str) -> Result<(), RepoError> {
        todo!()
    }

    fn delete_sessions_for_account(&self, _account_id: &Id) -> Result<(), RepoError> {
        todo!()
    }
}
