use chrono::{DateTime, Utc};
use std::{fmt, str::FromStr};
use uuid::Uuid;

macro_rules! string_enum {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $name { $($variant),+ }

        impl $name {
            pub const fn as_str(self) -> &'static str {
                match self { $(Self::$variant => $value),+ }
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.write_str(self.as_str()) }
        }

        impl FromStr for $name {
            type Err = String;
            fn from_str(value: &str) -> Result<Self, Self::Err> {
                match value { $($value => Ok(Self::$variant),)+ _ => Err(format!("invalid {}: {value}", stringify!($name))) }
            }
        }
    };
}

string_enum!(PrincipalKind { Human => "human", Service => "service" });
string_enum!(PrincipalStatus { Active => "active", Retired => "retired" });
string_enum!(ScopeRole { Owner => "owner", Editor => "editor", Viewer => "viewer" });
string_enum!(ScopeStatus { Active => "active", Archived => "archived" });
string_enum!(ScopeMemberStatus { Active => "active", Revoked => "revoked" });
string_enum!(ScopeInvitationStatus {
    Pending => "pending",
    PendingAdmission => "pending_admission",
    Accepted => "accepted",
    Active => "active",
    Cancelled => "cancelled",
    Expired => "expired",
    Revoked => "revoked"
});

/// Named grants are intentionally separate from roles.  Roles are the v1
/// presets; future membership work can add overrides without changing callers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopePermission {
    ViewItems,
    CreateItems,
    UpdateItems,
    DeleteItems,
    ManageMembers,
    ManageScopeSettings,
}

impl ScopeRole {
    pub const fn grants(self, permission: ScopePermission) -> bool {
        match self {
            Self::Owner => true,
            Self::Editor => matches!(
                permission,
                ScopePermission::ViewItems
                    | ScopePermission::CreateItems
                    | ScopePermission::UpdateItems
                    | ScopePermission::DeleteItems
            ),
            Self::Viewer => matches!(permission, ScopePermission::ViewItems),
        }
    }
}

/// A stable, server-local identity. Human principals map one-to-one to an
/// account; service principals deliberately have no account or default scope.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServerPrincipal {
    pub principal_id: Uuid,
    pub server_id: Uuid,
    pub kind: PrincipalKind,
    pub account_id: Option<i64>,
    pub display_name: String,
    pub status: PrincipalStatus,
    pub default_scope_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub retired_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scope {
    pub scope_id: Uuid,
    pub server_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub status: ScopeStatus,
    pub owner_principal_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeMember {
    pub scope_id: Uuid,
    pub principal_id: Uuid,
    pub role: ScopeRole,
    pub status: ScopeMemberStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A scope-bound invitation capability. The bearer token is never part of
/// this record; only its secure hash and a signature binding it to the scope,
/// role, and server are persisted by the infrastructure layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeInvitation {
    pub invitation_id: Uuid,
    pub scope_id: Uuid,
    pub issuer_principal_id: Uuid,
    pub recipient_principal_id: Option<Uuid>,
    pub permitted_role: ScopeRole,
    pub token_hash: String,
    pub token_signature: String,
    pub status: ScopeInvitationStatus,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub terminal_at: Option<DateTime<Utc>>,
    pub correlation_id: Uuid,
}

/// Redacted, append-only lifecycle evidence. `reason_code` and `outcome` are
/// safe to expose to an authorized audit reader; no token or credential value
/// is carried here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeLifecycleEvent {
    pub event_id: Uuid,
    pub scope_id: Uuid,
    pub actor_principal_id: Option<Uuid>,
    pub subject_principal_id: Option<Uuid>,
    pub invitation_id: Option<Uuid>,
    pub event_type: String,
    pub outcome: String,
    pub reason_code: String,
    pub correlation_id: Uuid,
    pub occurred_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::{ScopePermission, ScopeRole};

    #[test]
    fn built_in_roles_have_the_documented_grants() {
        assert!(ScopeRole::Owner.grants(ScopePermission::ManageMembers));
        assert!(ScopeRole::Editor.grants(ScopePermission::DeleteItems));
        assert!(!ScopeRole::Editor.grants(ScopePermission::ManageScopeSettings));
        assert!(ScopeRole::Viewer.grants(ScopePermission::ViewItems));
        assert!(!ScopeRole::Viewer.grants(ScopePermission::CreateItems));
    }
}
