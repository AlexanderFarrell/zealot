use std::collections::{BTreeMap, HashMap};

use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_type::ItemTypeRepo};
use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, ItemTypeRef, ItemTypeSummary, UpdateItemTypeDto},
};

#[derive(Debug)]
pub struct ItemTypeSqliteRepo {
    pool: SqlitePool,
}

impl ItemTypeSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemTypeWithRequiredRow {
    type_id: i64,
    name: String,
    description: String,
    account_id: Option<i64>,
    required_key: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ItemTypeRefRow {
    item_id: i64,
    type_id: i64,
    name: String,
    account_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct ItemTypeSummaryRow {
    type_id: i64,
    name: String,
    account_id: Option<i64>,
    required_attributes_count: i64,
    item_count: i64,
}

fn rows_to_item_types(rows: Vec<ItemTypeWithRequiredRow>) -> Result<Vec<ItemType>, RepoError> {
    let mut map: BTreeMap<i64, ItemType> = BTreeMap::new();

    for row in rows {
        let entry = map.entry(row.type_id).or_insert(ItemType {
            type_id: Id::try_from(row.type_id).map_err(|err| RepoError::DatabaseError {
                err: err.to_string(),
            })?,
            is_system: row.account_id.is_none(),
            name: row.name.clone(),
            description: row.description.clone(),
            required_attributes: Vec::new(),
        });

        if let Some(key) = row.required_key {
            entry.required_attributes.push(key);
        }
    }

    Ok(map.into_values().collect())
}

fn rows_to_item_type_summaries(
    rows: Vec<ItemTypeSummaryRow>,
) -> Result<Vec<ItemTypeSummary>, RepoError> {
    rows.into_iter()
        .map(|row| {
            Ok(ItemTypeSummary {
                type_id: Id::try_from(row.type_id).map_err(|err| RepoError::DatabaseError {
                    err: err.to_string(),
                })?,
                is_system: row.account_id.is_none(),
                name: row.name,
                required_attributes_count: row.required_attributes_count,
                item_count: row.item_count,
            })
        })
        .collect()
}

const ITEM_TYPE_SELECT: &str = "
    SELECT it.type_id, it.name, it.description, it.account_id, ak.key AS required_key
    FROM item_type it
    LEFT JOIN item_type_attribute_kind_link lnk ON it.type_id = lnk.item_type_id
    LEFT JOIN attribute_kind ak ON lnk.attribute_kind_id = ak.kind_id
";

impl ItemTypeRepo for ItemTypeSqliteRepo {
    fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE (it.account_id = ? OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;
                rows_to_item_types(rows)
            })
        })
    }

    fn get_item_type_summaries(&self, account_id: &Id) -> Result<Vec<ItemTypeSummary>, RepoError> {
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemTypeSummaryRow>(
                    "SELECT
                        it.type_id,
                        it.name,
                        it.account_id,
                        COUNT(DISTINCT lnk.attribute_kind_id) AS required_attributes_count,
                        COUNT(DISTINCT i.item_id) AS item_count
                     FROM item_type it
                     LEFT JOIN item_type_attribute_kind_link lnk ON it.type_id = lnk.item_type_id
                     LEFT JOIN item_item_type_link item_lnk ON it.type_id = item_lnk.type_id
                     LEFT JOIN item i ON i.item_id = item_lnk.item_id AND i.account_id = ?
                     WHERE (it.account_id = ? OR it.account_id IS NULL)
                     GROUP BY it.type_id, it.name, it.account_id
                     ORDER BY LOWER(it.name), it.type_id",
                )
                .bind(account_id_val)
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;

                rows_to_item_type_summaries(rows)
            })
        })
    }

    fn get_item_type(
        &self,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let type_id_val = i64::from(*item_type_id);
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE it.type_id = ? AND (it.account_id = ? OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(type_id_val)
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;
                Ok(rows_to_item_types(rows)?.into_iter().next())
            })
        })
    }

    fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = name.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE it.name = ? AND (it.account_id = ? OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(&name)
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;
                Ok(rows_to_item_types(rows)?.into_iter().next())
            })
        })
    }

    fn get_item_type_refs_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemTypeRef>>, RepoError> {
        let item_ids = item_ids.clone();
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                if item_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
                let sql = format!(
                    "SELECT lnk.item_id, it.type_id, it.name, it.account_id
                     FROM item_item_type_link lnk
                     JOIN item_type it ON it.type_id = lnk.type_id
                     WHERE lnk.item_id IN ({}) AND (it.account_id = ? OR it.account_id IS NULL)
                     ORDER BY lnk.item_id, it.type_id",
                    placeholders
                );
                let mut query = sqlx::query_as::<_, ItemTypeRefRow>(&sql);
                for item_id in &item_ids {
                    query = query.bind(i64::from(*item_id));
                }
                query = query.bind(account_id_val);
                let rows = query.fetch_all(&self.pool).await.map_err(RepoError::from)?;

                let mut refs_by_item = HashMap::new();
                for row in rows {
                    let item_id =
                        Id::try_from(row.item_id).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })?;
                    let type_id =
                        Id::try_from(row.type_id).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })?;
                    refs_by_item
                        .entry(item_id)
                        .or_insert_with(Vec::new)
                        .push(ItemTypeRef {
                            type_id,
                            is_system: row.account_id.is_none(),
                            name: row.name,
                        });
                }

                Ok(refs_by_item)
            })
        })
    }

    fn get_item_ids_for_type_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = name.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let item_ids = sqlx::query_scalar::<_, i64>(
                    "SELECT lnk.item_id
                     FROM item_item_type_link lnk
                     JOIN item_type it ON it.type_id = lnk.type_id
                     JOIN item i ON i.item_id = lnk.item_id
                     WHERE it.name = ? AND i.account_id = ? AND (it.account_id = ? OR it.account_id IS NULL)
                     ORDER BY lnk.item_id",
                )
                .bind(&name)
                .bind(account_id_val)
                .bind(account_id_val)
                .fetch_all(&self.pool)
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

    fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = dto.name.clone();
        let description = dto.description.clone();
        let required = dto.required_attributes.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let type_id: i64 = sqlx::query_scalar(
                    "INSERT INTO item_type (name, description, account_id) VALUES (?, ?, ?) RETURNING type_id",
                )
                .bind(&name)
                .bind(&description)
                .bind(account_id_val)
                .fetch_one(&self.pool)
                .await
                .map_err(RepoError::from)?;

                for key in &required {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
                         SELECT ak.kind_id, ? FROM attribute_kind ak
                         WHERE ak.key = ? AND (ak.account_id = ? OR ak.is_system = 1)",
                    )
                    .bind(type_id)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map_err(RepoError::from)?;
                }

                let type_id_domain = Id::try_from(type_id)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                Ok(Some(ItemType {
                    type_id: type_id_domain,
                    is_system: false,
                    name,
                    description,
                    required_attributes: required,
                }))
            })
        })
    }

    fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let type_id_val = dto.type_id;
        let name = dto.name.clone();
        let description = dto.description.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "UPDATE item_type
                     SET name = COALESCE(?, name),
                         description = COALESCE(?, description)
                     WHERE type_id = ? AND account_id = ?",
                )
                .bind(name)
                .bind(description)
                .bind(type_id_val)
                .bind(account_id_val)
                .execute(&self.pool)
                .await
                .map_err(RepoError::from)?;

                let type_id_domain = Id::try_from(type_id_val)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                self.get_item_type(&type_id_domain, &Id::try_from(account_id_val).unwrap())
            })
        })
    }

    fn delete_item_type(&self, item_type_id: &Id, account_id: &Id) -> Result<bool, RepoError> {
        let type_id_val = i64::from(*item_type_id);
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let result =
                    sqlx::query("DELETE FROM item_type WHERE type_id = ? AND account_id = ?")
                        .bind(type_id_val)
                        .bind(account_id_val)
                        .execute(&self.pool)
                        .await
                        .map_err(RepoError::from)?;

                Ok(result.rows_affected() > 0)
            })
        })
    }

    fn add_attr_kinds_to_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError> {
        let type_id_val = i64::from(*item_type_id);
        let account_id_val = i64::from(*account_id);
        let keys = attr_kinds.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for key in &keys {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
                         SELECT ak.kind_id, ? FROM attribute_kind ak
                         WHERE ak.key = ? AND (ak.account_id = ? OR ak.is_system = 1)",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map_err(RepoError::from)?;
                }
                Ok(())
            })
        })
    }

    fn remove_attr_kinds_from_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError> {
        let type_id_val = i64::from(*item_type_id);
        let account_id_val = i64::from(*account_id);
        let keys = attr_kinds.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for key in &keys {
                    sqlx::query(
                        "DELETE FROM item_type_attribute_kind_link
                         WHERE item_type_id = ?
                         AND attribute_kind_id IN (
                             SELECT ak.kind_id FROM attribute_kind ak
                             WHERE ak.key = ? AND (ak.account_id = ? OR ak.is_system = 1)
                         )",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map_err(RepoError::from)?;
                }
                Ok(())
            })
        })
    }

    fn assign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        let names = type_names.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for name in &names {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_item_type_link (item_id, type_id)
                         SELECT i.item_id, it.type_id
                         FROM item i
                         JOIN item_type it
                         WHERE i.item_id = ? AND i.account_id = ? AND it.name = ?
                           AND (it.account_id = ? OR it.account_id IS NULL)",
                    )
                    .bind(item_id_val)
                    .bind(account_id_val)
                    .bind(name)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map_err(RepoError::from)?;
                }
                Ok(())
            })
        })
    }

    fn unassign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        let names = type_names.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for name in &names {
                    sqlx::query(
                        "DELETE FROM item_item_type_link
                         WHERE item_id = ? AND type_id IN (
                             SELECT it.type_id
                             FROM item_type it
                             WHERE it.name = ? AND (it.account_id = ? OR it.account_id IS NULL)
                         )",
                    )
                    .bind(item_id_val)
                    .bind(name)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map_err(RepoError::from)?;
                }
                Ok(())
            })
        })
    }
}
