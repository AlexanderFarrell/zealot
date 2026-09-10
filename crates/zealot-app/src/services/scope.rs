use std::sync::Arc;
use uuid::Uuid;
use zealot_domain::scope::{Scope, ScopeMember, ScopeRole, ServerPrincipal};

use crate::{
    repos::common::RepoError,
    repos::scope::{ScopeRepo, ServerMetadata},
};

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
    pub fn default_scope_for_account(&self, account_id: i64) -> Result<Option<Scope>, RepoError> {
        self.repo.default_scope_for_account(account_id)
    }
    pub fn active_scopes_for_principal(&self, principal_id: Uuid) -> Result<Vec<Scope>, RepoError> {
        self.repo.active_scopes_for_principal(principal_id)
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
