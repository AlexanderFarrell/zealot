# TASK-LUA-006: `RuleService` CRUD and `run_rule_now`

## Context

`crates/zealot-app/src/services/rule.rs` is currently empty. The service wraps `RuleRepo` and `RuleRunnerPort`, providing the business logic layer for the HTTP handlers.

## Goal

Implement `RuleService` following the pattern of `ItemService` and `CommentService`.

## Requirements

### `crates/zealot-app/src/services/rule.rs`

```rust
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
    rule_repo:    Arc<dyn RuleRepo>,
    rule_runner:  Arc<dyn RuleRunnerPort>,
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
        Ok(self.rule_repo.add_rule(&dto, &account.account_id)?)
    }

    pub fn update_rule(&self, rule_id: &Id, dto: UpdateRuleDto, account: &Account) -> Result<Rule, RuleServiceError> {
        self.rule_repo
            .update_rule(rule_id, &dto, &account.account_id)?
            .ok_or(RuleServiceError::NotFound)
    }

    pub fn delete_rule(&self, rule_id: &Id, account: &Account) -> Result<(), RuleServiceError> {
        self.rule_repo.delete_rule(rule_id, &account.account_id)?;
        Ok(())
    }

    /// Run a rule immediately regardless of its trigger type.
    pub async fn run_rule_now(&self, rule_id: &Id, account: &Account) -> Result<RuleRunResult, RuleServiceError> {
        let rule = self.get_rule(rule_id, account)?;
        let result = self.rule_runner.run_rule(&rule, RuleContext::Manual).await;
        Ok(result)
    }
}
```

### Wire into `ZealotServices`

In `crates/zealot-app/src/services/mod.rs`:
- Add `rule: RuleService` field to `ZealotServices`
- Instantiate in `ZealotServices::new(ports, repos)` using `RuleService::new(&repos.rule, &ports.rule_runner)`

## Dependencies

- TASK-LUA-004
- TASK-LUA-005

## Files to modify

- `crates/zealot-app/src/services/rule.rs`
- `crates/zealot-app/src/services/mod.rs`

## Verification

```bash
cargo check -p zealot-app
```
