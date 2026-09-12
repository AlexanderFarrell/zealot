use axum::{
    Extension, Json, Router,
    extract::{Path, State},
    http::StatusCode,
    middleware,
    routing::{delete, get, patch, post},
};
use chrono::{Duration, Utc};
use serde::Deserialize;
use uuid::Uuid;
use zealot_app::app::AppState;
use zealot_app::services::scope::{InvitationAcceptance, MembershipError};
use zealot_domain::{
    account::{ApiKeyRecordDto, CreateApiKeyResponseDto},
    auth::Actor,
    common::id::Id,
    scope::{ScopeInvitation, ScopeInvitationStatus, ScopePermission, ScopeRole},
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
        .route("/scopes/{scope_id}/members", get(list_scope_members))
        .route(
            "/scopes/{scope_id}/members/{principal_id}",
            patch(change_scope_member_role).delete(revoke_scope_member),
        )
        .route(
            "/scopes/{scope_id}/invitations",
            get(list_scope_invitations).post(create_scope_invitation),
        )
        .route(
            "/scopes/{scope_id}/invitations/{invitation_id}/approve",
            post(approve_scope_invitation),
        )
        .route(
            "/scopes/{scope_id}/invitations/{invitation_id}/cancel",
            post(cancel_scope_invitation),
        )
        .route(
            "/scopes/{scope_id}/invitations/{invitation_id}/revoke",
            post(revoke_scope_invitation),
        )
        .route("/invitations/accept", post(accept_scope_invitation))
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
    /// `owner` is intentionally disallowed here; service principals may hold
    /// explicit editor/viewer scope memberships but never scope ownership.
    role: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CreateScopeInvitationBody {
    role: String,
    recipient_principal_id: Option<Uuid>,
    ttl_seconds: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct ScopeMemberRoleBody {
    role: String,
}

#[derive(Debug, Deserialize)]
struct ApproveScopeInvitationBody {
    principal_id: Uuid,
}

#[derive(Debug, Deserialize)]
struct AcceptScopeInvitationBody {
    token: String,
}

#[derive(Debug, serde::Serialize)]
struct ScopeMemberResponse {
    principal_id: Uuid,
    display_name: String,
    role: String,
    status: String,
}

#[derive(Debug, serde::Serialize)]
struct ScopeInvitationResponse {
    invitation_id: Uuid,
    scope_id: Uuid,
    issuer_principal_id: Uuid,
    recipient_principal_id: Option<Uuid>,
    permitted_role: String,
    status: String,
    created_at: String,
    expires_at: String,
    accepted_at: Option<String>,
    terminal_at: Option<String>,
    correlation_id: Uuid,
}

#[derive(Debug, serde::Serialize)]
struct CreatedScopeInvitationResponse {
    invitation: ScopeInvitationResponse,
    /// This is the one-time bearer. It is never returned by list/read routes.
    token: String,
}

#[derive(Debug, serde::Serialize)]
struct AcceptScopeInvitationResponse {
    status: String,
    scope_id: Uuid,
    principal_id: Option<Uuid>,
}

fn parse_scope_role(value: &str) -> Result<ScopeRole, HttpError> {
    value.parse().map_err(|_| HttpError::UserError {
        err: "role must be owner, editor, or viewer".into(),
    })
}

fn invitation_response(invitation: ScopeInvitation) -> ScopeInvitationResponse {
    ScopeInvitationResponse {
        invitation_id: invitation.invitation_id,
        scope_id: invitation.scope_id,
        issuer_principal_id: invitation.issuer_principal_id,
        recipient_principal_id: invitation.recipient_principal_id,
        permitted_role: invitation.permitted_role.to_string(),
        status: invitation.status.to_string(),
        created_at: invitation.created_at.to_rfc3339(),
        expires_at: invitation.expires_at.to_rfc3339(),
        accepted_at: invitation.accepted_at.map(|value| value.to_rfc3339()),
        terminal_at: invitation.terminal_at.map(|value| value.to_rfc3339()),
        correlation_id: invitation.correlation_id,
    }
}

fn membership_http_error(error: MembershipError) -> HttpError {
    match error {
        MembershipError::Forbidden
        | MembershipError::NotFound
        | MembershipError::InvalidInvitation => {
            // Do not let an unauthorized caller distinguish an existing scope,
            // member, account, or invitation from a missing one.
            HttpError::NotFound
        }
        MembershipError::Expired => HttpError::Conflict {
            message: "Invitation expired".into(),
        },
        MembershipError::TerminalInvitation => HttpError::Conflict {
            message: "Invitation is no longer usable".into(),
        },
        MembershipError::RoleEscalation => HttpError::Forbidden,
        MembershipError::FinalOwner => HttpError::Conflict {
            message: "The final active owner cannot be removed or downgraded".into(),
        },
        MembershipError::InvalidState => HttpError::Conflict {
            message: "Invalid membership state".into(),
        },
        MembershipError::Repo(error) => {
            tracing::error!(%error, "scope membership operation failed");
            HttpError::Internal
        }
    }
}

fn actor_principal(actor: &Actor) -> Result<Uuid, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    actor.principal_id.ok_or(HttpError::Unauthorized)
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

async fn list_scope_members(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(scope_id): Path<Uuid>,
) -> Result<Json<Vec<ScopeMemberResponse>>, HttpError> {
    let principal_id = actor_principal(&actor).map_err(|_| HttpError::NotFound)?;
    let allowed = state
        .services
        .scope
        .authorize(principal_id, scope_id, ScopePermission::ViewItems)
        .map_err(|error| {
            tracing::error!(%error, "scope member listing authorization failed");
            HttpError::Internal
        })?;
    if !allowed {
        return Err(HttpError::NotFound);
    }
    let members = state
        .services
        .scope
        .members_for_scope(scope_id)
        .map_err(|error| {
            tracing::error!(%error, "scope member listing failed");
            HttpError::Internal
        })?;
    let mut response = Vec::with_capacity(members.len());
    for member in members {
        let Some(principal) = state
            .services
            .scope
            .principal_by_id(member.principal_id)
            .map_err(|error| {
                tracing::error!(%error, "scope member principal lookup failed");
                HttpError::Internal
            })?
        else {
            continue;
        };
        response.push(ScopeMemberResponse {
            principal_id: member.principal_id,
            display_name: principal.display_name,
            role: member.role.to_string(),
            status: member.status.to_string(),
        });
    }
    Ok(Json(response))
}

async fn change_scope_member_role(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, principal_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ScopeMemberRoleBody>,
) -> Result<Json<ScopeMemberResponse>, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    let role = parse_scope_role(&body.role)?;
    let member = state
        .services
        .scope
        .change_member_role(actor_principal_id, scope_id, principal_id, role)
        .map_err(membership_http_error)?;
    let principal = state
        .services
        .scope
        .principal_by_id(principal_id)
        .map_err(|error| {
            tracing::error!(%error, "changed member principal lookup failed");
            HttpError::Internal
        })?
        .ok_or(HttpError::NotFound)?;
    Ok(Json(ScopeMemberResponse {
        principal_id,
        display_name: principal.display_name,
        role: member.role.to_string(),
        status: member.status.to_string(),
    }))
}

async fn revoke_scope_member(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, principal_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    state
        .services
        .scope
        .revoke_member(actor_principal_id, scope_id, principal_id)
        .map_err(membership_http_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_scope_invitation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(scope_id): Path<Uuid>,
    Json(body): Json<CreateScopeInvitationBody>,
) -> Result<Json<CreatedScopeInvitationResponse>, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    let role = parse_scope_role(&body.role)?;
    let ttl_seconds = body.ttl_seconds.unwrap_or(7 * 24 * 60 * 60);
    if !(300..=30 * 24 * 60 * 60).contains(&ttl_seconds) {
        return Err(HttpError::UserError {
            err: "ttl_seconds must be between 300 and 2592000".into(),
        });
    }
    let created = state
        .services
        .scope
        .create_invitation(
            actor_principal_id,
            scope_id,
            role,
            body.recipient_principal_id,
            Some(Utc::now() + Duration::seconds(ttl_seconds)),
        )
        .map_err(membership_http_error)?;
    Ok(Json(CreatedScopeInvitationResponse {
        token: created.token,
        invitation: invitation_response(created.invitation),
    }))
}

async fn list_scope_invitations(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path(scope_id): Path<Uuid>,
) -> Result<Json<Vec<ScopeInvitationResponse>>, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    let invitations = state
        .services
        .scope
        .invitations_for_scope(actor_principal_id, scope_id)
        .map_err(membership_http_error)?;
    Ok(Json(
        invitations.into_iter().map(invitation_response).collect(),
    ))
}

async fn approve_scope_invitation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, invitation_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<ApproveScopeInvitationBody>,
) -> Result<StatusCode, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    state
        .services
        .scope
        .approve_invitation(
            actor_principal_id,
            scope_id,
            invitation_id,
            body.principal_id,
        )
        .map_err(membership_http_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn cancel_scope_invitation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    state
        .services
        .scope
        .cancel_invitation(actor_principal_id, scope_id, invitation_id)
        .map_err(membership_http_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn revoke_scope_invitation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Path((scope_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, HttpError> {
    let actor_principal_id = actor_principal(&actor)?;
    state
        .services
        .scope
        .revoke_invitation(actor_principal_id, scope_id, invitation_id)
        .map_err(membership_http_error)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn accept_scope_invitation(
    State(state): State<AppState>,
    Extension(actor): Extension<Actor>,
    Json(body): Json<AcceptScopeInvitationBody>,
) -> Result<(StatusCode, Json<AcceptScopeInvitationResponse>), HttpError> {
    let result = state
        .services
        .scope
        .accept_invitation(body.token.as_str(), actor.principal_id)
        .map_err(membership_http_error)?;
    match result {
        InvitationAcceptance::Active(member) => Ok((
            StatusCode::OK,
            Json(AcceptScopeInvitationResponse {
                status: "active".into(),
                scope_id: member.scope_id,
                principal_id: Some(member.principal_id),
            }),
        )),
        InvitationAcceptance::PendingAdmission(invitation) => Ok((
            StatusCode::ACCEPTED,
            Json(AcceptScopeInvitationResponse {
                status: ScopeInvitationStatus::PendingAdmission.to_string(),
                scope_id: invitation.scope_id,
                principal_id: None,
            }),
        )),
    }
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
