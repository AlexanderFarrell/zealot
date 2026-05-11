# TASK-LUA-002: Fill in `Rule` domain model and DTOs

## Context

`crates/zealot-domain/src/rule.rs` is currently empty. The `Rule` struct is the core domain object that will drive everything: the DB schema, the service layer, the HTTP API, and the Lua execution context.

## Goal

Define `Rule`, `TriggerKind`, `RuleDto`, `AddRuleDto`, `UpdateRuleDto` in `zealot-domain`.

## Requirements

### `Rule` struct

```rust
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::common::id::Id;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub rule_id:     Id,
    pub account_id:  Id,
    pub name:        String,
    pub description: String,
    pub trigger:     TriggerKind,
    pub script:      String,
    pub enabled:     bool,
    pub created_at:  NaiveDateTime,
    pub last_run_at: Option<NaiveDateTime>,
    pub last_error:  Option<String>,
    pub last_output: Option<String>,
}
```

### `TriggerKind` enum

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerKind {
    OnItemCreate,
    OnItemUpdate,
    OnItemDelete,
    OnCommentAdd,
    OnTypeAssign   { type_name: Option<String> },
    OnTypeUnassign { type_name: Option<String> },
    OnAttributeSet { attribute_key: Option<String> },
    Cron           { expression: String },
    Interval       { seconds: u64 },
    Manual,
}

impl TriggerKind {
    /// Returns the bare string tag used for DB storage and event matching.
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::OnItemCreate      => "on_item_create",
            Self::OnItemUpdate      => "on_item_update",
            Self::OnItemDelete      => "on_item_delete",
            Self::OnCommentAdd      => "on_comment_add",
            Self::OnTypeAssign {..} => "on_type_assign",
            Self::OnTypeUnassign{..}=> "on_type_unassign",
            Self::OnAttributeSet{..}=> "on_attribute_set",
            Self::Cron {..}         => "cron",
            Self::Interval {..}     => "interval",
            Self::Manual            => "manual",
        }
    }

    pub fn is_event_triggered(&self) -> bool {
        matches!(self,
            Self::OnItemCreate | Self::OnItemUpdate | Self::OnItemDelete |
            Self::OnCommentAdd | Self::OnTypeAssign {..} |
            Self::OnTypeUnassign {..} | Self::OnAttributeSet {..}
        )
    }

    pub fn is_scheduled(&self) -> bool {
        matches!(self, Self::Cron {..} | Self::Interval {..})
    }
}
```

### DTOs

```rust
/// Sent to the client.
pub type RuleDto = Rule;

/// Received when creating a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct AddRuleDto {
    pub name:        String,
    pub description: Option<String>,
    pub trigger:     TriggerKind,
    pub script:      String,
    pub enabled:     Option<bool>,
}

/// Received when updating a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRuleDto {
    pub name:        Option<String>,
    pub description: Option<String>,
    pub trigger:     Option<TriggerKind>,
    pub script:      Option<String>,
    pub enabled:     Option<bool>,
}
```

### Export in `lib.rs`

Add `pub mod rule;` to `crates/zealot-domain/src/lib.rs`.

## Dependencies

- TASK-LUA-001

## Files to modify

- `crates/zealot-domain/src/rule.rs`
- `crates/zealot-domain/src/lib.rs`

## Verification

```bash
cargo check -p zealot-domain
```
