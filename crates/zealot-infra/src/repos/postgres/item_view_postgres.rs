use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item_view::ItemViewRepo};
use zealot_domain::common::id::Id;

#[derive(Debug)]
pub struct ItemViewPostgresRepo {
    pool: PgPool,
}

impl ItemViewPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemViewRepo for ItemViewPostgresRepo {
    fn record_view(&self, item_id: &Id) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query("INSERT INTO item_view (item_id) VALUES ($1)")
                    .bind(item_id_val)
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn get_most_viewed(&self, limit: i64, account_id: &Id) -> Result<Vec<(Id, i64)>, RepoError> {
        self.get_most_viewed_with_archived(limit, account_id, false)
    }

    fn get_most_viewed_with_archived(
        &self,
        limit: i64,
        account_id: &Id,
        include_archived: bool,
    ) -> Result<Vec<(Id, i64)>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, (i32, i64)>(
                    "SELECT iv.item_id, COUNT(*) as cnt
                     FROM item_view iv
                     INNER JOIN item i ON i.item_id = iv.item_id
                     WHERE i.account_id = $1 AND ($2 OR NOT EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1))
                     GROUP BY iv.item_id
                     ORDER BY cnt DESC
                     LIMIT $3",
                )
                .bind(account_id_val)
                .bind(include_archived)
                .bind(limit)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter()
                    .map(|(item_id, cnt)| {
                        Id::try_from(item_id as i64)
                            .map(|id| (id, cnt))
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })
                    })
                    .collect()
            })
        })
    }
}
