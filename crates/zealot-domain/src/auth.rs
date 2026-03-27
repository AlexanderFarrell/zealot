use chrono::{DateTime, Utc};

use crate::account::Account;
use crate::common::id::Id;

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
    pub source: AuthSource,
}

impl Actor {
    pub fn is_authenticated(&self) -> bool {
        match self.source {
            AuthSource::Anonymous => false,
            _ => true
        }
    } 
}