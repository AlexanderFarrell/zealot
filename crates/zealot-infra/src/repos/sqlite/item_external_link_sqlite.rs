use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_external_link::ItemExternalLinkRepo};
use zealot_domain::{common::id::Id, item::ItemExternalLink};

#[derive(Debug)]
pub struct ItemExternalLinkSqliteRepo {
    pool: SqlitePool,
}

impl ItemExternalLinkSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemExternalLinkRow {
    link_id: i64,
    item_id: i64,
    url: String,
}

fn row_to_external_link(row: ItemExternalLinkRow) -> Result<ItemExternalLink, RepoError> {
    Ok(ItemExternalLink {
        link_id: Id::try_from(row.link_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        item_id: Id::try_from(row.item_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        url: row.url,
    })
}

impl ItemExternalLinkRepo for ItemExternalLinkSqliteRepo {
    fn get_for_item(&self, item_id: &Id) -> Result<Vec<ItemExternalLink>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemExternalLinkRow>(
                    "SELECT link_id, item_id, url
                     FROM item_external_link
                     WHERE item_id = ?
                     ORDER BY link_id ASC",
                )
                .bind(item_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_external_link).collect()
            })
        })
    }

    fn replace_for_item(&self, item_id: &Id, urls: &[String]) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let urls = urls.to_vec();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query("DELETE FROM item_external_link WHERE item_id = ?")
                    .bind(item_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                for url in &urls {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_external_link (item_id, url) VALUES (?, ?)",
                    )
                    .bind(item_id_val)
                    .bind(url)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }
}
