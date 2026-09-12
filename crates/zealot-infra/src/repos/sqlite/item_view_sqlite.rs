use sqlx::SqlitePool;
use uuid::Uuid;
use zealot_app::repos::{common::RepoError, item_view::ItemViewRepo};
use zealot_domain::common::id::Id;

#[derive(Debug)]
pub struct ItemViewSqliteRepo {
    pool: SqlitePool,
}

impl ItemViewSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ItemViewRepo for ItemViewSqliteRepo {
    fn record_view(&self, item_id: &Id) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query("INSERT INTO item_view (item_id) VALUES (?)")
                    .bind(item_id_val)
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn get_most_viewed(&self, limit: i64, account_id: &Id) -> Result<Vec<(Id, i64)>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, (i64, i64)>(
                    "SELECT iv.item_id, COUNT(*) as cnt
                     FROM item_view iv
                     INNER JOIN item i ON i.item_id = iv.item_id
                     WHERE i.account_id = ?
                     GROUP BY iv.item_id
                     ORDER BY cnt DESC
                     LIMIT ?",
                )
                .bind(account_id_val)
                .bind(limit)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter()
                    .map(|(item_id, cnt)| {
                        Id::try_from(item_id)
                            .map(|id| (id, cnt))
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })
                    })
                    .collect()
            })
        })
    }

    fn get_most_viewed_in_scopes(
        &self,
        limit: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(Id, i64)>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let scope_ids: Vec<String> = scope_ids.iter().map(Uuid::to_string).collect();
        let placeholders = scope_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let sql = format!(
            "SELECT iv.item_id, COUNT(*) as cnt
             FROM item_view iv
             INNER JOIN item i ON i.item_id = iv.item_id
             WHERE i.scope_id IN ({placeholders})
             GROUP BY iv.item_id
             ORDER BY cnt DESC
             LIMIT ?"
        );
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut query = sqlx::query_as::<_, (i64, i64)>(&sql);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                let rows = query
                    .bind(limit)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;
                rows.into_iter()
                    .map(|(item_id, count)| {
                        Id::try_from(item_id)
                            .map(|id| (id, count))
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })
                    })
                    .collect()
            })
        })
    }
}
