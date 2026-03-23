use sqlx::PgPool;
use zealot_app::repos::rule::RuleRepo;

#[derive(Debug)]
pub struct RulePostgresRepo {
    pool: PgPool,
}

impl RulePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RuleRepo for RulePostgresRepo {}
