use std::fmt::Debug;

use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, UpdateItemTypeDto},
};

use crate::repos::common::RepoError;

pub trait ItemTypeRepo: Debug + Send + Sync {
    fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, RepoError>;
    fn get_item_type(
        &self,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn get_item_types_for_item(
        &self,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<Vec<ItemType>, RepoError>;
    fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn add_attr_kinds_to_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
    fn remove_attr_kinds_from_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
}
