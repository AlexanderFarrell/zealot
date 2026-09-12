use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{get, post},
};
use zealot_app::{
    app::AppState, ports::rule_runner::RuleRunResult, services::rule::RuleServiceError,
};
use zealot_domain::{
    auth::Actor,
    common::id::Id,
    rule::{AddRuleDto, RuleDto, UpdateRuleDto},
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
    scope::{
        ScopeQuery, create_scope_access, delete_scope_access, read_scope_access,
        resolve_scope_owner_account, update_scope_access,
    },
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(list_rules).post(create_rule))
        .route(
            "/{id}",
            get(get_rule).patch(update_rule).delete(delete_rule),
        )
        .route("/{id}/run", post(run_rule))
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

fn rule_service_err(err: RuleServiceError) -> HttpError {
    match err {
        RuleServiceError::NotFound => HttpError::NotFound,
        RuleServiceError::Unauthorized => HttpError::Unauthorized,
        RuleServiceError::Repo(e) => {
            tracing::error!("Rule repo error: {e}");
            HttpError::Internal
        }
    }
}

fn parse_rule_id(raw: i64) -> Result<Id, HttpError> {
    Id::try_from(raw).map_err(|_| HttpError::NotFound)
}

async fn list_rules(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
) -> Result<Json<Vec<RuleDto>>, HttpError> {
    let access = read_scope_access(&state, &actor, &query)?;
    let rules = state
        .services
        .rule
        .get_rules_in_scopes(&access)
        .map_err(rule_service_err)?;
    Ok(Json(rules.iter().map(RuleDto::from).collect()))
}

async fn create_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
    Json(dto): Json<AddRuleDto>,
) -> Result<(StatusCode, Json<RuleDto>), HttpError> {
    let access = create_scope_access(&state, &actor, &query)?;
    let account = resolve_scope_owner_account(&state, &actor, &access)?;
    let rule = state
        .services
        .rule
        .add_rule_in_scope(dto, &account, &access)
        .map_err(rule_service_err)?;
    Ok((StatusCode::CREATED, Json(RuleDto::from(&rule))))
}

async fn get_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
) -> Result<Json<RuleDto>, HttpError> {
    let access = read_scope_access(&state, &actor, &query)?;
    let rule_id = parse_rule_id(id)?;
    let rule = state
        .services
        .rule
        .get_rule_in_scopes(&rule_id, &access)
        .map_err(rule_service_err)?;
    Ok(Json(RuleDto::from(&rule)))
}

async fn update_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
    Json(dto): Json<UpdateRuleDto>,
) -> Result<Json<RuleDto>, HttpError> {
    let access = update_scope_access(&state, &actor, &query)?;
    let rule_id = parse_rule_id(id)?;
    let rule = state
        .services
        .rule
        .update_rule_in_scopes(&rule_id, dto, &access)
        .map_err(rule_service_err)?;
    Ok(Json(RuleDto::from(&rule)))
}

async fn delete_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
) -> Result<StatusCode, HttpError> {
    let access = delete_scope_access(&state, &actor, &query)?;
    let rule_id = parse_rule_id(id)?;
    state
        .services
        .rule
        .delete_rule_in_scopes(&rule_id, &access)
        .map_err(rule_service_err)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run_rule(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
    axum::extract::Query(query): axum::extract::Query<ScopeQuery>,
) -> Result<Json<RuleRunResult>, HttpError> {
    let access = update_scope_access(&state, &actor, &query)?;
    let rule_id = parse_rule_id(id)?;
    let result = state
        .services
        .rule
        .run_rule_now_in_scopes(&rule_id, &access)
        .await
        .map_err(rule_service_err)?;
    Ok(Json(result))
}
