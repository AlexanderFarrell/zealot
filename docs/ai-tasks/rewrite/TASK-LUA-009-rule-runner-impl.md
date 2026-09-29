# TASK-LUA-009: `LuaRuleRunner` implementation

## Context

This task wires sandbox + bindings into a concrete `RuleRunnerPort` implementation: `LuaRuleRunner` in `crates/zealot-lua/src/runner.rs`. Each rule execution creates a fresh Lua VM, sandboxes it, injects the `zealot` globals, runs the script, captures results, and persists them.

## Goal

Implement `LuaRuleRunner` and its `RuleRunnerPort` impl.

## Requirements

### `crates/zealot-lua/src/runner.rs`

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use chrono::Utc;
use zealot_app::{
    ports::rule_runner::{RuleContext, RuleRunResult, RuleRunnerPort},
    ports::events::ZealotEvent,
    repos::ZealotRepos,
    services::ZealotServices,
};
use zealot_domain::{common::id::Id, rule::Rule};

#[derive(Debug, Clone)]
pub struct LuaRuleRunner {
    repos:    Arc<ZealotRepos>,
    services: Arc<ZealotServices>,
}

impl LuaRuleRunner {
    pub fn new(repos: Arc<ZealotRepos>, services: Arc<ZealotServices>) -> Self {
        Self { repos, services }
    }

    async fn execute_one(&self, rule: &Rule, context: RuleContext) -> RuleRunResult {
        let start = std::time::Instant::now();
        let output_buf: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(vec![]));

        let result = self.execute_script(rule, &context, output_buf.clone()).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        let output: Option<String> = {
            let lines = output_buf.lock().await;
            if lines.is_empty() { None } else { Some(lines.join("\n")) }
        };

        let (success, error) = match result {
            Ok(()) => (true, None),
            Err(e)  => (false, Some(e.to_string())),
        };

        // Persist run result
        let now = Utc::now().naive_utc();
        let _ = self.repos.rule.record_run(
            &rule.rule_id,
            now,
            error.as_deref(),
            output.as_deref(),
        );

        RuleRunResult { rule_id: rule.rule_id.clone(), success, output, error, duration_ms }
    }

    async fn execute_script(
        &self,
        rule: &Rule,
        context: &RuleContext,
        output_buf: Arc<Mutex<Vec<String>>>,
    ) -> mlua::Result<()> {
        use crate::sandbox::{new_sandbox, execute_script};
        use crate::bindings::setup_zealot_globals;

        let lua = new_sandbox()?;
        setup_zealot_globals(
            &lua,
            context,
            self.services.clone(),
            rule.account_id.clone(),
            output_buf,
        )?;
        execute_script(&lua, &rule.script).await
    }
}

#[async_trait::async_trait]
impl RuleRunnerPort for LuaRuleRunner {
    async fn run_event_rules(&self, event: ZealotEvent) -> Vec<RuleRunResult> {
        use zealot_app::ports::events::with_rule_depth_guard;

        let account_id = event.account_id().clone();
        let trigger_kind = event.trigger_kind();

        let rules = match self.repos.rule.get_enabled_event_rules(trigger_kind, &account_id) {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Failed to load event rules: {e}");
                return vec![];
            }
        };

        let mut results = vec![];
        for rule in rules {
            let context = RuleContext::Event(event.clone());
            // Guard prevents rules from triggering further events
            let result = {
                let runner = self.clone();
                with_rule_depth_guard(|| async move {
                    runner.execute_one(&rule, context).await
                }).await
            };
            results.push(result);
        }
        results
    }

    async fn run_rule(&self, rule: &Rule, context: RuleContext) -> RuleRunResult {
        self.execute_one(rule, context).await
    }
}
```

Note: `with_rule_depth_guard` needs to work with async code. Use a scoped increment/decrement around the await point, or use a `tokio::task_local!` instead of `thread_local!` since async tasks may hop threads. Revisit in implementation — `task_local!` is the correct approach for async.

## Dependencies

- TASK-LUA-007 (sandbox)
- TASK-LUA-008 (bindings)
- TASK-LUA-005 (repo — for `record_run`)
- TASK-LUA-006 (services — for passing to bindings)

## Files to create/modify

- `crates/zealot-lua/src/runner.rs`
- `crates/zealot-lua/src/lib.rs` (export `LuaRuleRunner`)

## Verification

Integration test: create a real rule with a script that calls `zealot.notify("ok")`, run it via `run_rule`, assert `result.success == true` and `result.output == Some("ok")`.

```bash
cargo test -p zealot-lua
```
