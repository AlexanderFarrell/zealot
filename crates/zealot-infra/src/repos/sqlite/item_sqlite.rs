use sqlx::SqlitePool;
use zealot_app::repos::item::ItemRepo;


#[derive(Debug)]
pub struct ItemSqliteRepo {
    pool: SqlitePool,
}

impl ItemSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self {pool}
    }
}

impl ItemRepo for ItemSqliteRepo {
    fn get_item_by_id(&self, item_id: &zealot_domain::common::id::Id, account: &zealot_domain::account::Account) -> Result<Option<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_items_by_title(&self, title: &str, account: &zealot_domain::account::Account) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn search_items_by_title(&self, term: &str, account: &zealot_domain::account::Account) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn regex_items_by_title(&self, term: &str, account: &zealot_domain::account::Account) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_items_by_type(&self, type_name: &str, account: &zealot_domain::account::Account)
    -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_items_containing_attribute(
        &self,
        attr_key: &str,
        account: &zealot_domain::account::Account,
    ) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_items_by_attr_filter(
        &self,
        filters: &Vec<zealot_domain::attribute::AttributeFilter>,
        account: &zealot_domain::account::Account,
    ) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_related_items(&self, item_id: &zealot_domain::common::id::Id, account: &zealot_domain::account::Account) -> Result<Vec<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn add_item(&self, dto: &zealot_domain::item::AddItemDto, account: &zealot_domain::account::Account) -> Result<Option<zealot_domain::item::Item>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn delete_item(&self, item_id: &zealot_domain::common::id::Id, account: &zealot_domain::account::Account) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn assign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &zealot_domain::common::id::Id,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn unassign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &zealot_domain::common::id::Id,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn is_item_valid_for_types(
        &self,
        type_names: &Vec<String>,
        item_id: &zealot_domain::common::id::Id,
        account: &zealot_domain::account::Account,
    ) -> Result<bool, zealot_app::repos::common::RepoError> {
        todo!()
    }
}