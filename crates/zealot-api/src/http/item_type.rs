use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use zealot_app::app::AppState;
use zealot_app::services::item_type::ItemTypeServiceError;
use zealot_domain::{
    auth::Actor,
    common::id::Id,
    item_type::{AddItemTypeDto, ItemTypeDto, ItemTypeSummaryDto, UpdateItemTypeDto},
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/summary", get(get_item_type_summaries))
        .route("/name/{name}", get(get_item_type_by_name))
        .route("/", get(get_item_types).post(add_item_type))
        .route(
            "/{type_id}",
            get(get_item_type)
                .patch(update_item_type)
                .delete(delete_item_type),
        )
        .route(
            "/{type_id}/attr_kind",
            post(add_attr_kinds).delete(remove_attr_kinds),
        )
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

fn item_type_service_err(err: ItemTypeServiceError) -> HttpError {
    match err {
        ItemTypeServiceError::NotFound => HttpError::NotFound,
        ItemTypeServiceError::ReadOnly(err) => HttpError::UserError { err },
        ItemTypeServiceError::Repo(_) => HttpError::Internal,
    }
}

async fn get_item_type_summaries(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<ItemTypeSummaryDto>>, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .item_type
        .get_item_type_summaries(&account.account_id)
        .map(|types| Json(types.iter().map(ItemTypeSummaryDto::from).collect()))
        .map_err(item_type_service_err)
}

async fn get_item_types(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<ItemTypeDto>>, HttpError> {
    let account = require_account(&actor)?;
    state
        .services
        .item_type
        .get_item_types(&account.account_id)
        .map(|types| Json(types.iter().map(ItemTypeDto::from).collect()))
        .map_err(|_| HttpError::Internal)
}

async fn get_item_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(type_id): Path<i64>,
) -> Result<Json<ItemTypeDto>, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(type_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    match state
        .services
        .item_type
        .get_item_type(&id, &account.account_id)
    {
        Ok(Some(t)) => Ok(Json(ItemTypeDto::from(&t))),
        Ok(None) => Err(HttpError::NotFound),
        Err(err) => Err(item_type_service_err(err)),
    }
}

async fn get_item_type_by_name(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(name): Path<String>,
) -> Result<Json<ItemTypeDto>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .item_type
        .get_item_type_by_name(&name, &account.account_id)
    {
        Ok(Some(t)) => Ok(Json(ItemTypeDto::from(&t))),
        Ok(None) => Err(HttpError::NotFound),
        Err(err) => Err(item_type_service_err(err)),
    }
}

async fn add_item_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddItemTypeDto>,
) -> Result<Json<ItemTypeDto>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .item_type
        .add_item_type(&dto, &account.account_id)
    {
        Ok(Some(t)) => Ok(Json(ItemTypeDto::from(&t))),
        Ok(None) => Err(HttpError::Internal),
        Err(err) => Err(item_type_service_err(err)),
    }
}

async fn update_item_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(type_id): Path<i64>,
    Json(mut dto): Json<UpdateItemTypeDto>,
) -> Result<Json<ItemTypeDto>, HttpError> {
    let account = require_account(&actor)?;
    dto.type_id = type_id;
    match state
        .services
        .item_type
        .update_item_type(&dto, &account.account_id)
    {
        Ok(Some(t)) => Ok(Json(ItemTypeDto::from(&t))),
        Ok(None) => Err(HttpError::NotFound),
        Err(err) => Err(item_type_service_err(err)),
    }
}

async fn delete_item_type(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(type_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(type_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    state
        .services
        .item_type
        .delete_item_type(&id, &account.account_id)
        .map(|_| StatusCode::OK)
        .map_err(item_type_service_err)
}

async fn add_attr_kinds(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(type_id): Path<i64>,
    Json(keys): Json<Vec<String>>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(type_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    state
        .services
        .item_type
        .add_attr_kinds_to_item_type(&keys, &id, &account.account_id)
        .map(|_| StatusCode::OK)
        .map_err(item_type_service_err)
}

async fn remove_attr_kinds(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(type_id): Path<i64>,
    Json(keys): Json<Vec<String>>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = Id::try_from(type_id).map_err(|e| HttpError::UserError { err: e.to_string() })?;
    state
        .services
        .item_type
        .remove_attr_kinds_from_item_type(&keys, &id, &account.account_id)
        .map(|_| StatusCode::OK)
        .map_err(item_type_service_err)
}
