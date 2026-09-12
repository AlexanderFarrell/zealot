use std::fmt::Debug;

use uuid::Uuid;
use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
};

use crate::repos::common::RepoError;

pub trait ItemRepo: Debug + Send + Sync {
    fn get_items_by_title_in_scopes(
        &self,
        title: &str,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(ItemCore, Id)>, RepoError> {
        let _ = (title, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item title lookup is not implemented by this repository"
                .to_string(),
        })
    }
    fn search_items_by_title_in_scopes(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(ItemCore, Id)>, RepoError> {
        let _ = (term, limit, offset, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item title search is not implemented by this repository"
                .to_string(),
        })
    }
    fn search_items_by_content_in_scopes(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(ItemCore, Id)>, RepoError> {
        let _ = (term, limit, offset, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item content search is not implemented by this repository"
                .to_string(),
        })
    }
    fn search_items_by_heading_in_scopes(
        &self,
        term: &str,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(ItemCore, String, Id)>, RepoError> {
        let _ = (term, limit, offset, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item heading search is not implemented by this repository"
                .to_string(),
        })
    }
    fn get_recent_items_in_scopes(
        &self,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(ItemCore, Id)>, RepoError> {
        let _ = (limit, offset, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified recent item lookup is not implemented by this repository"
                .to_string(),
        })
    }
    fn get_all_item_ids_in_scopes(&self, scope_ids: &[Uuid]) -> Result<Vec<Id>, RepoError> {
        let _ = scope_ids;
        Err(RepoError::DatabaseError {
            err: "scope-qualified item ID lookup is not implemented by this repository".to_string(),
        })
    }
    fn get_item_by_id_in_scopes(
        &self,
        item_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<(ItemCore, Id)>, RepoError> {
        let _ = (item_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item lookup is not implemented by this repository".to_string(),
        })
    }
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
    fn add_item_in_scope(
        &self,
        dto: &AddItemCoreDto,
        account: &Account,
        scope_id: Uuid,
    ) -> Result<Option<ItemCore>, RepoError> {
        let _ = (dto, account, scope_id);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item creation is not implemented by this repository".to_string(),
        })
    }
    fn update_item(
        &self,
        dto: &UpdateItemCoreDto,
        account: &Account,
    ) -> Result<Option<ItemCore>, RepoError>;
    fn delete_item(&self, item_id: &Id, account: &Account) -> Result<(), RepoError>;
}
