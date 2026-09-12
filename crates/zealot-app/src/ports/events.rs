use std::cell::Cell;
use std::fmt::Debug;
use uuid::Uuid;
use zealot_domain::{comment::Comment, common::id::Id, item::Item};

#[derive(Debug, Clone)]
pub enum ZealotEvent {
    ItemCreated {
        account_id: Id,
        scope_id: Uuid,
        item: Item,
    },
    ItemUpdated {
        account_id: Id,
        scope_id: Uuid,
        item: Item,
    },
    ItemDeleted {
        account_id: Id,
        scope_id: Uuid,
        item_id: Id,
    },
    CommentAdded {
        account_id: Id,
        scope_id: Uuid,
        item_id: Id,
        comment: Comment,
    },
    TypeAssigned {
        account_id: Id,
        scope_id: Uuid,
        item: Item,
        type_name: String,
    },
    TypeUnassigned {
        account_id: Id,
        scope_id: Uuid,
        item: Item,
        type_name: String,
    },
    AttributeSet {
        account_id: Id,
        scope_id: Uuid,
        item: Item,
        attribute_key: String,
    },
}

impl ZealotEvent {
    pub fn account_id(&self) -> &Id {
        match self {
            Self::ItemCreated { account_id, .. } => account_id,
            Self::ItemUpdated { account_id, .. } => account_id,
            Self::ItemDeleted { account_id, .. } => account_id,
            Self::CommentAdded { account_id, .. } => account_id,
            Self::TypeAssigned { account_id, .. } => account_id,
            Self::TypeUnassigned { account_id, .. } => account_id,
            Self::AttributeSet { account_id, .. } => account_id,
        }
    }

    pub fn scope_id(&self) -> Uuid {
        match self {
            Self::ItemCreated { scope_id, .. }
            | Self::ItemUpdated { scope_id, .. }
            | Self::ItemDeleted { scope_id, .. }
            | Self::CommentAdded { scope_id, .. }
            | Self::TypeAssigned { scope_id, .. }
            | Self::TypeUnassigned { scope_id, .. }
            | Self::AttributeSet { scope_id, .. } => *scope_id,
        }
    }

    pub fn trigger_kind(&self) -> &'static str {
        match self {
            Self::ItemCreated { .. } => "on_item_create",
            Self::ItemUpdated { .. } => "on_item_update",
            Self::ItemDeleted { .. } => "on_item_delete",
            Self::CommentAdded { .. } => "on_comment_add",
            Self::TypeAssigned { .. } => "on_type_assign",
            Self::TypeUnassigned { .. } => "on_type_unassign",
            Self::AttributeSet { .. } => "on_attribute_set",
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

thread_local! {
    static RULE_EXECUTION_DEPTH: Cell<u32> = const { Cell::new(0) };
}

pub fn with_rule_depth_guard<F: FnOnce()>(f: F) {
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() + 1));
    f();
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() - 1));
}

pub async fn with_rule_depth_guard_async<F, Fut>(f: F) -> Fut::Output
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future,
{
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() + 1));
    let result = f().await;
    RULE_EXECUTION_DEPTH.with(|d| d.set(d.get() - 1));
    result
}

pub fn is_inside_rule_execution() -> bool {
    RULE_EXECUTION_DEPTH.with(|d| d.get() > 0)
}
