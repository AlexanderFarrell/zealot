use serde::{Deserialize, Serialize};

use crate::common::email::{Email, EmailError};
use crate::common::id::{Id, IdError};

// Domain

#[derive(Debug, Clone)]
pub struct Account {
    pub account_id: Id,
    pub username: String,
    pub email: Email,
    pub given_name: String,
    pub surname: String,
}

pub struct APIKey(String);

// Send DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDto {
    pub account_id: i64,
    pub username: String,
    pub email: String,
    pub given_name: String,
    pub surname: String,
}

// Receive DTOs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginBasicDto {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterBasicDto {
    pub username: String,
    pub password: String,
    pub email: String,
    pub given_name: String,
    pub surname: String,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum AccountError {
    #[error("invalid account id: {err:?}")]
    InvalidId { err: IdError },

    #[error("invalid username")]
    InvalidUsername,

    #[error("invalid email")]
    InvalidEmail { err: EmailError },

    #[error("error generating api key")]
    APIKeyGenError {err: String},

    #[error("not found")]
    NotFound,

    #[error("username already taken")]
    UsernameAlreadyTaken,
}

// Impls

impl Account {
    pub fn full_name_eng(&self) -> String {
        return format!("{} {}", self.given_name, self.surname);
    }

    pub fn full_name_formal_eng(&self) -> String {
        return format!("{}, {}", self.surname, self.given_name);
    }
}

impl TryFrom<AccountDto> for Account {
    type Error = AccountError;

    fn try_from(dto: AccountDto) -> Result<Self, Self::Error> {
        Ok(Self {
            account_id: Id::try_from(dto.account_id)
                .map_err(|err| AccountError::InvalidId { err })?,
            username: dto.username,
            email: Email::try_from(dto.email).map_err(|err| AccountError::InvalidEmail { err })?,
            given_name: dto.given_name,
            surname: dto.surname,
        })
    }
}

impl From<&Account> for AccountDto {
    fn from(value: &Account) -> Self {
        Self {
            account_id: value.account_id.into(),
            username: value.username.clone(),
            email: value.email.to_string(),
            given_name: value.given_name.clone(),
            surname: value.surname.clone(),
        }
    }
}

impl From<APIKey> for String {
    fn from(value: APIKey) -> Self {
        String::from(&value)
    }
} 

impl From<&APIKey> for String {
    fn from(value: &APIKey) -> Self {
        value.0.clone()
    }
}

// Tests

#[cfg(test)]
mod account_tests {
    use super::*;

    fn get_test_account() -> Account {
        Account {
            account_id: Id::try_from(5).unwrap(),
            username: String::from("Albert"),
            email: Email::try_from(String::from("a@a.com")).unwrap(),
            given_name: String::from("Albert"),
            surname: String::from("Smith"),
        }
    }

    #[test]
    fn full_name_test() {
        let account = get_test_account();
        assert_eq!(account.full_name_eng(), String::from("Albert Smith"));
    }

    #[test]
    fn full_name_formal_test() {
        let account = get_test_account();
        assert_eq!(
            account.full_name_formal_eng(),
            String::from("Smith, Albert")
        );
    }
}
