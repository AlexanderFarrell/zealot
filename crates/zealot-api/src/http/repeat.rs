use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{get, put},
};
use serde::Deserialize;
use sqlx::types::chrono::NaiveDate;
use zealot_app::{app::AppState, services::repeat::RepeatServiceError};
use zealot_domain::{auth::Actor, item::ItemDto, repeat::{RepeatEntryDto, UpdateRepeatEntryDto}};

use crate::http::{common::HttpError, middleware::{auth_middleware, csrf_middleware}};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/items", get(get_items))
        .route("/day/{date}", get(get_for_day))
        .route("/range", get(get_for_range))
        .route("/status", put(set_status))
        .route_layer(middleware::from_fn_with_state(state.clone(), csrf_middleware))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

#[derive(Deserialize)]
struct RangeParams {
    start: String,
    end: String,
}

// ─── Auth helper ─────────────────────────────────────────────────────────────

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn repeat_service_err(err: RepeatServiceError) -> HttpError {
    match err {
        RepeatServiceError::NotFound => HttpError::NotFound,
        RepeatServiceError::Repo(e) => { tracing::error!("Repeat repo error: {e}"); HttpError::Internal },
    }
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn get_for_day(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(date): Path<String>,
) -> Result<Json<Vec<RepeatEntryDto>>, HttpError> {
    let account = require_account(&actor)?;
    let day = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| HttpError::UserError { err: format!("invalid date '{}': {}", date, e) })?;
    let entries = state
        .services
        .repeat
        .get_for_day(&day, &account)
        .await
        .map_err(repeat_service_err)?;
    Ok(Json(entries.into_iter().map(RepeatEntryDto::from).collect()))
}

async fn get_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .repeat
        .get_all_repeat_items(&account)
        .map_err(repeat_service_err)?;
    Ok(Json(items.into_iter().map(|i| ItemDto::from(&i)).collect()))
}

async fn get_for_range(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<RangeParams>,
) -> Result<Json<Vec<RepeatEntryDto>>, HttpError> {
    let account = require_account(&actor)?;
    let start = NaiveDate::parse_from_str(&params.start, "%Y-%m-%d")
        .map_err(|e| HttpError::UserError { err: format!("invalid start date '{}': {}", params.start, e) })?;
    let end = NaiveDate::parse_from_str(&params.end, "%Y-%m-%d")
        .map_err(|e| HttpError::UserError { err: format!("invalid end date '{}': {}", params.end, e) })?;
    if end < start {
        return Err(HttpError::UserError { err: "end date must not be before start date".to_string() });
    }
    let entries = state
        .services
        .repeat
        .get_for_range(&start, &end, &account)
        .await
        .map_err(repeat_service_err)?;
    Ok(Json(entries.into_iter().map(RepeatEntryDto::from).collect()))
}

async fn set_status(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<UpdateRepeatEntryDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .repeat
        .set_status(&dto, &account)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(repeat_service_err)
}
