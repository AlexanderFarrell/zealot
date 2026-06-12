use chrono::NaiveDate;
use sqlx::SqlitePool;
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
    block_id:  i64,
    item_id:   i64,
    date:      String,
    start_min: i32,
    end_min:   i32,
    note:      String,
}

fn row_to_core(row: TimeBlockRow) -> Result<TimeBlockCore, RepoError> {
    let date = NaiveDate::parse_from_str(&row.date, "%Y-%m-%d")
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let block_id = Id::try_from(row.block_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let item_id = Id::try_from(row.item_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    Ok(TimeBlockCore {
        block_id,
        item_id,
        date,
        start_min: row.start_min,
        end_min:   row.end_min,
        note:      row.note,
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

    fn update(
        &self,
        dto: &UpdateTimeBlockDto,
        account: &Account,
    ) -> Result<(), RepoError> {
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

    fn delete(
        &self,
        block_id: Id,
        account: &Account,
    ) -> Result<(), RepoError> {
        let block_id_val = i64::from(block_id);
        let account_id = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM time_block WHERE block_id = ? AND account_id = ?",
                )
                .bind(block_id_val)
                .bind(account_id)
                .execute(&pool)
                .await
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                Ok(())
            })
        })
    }
}
