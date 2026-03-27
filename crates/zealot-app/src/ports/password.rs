use std::fmt::Debug;

use crate::ports::{common::PortError, password};


#[derive(Debug, thiserror::Error)]
pub enum PasswordError {
    #[error("failed to hash password")]
    HashFailed,

    #[error("failed to verify password")]
    VerifyFailed,
}

pub trait PasswordPort: Debug + Send + Sync {
    fn hash_password(&self, password: &str) -> Result<String, PortError<PasswordError>>;
    fn verify_password(
        &self,
        password: &str,
        password_hash: &str,
    ) -> Result<bool, PortError<PasswordError>>;
}