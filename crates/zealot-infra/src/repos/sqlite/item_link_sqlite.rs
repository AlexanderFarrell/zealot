use std::collections::{HashMap, HashSet};

use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_link::ItemLinkRepo};
use zealot_domain::{account::Account, common::id::Id, item::ItemLink};

#[derive(Debug)]
pub struct ItemLinkSqliteRepo {
    pool: SqlitePool,
}

impl ItemLinkSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemLinkRow {
    first_item_id: i64,
    second_item_id: i64,
    relationship: String,
}

impl ItemLinkRepo for ItemLinkSqliteRepo {
    fn get_links_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemLink>>, RepoError> {
        let item_ids = item_ids.clone();
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                if item_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let sql = format!(
                    "SELECT l.first_item_id, l.second_item_id, l.relationship
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE src.account_id = ? AND target.account_id = ? AND l.first_item_id IN ({})
                     ORDER BY l.first_item_id, l.second_item_id",
                    placeholders
                );

                let mut query = sqlx::query_as::<_, ItemLinkRow>(&sql)
                    .bind(account_id_val)
                    .bind(account_id_val);
                for item_id in &item_ids {
                    query = query.bind(i64::from(*item_id));
                }

                let rows = query.fetch_all(&pool).await.map_err(RepoError::from)?;
                let mut links_by_item: HashMap<Id, Vec<ItemLink>> = HashMap::new();
                for row in rows {
                    let first_item_id = Id::try_from(row.first_item_id).map_err(|err| {
                        RepoError::DatabaseError {
                            err: err.to_string(),
                        }
                    })?;
                    let second_item_id = Id::try_from(row.second_item_id).map_err(|err| {
                        RepoError::DatabaseError {
                            err: err.to_string(),
                        }
                    })?;
                    links_by_item
                        .entry(first_item_id)
                        .or_default()
                        .push(ItemLink {
                            other_item_id: second_item_id,
                            relationship: row.relationship,
                        });
                }

                Ok(links_by_item)
            })
        })
    }

    fn get_source_item_ids(
        &self,
        target_item_id: &Id,
        relationship: &str,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let target_item_id_val = i64::from(*target_item_id);
        let account_id_val = i64::from(*account_id);
        let relationship = relationship.to_owned();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let item_ids = sqlx::query_scalar::<_, i64>(
                    "SELECT l.first_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE target.item_id = ? AND target.account_id = ? AND src.account_id = ? AND l.relationship = ?
                     ORDER BY l.first_item_id",
                )
                .bind(target_item_id_val)
                .bind(account_id_val)
                .bind(account_id_val)
                .bind(&relationship)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                item_ids
                    .into_iter()
                    .map(|item_id| {
                        Id::try_from(item_id)
                            .map_err(|err| RepoError::DatabaseError { err: err.to_string() })
                    })
                    .collect()
            })
        })
    }

    fn get_related_item_ids(&self, item_id: &Id, account_id: &Id) -> Result<Vec<Id>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let outgoing = sqlx::query_scalar::<_, i64>(
                    "SELECT l.second_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE l.first_item_id = ? AND src.account_id = ? AND target.account_id = ?
                       AND l.relationship != 'parent'
                     ORDER BY l.second_item_id",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let incoming = sqlx::query_scalar::<_, i64>(
                    "SELECT l.first_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE l.second_item_id = ? AND src.account_id = ? AND target.account_id = ?
                       AND l.relationship != 'parent'
                     ORDER BY l.first_item_id",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut seen = HashSet::new();
                let mut related_ids = Vec::new();
                for raw_id in outgoing.into_iter().chain(incoming) {
                    let related_id =
                        Id::try_from(raw_id).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })?;
                    if seen.insert(related_id) {
                        related_ids.push(related_id);
                    }
                }

                Ok(related_ids)
            })
        })
    }

    fn replace_links_for_item(
        &self,
        item_id: &Id,
        links: &Vec<ItemLink>,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        // Deduplicate by (other_item_id, relationship).
        let mut deduped: HashMap<(Id, String), ()> = HashMap::new();
        for link in links {
            deduped.insert((link.other_item_id, link.relationship.clone()), ());
        }
        let links: Vec<ItemLink> = deduped
            .into_keys()
            .map(|(other_item_id, relationship)| ItemLink {
                other_item_id,
                relationship,
            })
            .collect();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM item WHERE item_id = ? AND account_id = ?)",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                if !exists {
                    return Err(RepoError::NotFound);
                }

                if !links.is_empty() {
                    let target_ids: Vec<i64> = links.iter().map(|link| i64::from(link.other_item_id)).collect();
                    let placeholders = target_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                    let sql = format!(
                        "SELECT COUNT(DISTINCT item_id) FROM item
                         WHERE account_id = ? AND item_id IN ({})",
                        placeholders
                    );
                    let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(account_id_val);
                    for target_id in &target_ids {
                        query = query.bind(*target_id);
                    }

                    let count = query.fetch_one(&pool).await.map_err(RepoError::from)?;
                    if count as usize != target_ids.len() {
                        return Err(RepoError::NotFound);
                    }
                }

                let mut tx = pool.begin().await.map_err(RepoError::from)?;
                sqlx::query("DELETE FROM item_item_link WHERE first_item_id = ?")
                    .bind(item_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                for link in &links {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
                         VALUES (?, ?, ?)",
                    )
                    .bind(item_id_val)
                    .bind(i64::from(link.other_item_id))
                    .bind(&link.relationship)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }

    fn replace_links_by_relationship(
        &self,
        item_id: &Id,
        relationship: &str,
        linked_ids: &[Id],
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let _account_id_val = i64::from(account.account_id);
        let relationship = relationship.to_owned();
        let target_ids: Vec<i64> = linked_ids.iter().map(|id| i64::from(*id)).collect();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query(
                    "DELETE FROM item_item_link WHERE first_item_id = ? AND relationship = ?",
                )
                .bind(item_id_val)
                .bind(&relationship)
                .execute(&mut *tx)
                .await
                .map_err(RepoError::from)?;

                for &target_id in &target_ids {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_item_link (first_item_id, second_item_id, relationship)
                         VALUES (?, ?, ?)",
                    )
                    .bind(item_id_val)
                    .bind(target_id)
                    .bind(&relationship)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }
}
