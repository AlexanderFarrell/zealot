use chrono::NaiveDate;
use sqlx::SqlitePool;
use uuid::Uuid;
use zealot_app::repos::{common::RepoError, time_block::TimeBlockRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    time_block::{CreateTimeBlockDto, TimeBlockCore, UpdateTimeBlockDto},
};

#[derive(Debug)]
pub struct TimeBlockSqliteRepo {
    pool: SqlitePool,
}

impl TimeBlockSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TimeBlockRow {
    block_id: i64,
    item_id: i64,
    date: String,
    start_min: i32,
    end_min: i32,
    note: String,
}

fn row_to_core(row: TimeBlockRow) -> Result<TimeBlockCore, RepoError> {
    let date = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let block_id =
        Id::try_from(row.block_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let item_id =
        Id::try_from(row.item_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    Ok(TimeBlockCore {
        block_id,
        item_id,
        date,
        start_min: row.start_min,
        end_min: row.end_min,
        note: row.note,
    })
}

impl TimeBlockRepo for TimeBlockSqliteRepo {
    fn get_for_day(
        &self,
        date: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let date_str = date.format("%Y-%m-%d").to_string();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT block_id, item_id, date, start_min, end_min, note
                     FROM time_block
                     WHERE account_id = ? AND date = ?
                     ORDER BY start_min",
                )
                .bind(account_id)
                .bind(&date_str)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter().map(row_to_core).collect()
            })
        })
    }

    fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT block_id, item_id, date, start_min, end_min, note
                     FROM time_block
                     WHERE account_id = ? AND date BETWEEN ? AND ?
                     ORDER BY date, start_min",
                )
                .bind(account_id)
                .bind(&start_str)
                .bind(&end_str)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter().map(row_to_core).collect()
            })
        })
    }

    fn get_for_item(
        &self,
        item_id: Id,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let item_id_val = i64::from(item_id);
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT block_id, item_id, date, start_min, end_min, note
                     FROM time_block
                     WHERE item_id = ? AND account_id = ?
                     ORDER BY date, start_min",
                )
                .bind(item_id_val)
                .bind(account_id)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter().map(row_to_core).collect()
            })
        })
    }

    fn create(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
    ) -> Result<TimeBlockCore, RepoError> {
        let item_id = dto.item_id;
        let date_str = dto.date.clone();
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone().unwrap_or_default();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, TimeBlockRow>(
                    "INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id)
                     VALUES (?, ?, ?, ?, ?, ?)
                     RETURNING block_id, item_id, date, start_min, end_min, note",
                )
                .bind(item_id)
                .bind(&date_str)
                .bind(start_min)
                .bind(end_min)
                .bind(&note)
                .bind(account_id)
                .fetch_one(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                row_to_core(row)
            })
        })
    }

    fn update(&self, dto: &UpdateTimeBlockDto, account: &Account) -> Result<(), RepoError> {
        let block_id = dto.block_id;
        let date_str = dto.date.clone();
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE time_block
                     SET date      = COALESCE(?, date),
                         start_min = COALESCE(?, start_min),
                         end_min   = COALESCE(?, end_min),
                         note      = COALESCE(?, note)
                     WHERE block_id = ? AND account_id = ?",
                )
                .bind(date_str)
                .bind(start_min)
                .bind(end_min)
                .bind(note)
                .bind(block_id)
                .bind(account_id)
                .execute(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                Ok(())
            })
        })
    }

    fn delete(&self, block_id: Id, account: &Account) -> Result<(), RepoError> {
        let block_id_val = i64::from(block_id);
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query("DELETE FROM time_block WHERE block_id = ? AND account_id = ?")
                    .bind(block_id_val)
                    .bind(account_id)
                    .execute(&pool)
                    .await
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                Ok(())
            })
        })
    }

    fn get_for_day_in_scopes(
        &self,
        date: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let date_str = date.format("%Y-%m-%d").to_string();
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE i.scope_id IN ({}) AND tb.date = ?
                     ORDER BY tb.start_min",
                    placeholders
                );
                let mut query = sqlx::query_as::<_, TimeBlockRow>(&sql);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query = query.bind(&date_str);
                query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .into_iter()
                    .map(row_to_core)
                    .collect()
            })
        })
    }

    fn get_for_range_in_scopes(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let start_str = start.format("%Y-%m-%d").to_string();
        let end_str = end.format("%Y-%m-%d").to_string();
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE i.scope_id IN ({}) AND tb.date BETWEEN ? AND ?
                     ORDER BY tb.date, tb.start_min",
                    placeholders
                );
                let mut query = sqlx::query_as::<_, TimeBlockRow>(&sql);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query = query.bind(&start_str).bind(&end_str);
                query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .into_iter()
                    .map(row_to_core)
                    .collect()
            })
        })
    }

    fn get_for_item_in_scopes(
        &self,
        item_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let item_id_val = i64::from(item_id);
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE tb.item_id = ? AND i.scope_id IN ({})
                     ORDER BY tb.date, tb.start_min",
                    placeholders
                );
                let mut query = sqlx::query_as::<_, TimeBlockRow>(&sql).bind(item_id_val);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .into_iter()
                    .map(row_to_core)
                    .collect()
            })
        })
    }

    fn create_in_scopes(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
        scope_ids: &[Uuid],
    ) -> Result<TimeBlockCore, RepoError> {
        if scope_ids.is_empty() {
            return Err(RepoError::NotFound);
        }
        let item_id = dto.item_id;
        let date_str = dto.date.clone();
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone().unwrap_or_default();
        let account_id = i64::from(account.account_id);
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT item_id FROM item WHERE item_id = ? AND scope_id IN ({})",
                    placeholders
                );
                let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(item_id);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                if query
                    .fetch_optional(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .is_none()
                {
                    return Err(RepoError::NotFound);
                }
                let row = sqlx::query_as::<_, TimeBlockRow>(
                    "INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id)
                     VALUES (?, ?, ?, ?, ?, ?)
                     RETURNING block_id, item_id, date, start_min, end_min, note",
                )
                .bind(item_id)
                .bind(&date_str)
                .bind(start_min)
                .bind(end_min)
                .bind(&note)
                .bind(account_id)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;
                row_to_core(row)
            })
        })
    }

    fn update_in_scopes(
        &self,
        dto: &UpdateTimeBlockDto,
        scope_ids: &[Uuid],
    ) -> Result<(), RepoError> {
        if scope_ids.is_empty() {
            return Ok(());
        }
        let block_id = dto.block_id;
        let date_str = dto.date.clone();
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone();
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "UPDATE time_block SET date = COALESCE(?, date),
                     start_min = COALESCE(?, start_min), end_min = COALESCE(?, end_min),
                     note = COALESCE(?, note)
                     WHERE block_id = ? AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id IN ({}))",
                    placeholders
                );
                let mut query = sqlx::query(&sql)
                    .bind(date_str)
                    .bind(start_min)
                    .bind(end_min)
                    .bind(note)
                    .bind(block_id);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn delete_in_scopes(&self, block_id: Id, scope_ids: &[Uuid]) -> Result<(), RepoError> {
        if scope_ids.is_empty() {
            return Ok(());
        }
        let block_id_val = i64::from(block_id);
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "DELETE FROM time_block WHERE block_id = ? AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id IN ({}))",
                    placeholders
                );
                let mut query = sqlx::query(&sql).bind(block_id_val);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }
}
