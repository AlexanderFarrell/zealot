use std::collections::HashMap;

use chrono::NaiveDate;
use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, item::ItemRepo};
use zealot_domain::{
    account::Account,
    attribute::{
        Attribute, AttributeBaseScalarType, AttributeBaseType, AttributeFilter,
        AttributeFilterOp, AttributeKind, AttributeListMode, AttributeScalar,
    },
    common::id::Id,
    item::{AddItemParsedDto, Item, UpdateItemParsedDto},
    item_type::ItemType,
};

#[derive(Debug)]
pub struct ItemSqliteRepo {
    pub(crate) pool: SqlitePool,
}

impl ItemSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

// ─── Row structs ────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct ItemRow {
    item_id: i64,
    title: String,
    content: String,
    account_id: i64,
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
struct ItemTypeForItemRow {
    item_id: i64,
    type_id: i64,
    name: String,
    description: String,
    account_id: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct AttrKindRow {
    kind_id: i64,
    key: String,
    base_type: String,
}

// ─── Attribute decoding helpers ──────────────────────────────────────────────

/// Decode a scalar attribute column set using the attribute kind for type info.
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
                    let date = NaiveDate::from_num_days_from_ce_opt(
                        (ts / 86400 + 719163) as i32
                    );
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

    // Fallback: first non-null column
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

/// Convert an `AttributeScalar` to `(value_text, value_num, value_int, value_date, value_item_id)`.
fn scalar_to_cols(scalar: &AttributeScalar) -> (Option<String>, Option<f64>, Option<i64>, Option<i64>, Option<i64>) {
    match scalar {
        AttributeScalar::Text(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Dropdown(s) => (Some(s.clone()), None, None, None, None),
        AttributeScalar::Week(w) => (Some(w.to_string()), None, None, None, None),
        AttributeScalar::Integer(i) => (None, None, Some(*i), None, None),
        AttributeScalar::Boolean(b) => (None, None, Some(*b as i64), None, None),
        AttributeScalar::Decimal(f) => (None, Some(*f), None, None, None),
        AttributeScalar::Date(d) => {
            // Store as Unix timestamp (seconds since epoch)
            let epoch = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
            let days = d.signed_duration_since(epoch).num_days();
            let ts = days * 86400;
            (None, None, None, Some(ts), None)
        }
        AttributeScalar::Item(id) => (None, None, None, None, Some(i64::from(*id))),
    }
}

// ─── Bulk hydration ───────────────────────────────────────────────────────────

async fn hydrate_items(
    rows: Vec<ItemRow>,
    account_id: i64,
    pool: &SqlitePool,
) -> Result<Vec<Item>, RepoError> {
    if rows.is_empty() {
        return Ok(vec![]);
    }

    let ids: Vec<i64> = rows.iter().map(|r| r.item_id).collect();

    // 1. Fetch attribute kinds for the account to decode column types
    let kind_rows = sqlx::query_as::<_, AttrKindRow>(
        "SELECT kind_id, key, base_type FROM attribute_kind
         WHERE account_id = ? OR is_system = 1",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .map_err(RepoError::from)?;

    // Build a map of key -> scalar base type (for lists, unwrap the inner type)
    let mut kind_scalar_map: HashMap<String, AttributeBaseScalarType> = HashMap::new();
    for kr in &kind_rows {
        let scalar = match kr.base_type.as_str() {
            "text" => AttributeBaseScalarType::Text,
            "integer" => AttributeBaseScalarType::Integer,
            "decimal" => AttributeBaseScalarType::Decimal,
            "date" => AttributeBaseScalarType::Date,
            "week" => AttributeBaseScalarType::Week,
            "dropdown" => AttributeBaseScalarType::Dropdown,
            "boolean" => AttributeBaseScalarType::Boolean,
            "item" => AttributeBaseScalarType::Item,
            // For list kinds the inner type is in config, but we don't have it here.
            // Use Text as a safe default for the fallback decoder.
            "list" => AttributeBaseScalarType::Text,
            _ => continue,
        };
        kind_scalar_map.insert(kr.key.clone(), scalar);
    }

    // 2. Fetch scalar attributes for all items
    let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
    let scalar_sql = format!(
        "SELECT item_id, key, value_text, value_num, value_int, value_date, value_item_id
         FROM attribute WHERE item_id IN ({})",
        placeholders
    );
    let mut scalar_query = sqlx::query_as::<_, AttrScalarRow>(&scalar_sql);
    for id in &ids {
        scalar_query = scalar_query.bind(id);
    }
    let scalar_rows = scalar_query.fetch_all(pool).await.map_err(RepoError::from)?;

    // 3. Fetch list attributes for all items
    let list_sql = format!(
        "SELECT item_id, key, ordinal, value_text, value_num, value_int, value_date, value_item_id
         FROM attribute_list_value WHERE item_id IN ({}) ORDER BY item_id, key, ordinal",
        placeholders
    );
    let mut list_query = sqlx::query_as::<_, AttrListRow>(&list_sql);
    for id in &ids {
        list_query = list_query.bind(id);
    }
    let list_rows = list_query.fetch_all(pool).await.map_err(RepoError::from)?;

    // 4. Fetch item types for all items
    let type_sql = format!(
        "SELECT lnk.item_id, it.type_id, it.name, it.description, it.account_id
         FROM item_item_type_link lnk
         JOIN item_type it ON lnk.type_id = it.type_id
         WHERE lnk.item_id IN ({}) AND (it.account_id = ? OR it.account_id IS NULL)",
        placeholders
    );
    let mut type_query = sqlx::query_as::<_, ItemTypeForItemRow>(&type_sql);
    for id in &ids {
        type_query = type_query.bind(id);
    }
    type_query = type_query.bind(account_id);
    let type_rows = type_query.fetch_all(pool).await.map_err(RepoError::from)?;

    // 5. Assemble items
    let mut item_map: HashMap<i64, Item> = HashMap::new();
    for row in rows {
        let item_id = Id::try_from(row.item_id)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        item_map.insert(row.item_id, Item {
            item_id,
            title: row.title,
            content: row.content,
            attributes: HashMap::new(),
            types: Vec::new(),
            related: Vec::new(),
        });
    }

    // 6. Decode scalar attributes
    for row in scalar_rows {
        if let Some(item) = item_map.get_mut(&row.item_id) {
            let bt = kind_scalar_map.get(&row.key);
            if let Some(scalar) = decode_scalar_from_cols(
                row.value_text.as_deref(),
                row.value_num,
                row.value_int,
                row.value_date,
                row.value_item_id,
                bt,
            ) {
                item.attributes.insert(row.key, Attribute::Scalar(scalar));
            }
        }
    }

    // 7. Decode list attributes — group by (item_id, key) maintaining ordinal order
    let mut list_map: HashMap<(i64, String), Vec<AttributeScalar>> = HashMap::new();
    for row in list_rows {
        let bt = kind_scalar_map.get(&row.key);
        if let Some(scalar) = decode_scalar_from_cols(
            row.value_text.as_deref(),
            row.value_num,
            row.value_int,
            row.value_date,
            row.value_item_id,
            bt,
        ) {
            list_map.entry((row.item_id, row.key)).or_default().push(scalar);
        }
    }
    for ((item_id, key), values) in list_map {
        if let Some(item) = item_map.get_mut(&item_id) {
            item.attributes.insert(key, Attribute::List(values));
        }
    }

    // 8. Attach types
    for row in type_rows {
        if let Some(item) = item_map.get_mut(&row.item_id) {
            let type_id = Id::try_from(row.type_id)
                .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
            item.types.push(ItemType {
                type_id,
                is_system: row.account_id.is_none(),
                name: row.name,
                description: row.description,
                required_attributes: Vec::new(), // not needed here
            });
        }
    }

    // Return in original order
    Ok(ids.into_iter().filter_map(|id| item_map.remove(&id)).collect())
}

// ─── Write helpers ────────────────────────────────────────────────────────────

async fn write_attributes(
    item_id: i64,
    attributes: &HashMap<String, Attribute>,
    pool: &SqlitePool,
) -> Result<(), RepoError> {
    for (key, attr) in attributes {
        match attr {
            Attribute::Scalar(scalar) => {
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
                .bind(item_id).bind(key)
                .bind(vt).bind(vn).bind(vi).bind(vd).bind(vid)
                .execute(pool)
                .await
                .map_err(RepoError::from)?;
            }
            Attribute::List(values) => {
                // Delete existing list values for this key
                sqlx::query(
                    "DELETE FROM attribute_list_value WHERE item_id = ? AND key = ?",
                )
                .bind(item_id).bind(key)
                .execute(pool)
                .await
                .map_err(RepoError::from)?;

                // Delete scalar attribute for this key if present
                sqlx::query(
                    "DELETE FROM attribute WHERE item_id = ? AND key = ?",
                )
                .bind(item_id).bind(key)
                .execute(pool)
                .await
                .map_err(RepoError::from)?;

                // Insert list values
                for (ordinal, scalar) in values.iter().enumerate() {
                    let (vt, vn, vi, vd, vid) = scalar_to_cols(scalar);
                    sqlx::query(
                        "INSERT INTO attribute_list_value
                         (item_id, key, ordinal, value_text, value_num, value_int, value_date, value_item_id)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(item_id).bind(key).bind(ordinal as i64)
                    .bind(vt).bind(vn).bind(vi).bind(vd).bind(vid)
                    .execute(pool)
                    .await
                    .map_err(RepoError::from)?;
                }
            }
        }
    }
    Ok(())
}

// ─── Filter query builder ─────────────────────────────────────────────────────

fn build_attr_filter_clause(
    filter: &AttributeFilter,
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

    // Determine which column and value to use
    let (col, value) = attr_filter_col_and_val(&filter.value)?;

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
                err: "list_mode 'all' is not yet supported in SQLite".to_string(),
            });
        }
    };

    Ok((clause, vec![
        SqlValue::Str(filter.key.clone()),
        value.clone(),
        SqlValue::Str(filter.key.clone()),
        value,
    ]))
}

#[derive(Clone)]
enum SqlValue {
    Str(String),
    Int(i64),
    Float(f64),
    Bool(bool),
}

fn attr_filter_col_and_val(value: &serde_json::Value) -> Result<(&'static str, SqlValue), RepoError> {
    match value {
        serde_json::Value::String(s) => Ok(("value_text", SqlValue::Str(s.clone()))),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(("value_int", SqlValue::Int(i)))
            } else if let Some(f) = n.as_f64() {
                Ok(("value_num", SqlValue::Float(f)))
            } else {
                Err(RepoError::DatabaseError { err: "unsupported number type in filter".into() })
            }
        }
        serde_json::Value::Bool(b) => Ok(("value_int", SqlValue::Int(*b as i64))),
        _ => Err(RepoError::DatabaseError { err: "unsupported filter value type".into() }),
    }
}

// ─── ItemRepo implementation ─────────────────────────────────────────────────

impl ItemRepo for ItemSqliteRepo {
    fn get_item_by_id(&self, item_id: &Id, account: &Account) -> Result<Option<Item>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let row = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content, account_id FROM item
                     WHERE item_id = ? AND account_id = ?",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                match row {
                    Some(r) => {
                        let items = hydrate_items(vec![r], account_id_val, &pool).await?;
                        Ok(items.into_iter().next())
                    }
                    None => Ok(None),
                }
            })
        })
    }

    fn get_items_by_title(&self, title: &str, account: &Account) -> Result<Vec<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let title = title.to_string();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content, account_id FROM item
                     WHERE title = ? AND account_id = ?",
                )
                .bind(&title)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn search_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let pattern = if term.trim().is_empty() {
            "%".to_string()
        } else {
            format!("%{}%", term)
        };
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content, account_id FROM item
                     WHERE title LIKE ? AND account_id = ?
                     ORDER BY item_id DESC LIMIT 20",
                )
                .bind(&pattern)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn regex_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<Item>, RepoError> {
        // TODO: SQLite regex requires a custom registered function. Falls back to LIKE.
        self.search_items_by_title(term, account)
    }

    fn get_items_by_type(&self, type_name: &str, account: &Account) -> Result<Vec<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let type_name = type_name.to_string();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT i.item_id, i.title, i.content, i.account_id FROM item i
                     JOIN item_item_type_link lnk ON lnk.item_id = i.item_id
                     JOIN item_type it ON it.type_id = lnk.type_id
                     WHERE it.name = ? AND i.account_id = ?",
                )
                .bind(&type_name)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn get_items_containing_attribute(&self, attr_key: &str, account: &Account) -> Result<Vec<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let attr_key = attr_key.to_string();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT i.item_id, i.title, i.content, i.account_id FROM item i
                     WHERE i.account_id = ?
                     AND (
                         EXISTS (SELECT 1 FROM attribute a WHERE a.item_id = i.item_id AND a.key = ?)
                         OR EXISTS (SELECT 1 FROM attribute_list_value alv WHERE alv.item_id = i.item_id AND alv.key = ?)
                     )",
                )
                .bind(account_id_val)
                .bind(&attr_key)
                .bind(&attr_key)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn get_items_by_attr_filter(
        &self,
        filters: &Vec<AttributeFilter>,
        account: &Account,
    ) -> Result<Vec<Item>, RepoError> {
        if filters.is_empty() {
            return Err(RepoError::DatabaseError { err: "no filters provided".into() });
        }

        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();

        // Build WHERE clause dynamically
        let mut where_parts: Vec<String> = Vec::new();
        let mut all_values: Vec<SqlValue> = Vec::new();

        for filter in filters {
            let (clause, values) = build_attr_filter_clause(filter)?;
            where_parts.push(format!("({})", clause));
            all_values.extend(values);
        }

        let sql = format!(
            "SELECT i.item_id, i.title, i.content, i.account_id FROM item i
             WHERE i.account_id = ? AND {}",
            where_parts.join(" AND ")
        );

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut query = sqlx::query_as::<_, ItemRow>(&sql).bind(account_id_val);
                for val in &all_values {
                    query = match val {
                        SqlValue::Str(s) => query.bind(s.clone()),
                        SqlValue::Int(i) => query.bind(*i),
                        SqlValue::Float(f) => query.bind(*f),
                        SqlValue::Bool(b) => query.bind(*b),
                    };
                }
                let rows = query.fetch_all(&pool).await.map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn get_related_items(&self, item_id: &Id, account: &Account) -> Result<Vec<Item>, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Items that have a list attribute pointing to this item_id
                let rows = sqlx::query_as::<_, ItemRow>(
                    "SELECT DISTINCT i.item_id, i.title, i.content, i.account_id FROM item i
                     JOIN attribute_list_value alv ON alv.item_id = i.item_id
                     WHERE alv.value_item_id = ? AND i.account_id = ? AND i.item_id != ?",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .bind(item_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                hydrate_items(rows, account_id_val, &pool).await
            })
        })
    }

    fn add_item(&self, dto: &AddItemParsedDto, account: &Account) -> Result<Option<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let title = dto.title.clone();
        let content = dto.content.clone();
        let attributes = dto.attributes.clone();
        let types = dto.types.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let item_id: i64 = sqlx::query_scalar(
                    "INSERT INTO item (title, content, account_id) VALUES (?, ?, ?) RETURNING item_id",
                )
                .bind(&title)
                .bind(&content)
                .bind(account_id_val)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                write_attributes(item_id, &attributes, &pool).await?;

                if let Some(type_names) = &types {
                    for name in type_names {
                        sqlx::query(
                            "INSERT OR IGNORE INTO item_item_type_link (item_id, type_id)
                             SELECT ?, type_id FROM item_type
                             WHERE name = ? AND (account_id = ? OR account_id IS NULL)",
                        )
                        .bind(item_id)
                        .bind(name)
                        .bind(account_id_val)
                        .execute(&pool)
                        .await
                        .map_err(RepoError::from)?;
                    }
                }

                let row = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content, account_id FROM item WHERE item_id = ?",
                )
                .bind(item_id)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;

                let items = hydrate_items(vec![row], account_id_val, &pool).await?;
                Ok(items.into_iter().next())
            })
        })
    }

    fn update_item(&self, dto: &UpdateItemParsedDto, account: &Account) -> Result<Option<Item>, RepoError> {
        let account_id_val = i64::from(account.account_id);
        let item_id_val = i64::from(dto.item_id);
        let title = dto.title.clone();
        let content = dto.content.clone();
        let attributes = dto.attributes.clone();
        let pool = self.pool.clone();

        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "UPDATE item SET
                       title = COALESCE(?, title),
                       content = COALESCE(?, content),
                       updated_at = strftime('%s', 'now')
                     WHERE item_id = ? AND account_id = ?",
                )
                .bind(title)
                .bind(content)
                .bind(item_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                if let Some(attrs) = &attributes {
                    write_attributes(item_id_val, attrs, &pool).await?;
                }

                let row = sqlx::query_as::<_, ItemRow>(
                    "SELECT item_id, title, content, account_id FROM item
                     WHERE item_id = ? AND account_id = ?",
                )
                .bind(item_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;

                match row {
                    Some(r) => {
                        let items = hydrate_items(vec![r], account_id_val, &pool).await?;
                        Ok(items.into_iter().next())
                    }
                    None => Ok(None),
                }
            })
        })
    }

    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query("DELETE FROM item WHERE item_id = ? AND account_id = ?")
                    .bind(item_id_val)
                    .bind(account_id_val)
                    .execute(&self.pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn set_item_attributes(
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
            tokio::runtime::Handle::current().block_on(async {
                // Verify the item belongs to the account
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
            tokio::runtime::Handle::current().block_on(async {
                let mut tx = pool.begin().await.map_err(RepoError::from)?;

                // Rename in attribute table (scalar)
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

                // Rename in attribute_list_value table
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
            tokio::runtime::Handle::current().block_on(async {
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

    fn assign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let names = type_names.clone();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for name in &names {
                    sqlx::query(
                        "INSERT OR IGNORE INTO item_item_type_link (item_id, type_id)
                         SELECT ?, type_id FROM item_type
                         WHERE name = ? AND (account_id = ? OR account_id IS NULL)",
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

    fn unassign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account: &Account,
    ) -> Result<(), RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let names = type_names.clone();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                for name in &names {
                    sqlx::query(
                        "DELETE FROM item_item_type_link
                         WHERE item_id = ? AND type_id = (
                             SELECT type_id FROM item_type
                             WHERE name = ? AND (account_id = ? OR account_id IS NULL)
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

    fn is_item_valid_for_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account: &Account,
    ) -> Result<bool, RepoError> {
        let item_id_val = i64::from(*item_id);
        let account_id_val = i64::from(account.account_id);
        let names = type_names.clone();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Get all attribute keys the item currently has
                let scalar_keys: Vec<String> = sqlx::query_scalar(
                    "SELECT DISTINCT key FROM attribute WHERE item_id = ?",
                )
                .bind(item_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let list_keys: Vec<String> = sqlx::query_scalar(
                    "SELECT DISTINCT key FROM attribute_list_value WHERE item_id = ?",
                )
                .bind(item_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;

                let mut item_keys: std::collections::HashSet<String> =
                    scalar_keys.into_iter().chain(list_keys).collect();

                for name in &names {
                    // Get required attribute keys for this type
                    let required: Vec<String> = sqlx::query_scalar(
                        "SELECT ak.key FROM attribute_kind ak
                         JOIN item_type_attribute_kind_link lnk ON lnk.attribute_kind_id = ak.kind_id
                         JOIN item_type it ON it.type_id = lnk.item_type_id
                         WHERE it.name = ? AND (it.account_id = ? OR it.account_id IS NULL)",
                    )
                    .bind(name)
                    .bind(account_id_val)
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?;

                    for key in required {
                        if !item_keys.contains(&key) {
                            return Ok(false);
                        }
                    }
                }

                Ok(true)
            })
        })
    }
}
