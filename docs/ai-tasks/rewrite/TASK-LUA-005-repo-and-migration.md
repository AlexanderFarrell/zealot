# TASK-LUA-005: `RuleRepo` trait, SQLite/Postgres impls, DB migration

## Context

Rules must persist in the database. This task creates the migration, the repo trait, and both SQLite and Postgres implementations following the exact pattern of existing repos (e.g. `CommentRepo`).

## Goal

DB migration + `RuleRepo` trait + both repo implementations.

## Requirements

### Migration: `crates/zealot-infra/migrations/sqlite/0004_rules.sql`

```sql
create table rule (
    rule_id        text    primary key,
    account_id     text    not null references account(account_id) on delete cascade,
    name           text    not null,
    description    text    not null default '',
    trigger_kind   text    not null,
    trigger_config text    not null default '{}',
    script         text    not null default '',
    enabled        integer not null default 1,
    created_at     integer not null,
    last_run_at    integer,
    last_error     text,
    last_output    text
);
create index idx_rule_account     on rule(account_id);
create index idx_rule_event_rules on rule(enabled, trigger_kind);
```

Create an equivalent `crates/zealot-infra/migrations/postgres/0004_rules.sql` using Postgres syntax (`boolean` instead of `integer`, `timestamptz` instead of `integer` timestamps, `text` remains `text`).

### `RuleRepo` trait — `crates/zealot-app/src/repos/rule.rs`

```rust
use zealot_domain::{
    common::id::Id,
    rule::{AddRuleDto, Rule, TriggerKind, UpdateRuleDto},
};
use crate::repos::common::RepoError;
use chrono::NaiveDateTime;

pub trait RuleRepo: Debug + Send + Sync {
    fn get_all_rules(&self, account_id: &Id) -> Result<Vec<Rule>, RepoError>;
    fn get_rule_by_id(&self, rule_id: &Id, account_id: &Id) -> Result<Option<Rule>, RepoError>;

    /// Returns all enabled scheduled rules across all accounts.
    /// Returns (Rule, account_id) pairs so the scheduler knows who owns each rule.
    fn get_enabled_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RepoError>;

    /// Returns enabled event rules matching the given trigger_kind for an account.
    fn get_enabled_event_rules(&self, trigger_kind: &str, account_id: &Id) -> Result<Vec<Rule>, RepoError>;

    fn add_rule(&self, dto: &AddRuleDto, account_id: &Id) -> Result<Rule, RepoError>;
    fn update_rule(&self, rule_id: &Id, dto: &UpdateRuleDto, account_id: &Id) -> Result<Option<Rule>, RepoError>;
    fn delete_rule(&self, rule_id: &Id, account_id: &Id) -> Result<(), RepoError>;

    /// Called after a rule execution to persist timing and error state.
    fn record_run(
        &self,
        rule_id: &Id,
        last_run_at: NaiveDateTime,
        error: Option<&str>,
        output: Option<&str>,
    ) -> Result<(), RepoError>;
}
```

### Repo implementations

**`crates/zealot-infra/src/sqlite/rule_repo.rs`** and **`crates/zealot-infra/src/postgres/rule_repo.rs`** — follow the exact pattern of `CommentSqliteRepo`/`CommentPostgresRepo`:

- `trigger_kind` stored as `TriggerKind::kind_str()` string
- `trigger_config` stored as `serde_json::to_string(&trigger)` (the full trigger JSON for config params); reconstruct with `serde_json::from_str`
- IDs stored as text (UUIDs)
- Timestamps stored as Unix epoch integers (SQLite) or `timestamptz` (Postgres)
- `enabled` stored as 0/1 integer (SQLite) or boolean (Postgres)

### Wire into `ZealotRepos`

Add `rule: Arc<dyn RuleRepo>` to the `ZealotRepos` struct in `crates/zealot-app/src/repos/mod.rs`.

## Dependencies

- TASK-LUA-002

## Files to create/modify

- `crates/zealot-infra/migrations/sqlite/0004_rules.sql` (new)
- `crates/zealot-infra/migrations/postgres/0004_rules.sql` (new)
- `crates/zealot-app/src/repos/rule.rs` (new)
- `crates/zealot-app/src/repos/mod.rs`
- `crates/zealot-infra/src/sqlite/rule_repo.rs` (new)
- `crates/zealot-infra/src/postgres/rule_repo.rs` (new)
- `crates/zealot-infra/src/sqlite/mod.rs`
- `crates/zealot-infra/src/postgres/mod.rs`

## Verification

```bash
cargo check -p zealot-infra
# Run migrations against a dev SQLite DB and verify the table is created
sqlx migrate run --database-url sqlite://dev.db
```
