use std::collections::HashMap;

use chrono::NaiveDate;
use serde_json::Value;
use sqlx::SqlitePool;
use zealot_app::repos::{
    common::RepoError,
    item_attribute_value::{ItemAttributeValueRepo, ItemQuery},
};
use zealot_domain::{
    account::Account,
    attribute::{
        Attribute, AttributeBaseScalarType, AttributeFilter, AttributeFilterOp, AttributeListMode,
        AttributeScalar,
    },
    common::id::Id,
    item::relationship,
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
                    .map(|value| ("value_int", SqlValue::Int(value)))
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
                    .map(|value| ("value_num", SqlValue::Float(value)))
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
                    let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                    let days = date.signed_duration_since(epoch).num_days();
                    Ok(("value_date", SqlValue::Int(days * 86400)))
                }
                _ => Err(RepoError::DatabaseError {
                    err: format!("filter '{}' expects a date string", key),
                }),
            },
            AttributeBaseScalarType::Item => match value {
                Value::Number(value) => value
                    .as_i64()
                    .map(|value| ("value_item_id", SqlValue::Int(value)))
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

fn bind_sql_value<'q>(
    query: sqlx::query::QueryScalar<'q, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'q>>,
    value: &SqlValue,
) -> sqlx::query::QueryScalar<'q, sqlx::Sqlite, i64, sqlx::sqlite::SqliteArguments<'q>> {
    match value {
        SqlValue::Str(v) => query.bind(v.clone()),
        SqlValue::Int(v) => query.bind(*v),
        SqlValue::Float(v) => query.bind(*v),
    }
}

fn build_attr_filter_clause(
    filter: &AttributeFilter,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<(String, Vec<SqlValue>), RepoError> {
    let (col, values) = attr_filter_col_and_values(filter, kind_map)?;
    let (op_str, values, case_insensitive) = match &filter.op {
        AttributeFilterOp::Equal => ("=", values, false),
        AttributeFilterOp::NotEqual => ("!=", values, false),
        AttributeFilterOp::GreaterThan => (">", values, false),
        AttributeFilterOp::LessThan => ("<", values, false),
        AttributeFilterOp::GreaterThanOrEqualTo => (">=", values, false),
        AttributeFilterOp::LessThanOrEqualTo => ("<=", values, false),
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
            ("LIKE", values, true)
        }
    };
    let list_comparison = if case_insensitive {
        format!("LOWER(alv.{col}) {op} LOWER(?)", col = col, op = op_str)
    } else {
        format!("alv.{col} {op} ?", col = col, op = op_str)
    };
    let scalar_comparison = if case_insensitive {
        format!("LOWER(a.{col}) {op} LOWER(?)", col = col, op = op_str)
    } else {
        format!("a.{col} {op} ?", col = col, op = op_str)
    };

    let mut clauses = Vec::new();
    let mut clause_values = Vec::new();
    for value in values {
        clauses.push(format!(
            "(EXISTS (SELECT 1 FROM attribute_list_value alv WHERE alv.item_id = i.item_id AND alv.key = ? AND {list_comparison})
             OR EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = ? AND {scalar_comparison}))",
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

/// Scalar column that stores values of this attribute type, for sorting.
fn sort_column(base_type: &AttributeBaseScalarType) -> &'static str {
    match base_type {
        AttributeBaseScalarType::Text
        | AttributeBaseScalarType::Dropdown
        | AttributeBaseScalarType::Week => "value_text",
        AttributeBaseScalarType::Integer | AttributeBaseScalarType::Boolean => "value_int",
        AttributeBaseScalarType::Decimal => "value_num",
        AttributeBaseScalarType::Date => "value_date",
        AttributeBaseScalarType::Item => "value_item_id",
    }
}

/// WHERE clause (after `i.account_id = ?`), its bind values, and the ORDER BY with its bind values.
struct QuerySql {
    where_sql: String,
    where_values: Vec<SqlValue>,
    order_sql: String,
    order_values: Vec<SqlValue>,
}

fn build_query_sql(
    query: &ItemQuery,
    kind_map: &HashMap<String, AttributeBaseScalarType>,
) -> Result<QuerySql, RepoError> {
    let mut parts: Vec<String> = Vec::new();
    if !query.include_archived {
        parts.push("NOT EXISTS (SELECT 1 FROM attribute archived WHERE archived.item_id = i.item_id AND archived.key = 'Archived' AND archived.value_int = 1)".into());
    }
    let mut values: Vec<SqlValue> = Vec::new();

    if !query.types.is_empty() {
        let type_exists = |placeholders: &str| {
            format!(
                "EXISTS (SELECT 1 FROM item_item_type_link l JOIN item_type t ON t.type_id = l.type_id
                 WHERE l.item_id = i.item_id AND t.name IN ({placeholders}))"
            )
        };
        if query.all_types {
            for name in &query.types {
                parts.push(type_exists("?"));
                values.push(SqlValue::Str(name.clone()));
            }
        } else {
            let placeholders = query
                .types
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            parts.push(type_exists(&placeholders));
            values.extend(query.types.iter().map(|n| SqlValue::Str(n.clone())));
        }
    }

    if let Some(root) = query.under {
        let root = i64::from(root);
        // UNION (not UNION ALL) drops repeats, so a cycle in Parent links terminates.
        parts.push(String::from(
            "(i.item_id IN (WITH RECURSIVE d(id) AS (
                 SELECT first_item_id FROM item_item_link WHERE second_item_id = ? AND relationship = ?
                 UNION
                 SELECT l.first_item_id FROM item_item_link l JOIN d ON l.second_item_id = d.id
                 WHERE l.relationship = ?
             ) SELECT id FROM d) AND i.item_id <> ?)",
        ));
        values.push(SqlValue::Int(root));
        values.push(SqlValue::Str(relationship::PARENT.to_string()));
        values.push(SqlValue::Str(relationship::PARENT.to_string()));
        values.push(SqlValue::Int(root));
    }

    for filter in &query.filters {
        let (clause, clause_values) = build_attr_filter_clause(filter, kind_map)?;
        parts.push(format!("({clause})"));
        values.extend(clause_values);
    }

    let mut order_values = Vec::new();
    let order_sql = match &query.sort {
        None => String::from("i.item_id"),
        Some(sort) => {
            let dir = if sort.descending { "DESC" } else { "ASC" };
            match sort.key.to_lowercase().as_str() {
                "title" => format!("LOWER(i.title) {dir}, i.item_id"),
                "created" => format!("i.created_at {dir}, i.item_id"),
                "updated" => format!("i.updated_at {dir}, i.item_id"),
                _ => {
                    let base_type = kind_map.get(&sort.key).ok_or_else(|| RepoError::DatabaseError {
                        err: format!(
                            "cannot sort by '{}': not title, created, updated, or a scalar attribute",
                            sort.key
                        ),
                    })?;
                    let expr = format!(
                        "(SELECT a.{} FROM attribute a WHERE a.item_id = i.item_id AND a.key = ?)",
                        sort_column(base_type)
                    );
                    // Items without the attribute sort last in either direction.
                    order_values.push(SqlValue::Str(sort.key.clone()));
                    order_values.push(SqlValue::Str(sort.key.clone()));
                    format!("{expr} IS NULL, {expr} {dir}, i.item_id")
                }
            }
        }
    };
    let where_sql = if parts.is_empty() {
        String::from("1 = 1")
    } else {
        parts.join(" AND ")
    };
    Ok(QuerySql {
        where_sql,
        where_values: values,
        order_sql,
        order_values,
    })
}

impl ItemAttributeValueRepo for ItemAttributeValueSqliteRepo {
    fn set_archived(
        &self,
        root: &Id,
        archived: bool,
        subtree: bool,
        account: &Account,
    ) -> Result<Vec<Id>, RepoError> {
        let root = i64::from(*root);
        let account_id = i64::from(account.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
            let mut tx = self.pool.begin().await.map_err(RepoError::from)?;
            // UNION terminates cycles and deduplicates descendants with multiple parents.
            let ids: Vec<i64> = sqlx::query_scalar("WITH RECURSIVE d(id) AS (
                SELECT item_id FROM item WHERE item_id = ?1 AND account_id = ?2
                UNION
                SELECT l.first_item_id FROM item_item_link l JOIN d ON l.second_item_id = d.id
                JOIN item child ON child.item_id = l.first_item_id
                WHERE l.relationship = 'parent' AND child.account_id = ?2 AND ?3
            ) SELECT id FROM d ORDER BY id")
                .bind(root).bind(account_id).bind(subtree).fetch_all(&mut *tx).await.map_err(RepoError::from)?;
            for id in &ids {
                sqlx::query("INSERT INTO attribute (item_id, key, value_int) VALUES (?1, 'Archived', ?2)
                    ON CONFLICT (item_id, key) DO UPDATE SET value_int = excluded.value_int")
                    .bind(*id).bind(if archived { 1i64 } else { 0i64 })
                    .execute(&mut *tx).await.map_err(RepoError::from)?;
            }
            tx.commit().await.map_err(RepoError::from)?;
            ids.into_iter().map(|id| Id::try_from(id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })).collect()
        })
        })
    }

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

    fn query_item_ids(
        &self,
        query: &ItemQuery,
        account_id: &Id,
    ) -> Result<(Vec<Id>, i64), RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        let query = query.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let kind_map = get_kind_scalar_map(account_id_val, &pool).await?;
                let built = build_query_sql(&query, &kind_map)?;

                let count_sql = format!(
                    "SELECT COUNT(*) FROM item i WHERE i.account_id = ? AND {}",
                    built.where_sql
                );
                let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql).bind(account_id_val);
                for value in &built.where_values {
                    count_query = bind_sql_value(count_query, value);
                }
                let total = count_query
                    .fetch_one(&pool)
                    .await
                    .map_err(RepoError::from)?;

                let page_sql = format!(
                    "SELECT i.item_id FROM item i WHERE i.account_id = ? AND {}
                     ORDER BY {} LIMIT ? OFFSET ?",
                    built.where_sql, built.order_sql
                );
                let mut page_query = sqlx::query_scalar::<_, i64>(&page_sql).bind(account_id_val);
                for value in built.where_values.iter().chain(&built.order_values) {
                    page_query = bind_sql_value(page_query, value);
                }
                let ids = page_query
                    .bind(query.limit)
                    .bind(query.offset)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;
                let ids = ids
                    .into_iter()
                    .map(|item_id| {
                        Id::try_from(item_id).map_err(|err| RepoError::DatabaseError {
                            err: err.to_string(),
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((ids, total))
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
                let mut where_parts = Vec::new();
                let mut values = Vec::new();
                let kind_map = get_kind_scalar_map(account_id_val, &pool).await?;

                for filter in &filters {
                    let (clause, clause_values) = build_attr_filter_clause(filter, &kind_map)?;
                    where_parts.push(format!("({})", clause));
                    values.extend(clause_values);
                }

                let mut sql = format!(
                    "SELECT DISTINCT i.item_id FROM item i
                     WHERE i.account_id = ? AND {}
                     ORDER BY i.item_id",
                    where_parts.join(" AND ")
                );
                if limit.is_some() {
                    sql.push_str(" LIMIT ? OFFSET ?");
                }

                let mut query = sqlx::query_scalar::<_, i64>(&sql).bind(account_id_val);
                for value in &values {
                    query = match value {
                        SqlValue::Str(value) => query.bind(value.clone()),
                        SqlValue::Int(value) => query.bind(*value),
                        SqlValue::Float(value) => query.bind(*value),
                    };
                }
                if let Some(limit) = limit {
                    query = query.bind(limit).bind(offset);
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

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map()).unwrap();

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

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map()).unwrap();

        assert!(clause.contains(" OR "));
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

        let (clause, values) = build_attr_filter_clause(&filter, &item_kind_map()).unwrap();

        assert!(clause.contains(" AND "));
        assert_eq!(values.len(), 8);
    }
}
