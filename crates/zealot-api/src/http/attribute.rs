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
        Err(AttributeServiceError::Duplicate(message)) => Err(HttpError::Conflict { message }),
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
        Err(AttributeServiceError::Duplicate(message)) => Err(HttpError::Conflict { message }),
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
    kind.to_json()
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
