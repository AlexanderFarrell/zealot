use crate::common::id::Id;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Rule {
    pub rule_id: Id,
    pub account_id: Id,
    pub name: String,
    pub description: String,
    pub trigger: TriggerKind,
    pub script: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub last_run_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,
    pub last_output: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TriggerKind {
    OnItemCreate,
    OnItemUpdate,
    OnItemDelete,
    OnCommentAdd,
    OnTypeAssign { type_name: Option<String> },
    OnTypeUnassign { type_name: Option<String> },
    OnAttributeSet { attribute_key: Option<String> },
    Cron { expression: String },
    Interval { seconds: u64 },
    Manual,
}

impl TriggerKind {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Self::OnItemCreate => "on_item_create",
            Self::OnItemUpdate => "on_item_update",
            Self::OnItemDelete => "on_item_delete",
            Self::OnCommentAdd => "on_comment_add",
            Self::OnTypeAssign { .. } => "on_type_assign",
            Self::OnTypeUnassign { .. } => "on_type_unassign",
            Self::OnAttributeSet { .. } => "on_attribute_set",
            Self::Cron { .. } => "cron",
            Self::Interval { .. } => "interval",
            Self::Manual => "manual",
        }
    }

    pub fn is_event_triggered(&self) -> bool {
        matches!(
            self,
            Self::OnItemCreate
                | Self::OnItemUpdate
                | Self::OnItemDelete
                | Self::OnCommentAdd
                | Self::OnTypeAssign { .. }
                | Self::OnTypeUnassign { .. }
                | Self::OnAttributeSet { .. }
        )
    }

    pub fn is_scheduled(&self) -> bool {
        matches!(self, Self::Cron { .. } | Self::Interval { .. })
    }
}

/// Sent to the client.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDto {
    pub rule_id: i64,
    pub account_id: i64,
    pub name: String,
    pub description: String,
    pub trigger: TriggerKind,
    pub script: String,
    pub enabled: bool,
    pub created_at: NaiveDateTime,
    pub last_run_at: Option<NaiveDateTime>,
    pub last_error: Option<String>,
    pub last_output: Option<String>,
}

impl From<&Rule> for RuleDto {
    fn from(r: &Rule) -> Self {
        Self {
            rule_id: r.rule_id.into(),
            account_id: r.account_id.into(),
            name: r.name.clone(),
            description: r.description.clone(),
            trigger: r.trigger.clone(),
            script: r.script.clone(),
            enabled: r.enabled,
            created_at: r.created_at,
            last_run_at: r.last_run_at,
            last_error: r.last_error.clone(),
            last_output: r.last_output.clone(),
        }
    }
}

/// Received when creating a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct AddRuleDto {
    pub name: String,
    pub description: Option<String>,
    pub trigger: TriggerKind,
    pub script: String,
    pub enabled: Option<bool>,
}

/// Received when updating a rule.
#[derive(Debug, Clone, Deserialize)]
pub struct UpdateRuleDto {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerKind>,
    pub script: Option<String>,
    pub enabled: Option<bool>,
}
