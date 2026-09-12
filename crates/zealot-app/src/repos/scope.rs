use std::fmt::Debug;

use crate::repos::common::RepoError;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use zealot_domain::scope::{
    Scope, ScopeInvitation, ScopeInvitationStatus, ScopeLifecycleEvent, ScopeMember, ScopeRole,
    ServerPrincipal,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerMetadata {
    pub server_id: String,
    /// Latest SQL migration applied to this server. Future sync handshakes must
    /// require an exact match before exchanging scope data.
    pub schema_migration_version: i64,
}

pub trait ScopeRepo: Debug + Send + Sync {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError>;
    fn human_principal_for_account(
        &self,
        account_id: i64,
    ) -> Result<Option<ServerPrincipal>, RepoError>;
    fn principal_for_api_key_hash(
        &self,
        key_hash: &str,
    ) -> Result<Option<ServerPrincipal>, RepoError>;
    fn principal_by_id(&self, principal_id: Uuid) -> Result<Option<ServerPrincipal>, RepoError>;
    fn default_scope_for_account(&self, account_id: i64) -> Result<Option<Scope>, RepoError>;
    fn default_scope_for_principal(&self, principal_id: Uuid) -> Result<Option<Scope>, RepoError>;
    /// Resolves a scope independently of membership. Callers must still apply
    /// `authorize`; this exists to distinguish a known denied scope (403) from
    /// an unknown/inaccessible item reference (404).
    fn scope_by_id(&self, scope_id: Uuid) -> Result<Option<Scope>, RepoError>;
    fn active_scopes_for_principal(&self, principal_id: Uuid) -> Result<Vec<Scope>, RepoError>;
    fn members_for_scope(&self, scope_id: Uuid) -> Result<Vec<ScopeMember>, RepoError>;
    fn server_enrolment_policy(&self) -> Result<String, RepoError>;
    fn server_invitation_signing_key(&self) -> Result<Option<String>, RepoError>;
    fn set_server_invitation_signing_key(&self, key: &str) -> Result<String, RepoError>;
    fn create_invitation(&self, invitation: &ScopeInvitation)
    -> Result<ScopeInvitation, RepoError>;
    fn invitation_by_token_hash(
        &self,
        token_hash: &str,
    ) -> Result<Option<ScopeInvitation>, RepoError>;
    fn invitations_for_scope(&self, scope_id: Uuid) -> Result<Vec<ScopeInvitation>, RepoError>;
    fn transition_invitation(
        &self,
        invitation_id: Uuid,
        status: ScopeInvitationStatus,
        when: DateTime<Utc>,
    ) -> Result<Option<ScopeInvitation>, RepoError>;
    /// Consume a valid invitation and activate/update the target membership in
    /// one database transaction. `None` means the invitation was not in an
    /// acceptable state for this atomic transition.
    fn activate_invitation(
        &self,
        invitation_id: Uuid,
        principal_id: Uuid,
        when: DateTime<Utc>,
    ) -> Result<Option<ScopeMember>, RepoError>;
    fn change_member_role(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        role: ScopeRole,
        when: DateTime<Utc>,
    ) -> Result<Option<ScopeMember>, RepoError>;
    fn revoke_member(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        when: DateTime<Utc>,
    ) -> Result<Option<ScopeMember>, RepoError>;
    fn append_lifecycle_event(&self, event: &ScopeLifecycleEvent) -> Result<(), RepoError>;
    fn lifecycle_events_for_scope(
        &self,
        scope_id: Uuid,
    ) -> Result<Vec<ScopeLifecycleEvent>, RepoError>;
    fn create_service_principal(&self, display_name: &str) -> Result<ServerPrincipal, RepoError>;
    fn service_principal_by_name(
        &self,
        display_name: &str,
    ) -> Result<Option<ServerPrincipal>, RepoError>;
    fn upsert_member(
        &self,
        scope_id: Uuid,
        principal_id: Uuid,
        role: ScopeRole,
    ) -> Result<ScopeMember, RepoError>;
}
