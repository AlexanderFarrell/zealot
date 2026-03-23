use std::sync::Arc;

use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeError, AttributeKind, UpdateAttributeKindDto},
    common::id::Id,
};

use crate::{repos::attribute::AttributeRepo, services::common::ServiceError};

#[derive(Debug)]
pub struct AttributeService {
    repo: Arc<dyn AttributeRepo>,
}

impl AttributeService {
    pub fn new(repo: &Arc<dyn AttributeRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn get_kind_by_key(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, ServiceError<AttributeError>> {
        todo!()
    }

    pub fn get_kind_by_id(
        &self,
        kind_id: &Id,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, ServiceError<AttributeError>> {
        todo!()
    }

    pub fn get_kinds_for_user(
        &self,
        account_id: &Id,
    ) -> Result<Vec<AttributeKind>, ServiceError<AttributeError>> {
        todo!()
    }

    pub fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, ServiceError<AttributeError>> {
        todo!()
    }

    pub fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, ServiceError<AttributeError>> {
        todo!()
    }

    pub fn delete_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<(), ServiceError<AttributeError>> {
        todo!()
    }
}
