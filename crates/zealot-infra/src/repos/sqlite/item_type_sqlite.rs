use sqlx::SqlitePool;
use zealot_app::repos::item_type::ItemTypeRepo;


#[derive(Debug)]
pub struct ItemTypeSqliteRepo {
    pool: SqlitePool,
}

impl ItemTypeSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ItemTypeRepo for ItemTypeSqliteRepo {
    fn get_item_types(&self, account_id: &zealot_domain::common::id::Id) -> Result<Vec<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_item_type(
        &self,
        item_type_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_item_types_for_item(
        &self,
        item_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Vec<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn add_item_type(
        &self,
        dto: &zealot_domain::item_type::AddItemTypeDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn update_item_type(
        &self,
        dto: &zealot_domain::item_type::UpdateItemTypeDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::item_type::ItemType>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn add_attr_kinds_to_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn remove_attr_kinds_from_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
