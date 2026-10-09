use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{account::Account, common::id::Id, item::ItemLink};

use crate::repos::common::RepoError;

pub trait ItemLinkRepo: Debug + Send + Sync {
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

    /// Replaces the item's `wikilink` links with `linked_ids` and records
    /// `titles` as the `[[...]]` references its content makes, in one
    /// transaction. `titles` are the extracted targets, resolved or not.
    fn replace_wikilinks(
        &self,
        item_id: &Id,
        titles: &[String],
        linked_ids: &[Id],
        account: &Account,
    ) -> Result<(), RepoError>;

    /// Ids of the account's items whose recorded references include any of
    /// `titles`.
    fn get_wikilink_source_ids(
        &self,
        titles: &[String],
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError>;

    /// Whether recorded references cover every item of the account, so a
    /// rename may rely on them.
    fn wikilink_refs_complete(&self, account_id: &Id) -> Result<bool, RepoError>;

    fn mark_wikilink_refs_complete(&self, account_id: &Id) -> Result<(), RepoError>;
}
