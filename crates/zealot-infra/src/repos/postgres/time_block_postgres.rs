use chrono::NaiveDate;
use sqlx::PgPool;
use uuid::Uuid;
use zealot_app::repos::{common::RepoError, time_block::TimeBlockRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    time_block::{CreateTimeBlockDto, TimeBlockCore, UpdateTimeBlockDto},
};

#[derive(Debug)]
pub struct TimeBlockPostgresRepo {
    pool: PgPool,
}

impl TimeBlockPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct TimeBlockRow {
    block_id: i64,
    item_id: i64,
    date: NaiveDate,
    start_min: i32,
    end_min: i32,
    note: String,
}

fn row_to_core(row: TimeBlockRow) -> Result<TimeBlockCore, RepoError> {
    let block_id =
        Id::try_from(row.block_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let item_id =
        Id::try_from(row.item_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    Ok(TimeBlockCore {
        block_id,
        item_id,
        date: row.date,
        start_min: row.start_min,
        end_min: row.end_min,
        note: row.note,
    })
}

impl TimeBlockRepo for TimeBlockPostgresRepo {
    fn get_for_day(
        &self,
        date: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let date = *date;
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT block_id, item_id, date, start_min, end_min, note
                     FROM time_block
                     WHERE account_id = $1 AND date = $2
                     ORDER BY start_min",
                )
                .bind(account_id)
                .bind(date)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter()
                    .map(row_to_core)
                    .collect::<Result<Vec<_>, _>>()
            })
        })
    }

    fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let start = *start;
        let end = *end;
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT block_id, item_id, date, start_min, end_min, note
                     FROM time_block
                     WHERE account_id = $1 AND date BETWEEN $2 AND $3
                     ORDER BY date, start_min",
                )
                .bind(account_id)
                .bind(start)
                .bind(end)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter()
                    .map(row_to_core)
                    .collect::<Result<Vec<_>, _>>()
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
                     WHERE item_id = $1 AND account_id = $2
                     ORDER BY date, start_min",
                )
                .bind(item_id_val)
                .bind(account_id)
                .fetch_all(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                rows.into_iter()
                    .map(row_to_core)
                    .collect::<Result<Vec<_>, _>>()
            })
        })
    }

    fn create(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
    ) -> Result<TimeBlockCore, RepoError> {
        let date = NaiveDate::parse_from_str(&dto.date, "%Y-%m-%d")
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let item_id = dto.item_id;
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone().unwrap_or_default();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, TimeBlockRow>(
                    "INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     RETURNING block_id, item_id, date, start_min, end_min, note",
                )
                .bind(item_id)
                .bind(date)
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
        let date = dto
            .date
            .as_deref()
            .map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone();
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE time_block
                     SET date      = COALESCE($1, date),
                         start_min = COALESCE($2, start_min),
                         end_min   = COALESCE($3, end_min),
                         note      = COALESCE($4, note)
                     WHERE block_id = $5 AND account_id = $6",
                )
                .bind(date)
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
                sqlx::query("DELETE FROM time_block WHERE block_id = $1 AND account_id = $2")
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
        let date = *date;
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE i.scope_id = ANY($1) AND tb.date = $2
                     ORDER BY tb.start_min",
                )
                .bind(&scope_ids)
                .bind(date)
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
        let start = *start;
        let end = *end;
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE i.scope_id = ANY($1) AND tb.date BETWEEN $2 AND $3
                     ORDER BY tb.date, tb.start_min",
                )
                .bind(&scope_ids)
                .bind(start)
                .bind(end)
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
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, TimeBlockRow>(
                    "SELECT tb.block_id, tb.item_id, tb.date, tb.start_min, tb.end_min, tb.note
                     FROM time_block tb JOIN item i ON i.item_id = tb.item_id
                     WHERE tb.item_id = $1 AND i.scope_id = ANY($2)
                     ORDER BY tb.date, tb.start_min",
                )
                .bind(item_id_val)
                .bind(&scope_ids)
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
        let date = NaiveDate::parse_from_str(&dto.date, "%Y-%m-%d")
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let item_id = dto.item_id;
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone().unwrap_or_default();
        let account_id = i64::from(account.account_id);
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let exists: Option<i32> = sqlx::query_scalar(
                    "SELECT item_id FROM item WHERE item_id = $1 AND scope_id = ANY($2)",
                )
                .bind(item_id)
                .bind(&scope_ids)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                if exists.is_none() {
                    return Err(RepoError::NotFound);
                }
                let row = sqlx::query_as::<_, TimeBlockRow>(
                    "INSERT INTO time_block (item_id, date, start_min, end_min, note, account_id)
                     VALUES ($1, $2, $3, $4, $5, $6)
                     RETURNING block_id, item_id, date, start_min, end_min, note",
                )
                .bind(item_id)
                .bind(date)
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
        let date = dto
            .date
            .as_deref()
            .map(|value| NaiveDate::parse_from_str(value, "%Y-%m-%d"))
            .transpose()
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let start_min = dto.start_min;
        let end_min = dto.end_min;
        let note = dto.note.clone();
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE time_block SET date = COALESCE($1, date),
                     start_min = COALESCE($2, start_min), end_min = COALESCE($3, end_min),
                     note = COALESCE($4, note)
                     WHERE block_id = $5 AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id = ANY($6))",
                )
                .bind(date)
                .bind(start_min)
                .bind(end_min)
                .bind(note)
                .bind(block_id)
                .bind(&scope_ids)
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
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM time_block WHERE block_id = $1 AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id = ANY($2))",
                )
                .bind(block_id_val)
                .bind(&scope_ids)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
