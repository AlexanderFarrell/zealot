use std::{collections::HashMap, fmt::Debug};

use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeFilter},
    common::id::Id,
};

use crate::repos::common::RepoError;

pub trait ItemAttributeValueRepo: Debug + Send + Sync {
    fn get_attributes_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, HashMap<String, Attribute>>, RepoError>;
    fn find_item_ids_by_filters(
        &self,
        filters: &Vec<AttributeFilter>,
        account_id: &Id,
    ) -> Result<Vec<Id>, RepoError>;
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
