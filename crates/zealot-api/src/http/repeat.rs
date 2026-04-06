use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{get, put},
};
use sqlx::types::chrono::NaiveDate;
use zealot_app::{app::AppState, services::repeat::RepeatServiceError};
use zealot_domain::{auth::Actor, repeat::{RepeatEntryDto, UpdateRepeatEntryDto}};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/day/{date}", get(get_for_day))
        .route("/status", put(set_status))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
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
        RepeatServiceError::Repo(_) => HttpError::Internal,
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
