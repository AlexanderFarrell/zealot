use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeFilter},
    common::id::Id,
};

use crate::repos::common::RepoError;

/// Sort applied to an item query.
#[derive(Debug, Clone)]
pub struct ItemSort {
    /// `title`, `created`, `updated`, or a scalar attribute key
    pub key: String,
    pub descending: bool,
}

/// Item query combining types, subtree, attribute filters, sort, and paging.
#[derive(Debug, Clone)]
pub struct ItemQuery {
    pub types: Vec<String>,
    pub include_archived: bool,
    /// Require every type in `types` instead of any
    pub all_types: bool,
    pub under: Option<Id>,
    pub filters: Vec<AttributeFilter>,
    pub sort: Option<ItemSort>,
    pub limit: i64,
    pub offset: i64,
}

pub trait ItemAttributeValueRepo: Debug + Send + Sync {
    /// Atomically toggle only the archive attribute on a root and optionally its descendants.
    /// Returns all affected ids; an inaccessible root yields no ids. Traversal is account bounded.
    fn set_archived(
        &self,
        _root: &Id,
        _archived: bool,
        _subtree: bool,
        _account: &Account,
    ) -> Result<Vec<Id>, RepoError> {
        Err(RepoError::DatabaseError {
            err: "archive mutation is unsupported by this repository".into(),
        })
    }
    fn get_attributes_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, HashMap<String, Attribute>>, RepoError>;
    fn find_item_ids_by_filters(
        &self,
        filters: &Vec<AttributeFilter>,
        account_id: &Id,
        limit: Option<i64>,
        offset: i64,
    ) -> Result<Vec<Id>, RepoError>;
    /// Matching item ids for one page (sorted per the query) and the total match count.
    fn query_item_ids(
        &self,
        query: &ItemQuery,
        account_id: &Id,
    ) -> Result<(Vec<Id>, i64), RepoError>;
    fn replace_item_attributes(
        &self,
        item_id: &Id,
        attributes: &HashMap<String, Attribute>,
        account: &Account,
    ) -> Result<(), RepoError>;
    fn rename_item_attribute(
        &self,
        item_id: &Id,
        old_key: &str,
        new_key: &str,
        account: &Account,
    ) -> Result<(), RepoError>;
    fn delete_item_attribute(
        &self,
        item_id: &Id,
        key: &str,
        account: &Account,
    ) -> Result<(), RepoError>;
}
