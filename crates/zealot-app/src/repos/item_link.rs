use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{ItemLink, ItemRelationship},
};

use crate::repos::common::RepoError;

pub trait ItemLinkRepo: Debug + Send + Sync {
    fn get_links_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemLink>>, RepoError>;
    fn get_source_item_ids(
        &self,
        target_item_id: &Id,
        relationship: ItemRelationship,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError>;
    fn get_related_item_ids(&self, item_id: &Id, account_id: &Id) -> Result<Vec<Id>, RepoError>;
    fn replace_links_for_item(
        &self,
        item_id: &Id,
        links: &Vec<ItemLink>,
        account: &Account,
    ) -> Result<(), RepoError>;
}
