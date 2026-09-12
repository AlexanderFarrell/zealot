use chrono::{DateTime, NaiveDateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;
use zealot_app::repos::{common::RepoError, rule::RuleRepo};
use zealot_domain::{
    common::id::Id,
    rule::{AddRuleDto, Rule, TriggerKind, UpdateRuleDto},
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

#[derive(sqlx::FromRow)]
struct RuleRow {
    rule_id: i64,
    account_id: i64,
    scope_id: Uuid,
    name: String,
    description: String,
    trigger_kind: String,
    trigger_config: String,
    script: String,
    enabled: bool,
    created_at: DateTime<Utc>,
    last_run_at: Option<DateTime<Utc>>,
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
    Ok(Rule {
        rule_id,
        account_id,
        scope_id: row.scope_id,
        name: row.name,
        description: row.description,
        trigger,
        script: row.script,
        enabled: row.enabled,
        created_at: row.created_at.naive_utc(),
        last_run_at: row.last_run_at.map(|dt| dt.naive_utc()),
        last_error: row.last_error,
        last_output: row.last_output,
    })
}

const SELECT_COLS: &str = "rule_id, account_id, scope_id, name, description, trigger_kind, trigger_config, script, enabled, created_at, last_run_at, last_error, last_output";

impl RuleRepo for RulePostgresRepo {
    fn get_all_rules_in_scopes(&self, scope_ids: &[Uuid]) -> Result<Vec<Rule>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = (1..=scope_ids.len())
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let pool = self.pool.clone();
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE scope_id IN ({placeholders}) ORDER BY rule_id ASC"
                );
                let mut query = sqlx::query_as::<_, RuleRow>(&sql);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .fetch_all(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .into_iter()
                    .map(row_to_rule)
                    .collect()
            })
        })
    }

    fn get_rule_by_id_in_scopes(
        &self,
        rule_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<Rule>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        let placeholders = (2..=scope_ids.len() + 1)
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let pool = self.pool.clone();
        let rule_id_val = i64::from(*rule_id);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = $1 AND scope_id IN ({placeholders})"
                );
                let mut query = sqlx::query_as::<_, RuleRow>(&sql).bind(rule_id_val);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .fetch_optional(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .map(row_to_rule)
                    .transpose()
            })
        })
    }

    fn get_enabled_event_rules_in_scope(
        &self,
        trigger_kind: &str,
        scope_id: Uuid,
    ) -> Result<Vec<Rule>, RepoError> {
        let pool = self.pool.clone();
        let trigger_kind = trigger_kind.to_owned();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule
                     WHERE enabled = true AND trigger_kind = $1 AND scope_id = $2
                     ORDER BY rule_id ASC"
                ))
                .bind(trigger_kind)
                .bind(scope_id)
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?
                .into_iter()
                .map(row_to_rule)
                .collect()
            })
        })
    }

    fn add_rule_in_scope(
        &self,
        dto: &AddRuleDto,
        account_id: &Id,
        scope_id: Uuid,
    ) -> Result<Rule, RepoError> {
        let account_id_val = i64::from(*account_id);
        let name = dto.name.clone();
        let description = dto.description.clone().unwrap_or_default();
        let trigger_kind = dto.trigger.kind_str().to_owned();
        let trigger_config = serde_json::to_string(&dto.trigger)
            .map_err(|e| RepoError::DatabaseError { err: e.to_string() })?;
        let script = dto.script.clone();
        let enabled = dto.enabled.unwrap_or(true);
        let created_at: DateTime<Utc> = Utc::now();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, RuleRow>(&format!(
                    "INSERT INTO rule (account_id, scope_id, name, description, trigger_kind, trigger_config, script, enabled, created_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                     RETURNING {SELECT_COLS}"
                ))
                .bind(account_id_val)
                .bind(scope_id)
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

    fn update_rule_in_scopes(
        &self,
        rule_id: &Id,
        dto: &UpdateRuleDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<Rule>, RepoError> {
        if scope_ids.is_empty() {
            return Ok(None);
        }
        let rule_id_val = i64::from(*rule_id);
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
        let placeholders = (8..=scope_ids.len() + 7)
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let pool = self.pool.clone();
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql = format!(
                    "UPDATE rule
                     SET name = COALESCE($1, name),
                         description = COALESCE($2, description),
                         trigger_kind = COALESCE($3, trigger_kind),
                         trigger_config = COALESCE($4, trigger_config),
                         script = COALESCE($5, script),
                         enabled = COALESCE($6, enabled)
                     WHERE rule_id = $7 AND scope_id IN ({placeholders})"
                );
                let mut update = sqlx::query(&sql)
                    .bind(new_name)
                    .bind(new_description)
                    .bind(new_trigger_kind)
                    .bind(new_trigger_config)
                    .bind(new_script)
                    .bind(new_enabled)
                    .bind(rule_id_val);
                for scope_id in &scope_ids {
                    update = update.bind(scope_id);
                }
                update.execute(&pool).await.map_err(RepoError::from)?;

                let select_sql = format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = $1 AND scope_id IN ({})",
                    (2..=scope_ids.len() + 1)
                        .map(|index| format!("${index}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                let mut select = sqlx::query_as::<_, RuleRow>(&select_sql).bind(rule_id_val);
                for scope_id in &scope_ids {
                    select = select.bind(scope_id);
                }
                select
                    .fetch_optional(&pool)
                    .await
                    .map_err(RepoError::from)?
                    .map(row_to_rule)
                    .transpose()
            })
        })
    }

    fn delete_rule_in_scopes(&self, rule_id: &Id, scope_ids: &[Uuid]) -> Result<(), RepoError> {
        if scope_ids.is_empty() {
            return Ok(());
        }
        let placeholders = (2..=scope_ids.len() + 1)
            .map(|index| format!("${index}"))
            .collect::<Vec<_>>()
            .join(", ");
        let pool = self.pool.clone();
        let rule_id_val = i64::from(*rule_id);
        let scope_ids = scope_ids.to_vec();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let sql =
                    format!("DELETE FROM rule WHERE rule_id = $1 AND scope_id IN ({placeholders})");
                let mut query = sqlx::query(&sql).bind(rule_id_val);
                for scope_id in &scope_ids {
                    query = query.bind(scope_id);
                }
                query
                    .execute(&pool)
                    .await
                    .map(|_| ())
                    .map_err(RepoError::from)
            })
        })
    }

    fn get_all_rules(&self, account_id: &Id) -> Result<Vec<Rule>, RepoError> {
        let account_id_val = i64::from(*account_id);
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let rows = sqlx::query_as::<_, RuleRow>(&format!(
                    "SELECT {SELECT_COLS} FROM rule WHERE account_id = $1 ORDER BY rule_id ASC"
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
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = $1 AND account_id = $2"
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
                     WHERE enabled = true
                       AND trigger_kind IN ('cron', 'interval')
                     ORDER BY rule_id ASC"
                ))
                .fetch_all(&pool)
                .await
                .map_err(RepoError::from)?;
                rows.into_iter()
                    .map(|row| {
                        let account_id = Id::try_from(row.account_id as i64)
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
                     WHERE enabled = true AND trigger_kind = $1 AND account_id = $2
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
        let created_at: DateTime<Utc> = Utc::now();
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                let row = sqlx::query_as::<_, RuleRow>(&format!(
                    "INSERT INTO rule (account_id, scope_id, name, description, trigger_kind, trigger_config, script, enabled, created_at)
                     VALUES ($1, (SELECT default_scope_id FROM server_principal WHERE account_id = $1 AND kind = 'human'), $2, $3, $4, $5, $6, $7, $8)
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
                     SET name           = COALESCE($1, name),
                         description    = COALESCE($2, description),
                         trigger_kind   = COALESCE($3, trigger_kind),
                         trigger_config = COALESCE($4, trigger_config),
                         script         = COALESCE($5, script),
                         enabled        = COALESCE($6, enabled)
                     WHERE rule_id = $7 AND account_id = $8",
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
                    "SELECT {SELECT_COLS} FROM rule WHERE rule_id = $1 AND account_id = $2"
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
                sqlx::query("DELETE FROM rule WHERE rule_id = $1 AND account_id = $2")
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
        let ts: DateTime<Utc> = last_run_at.and_utc();
        let error = error.map(|s| s.to_owned());
        let output = output.map(|s| s.to_owned());
        let pool = self.pool.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async move {
                sqlx::query(
                    "UPDATE rule SET last_run_at = $1, last_error = $2, last_output = $3 WHERE rule_id = $4",
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
