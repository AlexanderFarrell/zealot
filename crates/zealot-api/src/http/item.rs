use std::collections::HashMap;

use axum::{
    Extension, Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use serde::Deserialize;
use serde_json::Value;
use zealot_app::{app::AppState, services::item::ItemServiceError};
use zealot_domain::{
    attribute::AttributeFilterDto,
    auth::Actor,
    common::id::Id,
    item::{AddItemDto, ItemDto, UpdateItemDto},
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_root_items).post(add_item))
        .route("/title/{title}", get(get_by_title))
        .route("/id/{item_id}", get(get_by_id))
        .route("/search", get(search_items))
        .route("/children/{item_id}", get(get_children))
        .route("/related/{item_id}", get(get_related))
        .route("/filter", post(filter_items))
        .route("/{item_id}", patch(update_item).delete(delete_item))
        .route("/{item_id}/attr", patch(set_attributes))
        .route("/{item_id}/attr/rename", patch(rename_attribute))
        .route("/{item_id}/attr/{key}", delete(delete_attribute))
        .route("/{item_id}/assign_type/{type_name}", post(assign_type).delete(unassign_type))
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

fn parse_item_id(raw: i64) -> Result<Id, HttpError> {
    Id::try_from(raw).map_err(|e| HttpError::UserError { err: e.to_string() })
}

// ─── Query param structs ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RootItemsParams {
    #[serde(rename = "type", default)]
    item_type: Option<String>,
}

#[derive(Deserialize)]
struct SearchParams {
    #[serde(default)]
    term: String,
}

#[derive(Deserialize)]
struct FilterBody {
    filters: Vec<AttributeFilterDto>,
}

#[derive(Deserialize)]
struct RenameAttributeDto {
    old_key: String,
    new_key: String,
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn get_root_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<RootItemsParams>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = if let Some(type_name) = params.item_type {
        state.services.item.get_items_by_type(&type_name, &account)
    } else {
        state.services.item.get_root_items(&account)
    }
    .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_by_title(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(title): Path<String>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .item
        .get_items_by_title(&title, &account)
        .map_err(item_service_err)?;
    items
        .into_iter()
        .next()
        .map(|i| Json(ItemDto::from(&i)))
        .ok_or(HttpError::NotFound)
}

async fn get_by_id(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    match state.services.item.get_item_by_id(&id, &account).map_err(item_service_err)? {
        Some(item) => Ok(Json(ItemDto::from(&item))),
        None => Err(HttpError::NotFound),
    }
}

async fn search_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .item
        .search_items_by_title(&params.term, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_children(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let items = state
        .services
        .item
        .get_children(&id, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn get_related(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    let items = state
        .services
        .item
        .get_related_items(&id, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn filter_items(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<FilterBody>,
) -> Result<Json<Vec<ItemDto>>, HttpError> {
    let account = require_account(&actor)?;
    let items = state
        .services
        .item
        .filter_items(&body.filters, &account)
        .map_err(item_service_err)?;
    Ok(Json(items.iter().map(ItemDto::from).collect()))
}

async fn add_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddItemDto>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    match state.services.item.add_item(&dto, &account).map_err(item_service_err)? {
        Some(item) => Ok(Json(ItemDto::from(&item))),
        None => Err(HttpError::Internal),
    }
}

async fn update_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(dto): Json<UpdateItemDto>,
) -> Result<Json<ItemDto>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    match state.services.item.update_item(&id, &dto, &account).map_err(item_service_err)? {
        Some(item) => Ok(Json(ItemDto::from(&item))),
        None => Err(HttpError::NotFound),
    }
}

async fn delete_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .delete_item(&id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn set_attributes(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(attrs): Json<HashMap<String, Value>>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .set_attributes(&id, &attrs, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn rename_attribute(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    Json(dto): Json<RenameAttributeDto>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .rename_attribute(&id, &dto.old_key, &dto.new_key, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn delete_attribute(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, key)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .delete_attribute(&id, &key, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn assign_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, type_name)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .assign_type(&type_name, &id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}

async fn unassign_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((item_id, type_name)): Path<(i64, String)>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_item_id(item_id)?;
    state
        .services
        .item
        .unassign_type(&type_name, &id, &account)
        .map(|_| StatusCode::OK)
        .map_err(item_service_err)
}
