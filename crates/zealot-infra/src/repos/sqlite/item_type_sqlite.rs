use std::collections::BTreeMap;

use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_type::ItemTypeRepo};
use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, UpdateItemTypeDto},
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

fn rows_to_item_types(rows: Vec<ItemTypeWithRequiredRow>) -> Result<Vec<ItemType>, RepoError> {
    let mut map: BTreeMap<i64, ItemType> = BTreeMap::new();

    for row in rows {
        let entry = map.entry(row.type_id).or_insert_with(|| ItemType {
            type_id: Id::try_from(row.type_id).unwrap_or(Id::try_from(0).unwrap()),
            is_system: row.account_id.is_none(),
            name: row.name,
            description: row.description,
            required_attributes: Vec::new(),
        });

        if let Some(key) = row.required_key {
            entry.required_attributes.push(key);
        }
    }

    Ok(map.into_values().collect())
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

    fn get_item_type(&self, item_type_id: &Id, account_id: &Id) -> Result<Option<ItemType>, RepoError> {
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

    fn get_item_type_by_name(&self, name: &str, account_id: &Id) -> Result<Option<ItemType>, RepoError> {
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

    fn get_item_types_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<ItemType>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} INNER JOIN item_item_type_link iitl ON iitl.type_id = it.type_id
                     WHERE iitl.item_id = ? AND (it.account_id = ? OR it.account_id IS NULL)
                     ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;
                rows_to_item_types(rows)
            })
        })
    }

    fn add_item_type(&self, dto: &AddItemTypeDto, account_id: &Id) -> Result<Option<ItemType>, RepoError> {
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

    fn update_item_type(&self, dto: &UpdateItemTypeDto, account_id: &Id) -> Result<Option<ItemType>, RepoError> {
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
}
