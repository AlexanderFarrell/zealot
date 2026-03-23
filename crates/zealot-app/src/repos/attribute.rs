use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto},
    common::id::Id,
};

use crate::repos::common::RepoError;

pub trait AttributeRepo: Debug + Send + Sync {
    fn get_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError>;
    fn get_attribute_kind_by_id(
        &self,
        id: &Id,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError>;
    fn get_attribute_kinds_for_user(
        &self,
        account_id: &Id,
    ) -> Result<HashMap<String, AttributeKind>, RepoError>;
    fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError>;
    fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, RepoError>;
    fn delete_attribute_kind(&self, key: &str, account_id: &Id) -> Result<(), RepoError>;
}
