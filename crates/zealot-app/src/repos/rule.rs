use std::fmt::Debug;

use chrono::NaiveDateTime;
use uuid::Uuid;
use zealot_domain::{
    common::id::Id,
    rule::{AddRuleDto, Rule, UpdateRuleDto},
};

use crate::repos::common::RepoError;

pub trait RuleRepo: Debug + Send + Sync {
    fn get_all_rules_in_scopes(&self, scope_ids: &[Uuid]) -> Result<Vec<Rule>, RepoError>;
    fn get_rule_by_id_in_scopes(
        &self,
        rule_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<Rule>, RepoError>;

    /// Returns enabled event-triggered rules owned by one authorized scope.
    fn get_enabled_event_rules_in_scope(
        &self,
        trigger_kind: &str,
        scope_id: Uuid,
    ) -> Result<Vec<Rule>, RepoError>;

    fn add_rule_in_scope(
        &self,
        dto: &AddRuleDto,
        account_id: &Id,
        scope_id: Uuid,
    ) -> Result<Rule, RepoError>;
    fn update_rule_in_scopes(
        &self,
        rule_id: &Id,
        dto: &UpdateRuleDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<Rule>, RepoError>;
    fn delete_rule_in_scopes(&self, rule_id: &Id, scope_ids: &[Uuid]) -> Result<(), RepoError>;

    fn get_all_rules(&self, account_id: &Id) -> Result<Vec<Rule>, RepoError>;
    fn get_rule_by_id(&self, rule_id: &Id, account_id: &Id) -> Result<Option<Rule>, RepoError>;

    /// Returns all enabled scheduled rules across all accounts, paired with their account_id.
    fn get_enabled_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RepoError>;

    /// Returns enabled event-triggered rules matching the given trigger_kind for an account.
    fn get_enabled_event_rules(
        &self,
        trigger_kind: &str,
        account_id: &Id,
    ) -> Result<Vec<Rule>, RepoError>;

    fn add_rule(&self, dto: &AddRuleDto, account_id: &Id) -> Result<Rule, RepoError>;
    fn update_rule(
        &self,
        rule_id: &Id,
        dto: &UpdateRuleDto,
        account_id: &Id,
    ) -> Result<Option<Rule>, RepoError>;
    fn delete_rule(&self, rule_id: &Id, account_id: &Id) -> Result<(), RepoError>;

    /// Persist timing and error state after a rule execution.
    fn record_run(
        &self,
        rule_id: &Id,
        last_run_at: NaiveDateTime,
        error: Option<&str>,
        output: Option<&str>,
    ) -> Result<(), RepoError>;
}
