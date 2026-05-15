use sqlx::PgPool;
use zealot_app::repos::{account::AccountRepo, common::RepoError};
use zealot_domain::{
    account::{Account, CreateAccountDto},
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
    api_key_hash: Option<String>,
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
        has_api_key: row.api_key_hash.is_some(),
    })
}

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
    fn get_password_hash_by_username(&self, username: &str) -> Result<Option<String>, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_scalar::<_, String>(
                    "SELECT password FROM account WHERE username = $1",
                )
                .bind(username)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)
            })
        })
    }

    fn get_account_by_id(&self, id: &Id) -> Result<Option<Account>, RepoError> {
        let id_val = i64::from(*id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AccountRow>(
                    "SELECT account_id, username, email, given_name, surname, settings::text, api_key_hash
                     FROM account WHERE account_id = $1",
                )
                .bind(id_val)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_account)
                .transpose()
            })
        })
    }

    fn get_account_by_username(&self, username: &str) -> Result<Option<Account>, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AccountRow>(
                    "SELECT account_id, username, email, given_name, surname, settings::text, api_key_hash
                     FROM account WHERE username = $1",
                )
                .bind(username)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_account)
                .transpose()
            })
        })
    }

    fn get_account_by_api_key(&self, key_hash: &str) -> Result<Option<Account>, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AccountRow>(
                    "SELECT account_id, username, email, given_name, surname, settings::text, api_key_hash
                     FROM account WHERE api_key_hash = $1",
                )
                .bind(key_hash)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_account)
                .transpose()
            })
        })
    }

    fn add_account(&self, account: &CreateAccountDto) -> Result<Account, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AccountRow>(
                    "INSERT INTO account (username, email, password, given_name, surname)
                     VALUES ($1, $2, $3, $4, $5)
                     RETURNING account_id, username, email, given_name, surname, settings::text, api_key_hash",
                )
                .bind(&account.username)
                .bind(&account.email)
                .bind(&account.password_hash)
                .bind(&account.given_name)
                .bind(&account.surname)
                .fetch_one(&self.pool)
                .await
                .map_err(RepoError::from)
                .and_then(row_to_account)
            })
        })
    }

    fn delete_account(&self, account_id: &Id) -> Result<(), RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query("DELETE FROM account WHERE account_id = $1")
                    .bind(id_val)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn upsert_api_key(&self, account_id: &Id, key_hash: &str) -> Result<(), RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "UPDATE account SET api_key_hash = $1 WHERE account_id = $2",
                )
                .bind(key_hash)
                .bind(id_val)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }

    fn delete_api_key(&self, account_id: &Id) -> Result<(), RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "UPDATE account SET api_key_hash = NULL WHERE account_id = $1",
                )
                .bind(id_val)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }

    fn update_settings(&self, account_id: &Id, settings: &serde_json::Value) -> Result<(), RepoError> {
        let id_val = i64::from(*account_id);
        let settings_str = serde_json::to_string(settings)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "UPDATE account SET settings = $1::jsonb WHERE account_id = $2",
                )
                .bind(settings_str)
                .bind(id_val)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
