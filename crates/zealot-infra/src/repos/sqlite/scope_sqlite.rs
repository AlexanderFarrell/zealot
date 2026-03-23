use sqlx::SqlitePool;
use zealot_app::repos::scope::ScopeRepo;

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
    
}