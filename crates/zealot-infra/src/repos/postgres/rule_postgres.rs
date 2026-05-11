use chrono::NaiveDateTime;
use sqlx::PgPool;
use zealot_app::repos::{common::RepoError, rule::RuleRepo};
use zealot_domain::{
    common::id::Id,
    rule::{AddRuleDto, Rule, UpdateRuleDto},
};

#[derive(Debug)]
pub struct RulePostgresRepo {
    pool: PgPool,
}

impl RulePostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl RuleRepo for RulePostgresRepo {
    fn get_all_rules(&self, _account_id: &Id) -> Result<Vec<Rule>, RepoError> {
        todo!()
    }

    fn get_rule_by_id(&self, _rule_id: &Id, _account_id: &Id) -> Result<Option<Rule>, RepoError> {
        todo!()
    }

    fn get_enabled_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RepoError> {
        todo!()
    }

    fn get_enabled_event_rules(&self, _trigger_kind: &str, _account_id: &Id) -> Result<Vec<Rule>, RepoError> {
        todo!()
    }

    fn add_rule(&self, _dto: &AddRuleDto, _account_id: &Id) -> Result<Rule, RepoError> {
        todo!()
    }

    fn update_rule(&self, _rule_id: &Id, _dto: &UpdateRuleDto, _account_id: &Id) -> Result<Option<Rule>, RepoError> {
        todo!()
    }

    fn delete_rule(&self, _rule_id: &Id, _account_id: &Id) -> Result<(), RepoError> {
        todo!()
    }

    fn record_run(
        &self,
        _rule_id: &Id,
        _last_run_at: NaiveDateTime,
        _error: Option<&str>,
        _output: Option<&str>,
    ) -> Result<(), RepoError> {
        todo!()
    }
}
