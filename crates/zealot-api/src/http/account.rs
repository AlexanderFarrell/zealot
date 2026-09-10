use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use serde::Deserialize;
use uuid::Uuid;
use zealot_app::app::AppState;
use zealot_domain::{
    account::{ApiKeyRecordDto, CreateApiKeyResponseDto},
    auth::Actor,
    common::id::Id,
    scope::{ScopePermission, ScopeRole},
};

use crate::http::{
    common::HttpError,
    middleware::{auth_middleware, csrf_middleware},
};

pub fn routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/settings", patch(update_settings))
        .route("/api-keys", get(list_api_keys))
        .route("/api-keys", post(create_api_key))
        .route("/api-keys/{id}", delete(revoke_api_key))
        .route("/scopes/{scope_id}/service-keys", post(create_service_key))
        .route(
            "/scopes/{scope_id}/service-keys/{principal_id}/{id}",
            delete(revoke_service_key),
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
struct CreateServiceKeyBody {
    display_name: String,
    label: Option<String>,
    /// `owner` is intentionally disallowed here; an owner must remain a human
    /// account until membership administration is implemented.
    role: Option<String>,
}

#[derive(serde::Serialize)]
struct CreateServiceKeyResponse {
    key: String,
    api_key_id: i64,
    principal_id: Uuid,
    label: String,
    created_at: String,
}

fn require_scope_owner(state: &AppState, actor: &Actor, scope_id: Uuid) -> Result<(), HttpError> {
    let principal_id = actor.principal_id.ok_or(HttpError::Unauthorized)?;
    let allowed = state
        .services
        .scope
        .authorize(principal_id, scope_id, ScopePermission::ManageScopeSettings)
        .map_err(|e| {
            tracing::error!(%e, "scope authorization failed");
            HttpError::Internal
        })?;
    if allowed {
        Ok(())
    } else {
        Err(HttpError::Unauthorized)
    }
}

async fn create_service_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(scope_id): Path<Uuid>,
    Json(body): Json<CreateServiceKeyBody>,
) -> Result<Json<CreateServiceKeyResponse>, HttpError> {
    require_scope_owner(&state, &actor, scope_id)?;
    if body.display_name.trim().is_empty() {
        return Err(HttpError::UserError {
            err: "display_name is required".into(),
        });
    }
    let role = match body.role.as_deref().unwrap_or("editor") {
        "editor" => ScopeRole::Editor,
        "viewer" => ScopeRole::Viewer,
        _ => {
            return Err(HttpError::UserError {
                err: "service role must be editor or viewer".into(),
            });
        }
    };
    let principal = state
        .services
        .scope
        .create_service_principal(body.display_name.trim())
        .map_err(|e| {
            tracing::error!(%e, "service principal creation failed");
            HttpError::Internal
        })?;
    state
        .services
        .scope
        .add_member(scope_id, principal.principal_id, role)
        .map_err(|e| {
            tracing::error!(%e, "service membership creation failed");
            HttpError::Internal
        })?;
    let label = body.label.unwrap_or_else(|| "Service".to_owned());
    let (record, key) = state
        .services
        .account
        .generate_service_api_key(principal.principal_id, &label)
        .map_err(|e| {
            tracing::error!(%e, "service key creation failed");
            HttpError::Internal
        })?;
    Ok(Json(CreateServiceKeyResponse {
        key,
        api_key_id: record.api_key_id.into(),
        principal_id: principal.principal_id,
        label: record.label,
        created_at: record.created_at,
    }))
}

async fn revoke_service_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, principal_id, id)): Path<(Uuid, Uuid, i64)>,
) -> Result<StatusCode, HttpError> {
    require_scope_owner(&state, &actor, scope_id)?;
    let id = Id::try_from(id).map_err(|_| HttpError::UserError {
        err: "Invalid API key id".into(),
    })?;
    state
        .services
        .account
        .revoke_service_api_key(&id, principal_id)
        .map_err(|e| {
            tracing::error!(%e, "service key revoke failed");
            HttpError::Internal
        })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn update_settings(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(settings): Json<serde_json::Value>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    state
        .services
        .account
        .update_settings(&account.account_id, settings)
        .map_err(|e| {
            tracing::error!(account_id = ?account.account_id, %e, "failed to update settings");
            HttpError::Internal
        })?;
    Ok(StatusCode::OK)
}

async fn list_api_keys(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
) -> Result<Json<Vec<ApiKeyRecordDto>>, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let keys = state
        .services
        .account
        .list_api_keys(&account.account_id)
        .map_err(|e| {
            tracing::error!(account_id = ?account.account_id, %e, "failed to list api keys");
            HttpError::Internal
        })?;
    Ok(Json(keys.into_iter().map(ApiKeyRecordDto::from).collect()))
}

#[derive(Deserialize)]
struct CreateApiKeyBody {
    label: Option<String>,
}

async fn create_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<CreateApiKeyBody>,
) -> Result<Json<CreateApiKeyResponseDto>, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let label = body.label.unwrap_or_else(|| "Default".to_string());
    let (record, raw_key) = state
        .services
        .account
        .generate_api_key(&account.account_id, &label)
        .map_err(|e| {
            tracing::error!(account_id = ?account.account_id, %e, "failed to generate api key");
            HttpError::Internal
        })?;
    Ok(Json(CreateApiKeyResponseDto {
        key: raw_key,
        api_key_id: record.api_key_id.into(),
        label: record.label,
        created_at: record.created_at,
    }))
}

async fn revoke_api_key(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(id): Path<i64>,
) -> Result<StatusCode, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    let account = actor.account.ok_or(HttpError::Unauthorized)?;
    let api_key_id = Id::try_from(id).map_err(|_| HttpError::UserError {
        err: "Invalid API key id".to_string(),
    })?;
    state
        .services
        .account
        .revoke_api_key(&api_key_id, &account.account_id)
        .map_err(|e| {
            tracing::error!(account_id = ?account.account_id, ?api_key_id, %e, "failed to revoke api key");
            HttpError::Internal
        })?;
    Ok(StatusCode::NO_CONTENT)
}
