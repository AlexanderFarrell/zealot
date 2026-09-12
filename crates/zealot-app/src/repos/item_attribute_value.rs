use std::{collections::HashMap, fmt::Debug};

use uuid::Uuid;

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
        limit: Option<i64>,
        offset: i64,
    ) -> Result<Vec<Id>, RepoError>;
    fn find_item_ids_by_filters_in_scopes(
        &self,
        filters: &Vec<AttributeFilter>,
        scope_ids: &[Uuid],
        limit: Option<i64>,
        offset: i64,
    ) -> Result<Vec<(Id, Id)>, RepoError> {
        let _ = (filters, scope_ids, limit, offset);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item filter lookup is not implemented by this repository"
                .to_string(),
        })
    }
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
