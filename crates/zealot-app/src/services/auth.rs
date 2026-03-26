use std::{fs::remove_dir, sync::Arc};

use zealot_domain::{account::{Account, LoginBasicDto, RegisterBasicDto}, auth::{Actor, AuthSource}};

use crate::{repos::account::AccountRepo, services::common::ServiceError};


#[derive(Debug, Clone)]
pub struct AuthService {
    repo: Arc<dyn AccountRepo>,
}

impl AuthService {
    pub fn new(repo: &Arc<dyn AccountRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub async fn authenticate_password(&self, username: &str, password: &str) -> Result<Actor, ServiceError<AuthError>> {
        todo!()
    }

    pub async fn authenticate_api_key(&self, key: &str) -> Result<Actor, ServiceError<AuthError>> {
        todo!()
    }

    pub async fn authenticate_session(&self, session_id: &str) -> Result<Actor, ServiceError<AuthError>> {
        todo!()
    }

    pub fn get_anonymous_actor(&self) -> Actor {
        Actor {
            account: None,
            source: AuthSource::Anonymous,
        }
    }

    pub async fn register_account(
        &self,
        dto: &RegisterBasicDto,
    ) -> Result<Account, AuthError> {
        todo!()
    }

    pub async fn login_account(
        &self,
        dto: &LoginBasicDto,
    ) -> Result<Account, AuthError> {
        todo!()
    }

    pub async fn logout_account(
        &self,
        actor: &Actor,
    ) -> Result<(), AuthError> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("error registering account: {err:?}")]
    RegisterError{err: String},

    #[error("error logging in")]
    LoginError{err: String},

    #[error("server error")]
    ServerError,
}

