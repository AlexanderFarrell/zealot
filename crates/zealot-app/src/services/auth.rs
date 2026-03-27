use std::sync::Arc;

use chrono::{Duration, Utc};
use sha2::{Digest, Sha256};
use uuid::Uuid;
use zealot_domain::{
    account::{Account, CreateAccountDto, LoginBasicDto, RegisterBasicDto},
    auth::{Actor, AuthSource, CreateSessionDto},
    common::email::Email,
};

use crate::{
    ports::password::PasswordPort,
    repos::{account::AccountRepo, session::SessionRepo},
    services::common::ServiceError,
};

#[derive(Debug, Clone)]
pub struct AuthService {
    repo: Arc<dyn AccountRepo>,
    password: Arc<dyn PasswordPort>,
    session: Arc<dyn SessionRepo>,
}

impl AuthService {
    pub fn new(
        repo: &Arc<dyn AccountRepo>,
        password: &Arc<dyn PasswordPort>,
        session: &Arc<dyn SessionRepo>,
    ) -> Self {
        Self {
            repo: repo.clone(),
            password: password.clone(),
            session: session.clone(),
        }
    }

    pub async fn authenticate_password(
        &self,
        username: &str,
        password: &str,
    ) -> Result<Actor, ServiceError<AuthError>> {
        let username = username.trim();

        if username.is_empty() || password.trim().is_empty() {
            return Err(ServiceError::DomainError {
                err: Self::invalid_login_error(),
            });
        }

        let stored_hash = self
            .repo
            .get_password_hash_by_username(username)
            .map_err(|_| ServiceError::DomainError {
                err: Self::invalid_login_error(),
            })?;

        let Some(stored_hash) = stored_hash else {
            return Err(ServiceError::DomainError {
                err: Self::invalid_login_error(),
            });
        };

        let password_matches = self
            .password
            .verify_password(password, &stored_hash)
            .map_err(|_| ServiceError::DomainError {
                err: AuthError::ServerError,
            })?;

        if !password_matches {
            return Err(ServiceError::DomainError {
                err: Self::invalid_login_error(),
            });
        }

        let account = self
            .repo
            .get_account_by_username(username)
            .map_err(|_| ServiceError::DomainError {
                err: AuthError::ServerError,
            })?;

        let Some(account) = account else {
            return Err(ServiceError::DomainError {
                err: AuthError::ServerError,
            });
        };

        Ok(Self::actor_from_account(account, AuthSource::PlainLogin))
    }

    pub async fn authenticate_api_key(
        &self,
        key: &str,
    ) -> Result<Actor, ServiceError<AuthError>> {
        let key = key.trim();

        if key.is_empty() {
            return Err(ServiceError::DomainError {
                err: Self::invalid_api_key_error(),
            });
        }

        // TODO: Ensure that api key is encrypted at rest.
        let account = self
            .repo
            .get_account_by_api_key(key)
            .map_err(|_| ServiceError::DomainError {
                err: AuthError::ServerError,
            })?;

        let Some(account) = account else {
            return Err(ServiceError::DomainError {
                err: Self::invalid_api_key_error(),
            });
        };

        Ok(Self::actor_from_account(account, AuthSource::ApiKey))
    }

    pub async fn authenticate_session(
        &self,
        session_token: &str,
    ) -> Result<Actor, ServiceError<AuthError>> {
        let token_hash = Self::hash_token(session_token);

        let account = self
            .session
            .get_account_by_token_hash(&token_hash)
            .map_err(|_| ServiceError::DomainError {
                err: AuthError::ServerError,
            })?;

        let Some(account) = account else {
            return Err(ServiceError::DomainError {
                err: Self::invalid_session_error(),
            });
        };

        Ok(Self::actor_from_account(account, AuthSource::Session))
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
        let username = dto.username.trim();
        let given_name = dto.given_name.trim();
        let surname = dto.surname.trim();

        if username.is_empty() {
            return Err(AuthError::RegisterError {
                err: String::from("Username is required"),
            });
        };

        if dto.password.trim().is_empty() {
            return Err(AuthError::RegisterError {
                err: String::from("Password is required"),
            });
        };

        if given_name.is_empty() {
            return Err(AuthError::RegisterError {
                err: String::from("Surname is required"),
            });
        }

        let email = Email::try_from(dto.email.clone())
            .map_err(|err| AuthError::RegisterError {
                err: format!("Invalid email: {}", err),
            })?;

        let existing_account = self
            .repo
            .get_account_by_username(username)
            .map_err(|_| AuthError::ServerError)?;

        if existing_account.is_some() {
            return Err(AuthError::RegisterError {
                err: String::from("Username already taken"),
            });
        };

        let password_hash = self
            .password
            .hash_password(&dto.password)
            .map_err(|_| AuthError::ServerError)?;

        let create_dto = CreateAccountDto {
            username: username.to_string(),
            password_hash,
            email: email.to_string(),
            given_name: given_name.to_string(),
            surname: surname.to_string(),
        };

        self.repo
            .add_account(&create_dto)
            .map_err(|_| AuthError::ServerError)
    }

    /// Returns the account and the raw session token to be stored in the cookie.
    pub async fn login_account(
        &self,
        dto: &LoginBasicDto,
    ) -> Result<(Account, String), AuthError> {
        let actor = self
            .authenticate_password(&dto.username, &dto.password)
            .await
            .map_err(|error| match error {
                ServiceError::DomainError { err } => err,
            })?;

        let account = actor.account.ok_or(AuthError::ServerError)?;

        let (raw_token, token_hash) = Self::generate_session_token();
        let expires_at = Utc::now() + Duration::days(30);

        self.session
            .create_session(&CreateSessionDto {
                token_hash,
                account_id: account.account_id,
                expires_at,
            })
            .map_err(|_| AuthError::ServerError)?;

        Ok((account, raw_token))
    }

    pub async fn logout_account(
        &self,
        actor: &Actor,
        session_token: Option<&str>,
    ) -> Result<(), AuthError> {
        if !actor.is_authenticated() {
            return Err(AuthError::LoginError {
                err: String::from("Not logged in"),
            });
        }

        if let Some(token) = session_token {
            let token_hash = Self::hash_token(token);
            self.session
                .delete_session_by_token_hash(&token_hash)
                .map_err(|_| AuthError::ServerError)?;
        }

        Ok(())
    }

    fn actor_from_account(account: Account, source: AuthSource) -> Actor {
        Actor {
            account: Some(account),
            source,
        }
    }

    fn invalid_login_error() -> AuthError {
        AuthError::LoginError {
            err: String::from("Invalid username or password"),
        }
    }

    fn invalid_api_key_error() -> AuthError {
        AuthError::LoginError {
            err: String::from("Invalid API key"),
        }
    }

    fn invalid_session_error() -> AuthError {
        AuthError::LoginError {
            err: String::from("Invalid or expired session"),
        }
    }

    /// Generates a raw session token (sent to client) and its SHA-256 hash (stored in DB).
    fn generate_session_token() -> (String, String) {
        let raw_token = Uuid::new_v4().simple().to_string();
        let token_hash = Self::hash_token(&raw_token);
        (raw_token, token_hash)
    }

    fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("error registering account: {err:?}")]
    RegisterError { err: String },

    #[error("error logging in")]
    LoginError { err: String },

    #[error("server error")]
    ServerError,
}
