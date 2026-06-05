use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use zealot_app::repos::{common::RepoError, rule::RuleRepo};
use zealot_domain::{
    common::id::Id,
    rule::{AddRuleDto, Rule, TriggerKind, UpdateRuleDto},
};

#[derive(Debug)]
pub struct RuleSqliteRepo {
    pool: SqlitePool,
}

impl RuleSqliteRepo {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[derive(sqlx::FromRow)]
struct RuleRow {
    rule_id: i64,
    account_id: i64,
    name: String,
    description: String,
    trigger_kind: String,
    trigger_config: String,
    script: String,
    enabled: bool,
    created_at: i64,
    last_run_at: Option<i64>,
    last_error: Option<String>,
    last_output: Option<String>,
}

fn row_to_rule(row: RuleRow) -> Result<Rule, RepoError> {
    let rule_id =
        Id::try_from(row.rule_id).map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let account_id = Id::try_from(row.account_id)
        .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
    let trigger: TriggerKind =
        serde_json::from_str(&row.trigger_config).map_err(|e| RepoError::DatabaseError {
            err: format!("invalid trigger_config: {}", e),
        })?;
    let created_at = NaiveDateTime::from_timestamp_opt(row.created_at, 0).ok_or_else(|| {
        RepoError::DatabaseError {
            err: format!("invalid created_at timestamp: {}", row.created_at),
        }
    })?;
    let last_run_at = row
        .last_run_at
        .map(|ts| {
            NaiveDateTime::from_timestamp_opt(ts, 0).ok_or_else(|| RepoError::DatabaseError {
                err: format!("invalid last_run_at timestamp: {}", ts),
            })
        })
        .transpose()?;
    Ok(Rule {
        rule_id,
        account_id,
        name: row.name,
        description: row.description,
        trigger,
        script: row.script,
        enabled: row.enabled,
        created_at,
        last_run_at,
        last_error: row.last_error,
        last_output: row.last_output,
    })
}

const SELECT_COLS: &str = "rule_id, account_id, name, description, trigger_kind, trigger_config, script, enabled, created_at, last_run_at, last_error, last_output";

impl RuleRepo for RuleSqliteRepo {
    fn get_all_rules(&self, account_id: &Id) -> Result<Vec<Rule>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE account_id = ? ORDER BY rule_id ASC"
                ))
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows.into_iter().map(row_to_rule).collect()
            })
        })
    }

    fn get_rule_by_id(&self, rule_id: &Id, account_id: &Id) -> Result<Option<Rule>, RepoError> {
        let rule_id_val = i64::from(*rule_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = ? AND account_id = ?"
                ))
                .bind(rule_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                row.map(row_to_rule).transpose()
            })
        })
    }

    fn get_enabled_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RepoError> {
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule
                     WHERE enabled = 1
                       AND trigger_kind IN ('cron', 'interval')
                     ORDER BY rule_id ASC"
                ))
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows.into_iter()
                    .map(|row| {
                        let account_id = Id::try_from(row.account_id)
                            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
                        let rule = row_to_rule(row)?;
                        Ok((rule, account_id))
                    })
                    .collect()
            })
        })
    }

    fn get_enabled_event_rules(
        &self,
        trigger_kind: &str,
        account_id: &Id,
    ) -> Result<Vec<Rule>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let trigger_kind = trigger_kind.to_owned();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule
                     WHERE enabled = 1 AND trigger_kind = ? AND account_id = ?
                     ORDER BY rule_id ASC"
                ))
                .bind(&trigger_kind)
                .bind(account_id_val)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows.into_iter().map(row_to_rule).collect()
            })
        })
    }

    fn add_rule(&self, dto: &AddRuleDto, account_id: &Id) -> Result<Rule, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = dto.name.clone();
        let description = dto.description.clone().unwrap_or_default();
        let trigger_kind = dto.trigger.kind_str().to_owned();
        let trigger_config = serde_json::to_string(&dto.trigger)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let script = dto.script.clone();
        let enabled = dto.enabled.unwrap_or(true);
        let created_at = chrono::Utc::now().timestamp();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, RuleRow>(&format!(
                    "INSERT INTO rule (account_id, name, description, trigger_kind, trigger_config, script, enabled, created_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?)
                     RETURNING {SELECT_COLS}"
                ))
                .bind(account_id_val)
                .bind(&name)
                .bind(&description)
                .bind(&trigger_kind)
                .bind(&trigger_config)
                .bind(&script)
                .bind(enabled)
                .bind(created_at)
                .fetch_one(&pool)
                .await
                .map_err(RepoError::from)?;
                row_to_rule(row)
            })
        })
    }

    fn update_rule(
        &self,
        rule_id: &Id,
        dto: &UpdateRuleDto,
        account_id: &Id,
    ) -> Result<Option<Rule>, RepoError> {
        let rule_id_val = i64::from(*rule_id);
        let account_id_val = i64::from(*account_id);
        let new_name = dto.name.clone();
        let new_description = dto.description.clone();
        let new_trigger_kind = dto.trigger.as_ref().map(|t| t.kind_str().to_owned());
        let new_trigger_config = dto
            .trigger
            .as_ref()
            .map(|t| serde_json::to_string(t))
            .transpose()
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let new_script = dto.script.clone();
        let new_enabled = dto.enabled;
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE rule
                     SET name           = COALESCE(?, name),
                         description    = COALESCE(?, description),
                         trigger_kind   = COALESCE(?, trigger_kind),
                         trigger_config = COALESCE(?, trigger_config),
                         script         = COALESCE(?, script),
                         enabled        = COALESCE(?, enabled)
                     WHERE rule_id = ? AND account_id = ?",
                )
                .bind(new_name)
                .bind(new_description)
                .bind(new_trigger_kind)
                .bind(new_trigger_config)
                .bind(new_script)
                .bind(new_enabled)
                .bind(rule_id_val)
                .bind(account_id_val)
                .execute(&pool)
                .await
                .map_err(RepoError::from)?;

                let row = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = ? AND account_id = ?"
                ))
                .bind(rule_id_val)
                .bind(account_id_val)
                .fetch_optional(&pool)
                .await
                .map_err(RepoError::from)?;
                row.map(row_to_rule).transpose()
            })
        })
    }

    fn delete_rule(&self, rule_id: &Id, account_id: &Id) -> Result<(), RepoError> {
        let rule_id_val = i64::from(*rule_id);
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query("DELETE FROM rule WHERE rule_id = ? AND account_id = ?")
                    .bind(rule_id_val)
                    .bind(account_id_val)
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn record_run(
        &self,
        rule_id: &Id,
        last_run_at: NaiveDateTime,
        error: Option<&str>,
        output: Option<&str>,
    ) -> Result<(), RepoError> {
        let rule_id_val = i64::from(*rule_id);
        let ts = last_run_at.and_utc().timestamp();
        let error = error.map(|s| s.to_owned());
        let output = output.map(|s| s.to_owned());
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE rule SET last_run_at = ?, last_error = ?, last_output = ? WHERE rule_id = ?",
                )
                .bind(ts)
                .bind(error)
                .bind(output)
                .bind(rule_id_val)
                .execute(&pool)
                .await
                .map(|_| ())
                .map_err(RepoError::from)
            })
        })
    }
}
