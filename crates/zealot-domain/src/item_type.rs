use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{attribute::Attribute, common::id::Id};

#[derive(Debug, Clone)]
pub struct ItemType {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ItemTypeRef {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ItemTypeSummary {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
    pub required_attributes_count: i64,
    pub item_count: i64,
}

// Send DTO

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeDto {
    pub type_id: i64,
    pub is_system: bool,
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
    /// Present only on the response to a requirement change (including a dry run).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report: Option<RequiredChangeReport>,
}

/// What a requirement change did, or with `dry_run` would do. Counts are items of the type.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredChangeReport {
    pub dry_run: bool,
    pub added: Vec<String>,
    pub removed: Vec<String>,
    /// Items given a default value, per added attribute key.
    pub backfilled: BTreeMap<String, i64>,
    /// Items that lack an added attribute that has no default. When non-empty the change is rejected.
    pub non_compliant: NonCompliantSummary,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NonCompliantSummary {
    /// Distinct items lacking at least one added attribute without a default.
    pub total: i64,
    pub by_key: BTreeMap<String, i64>,
    /// Up to `NON_COMPLIANT_SAMPLE` item ids per key, ascending.
    pub sample_ids: BTreeMap<String, Vec<i64>>,
}

pub const NON_COMPLIANT_SAMPLE: i64 = 50;

/// Items of a type that lack some of the type's required attributes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompliantItemDto {
    pub item_id: i64,
    pub title: String,
    pub archived: bool,
    pub missing: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonCompliantPageDto {
    pub total: i64,
    pub items: Vec<NonCompliantItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeRefDto {
    pub type_id: i64,
    pub is_system: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemTypeSummaryDto {
    pub type_id: i64,
    pub is_system: bool,
    pub name: String,
    pub required_attributes_count: i64,
    pub item_count: i64,
}

// Receive DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddItemTypeDto {
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateItemTypeDto {
    pub type_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub required_attributes: Option<Vec<String>>,
    /// Report what a change to `required_attributes` would do without writing anything.
    #[serde(default)]
    pub dry_run: Option<bool>,
}

// Errors

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeError {}

// Impls

impl ItemType {
    pub fn is_valid(&self, attributes: &HashMap<String, Attribute>) -> bool {
        self.missing_required(attributes).is_empty()
    }

    /// Required keys absent from `attributes`, in the type's order.
    pub fn missing_required(&self, attributes: &HashMap<String, Attribute>) -> Vec<String> {
        self.required_attributes
            .iter()
            .filter(|key| !attributes.contains_key(*key))
            .cloned()
            .collect()
    }
}

impl From<&ItemType> for ItemTypeRef {
    fn from(value: &ItemType) -> Self {
        Self {
            type_id: value.type_id,
            is_system: value.is_system,
            name: value.name.clone(),
        }
    }
}

impl From<&ItemType> for ItemTypeDto {
    fn from(value: &ItemType) -> Self {
        Self {
            type_id: value.type_id.into(),
            is_system: value.is_system,
            name: value.name.clone(),
            description: value.description.clone(),
            required_attributes: value.required_attributes.clone(),
            report: None,
        }
    }
}

impl From<&ItemTypeRef> for ItemTypeRefDto {
    fn from(value: &ItemTypeRef) -> Self {
        Self {
            type_id: value.type_id.into(),
            is_system: value.is_system,
            name: value.name.clone(),
        }
    }
}

impl From<&ItemTypeSummary> for ItemTypeSummaryDto {
    fn from(value: &ItemTypeSummary) -> Self {
        Self {
            type_id: value.type_id.into(),
            is_system: value.is_system,
            name: value.name.clone(),
            required_attributes_count: value.required_attributes_count,
            item_count: value.item_count,
        }
    }
}
