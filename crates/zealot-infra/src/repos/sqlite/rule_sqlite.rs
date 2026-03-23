use sqlx::SqlitePool;
use zealot_app::repos::rule::RuleRepo;

#[derive(Debug)]
pub struct RuleSqliteRepo {
    pool: SqlitePool
}

impl RuleSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

impl RuleRepo for RuleSqliteRepo {
    
}