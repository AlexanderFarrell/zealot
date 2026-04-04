use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{get, patch, post},
};
use sqlx::types::chrono::NaiveDate;
use zealot_app::{app::AppState, services::comment::CommentServiceError};
use zealot_domain::{
    auth::Actor,
    comment::{AddCommentDto, CommentDto, UpdateCommentDto},
    common::id::Id,
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/day/:date", get(get_for_day))
        .route("/item/:item_id", get(get_for_item))
        .route("/", post(add_comment))
        .route("/:comment_id", patch(update_comment).delete(delete_comment))
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

fn comment_service_err(err: CommentServiceError) -> HttpError {
    match err {
        CommentServiceError::NotFound => HttpError::NotFound,
        CommentServiceError::Unauthorized => HttpError::Unauthorized,
        CommentServiceError::Repo(_) => HttpError::Internal,
    }
}

fn parse_id(raw: i64) -> Result<Id, HttpError> {
    Id::try_from(raw).map_err(|e| HttpError::UserError { err: e.to_string() })
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn get_for_day(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(date): Path<String>,
) -> Result<Json<Vec<CommentDto>>, HttpError> {
    let account = require_account(&actor)?;
    let day = NaiveDate::parse_from_str(&date, "%Y-%m-%d")
        .map_err(|e| HttpError::UserError { err: format!("invalid date '{}': {}", date, e) })?;
    let comments = state
        .services
        .comment
        .get_for_day(&day, &account)
        .await
        .map_err(comment_service_err)?;
    Ok(Json(comments.into_iter().map(CommentDto::from).collect()))
}

async fn get_for_item(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<CommentDto>>, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_id(item_id)?;
    let comments = state
        .services
        .comment
        .get_for_item(&id, &account)
        .await
        .map_err(comment_service_err)?;
    Ok(Json(comments.into_iter().map(CommentDto::from).collect()))
}

async fn add_comment(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddCommentDto>,
) -> Result<Json<CommentDto>, HttpError> {
    let account = require_account(&actor)?;
    match state
        .services
        .comment
        .add_comment(&dto, &account)
        .await
        .map_err(comment_service_err)?
    {
        Some(comment) => Ok(Json(CommentDto::from(comment))),
        None => Err(HttpError::Internal),
    }
}

async fn update_comment(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(comment_id): Path<i64>,
    Json(mut dto): Json<UpdateCommentDto>,
) -> Result<Json<CommentDto>, HttpError> {
    let account = require_account(&actor)?;
    // Ensure the comment_id in the path is authoritative.
    dto.comment_id = comment_id;
    match state
        .services
        .comment
        .update_comment(&dto, &account)
        .await
        .map_err(comment_service_err)?
    {
        Some(comment) => Ok(Json(CommentDto::from(comment))),
        None => Err(HttpError::NotFound),
    }
}

async fn delete_comment(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(comment_id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let id = parse_id(comment_id)?;
    state
        .services
        .comment
        .delete_comment(&id, &account)
        .await
        .map(|_| StatusCode::OK)
        .map_err(comment_service_err)
}
