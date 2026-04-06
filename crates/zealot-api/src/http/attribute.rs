use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use zealot_app::app::AppState;
use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto},
    auth::Actor,
    common::id::Id,
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_attribute_kinds).post(add_attribute_kind))
        .route("/id/{kind_id}", get(get_attribute_kind_by_id).patch(update_attribute_kind))
        .route("/key/{key}", get(get_attribute_kind_by_key).delete(delete_attribute_kind))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
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
        .map_err(|_| HttpError::Internal)
}

async fn get_attribute_kind_by_id(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(kind_id): Path<i64>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(kind_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    match state.services.attribute.get_kind_by_id(&id, &account.account_id) {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::NotFound),
        Err(_) => Err(HttpError::Internal),
    }
}

async fn get_attribute_kind_by_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(key): Path<String>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    match state.services.attribute.get_kind_by_key(&key, &account.account_id) {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::NotFound),
        Err(_) => Err(HttpError::Internal),
    }
}

async fn add_attribute_kind(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddAttributeKindDto>,
) -> Result<Json<serde_json::Value>, HttpError> {
    let account = require_account(&actor)?;
    match state.services.attribute.add_attribute_kind(&dto, &account.account_id) {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::Internal),
        Err(_) => Err(HttpError::Internal),
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
    match state.services.attribute.update_attribute_kind(&dto, &account.account_id) {
        Ok(Some(kind)) => Ok(Json(kind_to_json(&kind))),
        Ok(None) => Err(HttpError::NotFound),
        Err(_) => Err(HttpError::Internal),
    }
}

async fn delete_attribute_kind(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(key): Path<String>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .attribute
        .delete_attribute_kind(&key, &account.account_id)
        .map(|_| StatusCode::OK)
        .map_err(|_| HttpError::Internal)
}

fn kind_to_json(kind: &AttributeKind) -> serde_json::Value {
    serde_json::json!({
        "kind_id": i64::from(kind.kind_id),
        "key": kind.key,
        "description": kind.description,
        "is_system": kind.is_system,
        "base_type": format!("{:?}", kind.base_type),
    })
}
