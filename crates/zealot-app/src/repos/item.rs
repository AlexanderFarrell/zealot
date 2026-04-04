use std::fmt::Debug;

use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
};

use crate::repos::common::RepoError;

pub trait ItemRepo: Debug + Send + Sync {
    // Getters TODO: Add paging

    fn get_item_by_id(&self, item_id: &Id, account: &Account) -> Result<Option<ItemCore>, RepoError>;
    fn get_items_by_ids(&self, item_ids: &Vec<Id>, account: &Account) -> Result<Vec<ItemCore>, RepoError>;
    fn get_items_by_title(&self, title: &str, account: &Account) -> Result<Vec<ItemCore>, RepoError>;
    fn search_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<ItemCore>, RepoError>;
    fn regex_items_by_title(&self, term: &str, account: &Account) -> Result<Vec<ItemCore>, RepoError>;

    fn add_item(&self, dto: &AddItemCoreDto, account: &Account) -> Result<Option<ItemCore>, RepoError>;
    fn update_item(&self, dto: &UpdateItemCoreDto, account: &Account) -> Result<Option<ItemCore>, RepoError>;
    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError>;
}
