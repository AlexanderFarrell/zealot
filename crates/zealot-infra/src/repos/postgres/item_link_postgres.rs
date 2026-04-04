use std::collections::HashMap;

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item_link::ItemLinkRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{ItemLink, ItemRelationship},
};

#[derive(Debug)]
pub struct ItemLinkPostgresRepo {
    pool: PgPool,
}

impl ItemLinkPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemLinkRepo for ItemLinkPostgresRepo {
    fn get_links_for_items(
        &self,
        _item_ids: &Vec<Id>,
        _account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemLink>>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_source_item_ids(
        &self,
        _target_item_id: &Id,
        _relationship: ItemRelationship,
        _account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_related_item_ids(&self, _item_id: &Id, _account_id: &Id) -> Result<Vec<Id>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn replace_links_for_item(
        &self,
        _item_id: &Id,
        _links: &Vec<ItemLink>,
        _account: &Account,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }
}
