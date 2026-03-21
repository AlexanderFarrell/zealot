use std::collections::HashMap;

use zealot_domain::{attribute::{AddAttributeKindDto, AttributeError, AttributeKind, UpdateAttributeKindDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait AttributeRepo {
    async fn get_attribute_kind(key: &str, account_id: &Id) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
    async fn get_attribute_kind_by_id(id: &Id, account_id: &Id) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
    async fn get_attribute_kinds_for_user(account_id: &Id) -> Result<HashMap<String, AttributeKind>, RepoError<AttributeError>>;
    async fn add_attribute_kind(dto: &AddAttributeKindDto) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
    async fn update_attribute_kind(dto: &UpdateAttributeKindDto) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
    async fn delete_attribute_kind(key: &str, account_id: &Id) -> Result<(), RepoError<AttributeError>>;
}