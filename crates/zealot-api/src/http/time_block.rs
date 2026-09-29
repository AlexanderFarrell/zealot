use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use serde::Deserialize;
use sqlx::types::chrono::NaiveDate;
use zealot_app::{app::AppState, services::time_block::TimeBlockServiceError};
use zealot_domain::{
    auth::Actor,
    common::id::Id,
    time_block::{CreateTimeBlockDto, TimeBlockDto, UpdateTimeBlockDto},
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/day/{date}", get(get_for_day))
        .route("/range", get(get_for_range))
        .route("/item/{item_id}", get(get_for_item))
        .route("/", post(create))
        .route("/{block_id}", patch(update))
        .route("/{block_id}", delete(delete_block))
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

#[derive(Deserialize)]
struct RangeParams {
    start: String,
    end:   String,
}

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn service_err(err: TimeBlockServiceError) -> HttpError {
    match err {
        TimeBlockServiceError::NotFound => HttpError::NotFound,
        TimeBlockServiceError::Repo(e) => {
            tracing::error!("TimeBlock repo error: {e}");
            HttpError::Internal
        }
    }
}

async fn get_for_day(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(date): Path<String>,
) -> Result<Json<Vec<TimeBlockDto>>, HttpError> {
    let account = require_account(&actor)?;
    let day = NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| HttpError::UserError {
        err: format!("invalid date '{}': {}", date, e),
    })?;
    let blocks = state
        .services
        .time_block
        .get_for_day(&day, &account)
        .await
        .map_err(service_err)?;
    Ok(Json(blocks.into_iter().map(TimeBlockDto::from).collect()))
}

async fn get_for_range(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<RangeParams>,
) -> Result<Json<Vec<TimeBlockDto>>, HttpError> {
    let account = require_account(&actor)?;
    let start =
        NaiveDate::parse_from_str(&params.start, "%Y-%m-%d").map_err(|e| HttpError::UserError {
            err: format!("invalid start date '{}': {}", params.start, e),
        })?;
    let end =
        NaiveDate::parse_from_str(&params.end, "%Y-%m-%d").map_err(|e| HttpError::UserError {
            err: format!("invalid end date '{}': {}", params.end, e),
        })?;
    if end < start {
        return Err(HttpError::UserError {
            err: "end date must not be before start date".to_string(),
        });
    }
    let blocks = state
        .services
        .time_block
        .get_for_range(&start, &end, &account)
        .await
        .map_err(service_err)?;
    Ok(Json(blocks.into_iter().map(TimeBlockDto::from).collect()))
}

async fn get_for_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<TimeBlockDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(item_id).map_err(|_| HttpError::UserError {
        err: "invalid item_id".to_string(),
    })?;
    let blocks = state
        .services
        .time_block
        .get_for_item(id, &account)
        .await
        .map_err(service_err)?;
    Ok(Json(blocks.into_iter().map(TimeBlockDto::from).collect()))
}

async fn create(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<CreateTimeBlockDto>,
) -> Result<Json<TimeBlockDto>, HttpError> {
    let account = require_account(&actor)?;
    let block = state
        .services
        .time_block
        .create(&dto, &account)
        .await
        .map_err(service_err)?;
    Ok(Json(TimeBlockDto::from(block)))
}

async fn update(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(block_id): Path<i64>,
    Json(mut dto): Json<UpdateTimeBlockDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    dto.block_id = block_id;
    state
        .services
        .time_block
        .update(&dto, &account)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(service_err)
}

async fn delete_block(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(block_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(block_id).map_err(|_| HttpError::UserError {
        err: "invalid block_id".to_string(),
    })?;
    state
        .services
        .time_block
        .delete(id, &account)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(service_err)
}
