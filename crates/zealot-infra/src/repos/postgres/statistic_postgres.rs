use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use zealot_app::repos::{common::RepoError, statistic::StatisticRepo};
use zealot_domain::{account::Account, common::id::Id, statistic::StatisticEntryCore};

#[derive(Debug)]
pub struct StatisticPostgresRepo {
    pool: PgPool,
}

impl StatisticPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn get_scoped(
        &self,
        statistic_entry_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, StatisticRow>(&format!(
                    "{SELECT} WHERE statistic_entry_id = $1
                     AND item_id IN (SELECT item_id FROM item WHERE scope_id = ANY($2))"
                ))
                .bind(entry_id)
                .bind(&scope_ids)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                row.map(to_core).transpose()
            })
        })
    }

    fn list_scoped(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: Option<i64>,
        offset: Option<i64>,
        ascending: bool,
        scope_ids: &[Uuid],
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let pool = self.pool.clone();
        let item_id = i64::from(item_id);
        let scope_ids = scope_ids.to_vec();
        let order = if ascending {
            "occurred_at ASC, statistic_entry_id ASC"
        } else {
            "occurred_at DESC, statistic_entry_id DESC"
        };
        let sql = if limit.is_some() {
            format!(
                "{SELECT} WHERE item_id = $1
                 AND item_id IN (SELECT item_id FROM item WHERE scope_id = ANY($2))
                 AND ($3::timestamptz IS NULL OR occurred_at >= $3)
                 AND ($4::timestamptz IS NULL OR occurred_at < $4)
                 ORDER BY {order} LIMIT $5 OFFSET $6"
            )
        } else {
            format!(
                "{SELECT} WHERE item_id = $1
                 AND item_id IN (SELECT item_id FROM item WHERE scope_id = ANY($2))
                 AND ($3::timestamptz IS NULL OR occurred_at >= $3)
                 AND ($4::timestamptz IS NULL OR occurred_at < $4)
                 ORDER BY {order}"
            )
        };
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut query = sqlx::query_as::<_, StatisticRow>(&sql)
                    .bind(item_id)
                    .bind(&scope_ids)
                    .bind(start)
                    .bind(end);
                if let (Some(limit), Some(offset)) = (limit, offset) {
                    query = query.bind(limit).bind(offset);
                }
                query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .into_iter()
                    .map(to_core)
                    .collect()
            })
        })
    }

    fn create_scoped(
        &self,
        item_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
        scope_ids: &[Uuid],
    ) -> Result<StatisticEntryCore, RepoError> {
        if scope_ids.is_empty() {
            return Err(RepoError::NotFound);
        }
        let pool = self.pool.clone();
        let item_id_val = i64::from(item_id);
        let account_id = i64::from(account.account_id);
        let related = related_item_id.map(i64::from);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let exists: Option<i32> = sqlx::query_scalar(
                    "SELECT item_id FROM item WHERE item_id = $1 AND scope_id = ANY($2)",
                )
                .bind(item_id_val)
                .bind(&scope_ids)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                if exists.is_none() {
                    return Err(RepoError::NotFound);
                }
                let row = sqlx::query_as::<_, StatisticRow>(
                    "INSERT INTO statistic_entry
                         (item_id, account_id, value, occurred_at, related_item_id, comment)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     RETURNING statistic_entry_id, item_id, value, occurred_at,
                               related_item_id, comment, created_at, updated_at",
                )
                .bind(item_id_val)
                .bind(account_id)
                .bind(value)
                .bind(occurred_at)
                .bind(related)
                .bind(comment)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;
                to_core(row)
            })
        })
    }

    fn update_scoped(
        &self,
        statistic_entry_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let related = related_item_id.map(i64::from);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, StatisticRow>(
                    "UPDATE statistic_entry
                     SET value = $1, occurred_at = $2, related_item_id = $3,
                         comment = $4, updated_at = now()
                     WHERE statistic_entry_id = $5
                       AND item_id IN (SELECT item_id FROM item WHERE scope_id = ANY($6))
                     RETURNING statistic_entry_id, item_id, value, occurred_at,
                               related_item_id, comment, created_at, updated_at",
                )
                .bind(value)
                .bind(occurred_at)
                .bind(related)
                .bind(comment)
                .bind(entry_id)
                .bind(&scope_ids)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                row.map(to_core).transpose()
            })
        })
    }

    fn delete_scoped(&self, statistic_entry_id: Id, scope_ids: &[Uuid]) -> Result<bool, RepoError> {
        if scope_ids.is_empty() {
            return Ok(false);
        }
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let result = sqlx::query(
                    "DELETE FROM statistic_entry
                     WHERE statistic_entry_id = $1
                       AND item_id IN (SELECT item_id FROM item WHERE scope_id = ANY($2))",
                )
                .bind(entry_id)
                .bind(&scope_ids)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;
                Ok(result.rows_affected() == 1)
            })
        })
    }
}

#[derive(sqlx::FromRow)]
struct StatisticRow {
    statistic_entry_id: i64,
    item_id: i64,
    value: f64,
    occurred_at: DateTime<Utc>,
    related_item_id: Option<i64>,
    comment: Option<String>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn to_core(row: StatisticRow) -> Result<StatisticEntryCore, RepoError> {
    let id = |value: i64| {
        Id::try_from(value).map_err(|e| RepoError::DatabaseError { err: e.to_string() })
    };
    Ok(StatisticEntryCore {
        statistic_entry_id: id(row.statistic_entry_id)?,
        item_id: id(row.item_id)?,
        value: row.value,
        occurred_at: row.occurred_at,
        related_item_id: row.related_item_id.map(id).transpose()?,
        comment: row.comment,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

const SELECT: &str = "SELECT statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at FROM statistic_entry";

impl StatisticRepo for StatisticPostgresRepo {
    fn get_in_scopes(
        &self,
        statistic_entry_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        self.get_scoped(statistic_entry_id, scope_ids)
    }

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
                    "{SELECT} WHERE statistic_entry_id = $1 AND account_id = $2"
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let rows = sqlx::query_as::<_, StatisticRow>(&format!("{SELECT} WHERE item_id = $1 AND account_id = $2 AND ($3::timestamptz IS NULL OR occurred_at >= $3) AND ($4::timestamptz IS NULL OR occurred_at < $4) ORDER BY occurred_at DESC, statistic_entry_id DESC LIMIT $5 OFFSET $6"))
                .bind(item_id).bind(account_id).bind(start).bind(end).bind(limit).bind(offset).fetch_all(&pool).await?;
            rows.into_iter().map(to_core).collect()
        })
        })
    }

    fn list_in_scopes(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        self.list_scoped(
            item_id,
            start,
            end,
            Some(limit),
            Some(offset),
            false,
            scope_ids,
        )
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let rows = sqlx::query_as::<_, StatisticRow>(&format!("{SELECT} WHERE item_id = $1 AND account_id = $2 AND ($3::timestamptz IS NULL OR occurred_at >= $3) AND ($4::timestamptz IS NULL OR occurred_at < $4) ORDER BY occurred_at ASC, statistic_entry_id ASC"))
                .bind(item_id).bind(account_id).bind(start).bind(end).fetch_all(&pool).await?;
            rows.into_iter().map(to_core).collect()
        })
        })
    }

    fn list_all_in_scopes(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        scope_ids: &[Uuid],
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        self.list_scoped(item_id, start, end, None, None, true, scope_ids)
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let row = sqlx::query_as::<_, StatisticRow>("INSERT INTO statistic_entry (item_id, account_id, value, occurred_at, related_item_id, comment) VALUES ($1, $2, $3, $4, $5, $6) RETURNING statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at")
                .bind(item_id).bind(account_id).bind(value).bind(occurred_at).bind(related).bind(comment).fetch_one(&pool).await?;
            to_core(row)
        })
        })
    }

    fn create_in_scopes(
        &self,
        item_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
        scope_ids: &[Uuid],
    ) -> Result<StatisticEntryCore, RepoError> {
        self.create_scoped(
            item_id,
            value,
            occurred_at,
            related_item_id,
            comment,
            account,
            scope_ids,
        )
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
            let row = sqlx::query_as::<_, StatisticRow>("UPDATE statistic_entry SET value = $1, occurred_at = $2, related_item_id = $3, comment = $4, updated_at = now() WHERE statistic_entry_id = $5 AND account_id = $6 RETURNING statistic_entry_id, item_id, value, occurred_at, related_item_id, comment, created_at, updated_at")
                .bind(value).bind(occurred_at).bind(related).bind(comment).bind(entry_id).bind(account_id).fetch_optional(&pool).await?;
            row.map(to_core).transpose()
        })
        })
    }

    fn update_in_scopes(
        &self,
        statistic_entry_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        self.update_scoped(
            statistic_entry_id,
            value,
            occurred_at,
            related_item_id,
            comment,
            scope_ids,
        )
    }

    fn delete(&self, statistic_entry_id: Id, account: &Account) -> Result<bool, RepoError> {
        let pool = self.pool.clone();
        let entry_id = i64::from(statistic_entry_id);
        let account_id = i64::from(account.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let result = sqlx::query(
                    "DELETE FROM statistic_entry WHERE statistic_entry_id = $1 AND account_id = $2",
                )
                .bind(entry_id)
                .bind(account_id)
                .execute(&pool)
                .await?;
                Ok(result.rows_affected() == 1)
            })
        })
    }

    fn delete_in_scopes(
        &self,
        statistic_entry_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<bool, RepoError> {
        self.delete_scoped(statistic_entry_id, scope_ids)
    }
}
