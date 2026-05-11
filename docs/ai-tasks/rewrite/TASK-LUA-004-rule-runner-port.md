# TASK-LUA-004: `RuleRunnerPort` trait and result types

## Context

`crates/zealot-app/src/ports/rule_runner.rs` is currently empty. The port abstracts over the concrete Lua implementation so that `zealot-app` doesn't depend on `mlua` directly, keeping compile times fast for all crates that don't need Lua.

## Goal

Define the `RuleRunnerPort` trait, `RuleRunResult`, and `RuleContext` types. Wire the port into `ZealotPorts`.

## Requirements

### `crates/zealot-app/src/ports/rule_runner.rs`

```rust
use std::fmt::Debug;
use chrono::NaiveDateTime;
use zealot_domain::{common::id::Id, rule::Rule};
use crate::ports::events::ZealotEvent;

#[derive(Debug, Clone)]
pub struct RuleRunResult {
    pub rule_id:     Id,
    pub success:     bool,
    pub output:      Option<String>,   // captured zealot.notify() calls
    pub error:       Option<String>,   // LuaError message if failed
    pub duration_ms: u64,
}

#[derive(Debug, Clone)]
pub enum RuleContext {
    Event(ZealotEvent),
    Scheduled { now: NaiveDateTime },
    Manual,
}

#[async_trait::async_trait]
pub trait RuleRunnerPort: Debug + Send + Sync {
    /// Find and run all enabled event-triggered rules for this event.
    async fn run_event_rules(&self, event: ZealotEvent) -> Vec<RuleRunResult>;

    /// Run a single rule with the given context.
    async fn run_rule(&self, rule: &Rule, context: RuleContext) -> RuleRunResult;
}

/// No-op implementation for use in tests and contexts without Lua.
#[derive(Debug)]
pub struct NoopRuleRunner;

#[async_trait::async_trait]
impl RuleRunnerPort for NoopRuleRunner {
    async fn run_event_rules(&self, _event: ZealotEvent) -> Vec<RuleRunResult> {
        vec![]
    }
    async fn run_rule(&self, rule: &Rule, _context: RuleContext) -> RuleRunResult {
        RuleRunResult {
            rule_id: rule.rule_id.clone(),
            success: true,
            output: None,
            error: None,
            duration_ms: 0,
        }
    }
}
```

Note: `async_trait` needs to be added to `zealot-app`'s dependencies if not already present. Alternatively use the 2024 edition native async traits if the MSRV supports it.

### Wire into `ZealotPorts`

In `crates/zealot-app/src/ports/mod.rs`, add:
```rust
pub mod rule_runner;

pub struct ZealotPorts {
    pub events:      Arc<dyn EventPort>,
    pub rule_runner: Arc<dyn RuleRunnerPort>,
    // ... existing ports
}
```

## Dependencies

- TASK-LUA-003

## Files to modify

- `crates/zealot-app/src/ports/rule_runner.rs`
- `crates/zealot-app/src/ports/mod.rs`
- `crates/zealot-app/Cargo.toml` (add `async-trait` if needed)

## Verification

```bash
cargo check -p zealot-app
```
