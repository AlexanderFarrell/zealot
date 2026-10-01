use std::fmt::Debug;

use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
};

use crate::repos::common::RepoError;

pub trait ItemRepo: Debug + Send + Sync {
    fn get_item_by_id(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError>;
    fn get_items_by_ids(
        &self,
        item_ids: &Vec<Id>,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn get_items_by_title(
        &self,
        title: &str,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn search_items_by_title(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn search_items_by_content(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn search_items_by_heading(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<(ItemCore, String)>, RepoError>;
    fn regex_items_by_title(
        &self,
        term: &str,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn get_recent_items(
        &self,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<ItemCore>, RepoError>;
    fn get_all_item_ids_for_user(&self, account_id: &Id) -> Result<Vec<Id>, RepoError>;

    fn add_item(
        &self,
        dto: &AddItemCoreDto,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError>;
    fn update_item(
        &self,
        dto: &UpdateItemCoreDto,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError>;
    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError>;

    /// Set the item's content (and `title`, when given) only if its content
    /// still equals `expected_content`. Returns `false`, writing nothing, when
    /// the content has changed or the item is missing.
    ///
    /// This is the compare-and-swap that makes server-side edits safe against
    /// a concurrent write. Database backends override it with one conditional
    /// `UPDATE`; this default is a non-atomic fallback for test doubles.
    fn update_content_if_unchanged(
        &self,
        item_id: &Id,
        expected_content: &str,
        title: Option<&str>,
        content: &str,
        account: &Account,
    ) -> Result<bool, RepoError> {
        match self.get_item_by_id(item_id, account)? {
            Some(current) if current.content == expected_content => {
                let dto = UpdateItemCoreDto {
                    item_id: *item_id,
                    title: title.map(str::to_string),
                    content: Some(content.to_string()),
                };
                Ok(self.update_item(&dto, account)?.is_some())
            }
            _ => Ok(false),
        }
    }
}
