use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::get,
};
use serde::Deserialize;
use zealot_app::app::AppState;
use zealot_app::services::attribute::AttributeServiceError;
use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto},
    auth::Actor,
    common::id::Id,
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
};

#[derive(Deserialize)]
struct DeleteParams {
    #[serde(default)]
    force: bool,
}

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_attribute_kinds).post(add_attribute_kind))
        .route(
            "/id/{kind_id}",
            get(get_attribute_kind_by_id).patch(update_attribute_kind),
        )
        .route(
            "/key/{key}",
            get(get_attribute_kind_by_key).delete(delete_attribute_kind),
        )
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            csrf_middleware,
        ))
        .route_layer(middleware::map_request_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

async fn get_attribute_kinds(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<serde_json::Value>>, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .attribute
        .get_kinds_for_user(&account.account_id)
        .map(|kinds| Json(kinds.iter().map(kind_to_json).collect()))
        .map_err(|e| {
            tracing::error!(account_id = ?account.account_id, %e, "failed to get attribute kinds");
            HttpError::Internal
        })
}

async fn get_attribute_kind_by_id(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(kind_id): Path<i64>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(kind_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    match state
        .services
        .attribute
        .get_kind_by_id(&id, &account.account_id)
    {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::NotFound),
        Err(e) => {
            tracing::error!(account_id = ?account.account_id, %e, "failed to get attribute kind by id");
            Err(HttpError::Internal)
        }
    }
}

async fn get_attribute_kind_by_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .attribute
        .get_kind_by_key(&key, &account.account_id)
    {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::NotFound),
        Err(e) => {
            tracing::error!(account_id = ?account.account_id, key = %key, %e, "failed to get attribute kind by key");
            Err(HttpError::Internal)
        }
    }
}

async fn add_attribute_kind(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddAttributeKindDto>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .attribute
        .add_attribute_kind(&dto, &account.account_id)
    {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => {
            tracing::error!(account_id = ?account.account_id, "add_attribute_kind returned None");
            Err(HttpError::Internal)
        }
        Err(AttributeServiceError::InvalidDefault(err)) => Err(HttpError::UserError { err }),
        Err(e) => {
            tracing::error!(account_id = ?account.account_id, %e, "failed to add attribute kind");
            Err(HttpError::Internal)
        }
    }
}

async fn update_attribute_kind(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(kind_id): Path<i64>,
    Json(mut dto): Json<UpdateAttributeKindDto>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    dto.kind_id = kind_id;
    match state
        .services
        .attribute
        .update_attribute_kind(&dto, &account.account_id)
    {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) | Err(AttributeServiceError::NotFound) => Err(HttpError::NotFound),
        Err(AttributeServiceError::ReadOnly(err) | AttributeServiceError::InvalidDefault(err)) => {
            Err(HttpError::UserError { err })
        }
        Err(e) => {
            tracing::error!(account_id = ?account.account_id, %e, "failed to update attribute kind");
            Err(HttpError::Internal)
        }
    }
}

async fn delete_attribute_kind(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(key): Path<String>,
    Query(params): Query<DeleteParams>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .attribute
        .delete_attribute_kind(&key, &account.account_id, params.force)
        .map(|_| StatusCode::OK)
        .map_err(|e| match e {
            AttributeServiceError::InUse(msg) => HttpError::Conflict { message: msg },
            AttributeServiceError::ReadOnly(err) => HttpError::UserError { err },
            e => {
                tracing::error!(account_id = ?account.account_id, key = %key, %e, "failed to delete attribute kind");
                HttpError::Internal
            }
        })
}

fn kind_to_json(kind: &AttributeKind) -> serde_json::Value {
    let mut config = spec_to_config_json(&kind.spec);
    // A list kind carries its inner type's spec (dropdown values, min/max, ...); name the inner type too.
    if let zealot_domain::attribute::AttributeBaseType::List(inner) = &kind.base_type {
        let inner = base_type_str(&zealot_domain::attribute::AttributeBaseType::Scalar(
            inner.clone(),
        ));
        if let Some(map) = config.as_object_mut() {
            map.insert("list_type".into(), inner.into());
        }
    }
    // Services and clients read the default from `config.default`.
    if let (Some(default), Some(map)) = (&kind.default, config.as_object_mut()) {
        map.insert("default".into(), serde_json::json!(default));
    }
    serde_json::json!({
        "kind_id": i64::from(kind.kind_id),
        "key": kind.key,
        "description": kind.description,
        "is_system": kind.is_system,
        "base_type": base_type_str(&kind.base_type),
        "config": config,
    })
}

fn base_type_str(bt: &zealot_domain::attribute::AttributeBaseType) -> &'static str {
    use zealot_domain::attribute::{AttributeBaseScalarType, AttributeBaseType};
    match bt {
        AttributeBaseType::List(_) => "list",
        AttributeBaseType::Scalar(s) => match s {
            AttributeBaseScalarType::Text => "text",
            AttributeBaseScalarType::Integer => "integer",
            AttributeBaseScalarType::Decimal => "decimal",
            AttributeBaseScalarType::Date => "date",
            AttributeBaseScalarType::Week => "week",
            AttributeBaseScalarType::Dropdown => "dropdown",
            AttributeBaseScalarType::Boolean => "boolean",
            AttributeBaseScalarType::Item => "item",
            AttributeBaseScalarType::Email => "email",
            AttributeBaseScalarType::Telephone => "telephone",
            AttributeBaseScalarType::Url => "url",
            AttributeBaseScalarType::Media => "media",
            AttributeBaseScalarType::Place => "place",
        },
    }
}

fn spec_to_config_json(spec: &zealot_domain::attribute::AttributeKindSpec) -> serde_json::Value {
    use zealot_domain::attribute::{AttributeBaseType, AttributeKindSpec};
    match spec {
        AttributeKindSpec::Text {
            min_len,
            max_len,
            pattern,
        } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min_len {
                m.insert("min_len".into(), (*v as u64).into());
            }
            if let Some(v) = max_len {
                m.insert("max_len".into(), (*v as u64).into());
            }
            if let Some(v) = pattern {
                m.insert("pattern".into(), v.clone().into());
            }
            serde_json::Value::Object(m)
        }
        AttributeKindSpec::Integer { min, max } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min {
                m.insert("min".into(), (*v).into());
            }
            if let Some(v) = max {
                m.insert("max".into(), (*v).into());
            }
            serde_json::Value::Object(m)
        }
        AttributeKindSpec::Decimal { min, max } => {
            let mut m = serde_json::Map::new();
            if let Some(v) = min {
                if let Some(n) = serde_json::Number::from_f64(*v) {
                    m.insert("min".into(), n.into());
                }
            }
            if let Some(v) = max {
                if let Some(n) = serde_json::Number::from_f64(*v) {
                    m.insert("max".into(), n.into());
                }
            }
            serde_json::Value::Object(m)
        }
        AttributeKindSpec::Dropdown { values } => {
            let arr: Vec<serde_json::Value> = values.iter().map(|v| v.clone().into()).collect();
            let mut m = serde_json::Map::new();
            m.insert("values".into(), arr.into());
            serde_json::Value::Object(m)
        }
        AttributeKindSpec::List { list_type } => {
            let inner = match list_type {
                AttributeBaseType::List(t) | AttributeBaseType::Scalar(t) => {
                    base_type_str(&AttributeBaseType::Scalar(t.clone()))
                }
            };
            let mut m = serde_json::Map::new();
            m.insert("list_type".into(), inner.into());
            serde_json::Value::Object(m)
        }
        _ => serde_json::Value::Object(serde_json::Map::new()),
    }
}

#[cfg(test)]
mod kind_json_tests {
    use super::*;
    use zealot_domain::{
        attribute::{AttributeBaseScalarType, AttributeBaseType, AttributeKindSpec},
        common::id::Id,
    };

    /// A list of dropdown values (multi-select) reports its inner type and allowed values.
    #[test]
    fn list_kind_reports_inner_type_and_values() {
        let kind = AttributeKind {
            kind_id: Id::try_from(1).unwrap(),
            default: None,
            is_system: false,
            key: "Tags".into(),
            description: String::new(),
            base_type: AttributeBaseType::List(AttributeBaseScalarType::Dropdown),
            spec: AttributeKindSpec::Dropdown {
                values: vec!["a".into(), "b".into()],
            },
        };
        let json = kind_to_json(&kind);
        assert_eq!(json["base_type"], "list");
        assert_eq!(json["config"]["list_type"], "dropdown");
        assert_eq!(json["config"]["values"], serde_json::json!(["a", "b"]));
    }

    /// A kind's default is reported in `config.default`; a kind without one has no such key.
    #[test]
    fn kind_reports_default_in_config() {
        use zealot_domain::attribute::{Attribute, AttributeScalar};
        let mut kind = AttributeKind {
            kind_id: Id::try_from(1).unwrap(),
            default: None,
            is_system: false,
            key: "Priority".into(),
            description: String::new(),
            base_type: AttributeBaseType::Scalar(AttributeBaseScalarType::Integer),
            spec: AttributeKindSpec::Integer {
                min: Some(1),
                max: Some(10),
            },
        };
        assert!(kind_to_json(&kind)["config"].get("default").is_none());
        kind.default = Some(Attribute::Scalar(AttributeScalar::Integer(3)));
        assert_eq!(kind_to_json(&kind)["config"]["default"], 3);
    }
}
