use sqlx::SqlitePool;
use zealot_app::repos::{account::AccountRepo, common::RepoError};
use zealot_domain::{
    account::{Account, ApiKeyRecord, CreateAccountDto},
    common::{email::Email, id::Id},
};

#[derive(sqlx::FromRow)]
struct AccountRow {
    account_id: i64,
    username: String,
    email: String,
    given_name: String,
    surname: String,
    settings: String,
    has_api_key: bool,
}

fn row_to_account(row: AccountRow) -> Result<Account, RepoError> {
    let settings = serde_json::from_str(&row.settings)
        .unwrap_or(serde_json::Value::Object(Default::default()));
    Ok(Account {
        account_id: Id::try_from(row.account_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        username: row.username,
        email: Email::try_from(row.email)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        given_name: row.given_name,
        surname: row.surname,
        settings,
        has_api_key: row.has_api_key,
    })
}

#[derive(sqlx::FromRow)]
struct ApiKeyRow {
    api_key_id: i64,
    label: String,
    created_at: String,
}

fn row_to_api_key_record(row: ApiKeyRow) -> Result<ApiKeyRecord, RepoError> {
    Ok(ApiKeyRecord {
        api_key_id: Id::try_from(row.api_key_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        label: row.label,
        created_at: row.created_at,
    })
}

#[derive(Debug)]
pub struct AccountSqliteRepo {
    pool: SqlitePool,
}

impl AccountSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl AccountRepo for AccountSqliteRepo {
    fn get_password_hash_by_username(&self, username: &str) -> Result<Option<String>, RepoError> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_scalar::<_, String>(
                    "SELECT password FROM account WHERE username = ?",
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
                    "SELECT account_id, username, email, given_name, surname, settings,
                            EXISTS(SELECT 1 FROM api_key k WHERE k.account_id = account.account_id) AS has_api_key
                     FROM account WHERE account_id = ?",
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
                    "SELECT account_id, username, email, given_name, surname, settings,
                            EXISTS(SELECT 1 FROM api_key k WHERE k.account_id = account.account_id) AS has_api_key
                     FROM account WHERE username = ?",
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
                    "SELECT a.account_id, a.username, a.email, a.given_name, a.surname, a.settings,
                            1 AS has_api_key
                     FROM account a JOIN api_key k ON k.account_id = a.account_id
                     WHERE k.key_hash = ?",
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
                     VALUES (?, ?, ?, ?, ?)
                     RETURNING account_id, username, email, given_name, surname, settings,
                               0 AS has_api_key",
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
                sqlx::query("DELETE FROM account WHERE account_id = ?")
                    .bind(id_val)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn insert_api_key(&self, account_id: &Id, key_hash: &str, label: &str) -> Result<ApiKeyRecord, RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, ApiKeyRow>(
                    "INSERT INTO api_key (account_id, key_hash, label)
                     VALUES (?, ?, ?)
                     RETURNING api_key_id, label, created_at",
                )
                .bind(id_val)
                .bind(key_hash)
                .bind(label)
                .fetch_one(&self.pool)
                .await
                .map_err(RepoError::from)
                .and_then(row_to_api_key_record)
            })
        })
    }

    fn list_api_keys(&self, account_id: &Id) -> Result<Vec<ApiKeyRecord>, RepoError> {
        let id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, ApiKeyRow>(
                    "SELECT api_key_id, label, created_at
                     FROM api_key WHERE account_id = ?
                     ORDER BY created_at ASC",
                )
                .bind(id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?
                .into_iter()
                .map(row_to_api_key_record)
                .collect()
            })
        })
    }

    fn delete_api_key_by_id(&self, api_key_id: &Id, account_id: &Id) -> Result<(), RepoError> {
        let key_id_val = i64::from(*api_key_id);
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "DELETE FROM api_key WHERE api_key_id = ? AND account_id = ?",
                )
                .bind(key_id_val)
                .bind(account_id_val)
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
                    "UPDATE account SET settings = ? WHERE account_id = ?",
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
