use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{account::Account, common::id::Id, item::ItemLink};
use uuid::Uuid;

use crate::repos::common::RepoError;

pub trait ItemLinkRepo: Debug + Send + Sync {
    fn get_links_for_items_in_scopes(
        &self,
        item_ids: &Vec<Id>,
        scope_ids: &[Uuid],
    ) -> Result<HashMap<Id, Vec<ItemLink>>, RepoError> {
        let _ = (item_ids, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified link lookup is not implemented by this repository".to_string(),
        })
    }
    fn get_links_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemLink>>, RepoError>;

    /// Returns all item IDs that have a link pointing to `target_item_id` with
    /// the given `relationship` label.
    fn get_source_item_ids(
        &self,
        target_item_id: &Id,
        relationship: &str,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError>;

    fn get_related_item_ids(&self, item_id: &Id, account_id: &Id) -> Result<Vec<Id>, RepoError>;

    /// Replaces **all** outgoing links from `item_id` with the given set.
    fn replace_links_for_item(
        &self,
        item_id: &Id,
        links: &Vec<ItemLink>,
        account: &Account,
    ) -> Result<(), RepoError>;

    /// Replaces all outgoing links from `item_id` that have the given
    /// `relationship` label with a new set pointing to `linked_ids`.
    /// Links with other relationship labels are left untouched.
    fn replace_links_by_relationship(
        &self,
        item_id: &Id,
        relationship: &str,
        linked_ids: &[Id],
        account: &Account,
    ) -> Result<(), RepoError>;
}
