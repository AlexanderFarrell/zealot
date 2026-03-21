use std::collections::HashMap;

use zealot_domain::{attribute::{AddAttributeKindDto, AttributeError, AttributeKind, UpdateAttributeKindDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait AttributeRepo {
     fn get_attribute_kind(&self, key: &str, account_id: &Id) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
     fn get_attribute_kind_by_id(&self, id: &Id, account_id: &Id) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
     fn get_attribute_kinds_for_user(&self, account_id: &Id) -> Result<HashMap<String, AttributeKind>, RepoError<AttributeError>>;
     fn add_attribute_kind(&self, dto: &AddAttributeKindDto) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
     fn update_attribute_kind(&self, dto: &UpdateAttributeKindDto) -> Result<Option<AttributeKind>, RepoError<AttributeError>>;
     fn delete_attribute_kind(&self, key: &str, account_id: &Id) -> Result<(), RepoError<AttributeError>>;
}
