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
        _day: &sqlx::types::chrono::NaiveDate,
        _account: &zealot_domain::account::Account,
    ) -> Result<Vec<zealot_domain::repeat::RepeatEntryCore>, zealot_app::repos::common::RepoError>
    {
        todo!()
    }

    fn set_status(
        &self,
        _dto: &zealot_domain::repeat::UpdateRepeatEntryDto,
        _account: &zealot_domain::account::Account,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
