use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, statistic::StatisticRepo};
use zealot_domain::{account::Account, common::id::Id, statistic::StatisticEntryCore};

#[derive(Debug)]
pub struct StatisticSqliteRepo {
    pool: SqlitePool,
}

impl StatisticSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct StatisticRow {
    statistic_entry_id: i64,
    item_id: i64,
    value: f64,
    occurred_at: i64,
    related_item_id: Option<i64>,
    comment: Option<String>,
    created_at: i64,
    updated_at: i64,
}

fn timestamp(value: i64) -> Result<DateTime<Utc>, RepoError> {
    DateTime::from_timestamp(value, 0).ok_or_else(|| RepoError::DatabaseError {
        err: format!("invalid timestamp: {value}"),
    })
}

fn to_core(row: StatisticRow) -> Result<StatisticEntryCore, RepoError> {
    let id = |value: i64| {
        Id::try_from(value).map_err(|e| RepoError::DatabaseError { err: e.to_string() })
    };
    Ok(StatisticEntryCore {
        statistic_entry_id: id(row.statistic_entry_id)?,
        item_id: id(row.item_id)?,
        value: row.value,
        occurred_at: timestamp(row.occurred_at)?,
        related_item_id: row.related_item_id.map(id).transpose()?,
        comment: row.comment,
        created_at: timestamp(row.created_at)?,
        updated_at: timestamp(row.updated_at)?,
    })
}

const SELECT: &str = "SELECT statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at FROM statistic_entry";

impl StatisticRepo for StatisticSqliteRepo {
    fn get(
        &self,
        statistic_entry_id: Id,
        account: &Account,
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let account_id = i64::from(account.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, StatisticRow>(&format!(
                    "{SELECT} WHERE statistic_entry_id = ? AND account_id = ?"
                ))
                .bind(entry_id)
                .bind(account_id)
                .fetch_optional(&pool)
                .await?;
                row.map(to_core).transpose()
            })
        })
    }

    fn list(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        let pool = self.pool.clone();
        let item_id = i64::from(item_id);
        let account_id = i64::from(account.account_id);
        let start = start.map(|v| v.timestamp());
        let end = end.map(|v| v.timestamp());
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let rows = sqlx::query_as::<_, StatisticRow>(&format!("{SELECT} WHERE item_id = ? AND account_id = ? AND (? IS NULL OR occurred_at >= ?) AND (? IS NULL OR occurred_at < ?) ORDER BY occurred_at DESC, statistic_entry_id DESC LIMIT ? OFFSET ?"))
                .bind(item_id).bind(account_id).bind(start).bind(start).bind(end).bind(end).bind(limit).bind(offset).fetch_all(&pool).await?;
            rows.into_iter().map(to_core).collect()
        })
        })
    }

    fn list_all(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        account: &Account,
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        let pool = self.pool.clone();
        let item_id = i64::from(item_id);
        let account_id = i64::from(account.account_id);
        let start = start.map(|v| v.timestamp());
        let end = end.map(|v| v.timestamp());
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let rows = sqlx::query_as::<_, StatisticRow>(&format!("{SELECT} WHERE item_id = ? AND account_id = ? AND (? IS NULL OR occurred_at >= ?) AND (? IS NULL OR occurred_at < ?) ORDER BY occurred_at ASC, statistic_entry_id ASC"))
                .bind(item_id).bind(account_id).bind(start).bind(start).bind(end).bind(end).fetch_all(&pool).await?;
            rows.into_iter().map(to_core).collect()
        })
        })
    }

    fn create(
        &self,
        item_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
    ) -> Result<StatisticEntryCore, RepoError> {
        let pool = self.pool.clone();
        let item_id = i64::from(item_id);
        let account_id = i64::from(account.account_id);
        let related = related_item_id.map(i64::from);
        let occurred_at = occurred_at.timestamp();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let row = sqlx::query_as::<_, StatisticRow>("INSERT INTO statistic_entry (item_id, account_id, value, occurred_at, related_item_id, comment) VALUES (?, ?, ?, ?, ?, ?) RETURNING statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at")
                .bind(item_id).bind(account_id).bind(value).bind(occurred_at).bind(related).bind(comment).fetch_one(&pool).await?;
            to_core(row)
        })
        })
    }

    fn update(
        &self,
        statistic_entry_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let account_id = i64::from(account.account_id);
        let related = related_item_id.map(i64::from);
        let occurred_at = occurred_at.timestamp();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let row = sqlx::query_as::<_, StatisticRow>("UPDATE statistic_entry SET value = ?, occurred_at = ?, related_item_id = ?, comment = ?, updated_at = strftime('%s', 'now') WHERE statistic_entry_id = ? AND account_id = ? RETURNING statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at")
                .bind(value).bind(occurred_at).bind(related).bind(comment).bind(entry_id).bind(account_id).fetch_optional(&pool).await?;
            row.map(to_core).transpose()
        })
        })
    }

    fn delete(&self, statistic_entry_id: Id, account: &Account) -> Result<bool, RepoError> {
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let account_id = i64::from(account.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let result = sqlx::query(
                    "DELETE FROM statistic_entry WHERE statistic_entry_id = ? AND account_id = ?",
                )
                .bind(entry_id)
                .bind(account_id)
                .execute(&pool)
                .await?;
                Ok(result.rows_affected() == 1)
            })
        })
    }
}
