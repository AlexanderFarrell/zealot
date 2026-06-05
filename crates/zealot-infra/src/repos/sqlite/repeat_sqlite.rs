use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, repeat::RepeatRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    repeat::{RepeatEntryCore, RepeatStatus, UpdateRepeatEntryDto},
};

#[derive(Debug)]
pub struct RepeatSqliteRepo {
    pool: SqlitePool,
}

impl RepeatSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RepeatItemRow {
    item_id: i64,
}

#[derive(sqlx::FromRow)]
struct RepeatEntryRow {
    item_id: i64,
    status: String,
    comment: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RepeatItemScheduleRow {
    item_id: i64,
    schedule: String,
    end_date: Option<i64>, // unix seconds (days * 86400)
}

#[derive(sqlx::FromRow)]
struct RepeatEntryWithDateRow {
    item_id: i64,
    date: String, // "YYYY-MM-DD"
    status: String,
    comment: Option<String>,
}

impl RepeatRepo for RepeatSqliteRepo {
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
                // chrono's number_from_sunday() returns exactly this.
                let weekday_pos = day.weekday().number_from_sunday() as i64;
                let date_str = day.format("%Y-%m-%d").to_string();

                // End Date is stored as value_date = unix_seconds (days * 86400).
                let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                let today_unix_secs = day.signed_duration_since(epoch).num_days() * 86400;

                // Fetch all Repeat-typed items for this account that are scheduled
                // on this weekday and whose End Date, if set, has not passed.
                let item_rows = sqlx::query_as::<_, RepeatItemRow>(
                    "SELECT DISTINCT i.item_id
                     FROM item i
                     JOIN item_item_type_link lnk ON lnk.item_id = i.item_id
                     JOIN item_type it ON it.type_id = lnk.type_id
                     JOIN attribute sched ON sched.item_id = i.item_id
                         AND sched.key = 'Schedule'
                     WHERE i.account_id = ?
                       AND it.name = 'Repeat'
                       AND SUBSTR(sched.value_text, ?, 1) = '1'
                       AND NOT EXISTS (
                           SELECT 1 FROM attribute ed
                           WHERE ed.item_id = i.item_id
                             AND ed.key = 'End Date'
                             AND ed.value_date < ?
                       )",
                )
                .bind(account_id_val)
                .bind(weekday_pos)
                .bind(today_unix_secs)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                if item_rows.is_empty() {
                    return Ok(Vec::new());
                }

                let item_ids: Vec<i64> = item_rows.iter().map(|r| r.item_id).collect();

                // Batch-fetch existing repeat_entry records for these items on this date.
                let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let entry_sql = format!(
                    "SELECT re.item_id, re.status, re.comment
                     FROM repeat_entry re
                     JOIN item i ON i.item_id = re.item_id
                     WHERE re.date = ?
                       AND i.account_id = ?
                       AND re.item_id IN ({placeholders})"
                );

                let mut entry_query = sqlx::query_as::<_, RepeatEntryRow>(&entry_sql)
                    .bind(&date_str)
                    .bind(account_id_val);
                for id in &item_ids {
                    entry_query = entry_query.bind(*id);
                }
                let entry_rows = entry_query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;

                let mut entry_map: HashMap<i64, RepeatEntryRow> =
                    entry_rows.into_iter().map(|r| (r.item_id, r)).collect();

                // Build one RepeatEntryCore per scheduled item.
                // Items with no entry in repeat_entry default to NotComplete.
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

                        Ok(RepeatEntryCore {
                            item_id,
                            status,
                            date: day,
                            comment,
                        })
                    })
                    .collect()
            })
        })
    }

    fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntryCore>, RepoError> {
        let start = *start;
        let end = *end;
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                let start_unix_secs = start.signed_duration_since(epoch).num_days() * 86400;
                let start_str = start.format("%Y-%m-%d").to_string();
                let end_str = end.format("%Y-%m-%d").to_string();

                // Query 1: all active Repeat items with schedule and optional end date.
                // Excludes items whose End Date (stored as unix_secs) is before start.
                let item_rows = sqlx::query_as::<_, RepeatItemScheduleRow>(
                    "SELECT DISTINCT i.item_id, sched.value_text AS schedule,
                            ed.value_date AS end_date
                     FROM item i
                     JOIN item_item_type_link lnk ON lnk.item_id = i.item_id
                     JOIN item_type it ON it.type_id = lnk.type_id
                     JOIN attribute sched ON sched.item_id = i.item_id
                         AND sched.key = 'Schedule'
                     LEFT JOIN attribute ed ON ed.item_id = i.item_id
                         AND ed.key = 'End Date'
                     WHERE i.account_id = ?
                       AND it.name = 'Repeat'
                       AND (ed.value_date IS NULL OR ed.value_date >= ?)",
                )
                .bind(account_id_val)
                .bind(start_unix_secs)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                if item_rows.is_empty() {
                    return Ok(Vec::new());
                }

                // Convert SQLite end_date (unix_secs) to NaiveDate for each item.
                let item_schedule: Vec<(i64, String, Option<NaiveDate>)> = item_rows
                    .iter()
                    .map(|r| {
                        let end_date = r.end_date.map(|secs| epoch + Duration::days(secs / 86400));
                        (r.item_id, r.schedule.clone(), end_date)
                    })
                    .collect();

                let item_ids: Vec<i64> = item_rows.iter().map(|r| r.item_id).collect();

                // Query 2: all repeat_entry rows in the date range for those items.
                let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let entry_sql = format!(
                    "SELECT re.item_id, re.date, re.status, re.comment
                     FROM repeat_entry re
                     JOIN item i ON i.item_id = re.item_id
                     WHERE re.date >= ? AND re.date <= ?
                       AND i.account_id = ?
                       AND re.item_id IN ({placeholders})"
                );
                let mut entry_query = sqlx::query_as::<_, RepeatEntryWithDateRow>(&entry_sql)
                    .bind(&start_str)
                    .bind(&end_str)
                    .bind(account_id_val);
                for id in &item_ids {
                    entry_query = entry_query.bind(*id);
                }
                let entry_rows = entry_query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;

                let mut entry_map: HashMap<(i64, NaiveDate), RepeatEntryWithDateRow> = {
                    let mut map = HashMap::new();
                    for row in entry_rows {
                        let date =
                            NaiveDate::parse_from_str(&row.date, "%Y-%m-%d").map_err(|e| {
                                RepoError::DatabaseError {
                                    err: format!("invalid date '{}': {}", row.date, e),
                                }
                            })?;
                        map.insert((row.item_id, date), row);
                    }
                    map
                };

                // Loop over each day in [start, end] and produce one core per scheduled item.
                let mut result: Vec<RepeatEntryCore> = Vec::new();
                let mut current = start;
                loop {
                    let weekday_pos = current.weekday().number_from_sunday() as usize; // 1=Sun..7=Sat
                    for (item_id_val, schedule, item_end_date) in &item_schedule {
                        let scheduled = schedule
                            .as_bytes()
                            .get(weekday_pos - 1)
                            .map(|&b| b == b'1')
                            .unwrap_or(false);
                        if !scheduled {
                            continue;
                        }
                        if let Some(end_date) = item_end_date {
                            if *end_date < current {
                                continue;
                            }
                        }
                        let item_id = Id::try_from(*item_id_val)
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                        let key = (*item_id_val, current);
                        let (status, comment) = match entry_map.remove(&key) {
                            Some(entry) => {
                                let st = RepeatStatus::try_from(entry.status.as_str())
                                    .map_err(|e| RepoError::DatabaseError { err: e })?;
                                (st, entry.comment.unwrap_or_default())
                            }
                            None => (RepeatStatus::NotComplete, String::new()),
                        };
                        result.push(RepeatEntryCore {
                            item_id,
                            status,
                            date: current,
                            comment,
                        });
                    }
                    if current >= end {
                        break;
                    }
                    current = current.succ_opt().unwrap_or(end);
                }
                Ok(result)
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
                // Verify item belongs to this account before modifying.
                let exists: Option<i64> = sqlx::query_scalar(
                    "SELECT item_id FROM item WHERE item_id = ? AND account_id = ?",
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

                // Remove any existing entry for this (item, date).
                sqlx::query("DELETE FROM repeat_entry WHERE item_id = ? AND date = ?")
                    .bind(item_id_val)
                    .bind(&date_str)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                // A missing status or "Not Complete" means no record is stored.
                let status_str = status.as_deref().unwrap_or("Not Complete");
                let repeat_status = RepeatStatus::try_from(status_str)
                    .map_err(|e| RepoError::DatabaseError { err: e })?;

                if repeat_status != RepeatStatus::NotComplete {
                    sqlx::query(
                        "INSERT INTO repeat_entry (item_id, date, status, comment)
                         VALUES (?, ?, ?, ?)",
                    )
                    .bind(item_id_val)
                    .bind(&date_str)
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
