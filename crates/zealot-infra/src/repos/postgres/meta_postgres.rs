use sqlx::PgPool;
use zealot_app::repos::meta::MetaRepo;

#[derive(Debug)]
pub struct MetaPostgresRepo {
    pool: PgPool,
}

impl MetaPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl MetaRepo for MetaPostgresRepo {
    fn backup(&self) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn download(
        &self,
        target: zealot_app::repos::meta::MetaDownloadTarget,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn upload(&self) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
