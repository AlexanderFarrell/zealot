use sqlx::PgPool;
use zealot_app::repos::account::AccountRepo;

#[derive(Debug)]
pub struct AccountPostgresRepo {
    pool: PgPool,
}

impl AccountPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl AccountRepo for AccountPostgresRepo {
    fn get_hash_by_username(
        &self,
        username: &str,
    ) -> Result<Option<String>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_account_by_id(
        &self,
        id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::account::Account>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_account_by_title(
        &self,
        title: &str,
    ) -> Result<Option<zealot_domain::account::Account>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn add_account(
        &self,
        account: &zealot_domain::account::InsertAccountDto,
    ) -> Result<Option<zealot_domain::account::Account>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn delete_account(
        &self,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn upsert_api_key(
        &self,
        account_id: &zealot_domain::common::id::Id,
        key: &str,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn delete_api_key(
        &self,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
