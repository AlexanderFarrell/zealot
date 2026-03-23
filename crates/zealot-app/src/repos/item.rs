use std::fmt::Debug;

use zealot_domain::{account::Account, attribute::AttributeFilter, common::id::Id, item::{AddItemDto, Item}};

use crate::repos::common::RepoError;

pub trait ItemRepo: Debug + Send + Sync {
    // Getters TODO: Add paging
    
    fn get_item_by_id(&self, item_id: &Id, account: &Account) -> Result<Option<Item>, RepoError>;
    fn get_items_by_title(&self, title: &str, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn search_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn regex_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn get_items_by_type(&self, type_name: &str, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn get_items_containing_attribute(&self, attr_key: &str, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn get_items_by_attr_filter(&self, filters: &Vec<AttributeFilter>, account: &Account) -> Result<Vec<Item>, RepoError>;
    fn get_related_items(&self, item_id: &Id, account: &Account) -> Result<Vec<Item>, RepoError>;

    fn add_item(&self, dto: &AddItemDto, account: &Account) -> Result<Option<Item>, RepoError>;
    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError>;

    fn assign_item_types(&self, type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<(), RepoError>;
    fn unassign_item_types(&self, type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<(), RepoError>;
    fn is_item_valid_for_types(&self, type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<bool, RepoError>;
}
