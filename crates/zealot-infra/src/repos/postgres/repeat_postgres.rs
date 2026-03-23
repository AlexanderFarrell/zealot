use sqlx::PgPool;
use zealot_app::repos::repeat::RepeatRepo;

#[derive(Debug)]
pub struct RepeatPostgresRepo {
    pool: PgPool,
}

impl RepeatPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RepeatRepo for RepeatPostgresRepo {
    fn get_for_day(
        &self,
        day: &sqlx::types::chrono::NaiveDate,
        account: &zealot_domain::account::Account,
    ) -> Result<Vec<zealot_domain::repeat::RepeatEntry>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn set_status(
        &self,
        dto: &zealot_domain::repeat::RepeatEntryDto,
        account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
