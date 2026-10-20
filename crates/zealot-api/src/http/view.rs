use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    middleware,
    routing::{get, post},
};
use zealot_app::{app::AppState, services::view::ViewServiceError};
use zealot_domain::{
    account::Account,
    auth::Actor,
    common::id::Id,
    saved_view::{RunDefinitionDto, RunViewDto, ValidateViewDto, ViewInspectDto, ViewResultDto},
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
};

/// Saved View routes: run (saved or unsaved), inspect, and validate (Z161, Z196).
pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/validate", post(validate))
        .route("/run", post(run_definition))
        .route("/{item_id}/run", post(run))
        .route("/{item_id}/inspect", get(inspect))
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

fn require_account(actor: &Actor) -> Result<Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn view_id(value: i64) -> Result<Id, HttpError> {
    Id::try_from(value).map_err(|_| HttpError::UserError {
        err: String::from("invalid item_id"),
    })
}

fn service_error(error: ViewServiceError) -> HttpError {
    match error {
        ViewServiceError::NotFound => HttpError::NotFound,
        ViewServiceError::Invalid(err) => HttpError::UserError { err },
        ViewServiceError::Item(error) => {
            tracing::error!("Saved view item error: {error}");
            HttpError::Internal
        }
    }
}

/// `POST /view/{item_id}/run` with optional `{limit?, offset?}`.
async fn run(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
    body: Option<Json<RunViewDto>>,
) -> Result<Json<ViewResultDto>, HttpError> {
    let account = require_account(&actor)?;
    let page = body.map(|Json(dto)| dto).unwrap_or_default();
    let result = state
        .services
        .view
        .run(&view_id(item_id)?, page.limit, page.offset, &account)
        .map_err(service_error)?;
    Ok(Json(result))
}

/// `POST /view/run` with `{definition, limit?, offset?}`: run an unsaved definition.
async fn run_definition(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<RunDefinitionDto>,
) -> Result<Json<ViewResultDto>, HttpError> {
    let account = require_account(&actor)?;
    let result = state
        .services
        .view
        .run_definition(&dto.definition, dto.limit, dto.offset, &account)
        .map_err(service_error)?;
    Ok(Json(result))
}

/// `GET /view/{item_id}/inspect`.
async fn inspect(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(item_id): Path<i64>,
) -> Result<Json<ViewInspectDto>, HttpError> {
    let account = require_account(&actor)?;
    let report = state
        .services
        .view
        .inspect(&view_id(item_id)?, &account)
        .map_err(service_error)?;
    Ok(Json(report))
}

/// `POST /view/validate` with `{definition}`.
async fn validate(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<ValidateViewDto>,
) -> Result<Json<ViewInspectDto>, HttpError> {
    let account = require_account(&actor)?;
    let report = state
        .services
        .view
        .validate(&dto.definition, &account)
        .map_err(service_error)?;
    Ok(Json(report))
}
