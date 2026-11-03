use std::cell::Cell;
use std::fmt::Debug;
use zealot_domain::{
    attribute::Attribute, comment::Comment, common::id::Id, item::Item, rule::TriggerKind,
};

#[derive(Debug, Clone)]
pub enum ZealotEvent {
    ItemCreated {
        account_id: Id,
        item: Item,
    },
    ItemUpdated {
        account_id: Id,
        item: Item,
    },
    ItemDeleted {
        account_id: Id,
        item_id: Id,
    },
    CommentAdded {
        account_id: Id,
        item_id: Id,
        comment: Comment,
    },
    TypeAssigned {
        account_id: Id,
        item: Item,
        type_name: String,
    },
    TypeUnassigned {
        account_id: Id,
        item: Item,
        type_name: String,
    },
    AttributeSet {
        account_id: Id,
        item: Item,
        attribute_key: String,
        /// Value before the write; `None` when the attribute was unset.
        old_value: Option<Attribute>,
        /// Value after the write; `None` when the attribute is absent.
        new_value: Option<Attribute>,
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

    /// Whether a rule with this trigger should run for the event.
    ///
    /// The trigger kind must match, and the optional filter on the trigger
    /// (attribute key or type name) must match when set. An `AttributeSet`
    /// event whose old and new values are equal never fires: a write that
    /// leaves the value as it was is not a change, so a rule such as
    /// "Stamp Completed At" cannot be triggered by re-saving a `Status`.
    pub fn should_fire(&self, trigger: &TriggerKind) -> bool {
        match (self, trigger) {
            (Self::ItemCreated { .. }, TriggerKind::OnItemCreate)
            | (Self::ItemUpdated { .. }, TriggerKind::OnItemUpdate)
            | (Self::ItemDeleted { .. }, TriggerKind::OnItemDelete)
            | (Self::CommentAdded { .. }, TriggerKind::OnCommentAdd) => true,
            (
                Self::TypeAssigned { type_name, .. },
                TriggerKind::OnTypeAssign { type_name: want },
            )
            | (
                Self::TypeUnassigned { type_name, .. },
                TriggerKind::OnTypeUnassign { type_name: want },
            ) => want.as_ref().is_none_or(|want| want == type_name),
            (
                Self::AttributeSet {
                    attribute_key,
                    old_value,
                    new_value,
                    ..
                },
                TriggerKind::OnAttributeSet {
                    attribute_key: want,
                },
            ) => old_value != new_value && want.as_ref().is_none_or(|want| want == attribute_key),
            _ => false,
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use zealot_domain::attribute::AttributeScalar;

    fn id(n: i64) -> Id {
        Id::try_from(n).unwrap()
    }

    fn item() -> Item {
        Item {
            item_id: id(7),
            title: "Probe".into(),
            content: String::new(),
            attributes: HashMap::new(),
            types: vec![],
            links: vec![],
        }
    }

    fn text(value: &str) -> Option<Attribute> {
        Some(Attribute::Scalar(AttributeScalar::Text(value.into())))
    }

    fn attribute_set(key: &str, old: Option<Attribute>, new: Option<Attribute>) -> ZealotEvent {
        ZealotEvent::AttributeSet {
            account_id: id(1),
            item: item(),
            attribute_key: key.into(),
            old_value: old,
            new_value: new,
        }
    }

    fn on_attribute(key: Option<&str>) -> TriggerKind {
        TriggerKind::OnAttributeSet {
            attribute_key: key.map(String::from),
        }
    }

    #[test]
    fn attribute_set_fires_only_for_the_matching_key() {
        let event = attribute_set("Parent", text("a"), text("b"));
        assert!(!event.should_fire(&on_attribute(Some("Status"))));
        assert!(event.should_fire(&on_attribute(Some("Parent"))));
        assert!(event.should_fire(&on_attribute(None)));
    }

    #[test]
    fn attribute_set_skips_an_unchanged_value() {
        let same = attribute_set("Status", text("Complete"), text("Complete"));
        assert!(!same.should_fire(&on_attribute(Some("Status"))));
        assert!(!same.should_fire(&on_attribute(None)));
        let unset_again = attribute_set("Status", None, None);
        assert!(!unset_again.should_fire(&on_attribute(None)));
    }

    #[test]
    fn attribute_set_fires_when_set_changed_or_removed() {
        let trigger = on_attribute(Some("Status"));
        assert!(attribute_set("Status", None, text("Complete")).should_fire(&trigger));
        assert!(attribute_set("Status", text("To Do"), text("Complete")).should_fire(&trigger));
        assert!(attribute_set("Status", text("Complete"), None).should_fire(&trigger));
    }

    #[test]
    fn type_events_respect_the_type_name_filter() {
        let assigned = ZealotEvent::TypeAssigned {
            account_id: id(1),
            item: item(),
            type_name: "Ticket".into(),
        };
        let filter = |name: Option<&str>| TriggerKind::OnTypeAssign {
            type_name: name.map(String::from),
        };
        assert!(assigned.should_fire(&filter(None)));
        assert!(assigned.should_fire(&filter(Some("Ticket"))));
        assert!(!assigned.should_fire(&filter(Some("Plan"))));
        assert!(!assigned.should_fire(&TriggerKind::OnTypeUnassign { type_name: None }));
    }

    #[test]
    fn other_events_fire_only_their_own_trigger() {
        let created = ZealotEvent::ItemCreated {
            account_id: id(1),
            item: item(),
        };
        assert!(created.should_fire(&TriggerKind::OnItemCreate));
        assert!(!created.should_fire(&TriggerKind::OnItemUpdate));
        assert!(!created.should_fire(&on_attribute(None)));
        assert!(!created.should_fire(&TriggerKind::Manual));
    }
}
