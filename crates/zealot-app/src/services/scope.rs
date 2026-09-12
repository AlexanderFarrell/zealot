use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;
use zealot_domain::scope::{
    PrincipalKind, PrincipalStatus, Scope, ScopeInvitation, ScopeInvitationStatus,
    ScopeLifecycleEvent, ScopeMember, ScopeMemberStatus, ScopePermission, ScopeRole,
    ServerPrincipal,
};

use crate::{
    repos::common::RepoError,
    repos::scope::{ScopeRepo, ServerMetadata},
};

type InvitationHmac = Hmac<Sha256>;

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

#[derive(Debug, Clone)]
pub struct CreatedInvitation {
    pub invitation: ScopeInvitation,
    /// The opaque bearer is returned once to the caller. It is never persisted
    /// or placed in lifecycle-event metadata.
    pub token: String,
}

#[derive(Debug, Clone)]
pub enum InvitationAcceptance {
    Active(ScopeMember),
    PendingAdmission(ScopeInvitation),
}

#[derive(Debug, thiserror::Error)]
pub enum MembershipError {
    #[error("scope membership action forbidden")]
    Forbidden,
    #[error("scope or membership target not found")]
    NotFound,
    #[error("invalid invitation")]
    InvalidInvitation,
    #[error("invitation expired")]
    Expired,
    #[error("invitation is no longer usable")]
    TerminalInvitation,
    #[error("requested role exceeds issuer grant")]
    RoleEscalation,
    #[error("cannot remove or downgrade the final active owner")]
    FinalOwner,
    #[error("invalid membership state")]
    InvalidState,
    #[error("repository error: {0}")]
    Repo(#[from] RepoError),
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
    pub fn principal_by_id(
        &self,
        principal_id: Uuid,
    ) -> Result<Option<ServerPrincipal>, RepoError> {
        self.repo.principal_by_id(principal_id)
    }

    /// Authorizes a principal against the persisted active membership.  This is
    /// the single role-to-grant decision point for HTTP and application code.
    pub fn authorize(
        &self,
        principal_id: Uuid,
        scope_id: Uuid,
        permission: ScopePermission,
    ) -> Result<bool, RepoError> {
        let Some(principal) = self.repo.principal_by_id(principal_id)? else {
            return Ok(false);
        };
        if principal.status != PrincipalStatus::Active {
            return Ok(false);
        }
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
        let Some(principal) = self.repo.principal_by_id(principal_id)? else {
            return Err(ScopeAccessError::MissingPrincipal);
        };
        if principal.status != PrincipalStatus::Active {
            return Err(ScopeAccessError::MissingPrincipal);
        }
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
            return Ok(ScopeAccess {
                principal_id,
                scopes,
                permission,
                read_only,
            });
        }

        let scope = match requested {
            Some(scope_id) => self
                .scope_by_id(scope_id)?
                .ok_or(ScopeAccessError::NotFound)?,
            None => self
                .default_scope_for_principal(principal_id)?
                .ok_or(ScopeAccessError::DefaultUnavailable)?,
        };
        if !self.authorize(principal_id, scope.scope_id, permission)? {
            return Err(ScopeAccessError::Forbidden);
        }
        Ok(ScopeAccess {
            principal_id,
            scopes: vec![scope],
            permission,
            read_only,
        })
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

    /// Resolve the persisted scope owner as the execution identity for an
    /// automation rule. Rules have no interactive actor, so their scope and
    /// owner membership are checked at execution time instead of falling back
    /// to the legacy account boundary.
    pub fn rule_access(&self, scope_id: Uuid) -> Result<Option<ScopeAccess>, RepoError> {
        let Some(scope) = self.repo.scope_by_id(scope_id)? else {
            return Ok(None);
        };
        if scope.status != zealot_domain::scope::ScopeStatus::Active
            || !self.authorize(
                scope.owner_principal_id,
                scope_id,
                ScopePermission::UpdateItems,
            )?
        {
            return Ok(None);
        }
        Ok(Some(ScopeAccess {
            principal_id: scope.owner_principal_id,
            scopes: vec![scope],
            permission: ScopePermission::UpdateItems,
            read_only: false,
        }))
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

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    fn invitation_mac(
        signing_key: &str,
        invitation_id: Uuid,
        scope_id: Uuid,
        role: ScopeRole,
        recipient_principal_id: Option<Uuid>,
        nonce: &str,
    ) -> InvitationHmac {
        let mut mac = InvitationHmac::new_from_slice(signing_key.as_bytes())
            .expect("HMAC accepts signing keys of every length");
        mac.update(b"zealot-invitation-v1\0");
        mac.update(invitation_id.as_bytes());
        mac.update(b"\0");
        mac.update(scope_id.as_bytes());
        mac.update(b"\0");
        mac.update(role.as_str().as_bytes());
        mac.update(b"\0");
        if let Some(recipient_principal_id) = recipient_principal_id {
            mac.update(recipient_principal_id.as_bytes());
        } else {
            mac.update(b"unbound");
        }
        mac.update(b"\0");
        mac.update(nonce.as_bytes());
        mac
    }

    fn invitation_signature(
        signing_key: &str,
        invitation_id: Uuid,
        scope_id: Uuid,
        role: ScopeRole,
        recipient_principal_id: Option<Uuid>,
        nonce: &str,
    ) -> String {
        hex::encode(
            Self::invitation_mac(
                signing_key,
                invitation_id,
                scope_id,
                role,
                recipient_principal_id,
                nonce,
            )
            .finalize()
            .into_bytes(),
        )
    }

    fn invitation_signature_valid(
        signing_key: &str,
        invitation_id: Uuid,
        scope_id: Uuid,
        role: ScopeRole,
        recipient_principal_id: Option<Uuid>,
        nonce: &str,
        encoded_signature: &str,
    ) -> bool {
        let Ok(signature) = hex::decode(encoded_signature) else {
            return false;
        };
        Self::invitation_mac(
            signing_key,
            invitation_id,
            scope_id,
            role,
            recipient_principal_id,
            nonce,
        )
        .verify_slice(&signature)
        .is_ok()
    }

    fn invitation_signing_key(&self) -> Result<String, MembershipError> {
        if let Some(key) = self.repo.server_invitation_signing_key()? {
            return Ok(key);
        }
        // The key is generated from the OS-backed UUID v4 source and is
        // initialized once in the server-local metadata row. The repository's
        // conditional update makes concurrent first issuances converge on one
        // key without exposing it outside the service.
        let generated = format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple());
        Ok(self.repo.set_server_invitation_signing_key(&generated)?)
    }

    fn role_rank(role: ScopeRole) -> u8 {
        match role {
            ScopeRole::Viewer => 1,
            ScopeRole::Editor => 2,
            ScopeRole::Owner => 3,
        }
    }

    fn role_can_grant(issuer: ScopeRole, requested: ScopeRole) -> bool {
        Self::role_rank(requested) <= Self::role_rank(issuer)
    }

    fn role_allowed_for_principal(principal: &ServerPrincipal, role: ScopeRole) -> bool {
        principal.kind == PrincipalKind::Human || role != ScopeRole::Owner
    }

    fn manager_role(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
    ) -> Result<ScopeRole, MembershipError> {
        let scope = self
            .repo
            .scope_by_id(scope_id)?
            .ok_or(MembershipError::NotFound)?;
        if scope.status != zealot_domain::scope::ScopeStatus::Active {
            return Err(MembershipError::NotFound);
        }
        let principal = self
            .repo
            .principal_by_id(actor_principal_id)?
            .ok_or(MembershipError::Forbidden)?;
        if principal.status != PrincipalStatus::Active {
            return Err(MembershipError::Forbidden);
        }
        self.repo
            .members_for_scope(scope_id)?
            .into_iter()
            .find(|member| {
                member.principal_id == actor_principal_id
                    && member.status == ScopeMemberStatus::Active
                    && member.role.grants(ScopePermission::ManageMembers)
            })
            .map(|member| member.role)
            .ok_or(MembershipError::Forbidden)
    }

    pub fn create_invitation(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        permitted_role: ScopeRole,
        recipient_principal_id: Option<Uuid>,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<CreatedInvitation, MembershipError> {
        let issuer_role = self.manager_role(actor_principal_id, scope_id)?;
        if !Self::role_can_grant(issuer_role, permitted_role) {
            return Err(MembershipError::RoleEscalation);
        }
        if let Some(recipient) = recipient_principal_id {
            let principal = self
                .repo
                .principal_by_id(recipient)?
                .ok_or(MembershipError::NotFound)?;
            if principal.status != PrincipalStatus::Active
                || !Self::role_allowed_for_principal(&principal, permitted_role)
            {
                return Err(MembershipError::NotFound);
            }
        }
        let signing_key = self.invitation_signing_key()?;
        let invitation_id = Uuid::new_v4();
        let nonce = Uuid::new_v4().simple().to_string();
        let now = Utc::now();
        let expires_at = expires_at.unwrap_or_else(|| now + Duration::days(7));
        if expires_at <= now {
            return Err(MembershipError::InvalidInvitation);
        }
        let signature = Self::invitation_signature(
            &signing_key,
            invitation_id,
            scope_id,
            permitted_role,
            recipient_principal_id,
            &nonce,
        );
        let token = format!("z1.{nonce}.{signature}");
        let invitation = ScopeInvitation {
            invitation_id,
            scope_id,
            issuer_principal_id: actor_principal_id,
            recipient_principal_id,
            permitted_role,
            token_hash: Self::hash_token(&token),
            token_signature: signature,
            status: ScopeInvitationStatus::Pending,
            created_at: now,
            expires_at,
            accepted_at: None,
            terminal_at: None,
            correlation_id: Uuid::new_v4(),
        };
        let invitation = self.repo.create_invitation(&invitation)?;
        self.record_event(
            &invitation,
            Some(actor_principal_id),
            "invitation_created",
            "success",
            "created",
        )?;
        Ok(CreatedInvitation { invitation, token })
    }

    pub fn invitations_for_scope(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
    ) -> Result<Vec<ScopeInvitation>, MembershipError> {
        self.manager_role(actor_principal_id, scope_id)?;
        Ok(self.repo.invitations_for_scope(scope_id)?)
    }

    pub fn accept_invitation(
        &self,
        token: &str,
        actor_principal_id: Option<Uuid>,
    ) -> Result<InvitationAcceptance, MembershipError> {
        let token = token.trim();
        if token.is_empty() {
            return Err(MembershipError::InvalidInvitation);
        }
        let invitation = self
            .repo
            .invitation_by_token_hash(&Self::hash_token(token))?
            .ok_or(MembershipError::InvalidInvitation)?;
        let signing_key = self.invitation_signing_key()?;
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 || parts[0] != "z1" {
            return Err(MembershipError::InvalidInvitation);
        }
        if !Self::invitation_signature_valid(
            &signing_key,
            invitation.invitation_id,
            invitation.scope_id,
            invitation.permitted_role,
            invitation.recipient_principal_id,
            parts[1],
            parts[2],
        ) || invitation.token_signature != parts[2]
        {
            return Err(MembershipError::InvalidInvitation);
        }
        let scope = self
            .repo
            .scope_by_id(invitation.scope_id)?
            .ok_or(MembershipError::InvalidInvitation)?;
        if scope.status != zealot_domain::scope::ScopeStatus::Active {
            return Err(MembershipError::InvalidInvitation);
        }
        let now = Utc::now();
        if invitation.expires_at <= now {
            self.repo.transition_invitation(
                invitation.invitation_id,
                ScopeInvitationStatus::Expired,
                now,
            )?;
            self.record_event(
                &invitation,
                actor_principal_id,
                "invitation_expired",
                "denied",
                "expired",
            )?;
            return Err(MembershipError::Expired);
        }
        if !matches!(
            invitation.status,
            ScopeInvitationStatus::Pending | ScopeInvitationStatus::PendingAdmission
        ) {
            return Err(MembershipError::TerminalInvitation);
        }
        let policy = self.repo.server_enrolment_policy()?;
        if !matches!(
            policy.as_str(),
            "closed" | "admin-approved" | "invite-enabled" | "open-registration"
        ) {
            return Err(MembershipError::InvalidState);
        }
        let Some(principal_id) = actor_principal_id else {
            let pending = self
                .repo
                .transition_invitation(
                    invitation.invitation_id,
                    ScopeInvitationStatus::PendingAdmission,
                    now,
                )?
                .ok_or(MembershipError::InvalidInvitation)?;
            self.record_event(
                &pending,
                None,
                "invitation_admission_pending",
                "pending",
                &format!("authentication_required:{policy}"),
            )?;
            return Ok(InvitationAcceptance::PendingAdmission(pending));
        };
        if let Some(expected) = invitation.recipient_principal_id {
            if expected != principal_id {
                return Err(MembershipError::InvalidInvitation);
            }
        }
        let principal = self
            .repo
            .principal_by_id(principal_id)?
            .ok_or(MembershipError::InvalidInvitation)?;
        if principal.status != PrincipalStatus::Active
            || !Self::role_allowed_for_principal(&principal, invitation.permitted_role)
        {
            return Err(MembershipError::InvalidInvitation);
        }
        let member = self
            .repo
            .activate_invitation(invitation.invitation_id, principal_id, now)?
            .ok_or(MembershipError::TerminalInvitation)?;
        self.record_event(
            &invitation,
            Some(principal_id),
            "invitation_accepted",
            "success",
            "accepted",
        )?;
        Ok(InvitationAcceptance::Active(member))
    }

    pub fn approve_invitation(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        invitation_id: Uuid,
        principal_id: Uuid,
    ) -> Result<ScopeMember, MembershipError> {
        let issuer_role = self.manager_role(actor_principal_id, scope_id)?;
        let invitation = self
            .repo
            .invitations_for_scope(scope_id)?
            .into_iter()
            .find(|item| item.invitation_id == invitation_id)
            .ok_or(MembershipError::NotFound)?;
        if !matches!(
            invitation.status,
            ScopeInvitationStatus::Pending | ScopeInvitationStatus::PendingAdmission
        ) || !Self::role_can_grant(issuer_role, invitation.permitted_role)
        {
            return Err(MembershipError::TerminalInvitation);
        }
        if invitation.recipient_principal_id.is_some()
            && invitation.recipient_principal_id != Some(principal_id)
        {
            return Err(MembershipError::InvalidInvitation);
        }
        let principal = self
            .repo
            .principal_by_id(principal_id)?
            .ok_or(MembershipError::InvalidInvitation)?;
        if principal.status != PrincipalStatus::Active
            || !Self::role_allowed_for_principal(&principal, invitation.permitted_role)
        {
            return Err(MembershipError::InvalidInvitation);
        }
        let member = self
            .repo
            .activate_invitation(invitation_id, principal_id, Utc::now())?
            .ok_or(MembershipError::TerminalInvitation)?;
        self.record_event(
            &invitation,
            Some(actor_principal_id),
            "invitation_approved",
            "success",
            "approved",
        )?;
        Ok(member)
    }

    pub fn cancel_invitation(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), MembershipError> {
        self.manager_role(actor_principal_id, scope_id)?;
        self.transition_invitation(
            actor_principal_id,
            scope_id,
            invitation_id,
            ScopeInvitationStatus::Cancelled,
            "cancelled",
        )
    }

    pub fn revoke_invitation(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        invitation_id: Uuid,
    ) -> Result<(), MembershipError> {
        self.manager_role(actor_principal_id, scope_id)?;
        self.transition_invitation(
            actor_principal_id,
            scope_id,
            invitation_id,
            ScopeInvitationStatus::Revoked,
            "revoked",
        )
    }

    fn transition_invitation(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        invitation_id: Uuid,
        status: ScopeInvitationStatus,
        reason: &str,
    ) -> Result<(), MembershipError> {
        let invitation = self
            .repo
            .invitations_for_scope(scope_id)?
            .into_iter()
            .find(|item| item.invitation_id == invitation_id)
            .ok_or(MembershipError::NotFound)?;
        if !matches!(
            invitation.status,
            ScopeInvitationStatus::Pending | ScopeInvitationStatus::PendingAdmission
        ) {
            return Err(MembershipError::TerminalInvitation);
        }
        let updated = self
            .repo
            .transition_invitation(invitation_id, status, Utc::now())?
            .ok_or(MembershipError::TerminalInvitation)?;
        self.record_event(
            &updated,
            Some(actor_principal_id),
            &format!("invitation_{reason}"),
            "success",
            reason,
        )?;
        Ok(())
    }

    pub fn change_member_role(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        target_principal_id: Uuid,
        role: ScopeRole,
    ) -> Result<ScopeMember, MembershipError> {
        let issuer_role = self.manager_role(actor_principal_id, scope_id)?;
        if !Self::role_can_grant(issuer_role, role) {
            return Err(MembershipError::RoleEscalation);
        }
        let target_principal = self
            .repo
            .principal_by_id(target_principal_id)?
            .ok_or(MembershipError::NotFound)?;
        if target_principal.status != PrincipalStatus::Active {
            return Err(MembershipError::NotFound);
        }
        if !Self::role_allowed_for_principal(&target_principal, role) {
            return Err(MembershipError::RoleEscalation);
        }
        let members = self.repo.members_for_scope(scope_id)?;
        let target = members
            .iter()
            .find(|member| {
                member.principal_id == target_principal_id
                    && member.status == ScopeMemberStatus::Active
            })
            .ok_or(MembershipError::NotFound)?;
        if target.role == ScopeRole::Owner
            && role != ScopeRole::Owner
            && members
                .iter()
                .filter(|member| {
                    member.status == ScopeMemberStatus::Active && member.role == ScopeRole::Owner
                })
                .count()
                <= 1
        {
            return Err(MembershipError::FinalOwner);
        }
        let member = self
            .repo
            .change_member_role(scope_id, target_principal_id, role, Utc::now())?
            .ok_or(MembershipError::NotFound)?;
        self.record_member_event(
            scope_id,
            Some(actor_principal_id),
            Some(target_principal_id),
            "membership_role_changed",
            "success",
            "role_changed",
        )?;
        Ok(member)
    }

    pub fn revoke_member(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
        target_principal_id: Uuid,
    ) -> Result<(), MembershipError> {
        self.manager_role(actor_principal_id, scope_id)?;
        let members = self.repo.members_for_scope(scope_id)?;
        let target = members
            .iter()
            .find(|member| {
                member.principal_id == target_principal_id
                    && member.status == ScopeMemberStatus::Active
            })
            .ok_or(MembershipError::NotFound)?;
        if target.role == ScopeRole::Owner
            && members
                .iter()
                .filter(|member| {
                    member.status == ScopeMemberStatus::Active && member.role == ScopeRole::Owner
                })
                .count()
                <= 1
        {
            return Err(MembershipError::FinalOwner);
        }
        self.repo
            .revoke_member(scope_id, target_principal_id, Utc::now())?
            .ok_or(MembershipError::NotFound)?;
        self.record_member_event(
            scope_id,
            Some(actor_principal_id),
            Some(target_principal_id),
            "membership_revoked",
            "success",
            "revoked",
        )?;
        Ok(())
    }

    pub fn lifecycle_events_for_scope(
        &self,
        actor_principal_id: Uuid,
        scope_id: Uuid,
    ) -> Result<Vec<ScopeLifecycleEvent>, MembershipError> {
        self.manager_role(actor_principal_id, scope_id)?;
        Ok(self.repo.lifecycle_events_for_scope(scope_id)?)
    }

    fn record_event(
        &self,
        invitation: &ScopeInvitation,
        actor_principal_id: Option<Uuid>,
        event_type: &str,
        outcome: &str,
        reason_code: &str,
    ) -> Result<(), MembershipError> {
        self.repo.append_lifecycle_event(&ScopeLifecycleEvent {
            event_id: Uuid::new_v4(),
            scope_id: invitation.scope_id,
            actor_principal_id,
            subject_principal_id: invitation.recipient_principal_id,
            invitation_id: Some(invitation.invitation_id),
            event_type: event_type.to_owned(),
            outcome: outcome.to_owned(),
            reason_code: reason_code.to_owned(),
            correlation_id: invitation.correlation_id,
            occurred_at: Utc::now(),
        })?;
        Ok(())
    }

    fn record_member_event(
        &self,
        scope_id: Uuid,
        actor_principal_id: Option<Uuid>,
        subject_principal_id: Option<Uuid>,
        event_type: &str,
        outcome: &str,
        reason_code: &str,
    ) -> Result<(), MembershipError> {
        self.repo.append_lifecycle_event(&ScopeLifecycleEvent {
            event_id: Uuid::new_v4(),
            scope_id,
            actor_principal_id,
            subject_principal_id,
            invitation_id: None,
            event_type: event_type.to_owned(),
            outcome: outcome.to_owned(),
            reason_code: reason_code.to_owned(),
            correlation_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
        })?;
        Ok(())
    }
}
