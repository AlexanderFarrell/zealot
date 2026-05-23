use axum::{Extension, Json, Router, extract::{Path, State}, http::StatusCode, middleware, routing::{delete, get, patch, post}};
use serde::Deserialize;
use zealot_app::app::AppState;
use zealot_domain::{account::{ApiKeyRecordDto, CreateApiKeyResponseDto}, auth::Actor, common::id::Id};

use crate::http::{common::HttpError, middleware::{auth_middleware, csrf_middleware}};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/settings", patch(update_settings))
        .route("/api-keys", get(list_api_keys))
        .route("/api-keys", post(create_api_key))
        .route("/api-keys/{id}", delete(revoke_api_key))
        .route_layer(middleware::from_fn_with_state(state.clone(), csrf_middleware))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

async fn update_settings(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(settings): Json<serde_json::Value>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    state.services.account
        .update_settings(&account.account_id, settings)
        .map_err(|_| HttpError::Internal)?;
    Ok(StatusCode::OK)
}

async fn list_api_keys(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<ApiKeyRecordDto>>, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let keys = state.services.account
        .list_api_keys(&account.account_id)
        .map_err(|_| HttpError::Internal)?;
    Ok(Json(keys.into_iter().map(ApiKeyRecordDto::from).collect()))
}

#[derive(Deserialize)]
struct CreateApiKeyBody {
    label: Option<String>,
}

async fn create_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<CreateApiKeyBody>,
) -> Result<Json<CreateApiKeyResponseDto>, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let label = body.label.unwrap_or_else(|| "Default".to_string());
    let (record, raw_key) = state.services.account
        .generate_api_key(&account.account_id, &label)
        .map_err(|_| HttpError::Internal)?;
    Ok(Json(CreateApiKeyResponseDto {
        key: raw_key,
        api_key_id: record.api_key_id.into(),
        label: record.label,
        created_at: record.created_at,
    }))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let api_key_id = Id::try_from(id).map_err(|_| HttpError::UserError { err: "Invalid API key id".to_string() })?;
    state.services.account
        .revoke_api_key(&api_key_id, &account.account_id)
        .map_err(|_| HttpError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}
