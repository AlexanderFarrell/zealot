# TASK-LUA-011: Scheduler loop for cron/interval rules

## Context

Rules with `TriggerKind::Cron` and `TriggerKind::Interval` need to fire on a time-based schedule. A Tokio background task polls every 60 seconds and fires any due rules.

## Goal

Implement `crates/zealot-app/src/scheduler.rs` and wire it into `apps/server/src/main.rs`.

## Requirements

### `crates/zealot-app/src/scheduler.rs`

```rust
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use chrono::Utc;
use zealot_domain::rule::TriggerKind;
use crate::{app::AppState, ports::rule_runner::RuleContext};

pub fn start(state: AppState) {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(60)).await;
            run_scheduled_rules(&state).await;
        }
    });
}

async fn run_scheduled_rules(state: &AppState) {
    let now = Utc::now().naive_local();

    let scheduled = match state.services.rule.get_all_scheduled_rules() {
        Ok(r) => r,    // returns Vec<(Rule, account_id)> from RuleRepo
        Err(e) => {
            tracing::error!("Scheduler: failed to load scheduled rules: {e}");
            return;
        }
    };

    for (rule, account_id) in scheduled {
        if is_due(&rule.trigger, rule.last_run_at, now) {
            let runner = state.ports.rule_runner.clone();
            let r = rule.clone();
            tokio::spawn(async move {
                runner.run_rule(&r, RuleContext::Scheduled { now }).await;
            });
        }
    }
}

fn is_due(trigger: &TriggerKind, last_run_at: Option<chrono::NaiveDateTime>, now: chrono::NaiveDateTime) -> bool {
    match trigger {
        TriggerKind::Cron { expression } => {
            // Use croner crate to evaluate if `now` is within the current minute window
            // and has not run in the last 60 seconds
            use croner::Cron;
            let Ok(cron) = Cron::new(expression).parse() else { return false };
            let last = last_run_at.unwrap_or(chrono::NaiveDateTime::MIN);
            let secs_since_last = (now - last).num_seconds();
            secs_since_last >= 55 && cron.is_time_matching(&now.and_utc()).unwrap_or(false)
        }
        TriggerKind::Interval { seconds } => {
            let last = last_run_at.unwrap_or(chrono::NaiveDateTime::MIN);
            (now - last).num_seconds() >= *seconds as i64
        }
        _ => false,
    }
}
```

### `RuleService::get_all_scheduled_rules`

Add this method to `RuleService` (it delegates to `RuleRepo::get_enabled_scheduled_rules`):
```rust
pub fn get_all_scheduled_rules(&self) -> Result<Vec<(Rule, Id)>, RuleServiceError> {
    Ok(self.rule_repo.get_enabled_scheduled_rules()?)
}
```

### Wire into `apps/server/src/main.rs`

After `AppState::new(...)`:
```rust
zealot_app::scheduler::start(state.clone());
```

Also start the event listener task here (see TASK-LUA-012).

### `AppState` needs `ports`

Currently `AppState` only exposes `services`. Add `ports: ZealotPorts` field to `AppState` so the scheduler can access `rule_runner`. Update `AppState::new` accordingly.

## Dependencies

- TASK-LUA-009 (runner impl)
- TASK-LUA-005 (repo — `get_enabled_scheduled_rules`)
- `croner` in workspace (TASK-LUA-001)

## Files to create/modify

- `crates/zealot-app/src/scheduler.rs` (new)
- `crates/zealot-app/src/lib.rs` (export `pub mod scheduler`)
- `crates/zealot-app/src/app.rs` (add `ports` field)
- `crates/zealot-app/src/services/rule.rs` (add `get_all_scheduled_rules`)
- `apps/server/src/main.rs`

## Verification

- Manual: create a rule with `Interval { seconds: 65 }`, wait 65 seconds, verify `last_run_at` is updated in DB.
- Unit test: test `is_due` for both `Cron` and `Interval` cases without needing a real DB.

```bash
cargo check -p zealot-app
cargo check -p server
```
