use axum::{Extension, Json, Router, extract::State, http::StatusCode, middleware, routing::{delete, patch, post}};
use serde::Serialize;
use zealot_app::app::AppState;
use zealot_domain::auth::Actor;

use crate::http::{common::HttpError, middleware::{auth_middleware, csrf_middleware}};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/settings", patch(update_settings))
        .route("/api-key", post(create_api_key))
        .route("/api-key", delete(revoke_api_key))
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

#[derive(Serialize)]
struct ApiKeyResponse {
    key: String,
}

async fn create_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<ApiKeyResponse>, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let raw_key = state.services.account
        .generate_api_key(&account.account_id)
        .map_err(|_| HttpError::Internal)?;
    Ok(Json(ApiKeyResponse { key: raw_key }))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    state.services.account
        .revoke_api_key(&account.account_id)
        .map_err(|_| HttpError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}
