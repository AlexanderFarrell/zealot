use std::sync::Arc;

use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto},
    common::id::Id,
};

use crate::repos::{attribute::AttributeRepo, common::RepoError};

#[derive(Debug)]
pub struct AttributeService {
    repo: Arc<dyn AttributeRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeServiceError {
    #[error("not found")]
    NotFound,
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl AttributeService {
    pub fn new(repo: &Arc<dyn AttributeRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn get_kind_by_key(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo.get_attribute_kind(key, account_id).map_err(AttributeServiceError::Repo)
    }

    pub fn get_kind_by_id(
        &self,
        kind_id: &Id,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo.get_attribute_kind_by_id(kind_id, account_id).map_err(AttributeServiceError::Repo)
    }

    pub fn get_kinds_for_user(
        &self,
        account_id: &Id,
    ) -> Result<Vec<AttributeKind>, AttributeServiceError> {
        self.repo
            .get_attribute_kinds_for_user(account_id)
            .map(|m| m.into_values().collect())
            .map_err(AttributeServiceError::Repo)
    }

    pub fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo.add_attribute_kind(dto, account_id).map_err(AttributeServiceError::Repo)
    }

    pub fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo.update_attribute_kind(dto, account_id).map_err(AttributeServiceError::Repo)
    }

    pub fn delete_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<(), AttributeServiceError> {
        self.repo.delete_attribute_kind(key, account_id).map_err(AttributeServiceError::Repo)
    }
}
