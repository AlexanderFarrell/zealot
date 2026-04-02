use std::collections::HashMap;

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item::ItemRepo};
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeFilter},
    common::id::Id,
    item::{AddItemParsedDto, Item, UpdateItemParsedDto},
};

#[derive(Debug)]
pub struct ItemPostgresRepo {
    pool: PgPool,
}

impl ItemPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemRepo for ItemPostgresRepo {
    fn get_item_by_id(&self, _item_id: &Id, _account: &Account) -> Result<Option<Item>, RepoError> { todo!() }
    fn get_items_by_title(&self, _title: &str, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn search_items_by_title(&self, _term: &str, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn regex_items_by_title(&self, _term: &str, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn get_items_by_type(&self, _type_name: &str, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn get_items_containing_attribute(&self, _attr_key: &str, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn get_items_by_attr_filter(&self, _filters: &Vec<AttributeFilter>, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn get_related_items(&self, _item_id: &Id, _account: &Account) -> Result<Vec<Item>, RepoError> { todo!() }
    fn add_item(&self, _dto: &AddItemParsedDto, _account: &Account) -> Result<Option<Item>, RepoError> { todo!() }
    fn update_item(&self, _dto: &UpdateItemParsedDto, _account: &Account) -> Result<Option<Item>, RepoError> { todo!() }
    fn delete_item(&self, _item_id: &Id, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn set_item_attributes(&self, _item_id: &Id, _attributes: &HashMap<String, Attribute>, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn rename_item_attribute(&self, _item_id: &Id, _old_key: &str, _new_key: &str, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn delete_item_attribute(&self, _item_id: &Id, _key: &str, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn assign_item_types(&self, _type_names: &Vec<String>, _item_id: &Id, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn unassign_item_types(&self, _type_names: &Vec<String>, _item_id: &Id, _account: &Account) -> Result<(), RepoError> { todo!() }
    fn is_item_valid_for_types(&self, _type_names: &Vec<String>, _item_id: &Id, _account: &Account) -> Result<bool, RepoError> { todo!() }
}
