use zealot_domain::{common::id::Id, item_type::{AddItemTypeDto, ItemType, ItemTypeError, UpdateItemTypeDto}};

use crate::repos::common::RepoError;

pub trait ItemTypeRepo {
    fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, RepoError<ItemTypeError>>;
    fn get_item_type(&self, item_type_id: &Id, account_id: &Id) -> Result<Option<ItemType>, RepoError<ItemTypeError>>;
    fn get_item_type_by_name(&self, name: &str, account_id: &Id) -> Result<Option<ItemType>, RepoError<ItemTypeError>>;
    fn get_item_types_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<ItemType>, RepoError<ItemTypeError>>;
    fn add_item_type(&self, dto: &AddItemTypeDto, account_id: &Id) -> Result<Option<ItemType>, RepoError<ItemTypeError>>;
    fn update_item_type(&self, dto: &UpdateItemTypeDto, account_id: &Id) -> Result<Option<ItemType>, RepoError<ItemTypeError>>;
    fn add_attr_kinds_to_item_type(&self, attr_kinds: &Vec<String>, item_type_id: &Id, account_id: &Id) -> Result<(), RepoError<ItemTypeError>>;
    fn remove_attr_kinds_from_item_type(&self, attr_kinds: &Vec<String>, item_type_id: &Id, account_id: &Id) -> Result<(), RepoError<ItemTypeError>>;
}
