use std::collections::HashMap;

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item_attribute_value::ItemAttributeValueRepo};
use zealot_domain::{
    account::Account,
    attribute::{Attribute, AttributeFilter},
    common::id::Id,
};

#[derive(Debug)]
pub struct ItemAttributeValuePostgresRepo {
    pool: PgPool,
}

impl ItemAttributeValuePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemAttributeValueRepo for ItemAttributeValuePostgresRepo {
    fn get_attributes_for_items(
        &self,
        _item_ids: &Vec<Id>,
        _account_id: &Id,
    ) -> Result<HashMap<Id, HashMap<String, Attribute>>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn find_item_ids_by_filters(
        &self,
        _filters: &Vec<AttributeFilter>,
        _account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn replace_item_attributes(
        &self,
        _item_id: &Id,
        _attributes: &HashMap<String, Attribute>,
        _account: &Account,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn rename_item_attribute(
        &self,
        _item_id: &Id,
        _old_key: &str,
        _new_key: &str,
        _account: &Account,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn delete_item_attribute(
        &self,
        _item_id: &Id,
        _key: &str,
        _account: &Account,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }
}
