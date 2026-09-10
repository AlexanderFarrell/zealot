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
