use zealot_app::ports::{
    common::PortError,
    password::{PasswordError, PasswordPort},
};

#[derive(Debug)]
pub struct BcryptPasswordPort;

impl BcryptPasswordPort {
    pub fn new() -> Self {
        Self
    }
}

impl PasswordPort for BcryptPasswordPort {
    fn hash_password(&self, password: &str) -> Result<String, PortError<PasswordError>> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|_| PortError::OtherError { err: PasswordError::HashFailed })
    }

    fn verify_password(
        &self,
        password: &str,
        password_hash: &str,
    ) -> Result<bool, PortError<PasswordError>> {
        bcrypt::verify(password, password_hash)
            .map_err(|_| PortError::OtherError { err: PasswordError::VerifyFailed })
    }
}
