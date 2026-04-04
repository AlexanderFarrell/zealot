use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use zealot_app::repos::{comment::CommentRepo, common::RepoError};
use zealot_domain::{
    comment::{AddCommentDto, CommentCore, UpdateCommentDto},
    common::id::Id,
};

#[derive(Debug)]
pub struct CommentSqliteRepo {
    pool: SqlitePool,
}

impl CommentSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CommentRow {
    comment_id: i64,
    item_id: i64,
    time: i64,
    content: String,
}

fn row_to_comment_core(row: CommentRow) -> Result<CommentCore, RepoError> {
    let comment_id = Id::try_from(row.comment_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let item_id = Id::try_from(row.item_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let timestamp = NaiveDateTime::from_timestamp_opt(row.time, 0).ok_or_else(|| {
        RepoError::DatabaseError {
            err: format!("invalid unix timestamp: {}", row.time),
        }
    })?;
    Ok(CommentCore { comment_id, item_id, timestamp, content: row.content })
}

fn parse_timestamp(s: &str) -> Result<i64, RepoError> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc().timestamp())
        .map_err(|e| RepoError::DatabaseError { err: format!("invalid timestamp '{}': {}", s, e) })
}

impl CommentRepo for CommentSqliteRepo {
    fn get_for_day(
        &self,
        day: &chrono::NaiveDate,
        account_id: &Id,
    ) -> Result<Vec<CommentCore>, RepoError> {
        let day_start = day.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let day_end = day.succ_opt().unwrap().and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c
                     JOIN item i ON i.item_id = c.item_id
                     WHERE i.account_id = ?
                       AND c.time >= ? AND c.time < ?
                     ORDER BY c.time ASC",
                )
                .bind(account_id_val)
                .bind(day_start)
                .bind(day_end)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_comment_core).collect()
            })
        })
    }

    fn get_for_item(
        &self,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<Vec<CommentCore>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c
                     JOIN item i ON i.item_id = c.item_id
                     WHERE c.item_id = ? AND i.account_id = ?
                     ORDER BY c.time ASC",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_comment_core).collect()
            })
        })
    }

    fn add_comment(
        &self,
        dto: &AddCommentDto,
        account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError> {
        let item_id_val = dto.item_id;
        let account_id_val = i64::from(*account_id);
        let timestamp_unix = parse_timestamp(&dto.timestamp)?;
        let content = dto.content.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                // Verify the item belongs to this account before inserting.
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

                let row = sqlx::query_as::<_, CommentRow>(
                    "INSERT INTO comment (item_id, time, content)
                     VALUES (?, ?, ?)
                     RETURNING comment_id, item_id, time, content",
                )
                .bind(item_id_val)
                .bind(timestamp_unix)
                .bind(&content)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                Ok(Some(row_to_comment_core(row)?))
            })
        })
    }

    fn update_comment(
        &self,
        dto: &UpdateCommentDto,
        account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError> {
        let comment_id_val = dto.comment_id;
        let account_id_val = i64::from(*account_id);
        let new_timestamp = dto
            .timestamp
            .as_deref()
            .map(parse_timestamp)
            .transpose()?;
        let new_content = dto.content.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE comment
                     SET time = COALESCE(?, time),
                         content = COALESCE(?, content),
                         last_updated = strftime('%s', 'now')
                     WHERE comment_id = ?
                       AND item_id IN (SELECT item_id FROM item WHERE account_id = ?)",
                )
                .bind(new_timestamp)
                .bind(new_content)
                .bind(comment_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                let row = sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c
                     JOIN item i ON i.item_id = c.item_id
                     WHERE c.comment_id = ? AND i.account_id = ?",
                )
                .bind(comment_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                row.map(row_to_comment_core).transpose()
            })
        })
    }

    fn delete_comment(
        &self,
        comment_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError> {
        let comment_id_val = i64::from(*comment_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM comment
                     WHERE comment_id = ?
                       AND item_id IN (SELECT item_id FROM item WHERE account_id = ?)",
                )
                .bind(comment_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
