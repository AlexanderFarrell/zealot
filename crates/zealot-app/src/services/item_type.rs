use std::sync::Arc;

use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, UpdateItemTypeDto},
};

use crate::repos::{common::RepoError, item_type::ItemTypeRepo};

#[derive(Debug, Clone)]
pub struct ItemTypeService {
    repo: Arc<dyn ItemTypeRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeServiceError {
    #[error("not found")]
    NotFound,
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl ItemTypeService {
    pub fn new(repo: &Arc<dyn ItemTypeRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, ItemTypeServiceError> {
        self.repo.get_item_types(account_id).map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type(
        &self,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.repo.get_item_type(type_id, account_id).map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.repo.get_item_type_by_name(name, account_id).map_err(ItemTypeServiceError::Repo)
    }

    pub fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.repo.add_item_type(dto, account_id).map_err(ItemTypeServiceError::Repo)
    }

    pub fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        let current = self
            .repo
            .get_item_type(&Id::try_from(dto.type_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?, account_id)
            .map_err(ItemTypeServiceError::Repo)?;

        let Some(current) = current else {
            return Ok(None);
        };

        let updated = self
            .repo
            .update_item_type(dto, account_id)
            .map_err(ItemTypeServiceError::Repo)?;

        if let Some(required_attributes) = &dto.required_attributes {
            let current_required: std::collections::HashSet<String> =
                current.required_attributes.into_iter().collect();
            let desired_required: std::collections::HashSet<String> =
                required_attributes.iter().cloned().collect();

            let to_add: Vec<String> = desired_required
                .difference(&current_required)
                .cloned()
                .collect();
            let to_remove: Vec<String> = current_required
                .difference(&desired_required)
                .cloned()
                .collect();

            let type_id =
                Id::try_from(dto.type_id).map_err(|e| ItemTypeServiceError::Repo(RepoError::DatabaseError {
                    err: e.to_string(),
                }))?;

            if !to_add.is_empty() {
                self.repo
                    .add_attr_kinds_to_item_type(&to_add, &type_id, account_id)
                    .map_err(ItemTypeServiceError::Repo)?;
            }

            if !to_remove.is_empty() {
                self.repo
                    .remove_attr_kinds_from_item_type(&to_remove, &type_id, account_id)
                    .map_err(ItemTypeServiceError::Repo)?;
            }

            return self.get_item_type(&type_id, account_id);
        }

        Ok(updated)
    }

    pub fn add_attr_kinds_to_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<(), ItemTypeServiceError> {
        self.repo
            .add_attr_kinds_to_item_type(attr_kind_keys, type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn remove_attr_kinds_from_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<(), ItemTypeServiceError> {
        self.repo
            .remove_attr_kinds_from_item_type(attr_kind_keys, type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }
}
