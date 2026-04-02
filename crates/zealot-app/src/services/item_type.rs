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
        self.repo.update_item_type(dto, account_id).map_err(ItemTypeServiceError::Repo)
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
