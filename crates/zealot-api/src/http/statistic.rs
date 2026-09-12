use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::get,
    Extension, Json, Router,
};
use serde::Deserialize;
use sqlx::types::chrono::{DateTime, Utc};
use zealot_app::{app::AppState, services::statistic::StatisticServiceError};
use zealot_domain::{
    auth::Actor,
    common::id::Id,
    item::ItemDto,
    statistic::{
        CreateStatisticEntryDto, StatisticDailyPointDto, StatisticEntryDto, StatisticEntryPageDto,
        StatisticSummaryDto, UpdateStatisticEntryDto,
    },
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
    scope::{
        create_scope_access, delete_scope_access, read_scope_access, resolve_scope_owner_account,
        update_scope_access, ScopeQuery,
    },
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/items", get(list_items))
        .route("/{item_id}/entries", get(list_entries).post(create_entry))
        .route("/{item_id}/daily", get(daily))
        .route("/{item_id}/summary", get(summary))
        .route(
            "/entries/{statistic_entry_id}",
            axum::routing::patch(update_entry).delete(delete_entry),
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

#[derive(Deserialize)]
struct ItemParams {
    parent_id: Option<i64>,
    #[serde(flatten)]
    scope: ScopeQuery,
}

#[derive(Deserialize)]
struct EntryParams {
    start: Option<String>,
    end: Option<String>,
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
    #[serde(flatten)]
    scope: ScopeQuery,
}

#[derive(Deserialize)]
struct RangeParams {
    start: Option<String>,
    end: Option<String>,
    #[serde(flatten)]
    scope: ScopeQuery,
}

fn default_limit() -> i64 {
    50
}

fn id(value: i64, name: &str) -> Result<Id, HttpError> {
    Id::try_from(value).map_err(|_| HttpError::UserError {
        err: format!("invalid {name}"),
    })
}

fn timestamp(value: Option<String>, name: &str) -> Result<Option<DateTime<Utc>>, HttpError> {
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|_| HttpError::UserError {
                    err: format!("invalid RFC 3339 {name}: {value}"),
                })
        })
        .transpose()
}

fn service_error(error: StatisticServiceError) -> HttpError {
    match error {
        StatisticServiceError::NotFound => HttpError::NotFound,
        StatisticServiceError::Invalid(err) => HttpError::UserError { err },
        StatisticServiceError::Repo(error) => {
            tracing::error!("Statistic repo error: {error}");
            HttpError::Internal
        }
    }
}

async fn list_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<ItemParams>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let access = read_scope_access(&state, &actor, &params.scope)?;
    let parent_id = params
        .parent_id
        .map(|value| id(value, "parent_id"))
        .transpose()?;
    let items = state
        .services
        .statistic
        .list_items_in_scopes(parent_id, &access)
        .map_err(service_error)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn list_entries(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Query(params): Query<EntryParams>,
) -> Result<Json<StatisticEntryPageDto>, HttpError> {
    if !(1..=100).contains(&params.limit) || params.offset < 0 {
        return Err(HttpError::UserError {
            err: "limit must be between 1 and 100 and offset must be non-negative".into(),
        });
    }
    let access = read_scope_access(&state, &actor, &params.scope)?;
    let page = state
        .services
        .statistic
        .list_entries_in_scopes(
            id(item_id, "item_id")?,
            timestamp(params.start, "start")?,
            timestamp(params.end, "end")?,
            params.limit,
            params.offset,
            &access,
        )
        .map_err(service_error)?;
    Ok(Json(page))
}

async fn daily(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Query(params): Query<RangeParams>,
) -> Result<Json<Vec<StatisticDailyPointDto>>, HttpError> {
    let access = read_scope_access(&state, &actor, &params.scope)?;
    let points = state
        .services
        .statistic
        .daily_in_scopes(
            id(item_id, "item_id")?,
            timestamp(params.start, "start")?,
            timestamp(params.end, "end")?,
            &access,
        )
        .map_err(service_error)?;
    Ok(Json(points))
}

async fn summary(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Query(params): Query<RangeParams>,
) -> Result<Json<StatisticSummaryDto>, HttpError> {
    let access = read_scope_access(&state, &actor, &params.scope)?;
    let result = state
        .services
        .statistic
        .summary_in_scopes(
            id(item_id, "item_id")?,
            timestamp(params.start, "start")?,
            timestamp(params.end, "end")?,
            &access,
        )
        .map_err(service_error)?;
    Ok(Json(result))
}

async fn create_entry(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Query(query): Query<ScopeQuery>,
    Json(dto): Json<CreateStatisticEntryDto>,
) -> Result<(StatusCode, Json<StatisticEntryDto>), HttpError> {
    let access = create_scope_access(&state, &actor, &query)?;
    let owner_account = resolve_scope_owner_account(&state, &actor, &access)?;
    let entry = state
        .services
        .statistic
        .create_in_scopes(id(item_id, "item_id")?, &dto, &owner_account, &access)
        .map_err(service_error)?;
    Ok((StatusCode::CREATED, Json(StatisticEntryDto::from(&entry))))
}

async fn update_entry(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(statistic_entry_id): Path<i64>,
    Query(query): Query<ScopeQuery>,
    Json(dto): Json<UpdateStatisticEntryDto>,
) -> Result<Json<StatisticEntryDto>, HttpError> {
    let access = update_scope_access(&state, &actor, &query)?;
    let entry = state
        .services
        .statistic
        .update_in_scopes(id(statistic_entry_id, "statistic_entry_id")?, &dto, &access)
        .map_err(service_error)?;
    Ok(Json(StatisticEntryDto::from(&entry)))
}

async fn delete_entry(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(statistic_entry_id): Path<i64>,
    Query(query): Query<ScopeQuery>,
) -> Result<StatusCode, HttpError> {
    let access = delete_scope_access(&state, &actor, &query)?;
    state
        .services
        .statistic
        .delete_in_scopes(id(statistic_entry_id, "statistic_entry_id")?, &access)
        .map_err(service_error)?;
    Ok(StatusCode::NO_CONTENT)
}
