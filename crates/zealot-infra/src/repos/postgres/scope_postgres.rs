use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, scope::{ScopeRepo, ServerMetadata}};

#[derive(Debug)]
pub struct ScopePostgresRepo {
    pool: PgPool,
}

impl ScopePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ScopeRepo for ScopePostgresRepo {
    fn server_metadata(&self) -> Result<ServerMetadata, RepoError> {
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(async move {
            sqlx::query_as::<_, (uuid::Uuid, i64)>("SELECT server_id, schema_migration_version FROM server LIMIT 1")
                .fetch_one(&pool).await.map(|(server_id, schema_migration_version)| ServerMetadata { server_id: server_id.to_string(), schema_migration_version }).map_err(RepoError::from)
        }))
    }
}
