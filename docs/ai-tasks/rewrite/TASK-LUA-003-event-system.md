# TASK-LUA-003: Event system — `ZealotEvent`, `EventPort`, broadcast impl

## Context

Rules with event-based triggers need to fire when service mutations happen (item created, comment added, etc.). This requires an event bus. The design uses a `tokio::sync::broadcast` channel; services emit fire-and-forget events; a background task in `apps/server` receives events and dispatches matching rules.

`crates/zealot-app/src/ports/events.rs` is currently empty.

## Goal

Implement the `ZealotEvent` enum, `EventPort` trait, and `BroadcastEventPort` concrete implementation. Wire `events` into `ZealotPorts`.

## Requirements

### `crates/zealot-app/src/ports/events.rs`

```rust
use std::fmt::Debug;
use std::sync::Arc;
use zealot_domain::{
    comment::Comment,
    common::id::Id,
    item::Item,
};

#[derive(Debug, Clone)]
pub enum ZealotEvent {
    ItemCreated    { account_id: Id, item: Item },
    ItemUpdated    { account_id: Id, item: Item },
    ItemDeleted    { account_id: Id, item_id: Id },
    CommentAdded   { account_id: Id, item_id: Id, comment: Comment },
    TypeAssigned   { account_id: Id, item: Item, type_name: String },
    TypeUnassigned { account_id: Id, item: Item, type_name: String },
    AttributeSet   { account_id: Id, item: Item, attribute_key: String },
}

impl ZealotEvent {
    pub fn account_id(&self) -> &Id {
        match self {
            Self::ItemCreated    { account_id, .. } => account_id,
            Self::ItemUpdated    { account_id, .. } => account_id,
            Self::ItemDeleted    { account_id, .. } => account_id,
            Self::CommentAdded   { account_id, .. } => account_id,
            Self::TypeAssigned   { account_id, .. } => account_id,
            Self::TypeUnassigned { account_id, .. } => account_id,
            Self::AttributeSet   { account_id, .. } => account_id,
        }
    }

    /// Returns the TriggerKind string tag this event corresponds to.
    pub fn trigger_kind(&self) -> &'static str {
        match self {
            Self::ItemCreated {..}    => "on_item_create",
            Self::ItemUpdated {..}    => "on_item_update",
            Self::ItemDeleted {..}    => "on_item_delete",
            Self::CommentAdded {..}   => "on_comment_add",
            Self::TypeAssigned {..}   => "on_type_assign",
            Self::TypeUnassigned {..} => "on_type_unassign",
            Self::AttributeSet {..}   => "on_attribute_set",
        }
    }
}

pub trait EventPort: Debug + Send + Sync {
    /// Fire-and-forget — never blocks.
    fn emit(&self, event: ZealotEvent);
}

/// No-op implementation used in tests and CLI.
#[derive(Debug)]
pub struct NoopEventPort;
impl EventPort for NoopEventPort {
    fn emit(&self, _event: ZealotEvent) {}
}
```

### `crates/zealot-infra/src/ports/broadcast_event_port.rs` (new file)

```rust
use tokio::sync::broadcast;
use zealot_app::ports::events::{EventPort, ZealotEvent};

#[derive(Debug, Clone)]
pub struct BroadcastEventPort {
    tx: broadcast::Sender<ZealotEvent>,
}

impl BroadcastEventPort {
    pub fn new(tx: broadcast::Sender<ZealotEvent>) -> Self {
        Self { tx }
    }
}

impl EventPort for BroadcastEventPort {
    fn emit(&self, event: ZealotEvent) {
        // Ignore send errors — no receivers is fine (rules disabled or server shutting down)
        let _ = self.tx.send(event);
    }
}
```

### Anti-recursion guard

Add a thread-local in `crates/zealot-app/src/ports/events.rs`:

```rust
use std::cell::Cell;
thread_local! {
    static RULE_EXECUTION_DEPTH: Cell<u32> = const { Cell::new(0) };
}

pub fn with_rule_depth_guard<F: FnOnce()>(f: F) {
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() + 1));
    f();
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() - 1));
}

pub fn is_inside_rule_execution() -> bool {
    RULE_EXECUTION_DEPTH.with(|d| d.get() > 0)
}
```

`BroadcastEventPort::emit` checks `is_inside_rule_execution()` and skips the send when true. This prevents event → rule → event → rule infinite loops.

### Wire into `ZealotPorts`

In `crates/zealot-app/src/ports/mod.rs`, add `events: Arc<dyn EventPort>` to the `ZealotPorts` struct.

## Dependencies

- TASK-LUA-002

## Files to create/modify

- `crates/zealot-app/src/ports/events.rs`
- `crates/zealot-app/src/ports/mod.rs`
- `crates/zealot-infra/src/ports/broadcast_event_port.rs` (new)
- `crates/zealot-infra/src/ports/mod.rs` (export new file)

## Verification

```bash
cargo check -p zealot-app
cargo check -p zealot-infra
```
