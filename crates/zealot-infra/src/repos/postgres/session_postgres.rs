use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, session::SessionRepo};
use zealot_domain::{
    account::Account,
    auth::CreateSessionDto,
    common::{email::Email, id::Id},
};

#[derive(sqlx::FromRow)]
struct AccountRow {
    account_id: i32,
    username: String,
    email: String,
    given_name: String,
    surname: String,
    settings: String,
}

fn row_to_account(row: AccountRow) -> Result<Account, RepoError> {
    let settings = serde_json::from_str(&row.settings)
        .unwrap_or(serde_json::Value::Object(Default::default()));
    Ok(Account {
        account_id: Id::try_from(row.account_id as i64)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        username: row.username,
        email: Email::try_from(row.email)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        given_name: row.given_name,
        surname: row.surname,
        settings,
    })
}

#[derive(Debug)]
pub struct SessionPostgresRepo {
    pool: PgPool,
}

impl SessionPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl SessionRepo for SessionPostgresRepo {
    fn create_session(&self, dto: &CreateSessionDto) -> Result<(), RepoError> {
        let account_id = i64::from(dto.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "INSERT INTO session (token_hash, account_id, expires_at)
                     VALUES ($1, $2, $3)",
                )
                .bind(&dto.token_hash)
                .bind(account_id)
                .bind(dto.expires_at)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }

    fn get_account_by_token_hash(&self, token_hash: &str) -> Result<Option<Account>, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AccountRow>(
                    "SELECT a.account_id, a.username, a.email, a.given_name, a.surname, a.settings::text
                     FROM account a
                     JOIN session s ON s.account_id = a.account_id
                     WHERE s.token_hash = $1
                       AND s.expires_at > now()",
                )
                .bind(token_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_account)
                .transpose()
            })
        })
    }

    fn delete_session_by_token_hash(&self, token_hash: &str) -> Result<(), RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query("DELETE FROM session WHERE token_hash = $1")
                    .bind(token_hash)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn delete_sessions_for_account(&self, account_id: &Id) -> Result<(), RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query("DELETE FROM session WHERE account_id = $1")
                    .bind(id_val)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }
}
