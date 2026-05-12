use std::collections::HashMap;

use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, repeat::RepeatRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    repeat::{RepeatEntryCore, RepeatStatus, UpdateRepeatEntryDto},
};

#[derive(Debug)]
pub struct RepeatPostgresRepo {
    pool: PgPool,
}

impl RepeatPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RepeatItemRow {
    item_id: i32,
}

#[derive(sqlx::FromRow)]
struct RepeatEntryRow {
    item_id: i32,
    status: String,
    comment: Option<String>,
}

impl RepeatRepo for RepeatPostgresRepo {
    fn get_for_day(
        &self,
        day: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntryCore>, RepoError> {
        let day = *day;
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                // Position in the Schedule string: Sun=1, Mon=2, ..., Sat=7.
                let weekday_pos = day.weekday().number_from_sunday() as i32;

                // Fetch all Repeat-typed items for this account that are scheduled
                // on this weekday and whose End Date, if set, has not passed.
                // Postgres: value_date is timestamptz; cast to date for comparison.
                let item_rows = sqlx::query_as::<_, RepeatItemRow>(
                    "SELECT DISTINCT i.item_id
                     FROM item i
                     JOIN item_item_type_link lnk ON lnk.item_id = i.item_id
                     JOIN item_type it ON it.type_id = lnk.type_id
                     JOIN attribute sched ON sched.item_id = i.item_id
                         AND sched.key = 'Schedule'
                     WHERE i.account_id = $1
                       AND it.name = 'Repeat'
                       AND SUBSTR(sched.value_text, $2, 1) = '1'
                       AND NOT EXISTS (
                           SELECT 1 FROM attribute ed
                           WHERE ed.item_id = i.item_id
                             AND ed.key = 'End Date'
                             AND ed.value_date::date < $3
                       )",
                )
                .bind(account_id_val)
                .bind(weekday_pos)
                .bind(day)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                if item_rows.is_empty() {
                    return Ok(Vec::new());
                }

                let item_ids: Vec<i64> = item_rows.iter().map(|r| r.item_id as i64).collect();

                // Batch-fetch existing repeat_entry records for these items on this date.
                let placeholders = (1..=item_ids.len())
                    .map(|i| format!("${}", i + 2))
                    .collect::<Vec<_>>()
                    .join(", ");
                let entry_sql = format!(
                    "SELECT re.item_id, re.status::text, re.comment
                     FROM repeat_entry re
                     JOIN item i ON i.item_id = re.item_id
                     WHERE re.date = $1
                       AND i.account_id = $2
                       AND re.item_id IN ({placeholders})"
                );

                let mut entry_query = sqlx::query_as::<_, RepeatEntryRow>(&entry_sql)
                    .bind(day)
                    .bind(account_id_val);
                for id in &item_ids {
                    entry_query = entry_query.bind(*id);
                }
                let entry_rows = entry_query.fetch_all(&pool).await.map_err(RepoError::from)?;

                let mut entry_map: HashMap<i64, RepeatEntryRow> =
                    entry_rows.into_iter().map(|r| (r.item_id as i64, r)).collect();

                item_ids
                    .into_iter()
                    .map(|item_id_val| {
                        let item_id = Id::try_from(item_id_val)
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                        let (status, comment) = match entry_map.remove(&item_id_val) {
                            Some(entry) => {
                                let st = RepeatStatus::try_from(entry.status.as_str())
                                    .map_err(|e| RepoError::DatabaseError { err: e })?;
                                (st, entry.comment.unwrap_or_default())
                            }
                            None => (RepeatStatus::NotComplete, String::new()),
                        };

                        Ok(RepeatEntryCore { item_id, status, date: day, comment })
                    })
                    .collect()
            })
        })
    }

    fn set_status(&self, dto: &UpdateRepeatEntryDto, account: &Account) -> Result<(), RepoError> {
        let item_id_val = dto.item_id;
        let date_str = dto.date.clone();
        let account_id_val = i64::from(account.account_id);
        let status = dto.status.clone();
        let comment = dto.comment.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d").map_err(|e| {
                    RepoError::DatabaseError { err: format!("invalid date '{}': {}", date_str, e) }
                })?;

                let exists: Option<i64> = sqlx::query_scalar(
                    "SELECT item_id FROM item WHERE item_id = $1 AND account_id = $2",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                if exists.is_none() {
                    return Err(RepoError::NotFound);
                }

                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query("DELETE FROM repeat_entry WHERE item_id = $1 AND date = $2")
                    .bind(item_id_val)
                    .bind(date)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                let status_str = status.as_deref().unwrap_or("Not Complete");
                let repeat_status = RepeatStatus::try_from(status_str)
                    .map_err(|e| RepoError::DatabaseError { err: e })?;

                if repeat_status != RepeatStatus::NotComplete {
                    sqlx::query(
                        "INSERT INTO repeat_entry (item_id, date, status, comment)
                         VALUES ($1, $2, $3::repeat_entry_status, $4)",
                    )
                    .bind(item_id_val)
                    .bind(date)
                    .bind(repeat_status.to_string())
                    .bind(comment.unwrap_or_default())
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }
}
