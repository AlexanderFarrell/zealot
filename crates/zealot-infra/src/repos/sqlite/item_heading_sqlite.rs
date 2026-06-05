use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_heading::ItemHeadingRepo};
use zealot_domain::{common::id::Id, item::ItemHeading};

#[derive(Debug)]
pub struct ItemHeadingSqliteRepo {
    pool: SqlitePool,
}

impl ItemHeadingSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemHeadingRow {
    heading_id: i64,
    item_id: i64,
    level: i64,
    ordinal: i64,
    text: String,
}

fn row_to_heading(row: ItemHeadingRow) -> Result<ItemHeading, RepoError> {
    Ok(ItemHeading {
        heading_id: Id::try_from(row.heading_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        item_id: Id::try_from(row.item_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?,
        level: row.level as u8,
        ordinal: row.ordinal as u32,
        text: row.text,
    })
}

impl ItemHeadingRepo for ItemHeadingSqliteRepo {
    fn get_for_item(&self, item_id: &Id) -> Result<Vec<ItemHeading>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemHeadingRow>(
                    "SELECT heading_id, item_id, level, ordinal, text
                     FROM item_heading
                     WHERE item_id = ?
                     ORDER BY ordinal ASC",
                )
                .bind(item_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_heading).collect()
            })
        })
    }

    fn replace_for_item(&self, item_id: &Id, headings: &[(u8, u32, String)]) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let headings = headings.to_vec();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query("DELETE FROM item_heading WHERE item_id = ?")
                    .bind(item_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                for (level, ordinal, text) in &headings {
                    sqlx::query(
                        "INSERT INTO item_heading (item_id, level, ordinal, text) VALUES (?, ?, ?, ?)",
                    )
                    .bind(item_id_val)
                    .bind(*level as i64)
                    .bind(*ordinal as i64)
                    .bind(text)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }
}
