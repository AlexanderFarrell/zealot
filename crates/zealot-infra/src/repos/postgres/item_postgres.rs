use std::collections::HashMap;

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item::ItemRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
};

#[derive(Debug)]
pub struct ItemPostgresRepo {
    pool: PgPool,
}

impl ItemPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemRow {
    item_id: i32,
    title: String,
    content: String,
}

fn row_to_item_core(row: ItemRow) -> Result<ItemCore, RepoError> {
    Ok(ItemCore {
        item_id: Id::try_from(row.item_id as i64)
            .map_err(|err| RepoError::DatabaseError { err: err.to_string() })?,
        title: row.title,
        content: row.content,
    })
}

async fn fetch_items_by_ids(
    item_ids: &[Id],
    account_id: i64,
    pool: &PgPool,
) -> Result<Vec<ItemCore>, RepoError> {
    if item_ids.is_empty() {
        return Ok(Vec::new());
    }

    let id_vals: Vec<i64> = item_ids.iter().map(|id| i64::from(*id)).collect();
    let rows = sqlx::query_as::<_, ItemRow>(
        "SELECT item_id, title, content FROM item
         WHERE account_id = $1 AND item_id = ANY($2)",
    )
    .bind(account_id)
    .bind(&id_vals)
    .fetch_all(pool)
    .await
    .map_err(RepoError::from)?;

    let mut items_by_id: HashMap<i64, ItemCore> = HashMap::new();
    for row in rows {
        items_by_id.insert(row.item_id as i64, row_to_item_core(row)?);
    }

    Ok(item_ids
        .iter()
        .filter_map(|item_id| items_by_id.remove(&i64::from(*item_id)))
        .collect())
}

impl ItemRepo for ItemPostgresRepo {
    fn get_item_by_id(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE item_id = $1 AND account_id = $2",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                row.map(row_to_item_core).transpose()
            })
        })
    }

    fn get_items_by_ids(
        &self,
        item_ids: &Vec<Id>,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError> {
        let pool = self.pool.clone();
        let item_ids = item_ids.clone();
        let account_id_val = i64::from(account.account_id);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async move { fetch_items_by_ids(&item_ids, account_id_val, &pool).await })
        })
    }

    fn get_items_by_title(
        &self,
        title: &str,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError> {
        let title = title.to_string();
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE title = $1 AND account_id = $2",
                )
                .bind(&title)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_item_core).collect()
            })
        })
    }

    fn search_items_by_title(
        &self,
        term: &str,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError> {
        let pattern = if term.trim().is_empty() {
            String::from("%")
        } else {
            format!("%{}%", term)
        };
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE title ILIKE $1 AND account_id = $2
                     ORDER BY item_id DESC LIMIT 20",
                )
                .bind(&pattern)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_item_core).collect()
            })
        })
    }

    fn regex_items_by_title(
        &self,
        term: &str,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError> {
        let term = term.to_string();
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE title ~* $1 AND account_id = $2
                     ORDER BY item_id DESC LIMIT 20",
                )
                .bind(&term)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_item_core).collect()
            })
        })
    }

    fn get_recent_items(&self, limit: i64, offset: i64, account: &Account) -> Result<Vec<ItemCore>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE account_id = $1
                     ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                )
                .bind(account_id_val)
                .bind(limit)
                .bind(offset)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                rows.into_iter().map(row_to_item_core).collect()
            })
        })
    }

    fn add_item(
        &self,
        dto: &AddItemCoreDto,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError> {
        let title = dto.title.clone();
        let content = dto.content.clone();
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, ItemRow>(
                    "INSERT INTO item (title, content, account_id)
                     VALUES ($1, $2, $3)
                     RETURNING item_id, title, content",
                )
                .bind(&title)
                .bind(&content)
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                Ok(Some(row_to_item_core(row)?))
            })
        })
    }

    fn update_item(
        &self,
        dto: &UpdateItemCoreDto,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError> {
        let title = dto.title.clone();
        let content = dto.content.clone();
        let item_id_val = i64::from(dto.item_id);
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE item SET
                       title = COALESCE($1, title),
                       content = COALESCE($2, content),
                       updated_at = now()
                     WHERE item_id = $3 AND account_id = $4",
                )
                .bind(title)
                .bind(content)
                .bind(item_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                let row = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content FROM item
                     WHERE item_id = $1 AND account_id = $2",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                row.map(row_to_item_core).transpose()
            })
        })
    }

    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query("DELETE FROM item WHERE item_id = $1 AND account_id = $2")
                    .bind(item_id_val)
                    .bind(account_id_val)
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }
}
