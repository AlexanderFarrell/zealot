use serde::Deserialize;
use uuid::Uuid;
use zealot_app::{
    app::AppState,
    services::scope::{ScopeAccess, ScopeAccessError},
};
use zealot_domain::{account::Account, common::id::Id};
use zealot_domain::{auth::Actor, scope::ScopePermission};

use super::common::HttpError;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ScopeQuery {
    pub scope_id: Option<Uuid>,
    #[serde(default)]
    pub all_scopes: bool,
}

pub fn resolve_scope_access(
    state: &AppState,
    actor: &Actor,
    query: &ScopeQuery,
    permission: ScopePermission,
    read_only: bool,
) -> Result<ScopeAccess, HttpError> {
    if !actor.is_authenticated() {
        return Err(HttpError::Unauthorized);
    }
    state
        .services
        .scope
        .resolve_access(
            actor.principal_id,
            query.scope_id,
            query.all_scopes,
            permission,
            read_only,
        )
        .map_err(scope_access_error)
}

pub fn scope_access_error(error: ScopeAccessError) -> HttpError {
    match error {
        ScopeAccessError::MissingPrincipal | ScopeAccessError::DefaultUnavailable => {
            HttpError::Unauthorized
        }
        ScopeAccessError::NotFound => HttpError::NotFound,
        ScopeAccessError::Forbidden | ScopeAccessError::AllScopesMutation => HttpError::Forbidden,
        ScopeAccessError::Unauthenticated => HttpError::Unauthorized,
        ScopeAccessError::Repo(error) => {
            tracing::error!(%error, "Scope access resolution failed");
            HttpError::Internal
        }
    }
}

pub fn read_scope_access(
    state: &AppState,
    actor: &Actor,
    query: &ScopeQuery,
) -> Result<ScopeAccess, HttpError> {
    resolve_scope_access(state, actor, query, ScopePermission::ViewItems, true)
}

pub fn update_scope_access(
    state: &AppState,
    actor: &Actor,
    query: &ScopeQuery,
) -> Result<ScopeAccess, HttpError> {
    resolve_scope_access(state, actor, query, ScopePermission::UpdateItems, false)
}

pub fn create_scope_access(
    state: &AppState,
    actor: &Actor,
    query: &ScopeQuery,
) -> Result<ScopeAccess, HttpError> {
    resolve_scope_access(state, actor, query, ScopePermission::CreateItems, false)
}

pub fn delete_scope_access(
    state: &AppState,
    actor: &Actor,
    query: &ScopeQuery,
) -> Result<ScopeAccess, HttpError> {
    resolve_scope_access(state, actor, query, ScopePermission::DeleteItems, false)
}

pub fn resolve_scope_owner_account(
    state: &AppState,
    actor: &Actor,
    access: &ScopeAccess,
) -> Result<Account, HttpError> {
    if let Some(account) = actor.account.clone() {
        return Ok(account);
    }
    let scope = access.scopes.first().ok_or(HttpError::Forbidden)?;
    let principal = state
        .services
        .scope
        .principal_by_id(scope.owner_principal_id)
        .map_err(|error| {
            tracing::error!(%error, "Failed to resolve scope owner principal");
            HttpError::Internal
        })?
        .ok_or(HttpError::Forbidden)?;
    let account_id = principal.account_id.ok_or(HttpError::Forbidden)?;
    let account_id = Id::try_from(account_id).map_err(|error| {
        tracing::error!(%error, "Scope owner has an invalid account id");
        HttpError::Internal
    })?;
    state
        .services
        .account
        .get_account_by_id(&account_id)
        .map_err(|error| {
            tracing::error!(%error, "Failed to resolve scope owner account");
            HttpError::Internal
        })?
        .ok_or(HttpError::Forbidden)
}
