use sqlx::SqlitePool;
use zealot_app::repos::repeat::RepeatRepo;

#[derive(Debug)]
pub struct RepeatSqliteRepo {
    pool: SqlitePool,
}

impl RepeatSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl RepeatRepo for RepeatSqliteRepo {
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
