use std::collections::{BTreeMap, HashMap};

use sqlx::PgPool;
use zealot_app::repos::{
    common::RepoError,
    item_type::{ItemTypeRepo, MissingGroup, Offender, RequiredChange},
};
use zealot_domain::{
    attribute::Attribute,
    common::id::Id,
    item_type::{
        AddItemTypeDto, ItemType, ItemTypeRef, ItemTypeSummary, NonCompliantItemDto,
        NonCompliantPageDto, UpdateItemTypeDto,
    },
};

use super::item_attribute_value_postgres::scalar_to_cols;
use crate::repos::override_sets;

#[derive(Debug)]
pub struct ItemTypePostgresRepo {
    pool: PgPool,
}

impl ItemTypePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct ItemTypeWithRequiredRow {
    type_id: i32,
    name: String,
    description: String,
    account_id: Option<i32>,
    required_key: Option<String>,
    required_default: Option<String>,
}

#[derive(sqlx::FromRow)]
struct ItemTypeRefRow {
    item_id: i32,
    type_id: i32,
    name: String,
    account_id: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct ItemTypeSummaryRow {
    type_id: i32,
    name: String,
    account_id: Option<i32>,
    required_attributes_count: i64,
    item_count: i64,
}

fn rows_to_item_types(rows: Vec<ItemTypeWithRequiredRow>) -> Result<Vec<ItemType>, RepoError> {
    let mut map: BTreeMap<i32, ItemType> = BTreeMap::new();

    for row in rows {
        let entry = map.entry(row.type_id).or_insert(ItemType {
            type_id: Id::try_from(row.type_id as i64).map_err(|err| RepoError::DatabaseError {
                err: err.to_string(),
            })?,
            is_system: row.account_id.is_none(),
            name: row.name.clone(),
            description: row.description.clone(),
            required_attributes: Vec::new(),
            required_defaults: BTreeMap::new(),
        });

        if let Some(key) = row.required_key {
            if let Some(raw) = &row.required_default {
                let value = serde_json::from_str(raw)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                entry.required_defaults.insert(key.clone(), value);
            }
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
                type_id: Id::try_from(row.type_id as i64).map_err(|err| {
                    RepoError::DatabaseError {
                        err: err.to_string(),
                    }
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
    SELECT it.type_id, it.name, it.description, it.account_id, ak.key AS required_key, lnk.default_value::text AS required_default
    FROM item_type it
    LEFT JOIN item_type_attribute_kind_link lnk ON it.type_id = lnk.item_type_id
    LEFT JOIN attribute_kind ak ON lnk.attribute_kind_id = ak.kind_id
";

impl ItemTypeRepo for ItemTypePostgresRepo {
    fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE (it.account_id = $1 OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows_to_item_types(rows)
            })
        })
    }

    fn get_item_type_summaries(&self, account_id: &Id) -> Result<Vec<ItemTypeSummary>, RepoError> {
        self.get_item_type_summaries_with_archived(account_id, false)
    }

    fn get_item_type_summaries_with_archived(
        &self,
        account_id: &Id,
        include_archived: bool,
    ) -> Result<Vec<ItemTypeSummary>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
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
                     LEFT JOIN item i ON i.item_id = item_lnk.item_id AND i.account_id = $1 AND ($2 OR NOT EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = 'Archived' AND a.value_int = 1))
                     WHERE (it.account_id = $1 OR it.account_id IS NULL)
                     GROUP BY it.type_id, it.name, it.account_id
                     ORDER BY LOWER(it.name), it.type_id",
                )
                .bind(account_id_val)
                .bind(include_archived)
                .fetch_all(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE it.type_id = $1 AND (it.account_id = $2 OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(type_id_val)
                .bind(account_id_val)
                .fetch_all(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE it.name = $1 AND (it.account_id = $2 OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(&name)
                .bind(account_id_val)
                .fetch_all(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                if item_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let id_vals: Vec<i64> = item_ids.iter().map(|id| i64::from(*id)).collect();
                let rows = sqlx::query_as::<_, ItemTypeRefRow>(
                    "SELECT lnk.item_id, it.type_id, it.name, it.account_id
                     FROM item_item_type_link lnk
                     JOIN item_type it ON it.type_id = lnk.type_id
                     WHERE lnk.item_id = ANY($1) AND (it.account_id = $2 OR it.account_id IS NULL)
                     ORDER BY lnk.item_id, it.type_id",
                )
                .bind(&id_vals)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut refs_by_item: HashMap<Id, Vec<ItemTypeRef>> = HashMap::new();
                for row in rows {
                    let item_id = Id::try_from(row.item_id as i64).map_err(|err| {
                        RepoError::DatabaseError {
                            err: err.to_string(),
                        }
                    })?;
                    let type_id = Id::try_from(row.type_id as i64).map_err(|err| {
                        RepoError::DatabaseError {
                            err: err.to_string(),
                        }
                    })?;
                    refs_by_item.entry(item_id).or_default().push(ItemTypeRef {
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let item_ids = sqlx::query_scalar::<_, i32>(
                    "SELECT lnk.item_id
                     FROM item_item_type_link lnk
                     JOIN item_type it ON it.type_id = lnk.type_id
                     JOIN item i ON i.item_id = lnk.item_id
                     WHERE it.name = $1 AND i.account_id = $2
                       AND (it.account_id = $2 OR it.account_id IS NULL)
                     ORDER BY lnk.item_id",
                )
                .bind(&name)
                .bind(account_id_val)
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

    fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = dto.name.clone();
        let description = dto.description.clone();
        let required = dto.required_attributes.clone();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let type_id: i32 = sqlx::query_scalar(
                    "INSERT INTO item_type (name, description, account_id) VALUES ($1, $2, $3) RETURNING type_id",
                )
                .bind(&name)
                .bind(&description)
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                for key in &required {
                    sqlx::query(
                        "INSERT INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
                         SELECT ak.kind_id, $1 FROM attribute_kind ak
                         WHERE ak.key = $2 AND (ak.account_id = $3 OR ak.is_system = true)
                         ON CONFLICT DO NOTHING",
                    )
                    // i64 to match add_attr_kinds_to_item_type, which runs the
                    // same SQL text; sqlx caches statements by text (Z199).
                    .bind(i64::from(type_id))
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&pool)
                    .await
                    .map_err(RepoError::from)?;
                }

                let type_id_domain = Id::try_from(type_id as i64)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                Ok(Some(ItemType {
                    type_id: type_id_domain,
                    is_system: false,
                    name,
                    description,
                    required_attributes: required,
                    required_defaults: BTreeMap::new(),
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE item_type
                     SET name = COALESCE($1, name),
                         description = COALESCE($2, description)
                     WHERE type_id = $3 AND account_id = $4",
                )
                .bind(name)
                .bind(description)
                .bind(type_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                let type_id_domain = Id::try_from(type_id_val)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                let account_id_domain = Id::try_from(account_id_val)
                    .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

                // Borrow pool for the recursive call
                let rows = sqlx::query_as::<_, ItemTypeWithRequiredRow>(&format!(
                    "{} WHERE it.type_id = $1 AND (it.account_id = $2 OR it.account_id IS NULL) ORDER BY it.type_id",
                    ITEM_TYPE_SELECT
                ))
                .bind(i64::from(type_id_domain))
                .bind(i64::from(account_id_domain))
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                Ok(rows_to_item_types(rows)?.into_iter().next())
            })
        })
    }

    fn count_items_for_type(&self, type_id: &Id) -> Result<i64, RepoError> {
        let type_id_val = i64::from(*type_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM item_item_type_link WHERE type_id = $1",
                )
                .bind(type_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)
            })
        })
    }

    fn delete_item_type(&self, item_type_id: &Id, account_id: &Id) -> Result<bool, RepoError> {
        let type_id_val = i64::from(*item_type_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let result =
                    sqlx::query("DELETE FROM item_type WHERE type_id = $1 AND account_id = $2")
                        .bind(type_id_val)
                        .bind(account_id_val)
                        .execute(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                for key in &keys {
                    sqlx::query(
                        "INSERT INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
                         SELECT ak.kind_id, $1 FROM attribute_kind ak
                         WHERE ak.key = $2 AND (ak.account_id = $3 OR ak.is_system = true)
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                for key in &keys {
                    sqlx::query(
                        "DELETE FROM item_type_attribute_kind_link
                         WHERE item_type_id = $1
                         AND attribute_kind_id IN (
                             SELECT ak.kind_id FROM attribute_kind ak
                             WHERE ak.key = $2 AND (ak.account_id = $3 OR ak.is_system = true)
                         )",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                for name in &names {
                    sqlx::query(
                        "INSERT INTO item_item_type_link (item_id, type_id)
                         SELECT i.item_id, it.type_id
                         FROM item i
                         CROSS JOIN item_type it
                         WHERE i.item_id = $1 AND i.account_id = $2 AND it.name = $3
                           AND (it.account_id = $2 OR it.account_id IS NULL)
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(item_id_val)
                    .bind(account_id_val)
                    .bind(name)
                    .execute(&pool)
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
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                for name in &names {
                    sqlx::query(
                        "DELETE FROM item_item_type_link
                         WHERE item_id = $1 AND type_id IN (
                             SELECT it.type_id
                             FROM item_type it
                             WHERE it.name = $2 AND (it.account_id = $3 OR it.account_id IS NULL)
                         )",
                    )
                    .bind(item_id_val)
                    .bind(name)
                    .bind(account_id_val)
                    .execute(&pool)
                    .await
                    .map_err(RepoError::from)?;
                }
                Ok(())
            })
        })
    }

    fn missing_by_override_set(
        &self,
        type_id: &Id,
        key: &str,
        universe: &[i64],
        sample: i64,
        account_id: &Id,
    ) -> Result<Vec<MissingGroup>, RepoError> {
        if universe.len() > override_sets::MAX_UNIVERSE {
            return Err(RepoError::DatabaseError {
                err: format!("'{key}' has type defaults on too many types"),
            });
        }
        let type_id_val = i64::from(*type_id);
        let account_id_val = i64::from(*account_id);
        let key = key.to_string();
        let universe = universe.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let is_kind: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM attribute_kind
                     WHERE key = $1 AND (account_id = $2 OR is_system = true))",
                )
                .bind(&key)
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;
                if !is_kind {
                    return Ok(Vec::new());
                }

                let from = format!(
                    "FROM item_item_type_link l JOIN item i ON i.item_id = l.item_id
                     WHERE l.type_id = $1 AND i.account_id = $2 AND {}",
                    missing_value("$3")
                );
                let groups_sql = format!(
                    "SELECT mask, COUNT(*) FROM (SELECT CAST({} AS BIGINT) AS mask {from}) t
                     GROUP BY mask ORDER BY mask",
                    override_sets::mask_expr(&universe)
                );
                let groups: Vec<(i64, i64)> = sqlx::query_as(&groups_sql)
                    .bind(type_id_val)
                    .bind(account_id_val)
                    .bind(&key)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;

                let mut out = Vec::with_capacity(groups.len());
                for (mask, count) in groups {
                    let set = override_sets::set_from_mask(&universe, mask);
                    let sample_sql = format!(
                        "SELECT l.item_id::bigint {from} AND {} ORDER BY l.item_id LIMIT $4",
                        override_sets::exact_set(&universe, &set)
                    );
                    let sample_ids: Vec<i64> = sqlx::query_scalar(&sample_sql)
                        .bind(type_id_val)
                        .bind(account_id_val)
                        .bind(&key)
                        .bind(sample)
                        .fetch_all(&pool)
                        .await
                        .map_err(RepoError::from)?;
                    out.push(MissingGroup {
                        set,
                        count,
                        sample_ids,
                    });
                }
                Ok(out)
            })
        })
    }

    fn count_offenders(
        &self,
        type_id: &Id,
        offenders: &[Offender],
        account_id: &Id,
    ) -> Result<i64, RepoError> {
        if offenders.is_empty() {
            return Ok(0);
        }
        let type_id_val = i64::from(*type_id);
        let account_id_val = i64::from(*account_id);
        let offenders = offenders.to_vec();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let any = offenders
                    .iter()
                    .enumerate()
                    .map(|(i, offender)| {
                        let sets = offender
                            .sets
                            .iter()
                            .map(|set| {
                                format!("({})", override_sets::exact_set(&offender.universe, set))
                            })
                            .collect::<Vec<_>>()
                            .join(" OR ");
                        format!("({} AND ({sets}))", missing_value(&format!("${}", i + 3)))
                    })
                    .collect::<Vec<_>>()
                    .join(" OR ");
                let sql = format!(
                    "SELECT COUNT(*) FROM item_item_type_link l JOIN item i ON i.item_id = l.item_id
                     WHERE l.type_id = $1 AND i.account_id = $2 AND ({any})"
                );
                let mut query = sqlx::query_scalar::<_, i64>(&sql)
                    .bind(type_id_val)
                    .bind(account_id_val);
                for offender in &offenders {
                    query = query.bind(&offender.key);
                }
                query.fetch_one(&pool).await.map_err(RepoError::from)
            })
        })
    }

    fn overrides_for_key(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Vec<(i64, serde_json::Value)>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let key = key.to_string();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows: Vec<(i64, String)> = sqlx::query_as(
                    "SELECT lnk.item_type_id::bigint, lnk.default_value::text
                     FROM item_type_attribute_kind_link lnk
                     JOIN attribute_kind ak ON ak.kind_id = lnk.attribute_kind_id
                     JOIN item_type it ON it.type_id = lnk.item_type_id
                     WHERE ak.key = $1 AND (ak.account_id = $2 OR ak.is_system = true)
                       AND (it.account_id = $2 OR it.account_id IS NULL)
                       AND lnk.default_value IS NOT NULL
                     ORDER BY lnk.item_type_id",
                )
                .bind(&key)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows.into_iter()
                    .map(|(type_id, raw)| {
                        serde_json::from_str(&raw)
                            .map(|value| (type_id, value))
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })
                    })
                    .collect()
            })
        })
    }

    fn apply_required_change(
        &self,
        type_id: &Id,
        change: &RequiredChange,
        account_id: &Id,
    ) -> Result<BTreeMap<String, i64>, RepoError> {
        let type_id_val = i64::from(*type_id);
        let account_id_val = i64::from(*account_id);
        let change = change.clone();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                if change.name.is_some() || change.description.is_some() {
                    sqlx::query(
                        "UPDATE item_type
                         SET name = COALESCE($1, name),
                             description = COALESCE($2, description)
                         WHERE type_id = $3 AND account_id = $4",
                    )
                    .bind(&change.name)
                    .bind(&change.description)
                    .bind(type_id_val)
                    .bind(account_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                for key in &change.add {
                    sqlx::query(
                        "INSERT INTO item_type_attribute_kind_link (attribute_kind_id, item_type_id)
                         SELECT ak.kind_id, $1 FROM attribute_kind ak
                         WHERE ak.key = $2 AND (ak.account_id = $3 OR ak.is_system = true)
                         ON CONFLICT DO NOTHING",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                for key in &change.remove {
                    sqlx::query(
                        "DELETE FROM item_type_attribute_kind_link
                         WHERE item_type_id = $1
                         AND attribute_kind_id IN (
                             SELECT ak.kind_id FROM attribute_kind ak
                             WHERE ak.key = $2 AND (ak.account_id = $3 OR ak.is_system = true)
                         )",
                    )
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                for (key, value) in &change.defaults {
                    sqlx::query(
                        "UPDATE item_type_attribute_kind_link SET default_value = CAST($1 AS jsonb)
                         WHERE item_type_id = $2
                         AND attribute_kind_id IN (
                             SELECT ak.kind_id FROM attribute_kind ak
                             WHERE ak.key = $3 AND (ak.account_id = $4 OR ak.is_system = true)
                         )",
                    )
                    .bind(value.as_ref().map(|v| v.to_string()))
                    .bind(type_id_val)
                    .bind(key)
                    .bind(account_id_val)
                    .execute(&mut *tx)
                    .await
                    .map_err(RepoError::from)?;
                }

                let mut backfilled = BTreeMap::new();
                for backfill in &change.backfills {
                    let key = &backfill.key;
                    let from = format!(
                        "FROM item_item_type_link l JOIN item i ON i.item_id = l.item_id
                         WHERE l.type_id = $1 AND i.account_id = $2 AND {} AND {}",
                        missing_value("$3"),
                        override_sets::exact_set(&backfill.universe, &backfill.set)
                    );
                    let count = match &backfill.value {
                        Attribute::Scalar(scalar) => {
                            let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                            sqlx::query(&format!(
                                "INSERT INTO attribute
                                 (item_id, key, value_text, value_num, value_int, value_date, value_item_id)
                                 SELECT l.item_id, $3, $4, $5, $6, $7, $8 {from}"
                            ))
                            .bind(type_id_val)
                            .bind(account_id_val)
                            .bind(key)
                            .bind(vt)
                            .bind(vn)
                            .bind(vi)
                            .bind(vd)
                            .bind(vid)
                            .execute(&mut *tx)
                            .await
                            .map_err(RepoError::from)?
                            .rows_affected() as i64
                        }
                        Attribute::List(values) => {
                            let ids: Vec<i64> = sqlx::query_scalar(&format!(
                                "SELECT l.item_id::bigint {from} ORDER BY l.item_id"
                            ))
                            .bind(type_id_val)
                            .bind(account_id_val)
                            .bind(key)
                            .fetch_all(&mut *tx)
                            .await
                            .map_err(RepoError::from)?;
                            for id in &ids {
                                for (ordinal, scalar) in values.iter().enumerate() {
                                    let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                                    sqlx::query(
                                        "INSERT INTO attribute_list_value
                                         (item_id, key, ordinal, value_text, value_num, value_int, value_date, value_item_id)
                                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                                    )
                                    .bind(id)
                                    .bind(key)
                                    .bind(ordinal as i64)
                                    .bind(vt)
                                    .bind(vn)
                                    .bind(vi)
                                    .bind(vd)
                                    .bind(vid)
                                    .execute(&mut *tx)
                                    .await
                                    .map_err(RepoError::from)?;
                                }
                            }
                            ids.len() as i64
                        }
                    };
                    *backfilled.entry(key.clone()).or_insert(0) += count;
                }

                tx.commit().await.map_err(RepoError::from)?;
                Ok(backfilled)
            })
        })
    }

    fn list_non_compliant(
        &self,
        type_id: &Id,
        limit: i64,
        offset: i64,
        account_id: &Id,
    ) -> Result<NonCompliantPageDto, RepoError> {
        let type_id_val = i64::from(*type_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let keys: Vec<String> = sqlx::query_scalar(
                    "SELECT ak.key FROM item_type_attribute_kind_link lnk
                     JOIN attribute_kind ak ON ak.kind_id = lnk.attribute_kind_id
                     WHERE lnk.item_type_id = $1 ORDER BY ak.key",
                )
                .bind(type_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                if keys.is_empty() {
                    return Ok(NonCompliantPageDto {
                        total: 0,
                        items: Vec::new(),
                    });
                }

                let any_missing = (0..keys.len())
                    .map(|i| format!("({})", missing_value(&format!("${}", i + 3))))
                    .collect::<Vec<_>>()
                    .join(" OR ");
                let from = format!(
                    "FROM item_item_type_link l JOIN item i ON i.item_id = l.item_id
                     WHERE l.type_id = $1 AND i.account_id = $2 AND ({any_missing})"
                );

                let total_sql = format!("SELECT COUNT(*) {from}");
                let mut total = sqlx::query_scalar::<_, i64>(&total_sql)
                    .bind(type_id_val)
                    .bind(account_id_val);
                for key in &keys {
                    total = total.bind(key);
                }
                let total = total.fetch_one(&pool).await.map_err(RepoError::from)?;

                let limit_ph = format!("${}", keys.len() + 3);
                let offset_ph = format!("${}", keys.len() + 4);
                let page_sql = format!(
                    "SELECT i.item_id::bigint, i.title {from} ORDER BY i.item_id LIMIT {limit_ph} OFFSET {offset_ph}"
                );
                let mut page = sqlx::query_as::<_, (i64, String)>(&page_sql)
                .bind(type_id_val)
                .bind(account_id_val);
                for key in &keys {
                    page = page.bind(key);
                }
                let rows = page
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;

                let mut items = Vec::with_capacity(rows.len());
                for (item_id, title) in rows {
                    let present: Vec<String> = sqlx::query_scalar(
                        "SELECT key FROM attribute WHERE item_id = $1
                         UNION SELECT key FROM attribute_list_value WHERE item_id = $1",
                    )
                    .bind(item_id)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;
                    let archived: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM attribute
                         WHERE item_id = $1 AND key = 'Archived' AND value_int = 1)",
                    )
                    .bind(item_id)
                    .fetch_one(&pool)
                    .await
                    .map_err(RepoError::from)?;
                    items.push(NonCompliantItemDto {
                        item_id,
                        title,
                        archived,
                        missing: keys
                            .iter()
                            .filter(|key| !present.contains(key))
                            .cloned()
                            .collect(),
                    });
                }
                Ok(NonCompliantPageDto { total, items })
            })
        })
    }
}

/// SQL predicate: the item `l.item_id` has no scalar or list value for the key at placeholder `key`.
fn missing_value(key: &str) -> String {
    format!(
        "NOT EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = l.item_id AND a.key = {key}) \
         AND NOT EXISTS (SELECT 1 FROM attribute_list_value v WHERE v.item_id = l.item_id AND v.key = {key})"
    )
}
