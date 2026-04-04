use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, item::ItemRepo};
use zealot_domain::{
    account::Account,
    common::id::Id,
    item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
};

#[derive(Debug)]
pub struct ItemPostgresRepo {
    pool: PgPool,
}

impl ItemPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ItemRepo for ItemPostgresRepo {
    fn get_item_by_id(&self, _item_id: &Id, _account: &Account) -> Result<Option<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_items_by_ids(&self, _item_ids: &Vec<Id>, _account: &Account) -> Result<Vec<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn get_items_by_title(&self, _title: &str, _account: &Account) -> Result<Vec<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn search_items_by_title(&self, _term: &str, _account: &Account) -> Result<Vec<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn regex_items_by_title(&self, _term: &str, _account: &Account) -> Result<Vec<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn add_item(&self, _dto: &AddItemCoreDto, _account: &Account) -> Result<Option<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn update_item(&self, _dto: &UpdateItemCoreDto, _account: &Account) -> Result<Option<ItemCore>, RepoError> {
        let _ = &self.pool;
        todo!()
    }

    fn delete_item(&self, _item_id: &Id, _account: &Account) -> Result<(), RepoError> {
        let _ = &self.pool;
        todo!()
    }
}
