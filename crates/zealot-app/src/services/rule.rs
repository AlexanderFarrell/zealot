use std::sync::Arc;

use zealot_domain::{
    account::Account,
    common::id::Id,
    rule::{AddRuleDto, Rule, UpdateRuleDto},
};

use crate::{
    ports::rule_runner::{RuleContext, RuleRunResult, RuleRunnerPort},
    repos::{common::RepoError, rule::RuleRepo},
};

#[derive(Debug, Clone)]
pub struct RuleService {
    rule_repo: Arc<dyn RuleRepo>,
    rule_runner: Arc<dyn RuleRunnerPort>,
}

#[derive(Debug, thiserror::Error)]
pub enum RuleServiceError {
    #[error("not found")]
    NotFound,
    #[error("unauthorized")]
    Unauthorized,
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl RuleService {
    pub fn new(rule_repo: &Arc<dyn RuleRepo>, rule_runner: &Arc<dyn RuleRunnerPort>) -> Self {
        Self {
            rule_repo: rule_repo.clone(),
            rule_runner: rule_runner.clone(),
        }
    }

    pub fn get_rules(&self, account: &Account) -> Result<Vec<Rule>, RuleServiceError> {
        Ok(self.rule_repo.get_all_rules(&account.account_id)?)
    }

    pub fn get_rule(&self, rule_id: &Id, account: &Account) -> Result<Rule, RuleServiceError> {
        self.rule_repo
            .get_rule_by_id(rule_id, &account.account_id)?
            .ok_or(RuleServiceError::NotFound)
    }

    pub fn add_rule(&self, dto: AddRuleDto, account: &Account) -> Result<Rule, RuleServiceError> {
        let rule = self.rule_repo.add_rule(&dto, &account.account_id)?;
        tracing::info!(account_id = ?account.account_id, rule_id = ?rule.rule_id, rule_name = %rule.name, "rule created");
        Ok(rule)
    }

    pub fn update_rule(
        &self,
        rule_id: &Id,
        dto: UpdateRuleDto,
        account: &Account,
    ) -> Result<Rule, RuleServiceError> {
        let rule = self
            .rule_repo
            .update_rule(rule_id, &dto, &account.account_id)?
            .ok_or(RuleServiceError::NotFound)?;
        tracing::info!(account_id = ?account.account_id, rule_id = ?rule.rule_id, rule_name = %rule.name, "rule updated");
        Ok(rule)
    }

    pub fn delete_rule(&self, rule_id: &Id, account: &Account) -> Result<(), RuleServiceError> {
        self.rule_repo.delete_rule(rule_id, &account.account_id)?;
        tracing::info!(account_id = ?account.account_id, ?rule_id, "rule deleted");
        Ok(())
    }

    pub fn get_all_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RuleServiceError> {
        Ok(self.rule_repo.get_enabled_scheduled_rules()?)
    }

    /// Run a rule immediately regardless of its trigger type.
    pub async fn run_rule_now(
        &self,
        rule_id: &Id,
        account: &Account,
    ) -> Result<RuleRunResult, RuleServiceError> {
        let rule = self.get_rule(rule_id, account)?;
        tracing::info!(account_id = ?account.account_id, ?rule_id, rule_name = %rule.name, "running rule manually");
        let result = self.rule_runner.run_rule(&rule, RuleContext::Manual).await;
        if result.success {
            tracing::info!(account_id = ?account.account_id, ?rule_id, duration_ms = result.duration_ms, "manual rule completed");
        } else {
            tracing::error!(account_id = ?account.account_id, ?rule_id, error = ?result.error, duration_ms = result.duration_ms, "manual rule failed");
        }
        Ok(result)
    }
}
