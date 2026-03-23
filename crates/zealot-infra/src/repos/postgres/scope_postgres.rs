use sqlx::PgPool;
use zealot_app::repos::scope::ScopeRepo;

#[derive(Debug)]
pub struct ScopePostgresRepo {
    pool: PgPool,
}

impl ScopePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl ScopeRepo for ScopePostgresRepo {}
