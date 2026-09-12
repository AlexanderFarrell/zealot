use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use zealot_app::repos::{comment::CommentRepo, common::RepoError};
use zealot_domain::comment::{AddCommentDto, CommentCore, UpdateCommentDto};
use zealot_domain::common::id::Id;

#[derive(Debug)]
pub struct CommentPostgresRepo {
    pool: PgPool,
}

impl CommentPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct CommentRow {
    comment_id: i64,
    item_id: i32,
    time: DateTime<Utc>,
    content: String,
}

fn row_to_comment_core(row: CommentRow) -> Result<CommentCore, RepoError> {
    let comment_id = Id::try_from(row.comment_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let item_id = Id::try_from(row.item_id as i64)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    Ok(CommentCore {
        comment_id,
        item_id,
        timestamp: row.time.naive_utc(),
        content: row.content,
    })
}

fn parse_timestamp(s: &str) -> Result<DateTime<Utc>, RepoError> {
    NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
        .map(|dt| dt.and_utc())
        .map_err(|e| RepoError::DatabaseError {
            err: format!("invalid timestamp '{}': {}", s, e),
        })
}

impl CommentRepo for CommentPostgresRepo {
    fn get_for_day(
        &self,
        day: &sqlx::types::chrono::NaiveDate,
        account_id: &Id,
    ) -> Result<Vec<CommentCore>, RepoError> {
        let day_start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day
            .succ_opt()
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c
                     JOIN item i ON i.item_id = c.item_id
                     WHERE i.account_id = $1
                       AND c.time >= $2 AND c.time < $3
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

    fn get_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<CommentCore>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c
                     JOIN item i ON i.item_id = c.item_id
                     WHERE c.item_id = $1 AND i.account_id = $2
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
        let timestamp = parse_timestamp(&dto.timestamp)?;
        let content = dto.content.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let exists: Option<i32> = sqlx::query_scalar(
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

                let row = sqlx::query_as::<_, CommentRow>(
                    "INSERT INTO comment (item_id, time, content)
                     VALUES ($1, $2, $3)
                     RETURNING comment_id, item_id, time, content",
                )
                .bind(item_id_val)
                .bind(timestamp)
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
        let new_timestamp = dto.timestamp.as_deref().map(parse_timestamp).transpose()?;
        let new_content = dto.content.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE comment
                     SET time = COALESCE($1, time),
                         content = COALESCE($2, content),
                         last_updated = now()
                     WHERE comment_id = $3
                       AND item_id IN (SELECT item_id FROM item WHERE account_id = $4)",
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
                     WHERE c.comment_id = $1 AND i.account_id = $2",
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

    fn delete_comment(&self, comment_id: &Id, account_id: &Id) -> Result<(), RepoError> {
        let comment_id_val = i64::from(*comment_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM comment
                     WHERE comment_id = $1
                       AND item_id IN (SELECT item_id FROM item WHERE account_id = $2)",
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

    fn get_for_day_in_scopes(
        &self,
        day: &sqlx::types::chrono::NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<CommentCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let day_start = day.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let day_end = day
            .succ_opt()
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c JOIN item i ON i.item_id = c.item_id
                     WHERE i.scope_id = ANY($1) AND c.time >= $2 AND c.time < $3
                     ORDER BY c.time ASC",
                )
                .bind(&scope_ids)
                .bind(day_start)
                .bind(day_end)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?
                .into_iter()
                .map(row_to_comment_core)
                .collect()
            })
        })
    }

    fn get_for_item_in_scopes(
        &self,
        item_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<Vec<CommentCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let item_id_val = i64::from(*item_id);
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c JOIN item i ON i.item_id = c.item_id
                     WHERE c.item_id = $1 AND i.scope_id = ANY($2)
                     ORDER BY c.time ASC",
                )
                .bind(item_id_val)
                .bind(&scope_ids)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?
                .into_iter()
                .map(row_to_comment_core)
                .collect()
            })
        })
    }

    fn add_comment_in_scopes(
        &self,
        dto: &AddCommentDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<CommentCore>, RepoError> {
        if scope_ids.is_empty() {
            return Err(RepoError::NotFound);
        }
        let item_id_val = dto.item_id;
        let timestamp = parse_timestamp(&dto.timestamp)?;
        let content = dto.content.clone();
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
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
                let row = sqlx::query_as::<_, CommentRow>(
                    "INSERT INTO comment (item_id, time, content)
                     VALUES ($1, $2, $3) RETURNING comment_id, item_id, time, content",
                )
                .bind(item_id_val)
                .bind(timestamp)
                .bind(&content)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;
                Ok(Some(row_to_comment_core(row)?))
            })
        })
    }

    fn update_comment_in_scopes(
        &self,
        dto: &UpdateCommentDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<CommentCore>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        let comment_id_val = dto.comment_id;
        let new_timestamp = dto.timestamp.as_deref().map(parse_timestamp).transpose()?;
        let new_content = dto.content.clone();
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE comment SET time = COALESCE($1, time), content = COALESCE($2, content),
                     last_updated = now()
                     WHERE comment_id = $3 AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id = ANY($4))",
                )
                .bind(new_timestamp)
                .bind(new_content)
                .bind(comment_id_val)
                .bind(&scope_ids)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;
                sqlx::query_as::<_, CommentRow>(
                    "SELECT c.comment_id, c.item_id, c.time, c.content
                     FROM comment c JOIN item i ON i.item_id = c.item_id
                     WHERE c.comment_id = $1 AND i.scope_id = ANY($2)",
                )
                .bind(comment_id_val)
                .bind(&scope_ids)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_comment_core)
                .transpose()
            })
        })
    }

    fn delete_comment_in_scopes(
        &self,
        comment_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<(), RepoError> {
        if scope_ids.is_empty() {
            return Ok(());
        }
        let comment_id_val = i64::from(*comment_id);
        let scope_ids = scope_ids.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM comment WHERE comment_id = $1 AND item_id IN
                     (SELECT item_id FROM item WHERE scope_id = ANY($2))",
                )
                .bind(comment_id_val)
                .bind(&scope_ids)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
