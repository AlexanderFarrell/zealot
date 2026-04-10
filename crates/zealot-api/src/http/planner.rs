use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::get,
};
use sqlx::types::chrono::NaiveDate;
use zealot_app::{
    app::AppState,
    services::{item::ItemServiceError, planner::PlannerServiceError},
};
use zealot_domain::{attribute::Week, auth::Actor, item::ItemDto};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/day/{date}", get(get_for_day))
        .route("/week/{week}", get(get_for_week))
        .route("/month/{month}/year/{year}", get(get_for_month))
        .route("/year/{year}", get(get_for_year))
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

fn item_service_err(err: ItemServiceError) -> HttpError {
    match err {
        ItemServiceError::NotFound => HttpError::NotFound,
        ItemServiceError::Unauthorized => HttpError::Unauthorized,
        ItemServiceError::Attribute(e) => HttpError::UserError { err: e.to_string() },
        ItemServiceError::InvalidFilter(msg) => HttpError::UserError { err: msg },
        ItemServiceError::InvalidId(msg) => HttpError::UserError { err: msg },
        ItemServiceError::Repo(_) => HttpError::Internal,
    }
}

fn planner_service_err(err: PlannerServiceError) -> HttpError {
    match err {
        PlannerServiceError::Item(item_err) => item_service_err(item_err),
    }
}

async fn get_for_day(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(date): Path<String>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let parsed =
        NaiveDate::parse_from_str(&date, "%Y-%m-%d").map_err(|e| HttpError::UserError {
            err: format!("invalid date '{}': {}", date, e),
        })?;
    let items = state
        .services
        .planner
        .get_for_day(&parsed, &account)
        .map_err(planner_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_for_week(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(week): Path<String>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let parsed =
        Week::try_from(week.as_str()).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    let items = state
        .services
        .planner
        .get_for_week(&parsed, &account)
        .map_err(planner_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_for_month(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((month, year)): Path<(i64, i64)>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    if !(1..=12).contains(&month) {
        return Err(HttpError::UserError {
            err: format!("invalid month '{}': expected 1-12", month),
        });
    }

    let items = state
        .services
        .planner
        .get_for_month(month, year, &account)
        .map_err(planner_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_for_year(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(year): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .planner
        .get_for_year(year, &account)
        .map_err(planner_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}
