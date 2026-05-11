# TASK-LUA-012: HTTP API routes for rules

## Context

The frontend needs to CRUD rules and trigger manual runs. This task adds six routes to the Axum router following the exact pattern of `crates/zealot-api/src/http/item.rs` and `comment.rs`.

## Goal

Implement `crates/zealot-api/src/http/rule.rs` with all routes and register them in `http/mod.rs`.

## Requirements

### Routes

```
GET    /rule            — list all rules for current account
POST   /rule            — create rule
GET    /rule/{id}       — get rule by id
PATCH  /rule/{id}       — update rule
DELETE /rule/{id}       — delete rule
POST   /rule/{id}/run   — run rule immediately, return RuleRunResult
```

### `crates/zealot-api/src/http/rule.rs`

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, patch, post},
    Json, Router,
};
use zealot_app::app::AppState;
use zealot_domain::rule::{AddRuleDto, UpdateRuleDto};
use crate::http::{auth_middleware::Actor, error::HttpError};

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/",           get(list_rules).post(create_rule))
        .route("/{id}",       get(get_rule).patch(update_rule).delete(delete_rule))
        .route("/{id}/run",   post(run_rule))
        .with_state(state)
        // auth_middleware applied at the module level — confirm pattern from item.rs
}

async fn list_rules(State(state): State<AppState>, actor: Actor) -> Result<Json<Vec<Rule>>, HttpError> {
    let rules = state.services.rule.get_rules(&actor.account).map_err(HttpError::from)?;
    Ok(Json(rules))
}

async fn create_rule(
    State(state): State<AppState>,
    actor: Actor,
    Json(dto): Json<AddRuleDto>,
) -> Result<(StatusCode, Json<Rule>), HttpError> {
    let rule = state.services.rule.add_rule(dto, &actor.account).map_err(HttpError::from)?;
    Ok((StatusCode::CREATED, Json(rule)))
}

async fn get_rule(
    State(state): State<AppState>,
    actor: Actor,
    Path(id): Path<String>,
) -> Result<Json<Rule>, HttpError> {
    let rule_id = Id::parse(&id).map_err(|_| HttpError::NotFound)?;
    let rule = state.services.rule.get_rule(&rule_id, &actor.account).map_err(HttpError::from)?;
    Ok(Json(rule))
}

async fn update_rule(
    State(state): State<AppState>,
    actor: Actor,
    Path(id): Path<String>,
    Json(dto): Json<UpdateRuleDto>,
) -> Result<Json<Rule>, HttpError> {
    let rule_id = Id::parse(&id).map_err(|_| HttpError::NotFound)?;
    let rule = state.services.rule.update_rule(&rule_id, dto, &actor.account).map_err(HttpError::from)?;
    Ok(Json(rule))
}

async fn delete_rule(
    State(state): State<AppState>,
    actor: Actor,
    Path(id): Path<String>,
) -> Result<StatusCode, HttpError> {
    let rule_id = Id::parse(&id).map_err(|_| HttpError::NotFound)?;
    state.services.rule.delete_rule(&rule_id, &actor.account).map_err(HttpError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn run_rule(
    State(state): State<AppState>,
    actor: Actor,
    Path(id): Path<String>,
) -> Result<Json<RuleRunResult>, HttpError> {
    let rule_id = Id::parse(&id).map_err(|_| HttpError::NotFound)?;
    let result = state.services.rule.run_rule_now(&rule_id, &actor.account).await.map_err(HttpError::from)?;
    Ok(Json(result))
}
```

Map `RuleServiceError::NotFound` to `HttpError::NotFound` (same pattern as `ItemServiceError`).

### `RuleRunResult` must be serializable

Add `#[derive(Serialize)]` to `RuleRunResult` in `zealot-app/src/ports/rule_runner.rs`, or create a `RuleRunResultDto` in `zealot-domain` if preferred.

### Register in `crates/zealot-api/src/http/mod.rs`

```rust
pub mod rule;

// In the router builder:
.nest("/rule", rule::routes(state.clone()))
```

## Dependencies

- TASK-LUA-006 (service)
- TASK-LUA-009 (runner — needed for `run_rule_now`)

## Files to create/modify

- `crates/zealot-api/src/http/rule.rs` (new)
- `crates/zealot-api/src/http/mod.rs`

## Verification

```bash
cargo check -p zealot-api

# With server running:
curl -s -b "session_id=..." http://localhost:8080/api/rule | jq .
curl -s -b "session_id=..." -X POST http://localhost:8080/api/rule \
  -H 'Content-Type: application/json' \
  -d '{"name":"test","trigger":{"kind":"manual"},"script":"zealot.notify(\"hello\")"}' | jq .
```
