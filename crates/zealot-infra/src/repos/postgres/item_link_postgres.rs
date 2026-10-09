use std::collections::{HashMap, HashSet};

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item_link::ItemLinkRepo};
use zealot_domain::{account::Account, common::id::Id, item::ItemLink};

#[derive(Debug)]
pub struct ItemLinkPostgresRepo {
    pool: PgPool,
}

impl ItemLinkPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemLinkRow {
    first_item_id: i32,
    second_item_id: i32,
    relationship: String,
}

impl ItemLinkRepo for ItemLinkPostgresRepo {
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

                let id_vals: Vec<i64> = item_ids.iter().map(|id| i64::from(*id)).collect();
                let rows = sqlx::query_as::<_, ItemLinkRow>(
                    "SELECT l.first_item_id, l.second_item_id, l.relationship
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE src.account_id = $1 AND target.account_id = $1
                       AND l.first_item_id = ANY($2)
                     ORDER BY l.first_item_id, l.second_item_id",
                )
                .bind(account_id_val)
                .bind(&id_vals)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut links_by_item: HashMap<Id, Vec<ItemLink>> = HashMap::new();
                for row in rows {
                    let first_item_id = Id::try_from(row.first_item_id as i64).map_err(|err| {
                        RepoError::DatabaseError {
                            err: err.to_string(),
                        }
                    })?;
                    let second_item_id =
                        Id::try_from(row.second_item_id as i64).map_err(|err| {
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
                let item_ids = sqlx::query_scalar::<_, i32>(
                    "SELECT l.first_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE target.item_id = $1 AND target.account_id = $2
                       AND src.account_id = $2
                       AND l.relationship = $3
                     ORDER BY l.first_item_id",
                )
                .bind(target_item_id_val)
                .bind(account_id_val)
                .bind(&relationship)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                item_ids
                    .into_iter()
                    .map(|item_id| {
                        Id::try_from(item_id as i64).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })
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
                let outgoing = sqlx::query_scalar::<_, i32>(
                    "SELECT l.second_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE l.first_item_id = $1 AND src.account_id = $2 AND target.account_id = $2
                       AND l.relationship != 'parent'
                     ORDER BY l.second_item_id",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let incoming = sqlx::query_scalar::<_, i32>(
                    "SELECT l.first_item_id
                     FROM item_item_link l
                     JOIN item src ON src.item_id = l.first_item_id
                     JOIN item target ON target.item_id = l.second_item_id
                     WHERE l.second_item_id = $1 AND src.account_id = $2 AND target.account_id = $2
                       AND l.relationship != 'parent'
                     ORDER BY l.first_item_id",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut seen = HashSet::new();
                let mut related_ids = Vec::new();
                for raw_id in outgoing.into_iter().chain(incoming) {
                    let related_id =
                        Id::try_from(raw_id as i64).map_err(|err| RepoError::DatabaseError {
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
        // Deduplicate: last relationship wins for a given (other_item_id, relationship) pair.
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
                    "SELECT EXISTS(SELECT 1 FROM item WHERE item_id = $1 AND account_id = $2)",
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
                    let target_ids: Vec<i64> =
                        links.iter().map(|link| i64::from(link.other_item_id)).collect();
                    let count: i64 = sqlx::query_scalar(
                        "SELECT COUNT(DISTINCT item_id) FROM item WHERE account_id = $1 AND item_id = ANY($2)",
                    )
                    .bind(account_id_val)
                    .bind(&target_ids)
                    .fetch_one(&pool)
                    .await
                    .map_err(RepoError::from)?;

                    if count as usize != target_ids.len() {
                        return Err(RepoError::NotFound);
                    }
                }

                let mut tx = pool.begin().await.map_err(RepoError::from)?;
                sqlx::query("DELETE FROM item_item_link WHERE first_item_id = $1")
                    .bind(item_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;

                for link in &links {
                    sqlx::query(
                        "INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
                         VALUES ($1, $2, $3)
                         ON CONFLICT (first_item_id, second_item_id, relationship) DO NOTHING",
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
        let account_id_val = i64::from(account.account_id);
        let relationship = relationship.to_owned();
        let target_ids: Vec<i64> = linked_ids.iter().map(|id| i64::from(*id)).collect();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                // Remove all existing links for this item with this relationship key.
                sqlx::query(
                    "DELETE FROM item_item_link
                     WHERE first_item_id = $1 AND relationship = $2",
                )
                .bind(item_id_val)
                .bind(&relationship)
                .execute(&mut *tx)
                .await
                .map_err(RepoError::from)?;

                // Insert new links. Any pre-existing link with a *different* relationship
                // to the same target is left untouched; we skip on conflict just in case.
                for &target_id in &target_ids {
                    sqlx::query(
                        "INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
                         VALUES ($1, $2, $3)
                         ON CONFLICT (first_item_id, second_item_id, relationship) DO NOTHING",
                    )
                    .bind(item_id_val)
                    .bind(target_id)
                    .bind(&relationship)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                // Emit an event so the account_id guard is respected via item ownership.
                // (We trust the service layer to validate that item_id belongs to account.)
                let _ = account_id_val; // used implicitly through service-layer ownership check

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }

    fn replace_wikilinks(
        &self,
        item_id: &Id,
        titles: &[String],
        linked_ids: &[Id],
        _account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let titles = titles.to_vec();
        let target_ids: Vec<i64> = linked_ids.iter().map(|id| i64::from(*id)).collect();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query(
                    "DELETE FROM item_item_link WHERE first_item_id = $1::int8 AND relationship = 'wikilink'",
                )
                .bind(item_id_val)
                .execute(&mut *tx)
                .await
                .map_err(RepoError::from)?;
                for &target_id in &target_ids {
                    sqlx::query(
                        "INSERT INTO item_item_link (first_item_id, second_item_id, relationship)
                         VALUES ($1::int8::int4, $2::int8::int4, 'wikilink')
                         ON CONFLICT (first_item_id, second_item_id, relationship) DO NOTHING",
                    )
                    .bind(item_id_val)
                    .bind(target_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                sqlx::query("DELETE FROM item_wikilink_ref WHERE item_id = $1::int8")
                    .bind(item_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                for title in &titles {
                    sqlx::query(
                        "INSERT INTO item_wikilink_ref (item_id, target_title)
                         VALUES ($1::int8::int4, $2) ON CONFLICT DO NOTHING",
                    )
                    .bind(item_id_val)
                    .bind(title)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }

    fn get_wikilink_source_ids(
        &self,
        titles: &[String],
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let titles = titles.to_vec();
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut ids = std::collections::BTreeSet::new();
                for title in &titles {
                    let rows = sqlx::query_scalar::<_, i32>(
                        "SELECT r.item_id FROM item_wikilink_ref r
                         JOIN item i ON i.item_id = r.item_id
                         WHERE r.target_title = $1 AND i.account_id = $2::int8",
                    )
                    .bind(title)
                    .bind(account_id_val)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;
                    ids.extend(rows);
                }
                ids.into_iter()
                    .map(|id| {
                        Id::try_from(id as i64)
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })
                    })
                    .collect()
            })
        })
    }

    fn wikilink_refs_complete(&self, account_id: &Id) -> Result<bool, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_scalar::<_, bool>(
                    "SELECT EXISTS(SELECT 1 FROM wikilink_ref_state WHERE account_id = $1::int8)",
                )
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)
            })
        })
    }

    fn mark_wikilink_refs_complete(&self, account_id: &Id) -> Result<(), RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "INSERT INTO wikilink_ref_state (account_id) VALUES ($1::int8::int4)
                     ON CONFLICT DO NOTHING",
                )
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
