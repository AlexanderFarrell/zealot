use zealot_domain::{account::Account, attribute::AttributeFilter, common::id::Id, item::{self, AddItemDto, Item, ItemError}};

use crate::repos::common::RepoError;



pub trait ItemRepo {
    // Getters TODO: Add paging
    
    fn get_item_by_id(item_id: &Id, account: &Account) -> Result<Option<Item>, RepoError<ItemError>>;
    fn get_items_by_title(title: &str, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn search_items_by_title(term: &str, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn regex_items_by_title(term: &str, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn get_items_by_type(type_name: &str, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn get_items_containing_attribute(attr_key: &str, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn get_items_by_attr_filter(filters: &Vec<AttributeFilter>, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;
    fn get_related_items(item_id: &Id, account: &Account) -> Result<Vec<Item>, RepoError<ItemError>>;

    fn add_item(dto: &AddItemDto, account: &Account) -> Result<Option<Item>, RepoError<ItemError>>;
    fn delete_item(item_id: &Id, account: &Account) -> Result<(), RepoError<ItemError>>;

    fn assign_item_types(type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<(), RepoError<ItemError>>;
    fn unassign_item_types(type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<(), RepoError<ItemError>>;
    fn is_item_valid_for_types(type_names: &Vec<String>, item_id: &Id, account: &Account) -> Result<bool, RepoError<ItemError>>;
}