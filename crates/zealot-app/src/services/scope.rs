use std::sync::Arc;
use uuid::Uuid;
use zealot_domain::scope::{
    Scope, ScopeMember, ScopeMemberStatus, ScopePermission, ScopeRole, ServerPrincipal,
};

use crate::{
    repos::common::RepoError,
    repos::scope::{ScopeRepo, ServerMetadata},
};

/// A request-scoped, already-authorized selection. Repository/service APIs
/// must accept this instead of treating an account as the authorization
/// boundary. `all_scopes` is represented by more than one scope and is never
/// returned for a mutation.
#[derive(Debug, Clone)]
pub struct ScopeAccess {
    pub principal_id: Uuid,
    pub scopes: Vec<Scope>,
    pub permission: ScopePermission,
    pub read_only: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum ScopeAccessError {
    #[error("authentication required")]
    Unauthenticated,
    #[error("authenticated actor has no server principal")]
    MissingPrincipal,
    #[error("scope not found")]
    NotFound,
    #[error("scope access forbidden")]
    Forbidden,
    #[error("all_scopes is read-only")]
    AllScopesMutation,
    #[error("default scope unavailable")]
    DefaultUnavailable,
    #[error("scope repository error: {0}")]
    Repo(#[from] RepoError),
}

#[derive(Debug, Clone)]
pub struct ScopeService {
    repo: Arc<dyn ScopeRepo>,
}

impl ScopeService {
    pub fn new(repo: &Arc<dyn ScopeRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        self.repo.server_metadata()
    }

    pub fn human_principal_for_account(
        &self,
        account_id: i64,
    ) -> Result<Option<ServerPrincipal>, RepoError> {
        self.repo.human_principal_for_account(account_id)
    }
    pub fn principal_for_api_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<ServerPrincipal>, RepoError> {
        self.repo.principal_for_api_key_hash(key_hash)
    }

    /// Authorizes a principal against the persisted active membership.  This is
    /// the single role-to-grant decision point for HTTP and application code.
    pub fn authorize(
        &self,
        principal_id: Uuid,
        scope_id: Uuid,
        permission: ScopePermission,
    ) -> Result<bool, RepoError> {
        let Some(scope) = self.repo.scope_by_id(scope_id)? else {
            return Ok(false);
        };
        if scope.status != zealot_domain::scope::ScopeStatus::Active {
            return Ok(false);
        }
        Ok(self
            .repo
            .members_for_scope(scope_id)?
            .into_iter()
            .any(|member| {
                member.principal_id == principal_id
                    && member.status == ScopeMemberStatus::Active
                    && member.role.grants(permission)
            }))
    }
    pub fn default_scope_for_account(&self, account_id: i64) -> Result<Option<Scope>, RepoError> {
        self.repo.default_scope_for_account(account_id)
    }
    pub fn default_scope_for_principal(
        &self,
        principal_id: Uuid,
    ) -> Result<Option<Scope>, RepoError> {
        self.repo.default_scope_for_principal(principal_id)
    }

    /// Resolves a principal's scope selection once, before any item lookup or
    /// mutation. A denied explicit scope is intentionally not silently
    /// replaced by a default scope.
    pub fn resolve_access(
        &self,
        principal_id: Option<Uuid>,
        requested: Option<Uuid>,
        all_scopes: bool,
        permission: ScopePermission,
        read_only: bool,
    ) -> Result<ScopeAccess, ScopeAccessError> {
        let principal_id = principal_id.ok_or(ScopeAccessError::MissingPrincipal)?;
        if all_scopes {
            if !read_only {
                return Err(ScopeAccessError::AllScopesMutation);
            }
            let mut scopes = Vec::new();
            for scope in self.active_scopes_for_principal(principal_id)? {
                if self.authorize(principal_id, scope.scope_id, permission)? {
                    scopes.push(scope);
                }
            }
            return Ok(ScopeAccess { principal_id, scopes, permission, read_only });
        }

        let scope = match requested {
            Some(scope_id) => self.scope_by_id(scope_id)?.ok_or(ScopeAccessError::NotFound)?,
            None => self.default_scope_for_principal(principal_id)?.ok_or(ScopeAccessError::DefaultUnavailable)?,
        };
        if !self.authorize(principal_id, scope.scope_id, permission)? {
            return Err(ScopeAccessError::Forbidden);
        }
        Ok(ScopeAccess { principal_id, scopes: vec![scope], permission, read_only })
    }

    /// Resolves the API's scope controls.  A missing selection uses the
    /// principal default; all-scopes is intentionally available only to reads.
    pub fn authorize_selection(
        &self,
        principal_id: Uuid,
        requested: Option<Uuid>,
        all_scopes: bool,
        permission: ScopePermission,
        read_only: bool,
    ) -> Result<Vec<Scope>, RepoError> {
        if all_scopes {
            if !read_only {
                return Ok(Vec::new());
            }
            return Ok(self
                .repo
                .active_scopes_for_principal(principal_id)?
                .into_iter()
                .filter(|scope| {
                    self.authorize(principal_id, scope.scope_id, permission)
                        .unwrap_or(false)
                })
                .collect());
        }
        let scope = match requested {
            Some(id) => self
                .repo
                .active_scopes_for_principal(principal_id)?
                .into_iter()
                .find(|scope| scope.scope_id == id),
            None => self.repo.default_scope_for_principal(principal_id)?,
        };
        match scope {
            Some(scope) if self.authorize(principal_id, scope.scope_id, permission)? => {
                Ok(vec![scope])
            }
            _ => Ok(Vec::new()),
        }
    }
    pub fn active_scopes_for_principal(&self, principal_id: Uuid) -> Result<Vec<Scope>, RepoError> {
        self.repo.active_scopes_for_principal(principal_id)
    }
    pub fn scope_by_id(&self, scope_id: Uuid) -> Result<Option<Scope>, RepoError> {
        self.repo.scope_by_id(scope_id)
    }
    pub fn members_for_scope(&self, scope_id: Uuid) -> Result<Vec<ScopeMember>, RepoError> {
        self.repo.members_for_scope(scope_id)
    }
    pub fn create_service_principal(
        &self,
        display_name: &str,
    ) -> Result<ServerPrincipal, RepoError> {
        self.repo.create_service_principal(display_name)
    }
    pub fn service_principal_by_name(
        &self,
        display_name: &str,
    ) -> Result<Option<ServerPrincipal>, RepoError> {
        self.repo.service_principal_by_name(display_name)
    }
    pub fn add_member(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        role: ScopeRole,
    ) -> Result<ScopeMember, RepoError> {
        self.repo.upsert_member(scope_id, principal_id, role)
    }
}
