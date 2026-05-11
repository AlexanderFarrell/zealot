use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use zealot_app::{app::AppState, ports::rule_runner::RuleRunResult, services::rule::RuleServiceError};
use zealot_domain::{
    auth::Actor,
    common::id::Id,
    rule::{AddRuleDto, RuleDto, UpdateRuleDto},
};

use crate::http::{common::HttpError, middleware::auth_middleware};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_rules).post(create_rule))
        .route("/{id}", get(get_rule).patch(update_rule).delete(delete_rule))
        .route("/{id}/run", post(run_rule))
        .route_layer(middleware::map_request_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

fn require_account(actor: &Actor) -> Result<zealot_domain::account::Account, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.account.clone().ok_or(HttpError::Unauthorized)
}

fn rule_service_err(err: RuleServiceError) -> HttpError {
    match err {
        RuleServiceError::NotFound => HttpError::NotFound,
        RuleServiceError::Unauthorized => HttpError::Unauthorized,
        RuleServiceError::Repo(_) => HttpError::Internal,
    }
}

fn parse_rule_id(raw: i64) -> Result<Id, HttpError> {
    Id::try_from(raw).map_err(|_| HttpError::NotFound)
}

async fn list_rules(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<RuleDto>>, HttpError> {
    let account = require_account(&actor)?;
    let rules = state.services.rule.get_rules(&account).map_err(rule_service_err)?;
    Ok(Json(rules.iter().map(RuleDto::from).collect()))
}

async fn create_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(dto): Json<AddRuleDto>,
) -> Result<(StatusCode, Json<RuleDto>), HttpError> {
    let account = require_account(&actor)?;
    let rule = state.services.rule.add_rule(dto, &account).map_err(rule_service_err)?;
    Ok((StatusCode::CREATED, Json(RuleDto::from(&rule))))
}

async fn get_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
) -> Result<Json<RuleDto>, HttpError> {
    let account = require_account(&actor)?;
    let rule_id = parse_rule_id(id)?;
    let rule = state.services.rule.get_rule(&rule_id, &account).map_err(rule_service_err)?;
    Ok(Json(RuleDto::from(&rule)))
}

async fn update_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateRuleDto>,
) -> Result<Json<RuleDto>, HttpError> {
    let account = require_account(&actor)?;
    let rule_id = parse_rule_id(id)?;
    let rule = state.services.rule.update_rule(&rule_id, dto, &account).map_err(rule_service_err)?;
    Ok(Json(RuleDto::from(&rule)))
}

async fn delete_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    let account = require_account(&actor)?;
    let rule_id = parse_rule_id(id)?;
    state.services.rule.delete_rule(&rule_id, &account).map_err(rule_service_err)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
) -> Result<Json<RuleRunResult>, HttpError> {
    let account = require_account(&actor)?;
    let rule_id = parse_rule_id(id)?;
    let result = state
        .services
        .rule
        .run_rule_now(&rule_id, &account)
        .await
        .map_err(rule_service_err)?;
    Ok(Json(result))
}
