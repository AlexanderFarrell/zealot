use crate::account::Account;

#[derive(Debug, Clone)]
pub enum AuthSource {
    Anonymous,
    PlainLogin,
    OAuthJwt,
    ApiKey,
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