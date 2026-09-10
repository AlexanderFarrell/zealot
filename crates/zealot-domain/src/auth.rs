use chrono::{DateTime, Utc};

use crate::account::Account;
use crate::common::id::Id;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum AuthSource {
    Anonymous,
    PlainLogin,
    OAuthJwt,
    ApiKey,
    Session,
}

#[derive(Debug, Clone)]
pub struct Session {
    pub token_hash: String,
    pub account_id: Id,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateSessionDto {
    pub token_hash: String,
    pub account_id: Id,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Actor {
    pub account: Option<Account>,
    /// Stable server-local identity used for scope authorization.  Account is
    /// deliberately optional because service API keys have no human account.
    pub principal_id: Option<Uuid>,
    pub source: AuthSource,
}

impl Actor {
    pub fn is_authenticated(&self) -> bool {
        match self.source {
            AuthSource::Anonymous => false,
            _ => true,
        }
    }
}
