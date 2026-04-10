use std::collections::HashMap;

use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item_type::ItemTypeRepo};
use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, ItemTypeRef, ItemTypeSummary, UpdateItemTypeDto},
};

#[derive(Debug)]
pub struct ItemTypePostgresRepo {
    pool: PgPool,
}

impl ItemTypePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemTypeRepo for ItemTypePostgresRepo {
    fn get_item_types(&self, _account_id: &Id) -> Result<Vec<ItemType>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_item_type_summaries(&self, _account_id: &Id) -> Result<Vec<ItemTypeSummary>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_item_type(
        &self,
        _item_type_id: &Id,
        _account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_item_type_by_name(
        &self,
        _name: &str,
        _account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_item_type_refs_for_items(
        &self,
        _item_ids: &Vec<Id>,
        _account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemTypeRef>>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_item_ids_for_type_name(
        &self,
        _name: &str,
        _account_id: &Id,
    ) -> Result<Vec<Id>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn add_item_type(
        &self,
        _dto: &AddItemTypeDto,
        _account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn update_item_type(
        &self,
        _dto: &UpdateItemTypeDto,
        _account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn delete_item_type(&self, _item_type_id: &Id, _account_id: &Id) -> Result<bool, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn add_attr_kinds_to_item_type(
        &self,
        _attr_kinds: &Vec<String>,
        _item_type_id: &Id,
        _account_id: &Id,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn remove_attr_kinds_from_item_type(
        &self,
        _attr_kinds: &Vec<String>,
        _item_type_id: &Id,
        _account_id: &Id,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn assign_item_types(
        &self,
        _type_names: &Vec<String>,
        _item_id: &Id,
        _account_id: &Id,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn unassign_item_types(
        &self,
        _type_names: &Vec<String>,
        _item_id: &Id,
        _account_id: &Id,
    ) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }
}
