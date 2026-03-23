use sqlx::SqlitePool;
use zealot_app::repos::meta::MetaRepo;


#[derive(Debug)]
pub struct MetaSqliteRepo {
    pool: SqlitePool,
}

impl MetaSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self {pool}
    }
}

impl MetaRepo for MetaSqliteRepo {
    fn backup(&self) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn download(&self, target: zealot_app::repos::meta::MetaDownloadTarget) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn upload(&self) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}