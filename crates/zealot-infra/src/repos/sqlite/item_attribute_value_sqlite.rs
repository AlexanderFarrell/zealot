use std::collections::HashMap;

use chrono::NaiveDate;
use serde_json::Value;
use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item_attribute_value::ItemAttributeValueRepo};
use zealot_domain::{
    account::Account,
    attribute::{
        Attribute, AttributeBaseScalarType, AttributeFilter, AttributeFilterOp, AttributeListMode,
        AttributeScalar,
    },
    common::id::Id,
};

#[derive(Debug)]
pub struct ItemAttributeValueSqliteRepo {
    pool: SqlitePool,
}

impl ItemAttributeValueSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow, Clone)]
struct AttrScalarRow {
    item_id: i64,
    key: String,
    value_text: Option<String>,
    value_num: Option<f64>,
    value_int: Option<i64>,
    value_date: Option<i64>,
    value_item_id: Option<i64>,
}

#[derive(sqlx::FromRow, Clone)]
struct AttrListRow {
    item_id: i64,
    key: String,
    ordinal: i64,
    value_text: Option<String>,
    value_num: Option<f64>,
    value_int: Option<i64>,
    value_date: Option<i64>,
    value_item_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct AttrKindRow {
    key: String,
    base_type: String,
    config: String,
}

fn parse_scalar_type(s: &str) -> Option<AttributeBaseScalarType> {
    match s {
        "text" => Some(AttributeBaseScalarType::Text),
        "integer" => Some(AttributeBaseScalarType::Integer),
        "decimal" => Some(AttributeBaseScalarType::Decimal),
        "date" => Some(AttributeBaseScalarType::Date),
        "week" => Some(AttributeBaseScalarType::Week),
        "dropdown" => Some(AttributeBaseScalarType::Dropdown),
        "boolean" => Some(AttributeBaseScalarType::Boolean),
        "item" => Some(AttributeBaseScalarType::Item),
        _ => None,
    }
}

fn kind_scalar_type(row: &AttrKindRow) -> Option<AttributeBaseScalarType> {
    if row.base_type == "list" {
        let config: Value = serde_json::from_str(&row.config).unwrap_or(Value::Null);
        return config
            .get("list_type")
            .and_then(|value| value.as_str())
            .and_then(parse_scalar_type);
    }

    parse_scalar_type(&row.base_type)
}

fn decode_scalar_from_cols(
    value_text: Option<&str>,
    value_num: Option<f64>,
    value_int: Option<i64>,
    value_date: Option<i64>,
    value_item_id: Option<i64>,
    kind_base_type: Option<&AttributeBaseScalarType>,
) -> Option<AttributeScalar> {
    if let Some(bt) = kind_base_type {
        match bt {
            AttributeBaseScalarType::Text => {
                return value_text.map(|s| AttributeScalar::Text(s.to_string()));
            }
            AttributeBaseScalarType::Dropdown => {
                return value_text.map(|s| AttributeScalar::Dropdown(s.to_string()));
            }
            AttributeBaseScalarType::Week => {
                if let Some(s) = value_text {
                    use std::convert::TryFrom;
                    return zealot_domain::attribute::Week::try_from(s)
                        .ok()
                        .map(AttributeScalar::Week);
                }
                return None;
            }
            AttributeBaseScalarType::Integer => {
                return value_int.map(AttributeScalar::Integer);
            }
            AttributeBaseScalarType::Boolean => {
                return value_int.map(|i| AttributeScalar::Boolean(i != 0));
            }
            AttributeBaseScalarType::Decimal => {
                return value_num.map(AttributeScalar::Decimal);
            }
            AttributeBaseScalarType::Date => {
                if let Some(ts) = value_date {
                    let date = NaiveDate::from_num_days_from_ce_opt((ts / 86400 + 719163) as i32);
                    return date.map(AttributeScalar::Date);
                }
                return None;
            }
            AttributeBaseScalarType::Item => {
                if let Some(id) = value_item_id {
                    return Id::try_from(id).ok().map(AttributeScalar::Item);
                }
                return None;
            }
        }
    }

    if let Some(s) = value_text {
        return Some(AttributeScalar::Text(s.to_string()));
    }
    if let Some(i) = value_int {
        return Some(AttributeScalar::Integer(i));
    }
    if let Some(f) = value_num {
        return Some(AttributeScalar::Decimal(f));
    }
    if let Some(ts) = value_date {
        let date = NaiveDate::from_num_days_from_ce_opt((ts / 86400 + 719163) as i32);
        return date.map(AttributeScalar::Date);
    }
    if let Some(id) = value_item_id {
        return Id::try_from(id).ok().map(AttributeScalar::Item);
    }

    None
}

fn scalar_to_cols(
    scalar: &AttributeScalar,
) -> (
    Option<String>,
    Option<f64>,
    Option<i64>,
    Option<i64>,
    Option<i64>,
) {
    match scalar {
        AttributeScalar::Text(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Dropdown(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Week(w) => (Some(w.to_string()), None, None, None, None),
        AttributeScalar::Integer(i) => (None, None, Some(*i), None, None),
        AttributeScalar::Boolean(b) => (None, None, Some(*b as i64), None, None),
        AttributeScalar::Decimal(f) => (None, Some(*f), None, None, None),
        AttributeScalar::Date(d) => {
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            let days = d.signed_duration_since(epoch).num_days();
            (None, None, None, Some(days * 86400), None)
        }
        AttributeScalar::Item(id) => (None, None, None, None, Some(i64::from(*id))),
    }
}

async fn get_kind_scalar_map(
    account_id: i64,
    pool: &SqlitePool,
) -> Result<HashMap<String, AttributeBaseScalarType>, RepoError> {
    let rows = sqlx::query_as::<_, AttrKindRow>(
        "SELECT key, base_type, config FROM attribute_kind
         WHERE account_id = ? OR is_system = 1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(RepoError::from)?;

    let mut kinds = HashMap::new();
    for row in rows {
        if let Some(base_type) = kind_scalar_type(&row) {
            kinds.insert(row.key, base_type);
        }
    }

    Ok(kinds)
}

async fn write_attributes(
    item_id: i64,
    attributes: &HashMap<String, Attribute>,
    pool: &SqlitePool,
) -> Result<(), RepoError> {
    for (key, attr) in attributes {
        match attr {
            Attribute::Scalar(scalar) => {
                sqlx::query("DELETE FROM attribute_list_value WHERE item_id = ? AND key = ?")
                    .bind(item_id)
                    .bind(key)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;

                let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                sqlx::query(
                    "INSERT INTO attribute (item_id, key, value_text, value_num, value_int, value_date, value_item_id)
                     VALUES (?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(item_id, key) DO UPDATE SET
                       value_text = excluded.value_text,
                       value_num = excluded.value_num,
                       value_int = excluded.value_int,
                       value_date = excluded.value_date,
                       value_item_id = excluded.value_item_id",
                )
                .bind(item_id)
                .bind(key)
                .bind(vt)
                .bind(vn)
                .bind(vi)
                .bind(vd)
                .bind(vid)
                .execute(pool)
                .await
                .map_err(RepoError::from)?;
            }
            Attribute::List(values) => {
                sqlx::query("DELETE FROM attribute_list_value WHERE item_id = ? AND key = ?")
                    .bind(item_id)
                    .bind(key)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;

                sqlx::query("DELETE FROM attribute WHERE item_id = ? AND key = ?")
                    .bind(item_id)
                    .bind(key)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;

                for (ordinal, scalar) in values.iter().enumerate() {
                    let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                    sqlx::query(
                        "INSERT INTO attribute_list_value
                         (item_id, key, ordinal, value_text, value_num, value_int, value_date, value_item_id)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(item_id)
                    .bind(key)
                    .bind(ordinal as i64)
                    .bind(vt)
                    .bind(vn)
                    .bind(vi)
                    .bind(vd)
                    .bind(vid)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;
                }
            }
        }
    }

    Ok(())
}

#[derive(Clone)]
enum SqlValue {
    Str(String),
    Int(i64),
    Float(f64),
}

fn attr_filter_col_and_val(
    filter: &AttributeFilter,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<(&'static str, SqlValue), RepoError> {
    if let Some(base_type) = kind_map.get(&filter.key) {
        return match base_type {
            AttributeBaseScalarType::Text
            | AttributeBaseScalarType::Dropdown
            | AttributeBaseScalarType::Week => match &filter.value {
                Value::String(value) => Ok(("value_text", SqlValue::Str(value.clone()))),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a string value", filter.key),
                }),
            },
            AttributeBaseScalarType::Integer => match &filter.value {
                Value::Number(value) => value
                    .as_i64()
                    .map(|value| ("value_int", SqlValue::Int(value)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects an integer value", filter.key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects an integer value", filter.key),
                }),
            },
            AttributeBaseScalarType::Boolean => match &filter.value {
                Value::Bool(value) => Ok(("value_int", SqlValue::Int(*value as i64))),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a boolean value", filter.key),
                }),
            },
            AttributeBaseScalarType::Decimal => match &filter.value {
                Value::Number(value) => value
                    .as_f64()
                    .map(|value| ("value_num", SqlValue::Float(value)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects a numeric value", filter.key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a numeric value", filter.key),
                }),
            },
            AttributeBaseScalarType::Date => match &filter.value {
                Value::String(value) => {
                    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|err| {
                        RepoError::DatabaseError {
                            err: format!("filter '{}' expects YYYY-MM-DD: {}", filter.key, err),
                        }
                    })?;
                    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                    let days = date.signed_duration_since(epoch).num_days();
                    Ok(("value_date", SqlValue::Int(days * 86400)))
                }
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a date string", filter.key),
                }),
            },
            AttributeBaseScalarType::Item => match &filter.value {
                Value::Number(value) => value
                    .as_i64()
                    .map(|value| ("value_item_id", SqlValue::Int(value)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects an item id", filter.key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects an item id", filter.key),
                }),
            },
        };
    }

    let value = &filter.value;
    match value {
        Value::String(s) => Ok(("value_text", SqlValue::Str(s.clone()))),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(("value_int", SqlValue::Int(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(("value_num", SqlValue::Float(f)))
            } else {
                Err(RepoError::DatabaseError {
                    err: String::from("unsupported number type in filter"),
                })
            }
        }
        Value::Bool(b) => Ok(("value_int", SqlValue::Int(*b as i64))),
        _ => Err(RepoError::DatabaseError {
            err: String::from("unsupported filter value type"),
        }),
    }
}

fn build_attr_filter_clause(
    filter: &AttributeFilter,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<(String, Vec<SqlValue>), RepoError> {
    let op_str = match filter.op {
        AttributeFilterOp::Equal => "=",
        AttributeFilterOp::NotEqual => "!=",
        AttributeFilterOp::GreaterThan => ">",
        AttributeFilterOp::LessThan => "<",
        AttributeFilterOp::GreaterThanOrEqualTo => ">=",
        AttributeFilterOp::LessThanOrEqualTo => "<=",
        AttributeFilterOp::LikeCaseInsensitive => "LIKE",
    };

    let (col, value) = attr_filter_col_and_val(filter, kind_map)?;
    let clause = match filter.list_mode {
        AttributeListMode::Any => {
            format!(
                "(EXISTS (SELECT 1 FROM attribute_list_value alv WHERE alv.item_id = i.item_id AND alv.key = ? AND alv.{col} {op} ?)
                 OR EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = ? AND a.{col} {op} ?))",
                col = col,
                op = op_str,
            )
        }
        AttributeListMode::None => {
            format!(
                "(NOT EXISTS (SELECT 1 FROM attribute_list_value alv WHERE alv.item_id = i.item_id AND alv.key = ? AND alv.{col} {op} ?)
                 AND NOT EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = ? AND a.{col} {op} ?))",
                col = col,
                op = op_str,
            )
        }
        AttributeListMode::All => {
            return Err(RepoError::DatabaseError {
                err: String::from("list_mode 'all' is not yet supported in SQLite"),
            });
        }
    };

    Ok((
        clause,
        vec![
            SqlValue::Str(filter.key.clone()),
            value.clone(),
            SqlValue::Str(filter.key.clone()),
            value,
        ],
    ))
}

impl ItemAttributeValueRepo for ItemAttributeValueSqliteRepo {
    fn get_attributes_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, HashMap<String, Attribute>>, RepoError> {
        let item_ids = item_ids.clone();
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                if item_ids.is_empty() {
                    return Ok(HashMap::new());
                }

                let kind_scalar_map = get_kind_scalar_map(account_id_val, &pool).await?;
                let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");

                let scalar_sql = format!(
                    "SELECT a.item_id, a.key, a.value_text, a.value_num, a.value_int, a.value_date, a.value_item_id
                     FROM attribute a
                     JOIN item i ON i.item_id = a.item_id
                     WHERE i.account_id = ? AND a.item_id IN ({})",
                    placeholders
                );
                let mut scalar_query = sqlx::query_as::<_, AttrScalarRow>(&scalar_sql).bind(account_id_val);
                for item_id in &item_ids {
                    scalar_query = scalar_query.bind(i64::from(*item_id));
                }
                let scalar_rows = scalar_query.fetch_all(&pool).await.map_err(RepoError::from)?;

                let list_sql = format!(
                    "SELECT alv.item_id, alv.key, alv.ordinal, alv.value_text, alv.value_num, alv.value_int, alv.value_date, alv.value_item_id
                     FROM attribute_list_value alv
                     JOIN item i ON i.item_id = alv.item_id
                     WHERE i.account_id = ? AND alv.item_id IN ({})
                     ORDER BY alv.item_id, alv.key, alv.ordinal",
                    placeholders
                );
                let mut list_query = sqlx::query_as::<_, AttrListRow>(&list_sql).bind(account_id_val);
                for item_id in &item_ids {
                    list_query = list_query.bind(i64::from(*item_id));
                }
                let list_rows = list_query.fetch_all(&pool).await.map_err(RepoError::from)?;

                let mut attributes_by_item: HashMap<Id, HashMap<String, Attribute>> = HashMap::new();

                for row in scalar_rows {
                    let item_id = Id::try_from(row.item_id)
                        .map_err(|err| RepoError::DatabaseError { err: err.to_string() })?;
                    if let Some(scalar) = decode_scalar_from_cols(
                        row.value_text.as_deref(),
                        row.value_num,
                        row.value_int,
                        row.value_date,
                        row.value_item_id,
                        kind_scalar_map.get(&row.key),
                    ) {
                        attributes_by_item
                            .entry(item_id)
                            .or_default()
                            .insert(row.key, Attribute::Scalar(scalar));
                    }
                }

                let mut list_values: HashMap<(Id, String), Vec<AttributeScalar>> = HashMap::new();
                for row in list_rows {
                    let item_id = Id::try_from(row.item_id)
                        .map_err(|err| RepoError::DatabaseError { err: err.to_string() })?;
                    let _ = row.ordinal;
                    if let Some(scalar) = decode_scalar_from_cols(
                        row.value_text.as_deref(),
                        row.value_num,
                        row.value_int,
                        row.value_date,
                        row.value_item_id,
                        kind_scalar_map.get(&row.key),
                    ) {
                        list_values.entry((item_id, row.key)).or_default().push(scalar);
                    }
                }

                for ((item_id, key), values) in list_values {
                    attributes_by_item
                        .entry(item_id)
                        .or_default()
                        .insert(key, Attribute::List(values));
                }

                Ok(attributes_by_item)
            })
        })
    }

    fn find_item_ids_by_filters(
        &self,
        filters: &Vec<AttributeFilter>,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        if filters.is_empty() {
            return Err(RepoError::DatabaseError {
                err: String::from("no filters provided"),
            });
        }

        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        let filters = filters.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut where_parts = Vec::new();
                let mut values = Vec::new();
                let kind_map = get_kind_scalar_map(account_id_val, &pool).await?;

                for filter in &filters {
                    let (clause, clause_values) = build_attr_filter_clause(filter, &kind_map)?;
                    where_parts.push(format!("({})", clause));
                    values.extend(clause_values);
                }

                let sql = format!(
                    "SELECT DISTINCT i.item_id FROM item i
                     WHERE i.account_id = ? AND {}",
                    where_parts.join(" AND ")
                );

                let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(account_id_val);
                for value in &values {
                    query = match value {
                        SqlValue::Str(value) => query.bind(value.clone()),
                        SqlValue::Int(value) => query.bind(*value),
                        SqlValue::Float(value) => query.bind(*value),
                    };
                }

                let item_ids = query.fetch_all(&pool).await.map_err(RepoError::from)?;
                item_ids
                    .into_iter()
                    .map(|item_id| {
                        Id::try_from(item_id).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })
                    })
                    .collect()
            })
        })
    }

    fn replace_item_attributes(
        &self,
        item_id: &Id,
        attributes: &HashMap<String, Attribute>,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let attributes = attributes.clone();
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

                write_attributes(item_id_val, &attributes, &pool).await
            })
        })
    }

    fn rename_item_attribute(
        &self,
        item_id: &Id,
        old_key: &str,
        new_key: &str,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let old_key = old_key.to_string();
        let new_key = new_key.to_string();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                sqlx::query(
                    "UPDATE attribute SET key = ?
                     WHERE key = ? AND item_id = (
                         SELECT item_id FROM item WHERE item_id = ? AND account_id = ?
                     )",
                )
                .bind(&new_key)
                .bind(&old_key)
                .bind(item_id_val)
                .bind(account_id_val)
                .execute(&mut *tx)
                .await
                .map_err(RepoError::from)?;

                sqlx::query(
                    "UPDATE attribute_list_value SET key = ?
                     WHERE key = ? AND item_id = (
                         SELECT item_id FROM item WHERE item_id = ? AND account_id = ?
                     )",
                )
                .bind(&new_key)
                .bind(&old_key)
                .bind(item_id_val)
                .bind(account_id_val)
                .execute(&mut *tx)
                .await
                .map_err(RepoError::from)?;

                tx.commit().await.map_err(RepoError::from)
            })
        })
    }

    fn delete_item_attribute(
        &self,
        item_id: &Id,
        key: &str,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let key = key.to_string();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "DELETE FROM attribute
                     WHERE item_id = (SELECT item_id FROM item WHERE item_id = ? AND account_id = ?)
                     AND key = ?",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(&key)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                sqlx::query(
                    "DELETE FROM attribute_list_value
                     WHERE item_id = (SELECT item_id FROM item WHERE item_id = ? AND account_id = ?)
                     AND key = ?",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(&key)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
