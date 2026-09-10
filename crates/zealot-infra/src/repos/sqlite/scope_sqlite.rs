use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, scope::{ScopeRepo, ServerMetadata}};

#[derive(Debug)]
pub struct ScopeSqliteRepo {
    pool: SqlitePool,
}

impl ScopeSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl ScopeRepo for ScopeSqliteRepo {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async move {
            sqlx::query_as::<_, (String, i64)>("SELECT server_id, schema_migration_version FROM server LIMIT 1")
                .fetch_one(&pool).await.map(|(server_id, schema_migration_version)| ServerMetadata { server_id, schema_migration_version }).map_err(RepoError::from)
        }))
    }
}
