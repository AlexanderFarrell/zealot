use std::collections::HashMap;

use serde_json::Value;
use sqlx::SqlitePool;
use zealot_app::repos::{attribute::AttributeRepo, common::RepoError};
use zealot_domain::{
    attribute::{
        AddAttributeKindDto, AttributeBaseScalarType, AttributeBaseType, AttributeKind,
        AttributeKindSpec, UpdateAttributeKindDto,
    },
    common::id::Id,
};

#[derive(Debug)]
pub struct AttributeSqliteRepo {
    pool: SqlitePool,
}

impl AttributeSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct AttributeKindRow {
    kind_id: i64,
    account_id: Option<i64>,
    key: String,
    description: String,
    base_type: String,
    config: String,
    is_system: i64,
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

fn scalar_type_to_str(t: &AttributeBaseScalarType) -> &'static str {
    match t {
        AttributeBaseScalarType::Text => "text",
        AttributeBaseScalarType::Integer => "integer",
        AttributeBaseScalarType::Decimal => "decimal",
        AttributeBaseScalarType::Date => "date",
        AttributeBaseScalarType::Week => "week",
        AttributeBaseScalarType::Dropdown => "dropdown",
        AttributeBaseScalarType::Boolean => "boolean",
        AttributeBaseScalarType::Item => "item",
    }
}

fn row_to_attribute_kind(row: AttributeKindRow) -> Result<AttributeKind, RepoError> {
    let config: Value = serde_json::from_str(&row.config)
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let (base_type, spec) = match row.base_type.as_str() {
        "text" => {
            let min_len = config.get("min_len").and_then(|v| v.as_u64()).map(|v| v as usize);
            let max_len = config.get("max_len").and_then(|v| v.as_u64()).map(|v| v as usize);
            let pattern = config.get("pattern").and_then(|v| v.as_str()).map(String::from);
            (
                AttributeBaseType::Scalar(AttributeBaseScalarType::Text),
                AttributeKindSpec::Text { min_len, max_len, pattern },
            )
        }
        "integer" => {
            let min = config.get("min").and_then(|v| v.as_i64());
            let max = config.get("max").and_then(|v| v.as_i64());
            (
                AttributeBaseType::Scalar(AttributeBaseScalarType::Integer),
                AttributeKindSpec::Integer { min, max },
            )
        }
        "decimal" => {
            let min = config.get("min").and_then(|v| v.as_f64());
            let max = config.get("max").and_then(|v| v.as_f64());
            (
                AttributeBaseType::Scalar(AttributeBaseScalarType::Decimal),
                AttributeKindSpec::Decimal { min, max },
            )
        }
        "date" => (
            AttributeBaseType::Scalar(AttributeBaseScalarType::Date),
            AttributeKindSpec::Date,
        ),
        "week" => (
            AttributeBaseType::Scalar(AttributeBaseScalarType::Week),
            AttributeKindSpec::Week,
        ),
        "dropdown" => {
            let values = config
                .get("values")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            (
                AttributeBaseType::Scalar(AttributeBaseScalarType::Dropdown),
                AttributeKindSpec::Dropdown { values },
            )
        }
        "boolean" => (
            AttributeBaseType::Scalar(AttributeBaseScalarType::Boolean),
            AttributeKindSpec::Boolean,
        ),
        "item" => (
            AttributeBaseType::Scalar(AttributeBaseScalarType::Item),
            AttributeKindSpec::Item,
        ),
        "list" => {
            let inner_type_str = config
                .get("list_type")
                .and_then(|v| v.as_str())
                .unwrap_or("text");
            let inner = parse_scalar_type(inner_type_str)
                .unwrap_or(AttributeBaseScalarType::Text);
            (
                AttributeBaseType::List(inner.clone()),
                AttributeKindSpec::List {
                    list_type: AttributeBaseType::List(inner),
                },
            )
        }
        other => {
            return Err(RepoError::DatabaseError {
                err: format!("unknown base_type: {}", other),
            })
        }
    };

    let kind_id = Id::try_from(row.kind_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;

    Ok(AttributeKind {
        kind_id,
        is_system: row.is_system != 0,
        key: row.key,
        description: row.description,
        base_type,
        spec,
    })
}

fn spec_to_config(spec: &AttributeKindSpec) -> Value {
    match spec {
        AttributeKindSpec::Text { min_len, max_len, pattern } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min_len { m.insert("min_len".into(), Value::Number((*v as u64).into())); }
            if let Some(v) = max_len { m.insert("max_len".into(), Value::Number((*v as u64).into())); }
            if let Some(v) = pattern { m.insert("pattern".into(), Value::String(v.clone())); }
            Value::Object(m)
        }
        AttributeKindSpec::Integer { min, max } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min { m.insert("min".into(), Value::Number((*v).into())); }
            if let Some(v) = max { m.insert("max".into(), Value::Number((*v).into())); }
            Value::Object(m)
        }
        AttributeKindSpec::Decimal { min, max } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min {
                m.insert("min".into(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or(0.into())));
            }
            if let Some(v) = max {
                m.insert("max".into(), Value::Number(serde_json::Number::from_f64(*v).unwrap_or(0.into())));
            }
            Value::Object(m)
        }
        AttributeKindSpec::Dropdown { values } => {
            let arr: Vec<Value> = values.iter().map(|v| Value::String(v.clone())).collect();
            let mut m = serde_json::Map::new();
            m.insert("values".into(), Value::Array(arr));
            Value::Object(m)
        }
        AttributeKindSpec::List { list_type } => {
            let inner_str = match list_type {
                AttributeBaseType::List(t) => scalar_type_to_str(t),
                AttributeBaseType::Scalar(t) => scalar_type_to_str(t),
            };
            let mut m = serde_json::Map::new();
            m.insert("list_type".into(), Value::String(inner_str.into()));
            Value::Object(m)
        }
        _ => Value::Object(serde_json::Map::new()),
    }
}

fn base_type_to_db_str(bt: &AttributeBaseType) -> &'static str {
    match bt {
        AttributeBaseType::Scalar(t) => scalar_type_to_str(t),
        AttributeBaseType::List(_) => "list",
    }
}

impl AttributeRepo for AttributeSqliteRepo {
    fn get_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let key = key.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AttributeKindRow>(
                    "SELECT kind_id, account_id, key, description, base_type, config, is_system
                     FROM attribute_kind
                     WHERE key = ? AND (account_id = ? OR is_system = 1)",
                )
                .bind(&key)
                .bind(account_id_val)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_attribute_kind)
                .transpose()
            })
        })
    }

    fn get_attribute_kind_by_id(
        &self,
        id: &Id,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError> {
        let id_val = i64::from(*id);
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AttributeKindRow>(
                    "SELECT kind_id, account_id, key, description, base_type, config, is_system
                     FROM attribute_kind
                     WHERE kind_id = ? AND (account_id = ? OR is_system = 1)",
                )
                .bind(id_val)
                .bind(account_id_val)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_attribute_kind)
                .transpose()
            })
        })
    }

    fn get_attribute_kinds_for_user(
        &self,
        account_id: &Id,
    ) -> Result<HashMap<String, AttributeKind>, RepoError> {
        let account_id_val = i64::from(*account_id);
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let rows = sqlx::query_as::<_, AttributeKindRow>(
                    "SELECT kind_id, account_id, key, description, base_type, config, is_system
                     FROM attribute_kind
                     WHERE account_id = ? OR is_system = 1",
                )
                .bind(account_id_val)
                .fetch_all(&self.pool)
                .await
                .map_err(RepoError::from)?;

                let mut result = HashMap::new();
                for row in rows {
                    let key = row.key.clone();
                    let kind = row_to_attribute_kind(row)?;
                    result.insert(key, kind);
                }
                Ok(result)
            })
        })
    }

    fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let key = dto.key.clone();
        let description = dto.description.clone();
        let base_type = dto.base_type.clone();
        let config = dto.config.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AttributeKindRow>(
                    "INSERT INTO attribute_kind (account_id, key, description, base_type, config, is_system)
                     VALUES (?, ?, ?, ?, ?, 0)
                     RETURNING kind_id, account_id, key, description, base_type, config, is_system",
                )
                .bind(account_id_val)
                .bind(&key)
                .bind(&description)
                .bind(&base_type)
                .bind(&config)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_attribute_kind)
                .transpose()
            })
        })
    }

    fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let kind_id_val = dto.kind_id;
        let key = dto.key.clone();
        let description = dto.description.clone();
        let base_type = dto.base_type.clone();
        let config = dto.config.as_ref().map(|c| c.to_string());
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query_as::<_, AttributeKindRow>(
                    "UPDATE attribute_kind
                     SET key = COALESCE(?, key),
                         description = COALESCE(?, description),
                         base_type = COALESCE(?, base_type),
                         config = COALESCE(?, config)
                     WHERE kind_id = ? AND account_id = ?
                     RETURNING kind_id, account_id, key, description, base_type, config, is_system",
                )
                .bind(key)
                .bind(description)
                .bind(base_type)
                .bind(config)
                .bind(kind_id_val)
                .bind(account_id_val)
                .fetch_optional(&self.pool)
                .await
                .map_err(RepoError::from)?
                .map(row_to_attribute_kind)
                .transpose()
            })
        })
    }

    fn delete_attribute_kind(&self, key: &str, account_id: &Id) -> Result<(), RepoError> {
        let account_id_val = i64::from(*account_id);
        let key = key.to_string();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                sqlx::query(
                    "DELETE FROM attribute_kind WHERE key = ? AND account_id = ?",
                )
                .bind(&key)
                .bind(account_id_val)
                .execute(&self.pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
