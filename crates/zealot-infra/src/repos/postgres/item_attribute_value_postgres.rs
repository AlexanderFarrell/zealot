use std::collections::HashMap;

use chrono::NaiveDate;
use serde_json::Value;
use sqlx::PgPool;
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
pub struct ItemAttributeValuePostgresRepo {
    pool: PgPool,
}

impl ItemAttributeValuePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow, Clone)]
struct AttrScalarRow {
    item_id: i32,
    key: String,
    value_text: Option<String>,
    value_num: Option<f64>,
    value_int: Option<i32>,
    value_date: Option<chrono::DateTime<chrono::Utc>>,
    value_item_id: Option<i32>,
}

#[derive(sqlx::FromRow, Clone)]
struct AttrListRow {
    item_id: i32,
    key: String,
    ordinal: i32,
    value_text: Option<String>,
    value_num: Option<f64>,
    value_int: Option<i32>,
    value_date: Option<chrono::DateTime<chrono::Utc>>,
    value_item_id: Option<i32>,
}

#[derive(sqlx::FromRow)]
struct AttrKindRow {
    key: String,
    base_type: String,
    config: serde_json::Value,
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
        return row
            .config
            .get("list_type")
            .and_then(|v| v.as_str())
            .and_then(parse_scalar_type);
    }
    parse_scalar_type(&row.base_type)
}

fn decode_scalar_from_cols(
    value_text: Option<&str>,
    value_num: Option<f64>,
    value_int: Option<i32>,
    value_date: Option<chrono::DateTime<chrono::Utc>>,
    value_item_id: Option<i32>,
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
                    return zealot_domain::attribute::Week::try_from(s)
                        .ok()
                        .map(AttributeScalar::Week);
                }
                return None;
            }
            AttributeBaseScalarType::Integer => {
                return value_int.map(|i| AttributeScalar::Integer(i as i64));
            }
            AttributeBaseScalarType::Boolean => {
                return value_int.map(|i| AttributeScalar::Boolean(i != 0));
            }
            AttributeBaseScalarType::Decimal => {
                return value_num.map(AttributeScalar::Decimal);
            }
            AttributeBaseScalarType::Date => {
                return value_date.map(|dt| AttributeScalar::Date(dt.date_naive()));
            }
            AttributeBaseScalarType::Item => {
                if let Some(id) = value_item_id {
                    return Id::try_from(id as i64).ok().map(AttributeScalar::Item);
                }
                return None;
            }
        }
    }

    if let Some(s) = value_text {
        return Some(AttributeScalar::Text(s.to_string()));
    }
    if let Some(i) = value_int {
        return Some(AttributeScalar::Integer(i as i64));
    }
    if let Some(f) = value_num {
        return Some(AttributeScalar::Decimal(f));
    }
    if let Some(dt) = value_date {
        return Some(AttributeScalar::Date(dt.date_naive()));
    }
    if let Some(id) = value_item_id {
        return Id::try_from(id as i64).ok().map(AttributeScalar::Item);
    }

    None
}

fn scalar_to_cols(
    scalar: &AttributeScalar,
) -> (
    Option<String>,
    Option<f64>,
    Option<i32>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<i32>,
) {
    match scalar {
        AttributeScalar::Text(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Dropdown(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Week(w) => (Some(w.to_string()), None, None, None, None),
        AttributeScalar::Integer(i) => (None, None, Some(*i as i32), None, None),
        AttributeScalar::Boolean(b) => (None, None, Some(*b as i32), None, None),
        AttributeScalar::Decimal(f) => (None, Some(*f), None, None, None),
        AttributeScalar::Date(d) => {
            let dt = d.and_hms_opt(0, 0, 0).map(|ndt| ndt.and_utc());
            (None, None, None, dt, None)
        }
        AttributeScalar::Item(id) => (None, None, None, None, Some(i64::from(*id) as i32)),
    }
}

async fn get_kind_scalar_map(
    account_id: i64,
    pool: &PgPool,
) -> Result<HashMap<String, AttributeBaseScalarType>, RepoError> {
    let rows = sqlx::query_as::<_, AttrKindRow>(
        "SELECT key, base_type, config FROM attribute_kind
         WHERE account_id = $1 OR is_system = true",
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
    pool: &PgPool,
) -> Result<(), RepoError> {
    for (key, attr) in attributes {
        match attr {
            Attribute::Scalar(scalar) => {
                sqlx::query("DELETE FROM attribute_list_value WHERE item_id = $1 AND key = $2")
                    .bind(item_id)
                    .bind(key)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;

                let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                sqlx::query(
                    "INSERT INTO attribute (item_id, key, value_text, value_num, value_int, value_date, value_item_id)
                     VALUES ($1, $2, $3, $4, $5, $6, $7)
                     ON CONFLICT (item_id, key) DO UPDATE SET
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
                sqlx::query("DELETE FROM attribute_list_value WHERE item_id = $1 AND key = $2")
                    .bind(item_id)
                    .bind(key)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;

                sqlx::query("DELETE FROM attribute WHERE item_id = $1 AND key = $2")
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
                         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
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
    Date(NaiveDate),
}

fn attr_filter_value_col_and_val(
    key: &str,
    value: &Value,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<(&'static str, SqlValue), RepoError> {
    if let Some(base_type) = kind_map.get(key) {
        return match base_type {
            AttributeBaseScalarType::Text
            | AttributeBaseScalarType::Dropdown
            | AttributeBaseScalarType::Week => match value {
                Value::String(value) => Ok(("value_text", SqlValue::Str(value.clone()))),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a string value", key),
                }),
            },
            AttributeBaseScalarType::Integer => match value {
                Value::Number(value) => value
                    .as_i64()
                    .map(|v| ("value_int", SqlValue::Int(v)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects an integer value", key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects an integer value", key),
                }),
            },
            AttributeBaseScalarType::Boolean => match value {
                Value::Bool(value) => Ok(("value_int", SqlValue::Int(*value as i64))),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a boolean value", key),
                }),
            },
            AttributeBaseScalarType::Decimal => match value {
                Value::Number(value) => value
                    .as_f64()
                    .map(|v| ("value_num", SqlValue::Float(v)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects a numeric value", key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a numeric value", key),
                }),
            },
            AttributeBaseScalarType::Date => match value {
                Value::String(value) => {
                    let date = NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|err| {
                        RepoError::DatabaseError {
                            err: format!("filter '{}' expects YYYY-MM-DD: {}", key, err),
                        }
                    })?;
                    Ok(("value_date::date", SqlValue::Date(date)))
                }
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a date string", key),
                }),
            },
            AttributeBaseScalarType::Item => match value {
                Value::Number(value) => value
                    .as_i64()
                    .map(|v| ("value_item_id", SqlValue::Int(v)))
                    .ok_or(RepoError::DatabaseError {
                        err: format!("filter '{}' expects an item id", key),
                    }),
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects an item id", key),
                }),
            },
        };
    }

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

fn attr_filter_col_and_values(
    filter: &AttributeFilter,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<(&'static str, Vec<SqlValue>), RepoError> {
    let raw_values: Vec<&Value> = match &filter.value {
        Value::Array(values) => {
            if values.is_empty() {
                return Err(RepoError::DatabaseError {
                    err: format!("filter '{}' value array must not be empty", filter.key),
                });
            }
            values.iter().collect()
        }
        value => vec![value],
    };

    let mut parsed = Vec::new();
    let mut column = None;
    for value in raw_values {
        let (value_col, sql_value) = attr_filter_value_col_and_val(&filter.key, value, kind_map)?;
        if let Some(column) = column {
            if column != value_col {
                return Err(RepoError::DatabaseError {
                    err: format!("filter '{}' array values must have one type", filter.key),
                });
            }
        } else {
            column = Some(value_col);
        }
        parsed.push(sql_value);
    }

    Ok((column.unwrap(), parsed))
}

fn build_attr_filter_clause(
    filter: &AttributeFilter,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
    param_offset: usize,
) -> Result<(String, Vec<SqlValue>), RepoError> {
    let (col, values) = attr_filter_col_and_values(filter, kind_map)?;
    let (op_str, values) = match &filter.op {
        AttributeFilterOp::Equal => ("=", values),
        AttributeFilterOp::NotEqual => ("!=", values),
        AttributeFilterOp::GreaterThan => (">", values),
        AttributeFilterOp::LessThan => ("<", values),
        AttributeFilterOp::GreaterThanOrEqualTo => (">=", values),
        AttributeFilterOp::LessThanOrEqualTo => ("<=", values),
        AttributeFilterOp::LikeCaseInsensitive => {
            if col != "value_text" {
                return Err(RepoError::DatabaseError {
                    err: format!(
                        "filter '{}' only supports ilike for text values",
                        filter.key
                    ),
                });
            }
            let values = values
                .into_iter()
                .map(|value| match value {
                    SqlValue::Str(value) => Ok(SqlValue::Str(format!("%{}%", value))),
                    _ => Err(RepoError::DatabaseError {
                        err: format!("filter '{}' expects a string value for ilike", filter.key),
                    }),
                })
                .collect::<Result<Vec<_>, _>>()?;
            ("ILIKE", values)
        }
    };

    let mut clauses = Vec::new();
    let mut clause_values = Vec::new();
    for value in values {
        let p1 = param_offset + clause_values.len();
        let p2 = p1 + 1;
        let p3 = p1 + 2;
        let p4 = p1 + 3;
        clauses.push(format!(
            "(EXISTS (SELECT 1 FROM attribute_list_value alv WHERE alv.item_id = i.item_id AND alv.key = ${p1} AND alv.{col} {op} ${p2})
             OR EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = ${p3} AND a.{col} {op} ${p4}))",
            col = col,
            op = op_str,
        ));
        clause_values.push(SqlValue::Str(filter.key.clone()));
        clause_values.push(value.clone());
        clause_values.push(SqlValue::Str(filter.key.clone()));
        clause_values.push(value);
    }

    let joined = clauses.join(" OR ");
    let clause = match &filter.list_mode {
        AttributeListMode::Any => format!("({joined})"),
        AttributeListMode::None => format!("NOT ({joined})"),
        AttributeListMode::All => format!("({})", clauses.join(" AND ")),
    };

    Ok((clause, clause_values))
}

impl ItemAttributeValueRepo for ItemAttributeValuePostgresRepo {
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
                let id_vals: Vec<i64> = item_ids.iter().map(|id| i64::from(*id)).collect();

                let scalar_rows = sqlx::query_as::<_, AttrScalarRow>(
                    "SELECT a.item_id, a.key, a.value_text, a.value_num, a.value_int, a.value_date, a.value_item_id
                     FROM attribute a
                     JOIN item i ON i.item_id = a.item_id
                     WHERE i.account_id = $1 AND a.item_id = ANY($2)",
                )
                .bind(account_id_val)
                .bind(&id_vals)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let list_rows = sqlx::query_as::<_, AttrListRow>(
                    "SELECT alv.item_id, alv.key, alv.ordinal, alv.value_text, alv.value_num, alv.value_int, alv.value_date, alv.value_item_id
                     FROM attribute_list_value alv
                     JOIN item i ON i.item_id = alv.item_id
                     WHERE i.account_id = $1 AND alv.item_id = ANY($2)
                     ORDER BY alv.item_id, alv.key, alv.ordinal",
                )
                .bind(account_id_val)
                .bind(&id_vals)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut attributes_by_item: HashMap<Id, HashMap<String, Attribute>> =
                    HashMap::new();

                for row in scalar_rows {
                    let item_id = Id::try_from(row.item_id as i64)
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
                    let item_id = Id::try_from(row.item_id as i64)
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
        limit: Option<i64>,
        offset: i64,
    ) -> Result<Vec<Id>, RepoError> {
        if filters.is_empty() {
            return Err(RepoError::DatabaseError {
                err: String::from("no filters provided"),
            });
        }

        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        let filters = filters.clone();
        let limit = limit.map(|limit| limit.max(1));
        let offset = offset.max(0);

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let kind_map = get_kind_scalar_map(account_id_val, &pool).await?;
                let mut where_parts = Vec::new();
                let mut values: Vec<SqlValue> = Vec::new();

                // $1 is reserved for account_id; filter params start at $2
                for filter in &filters {
                    let param_offset = 2 + values.len();
                    let (clause, clause_values) =
                        build_attr_filter_clause(filter, &kind_map, param_offset)?;
                    where_parts.push(clause);
                    values.extend(clause_values);
                }

                let mut sql = format!(
                    "SELECT DISTINCT i.item_id FROM item i
                     WHERE i.account_id = $1 AND {}
                     ORDER BY i.item_id",
                    where_parts.join(" AND ")
                );
                if limit.is_some() {
                    let limit_param = 2 + values.len();
                    let offset_param = limit_param + 1;
                    sql.push_str(&format!(" LIMIT ${limit_param} OFFSET ${offset_param}"));
                }

                let mut query = sqlx::query_scalar::<_, i32>(&sql).bind(account_id_val);
                for value in &values {
                    query = match value {
                        SqlValue::Str(v) => query.bind(v.clone()),
                        SqlValue::Int(v) => query.bind(*v as i32),
                        SqlValue::Float(v) => query.bind(*v),
                        SqlValue::Date(v) => query.bind(*v),
                    };
                }
                if let Some(limit) = limit {
                    query = query.bind(limit).bind(offset);
                }

                let item_ids = query.fetch_all(&pool).await.map_err(RepoError::from)?;
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
                    "UPDATE attribute SET key = $1
                     WHERE key = $2 AND item_id = (
                         SELECT item_id FROM item WHERE item_id = $3 AND account_id = $4
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
                    "UPDATE attribute_list_value SET key = $1
                     WHERE key = $2 AND item_id = (
                         SELECT item_id FROM item WHERE item_id = $3 AND account_id = $4
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
                     WHERE item_id = (SELECT item_id FROM item WHERE item_id = $1 AND account_id = $2)
                     AND key = $3",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(&key)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                sqlx::query(
                    "DELETE FROM attribute_list_value
                     WHERE item_id = (SELECT item_id FROM item WHERE item_id = $1 AND account_id = $2)
                     AND key = $3",
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn item_kind_map() -> HashMap<String, AttributeBaseScalarType> {
        HashMap::from([(String::from("Parent"), AttributeBaseScalarType::Item)])
    }

    #[test]
    fn all_list_mode_accepts_scalar_values() {
        let filter = AttributeFilter {
            key: String::from("Parent"),
            op: AttributeFilterOp::Equal,
            value: json!(68),
            list_mode: AttributeListMode::All,
        };

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map(), 2).unwrap();

        assert!(clause.contains("EXISTS"));
        assert_eq!(values.len(), 4);
    }

    #[test]
    fn array_values_expand_for_any_list_mode() {
        let filter = AttributeFilter {
            key: String::from("Parent"),
            op: AttributeFilterOp::Equal,
            value: json!([68, 1808]),
            list_mode: AttributeListMode::Any,
        };

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map(), 2).unwrap();

        assert!(clause.contains(" OR "));
        assert!(clause.contains("$9"));
        assert_eq!(values.len(), 8);
    }

    #[test]
    fn array_values_expand_for_all_list_mode() {
        let filter = AttributeFilter {
            key: String::from("Parent"),
            op: AttributeFilterOp::Equal,
            value: json!([68, 1808]),
            list_mode: AttributeListMode::All,
        };

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map(), 2).unwrap();

        assert!(clause.contains(" AND "));
        assert!(clause.contains("$9"));
        assert_eq!(values.len(), 8);
    }
}
