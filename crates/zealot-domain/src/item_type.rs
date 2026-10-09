use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use serde_json::Value;

use crate::{attribute::Attribute, common::id::Id};

#[derive(Debug, Clone)]
pub struct ItemType {
    pub type_id: Id,
    pub is_system: bool,
    pub name: String,
    pub description: String,
    pub required_attributes: Vec<String>,
    /// Type-level default per required key (raw JSON, validated against the kind when written).
    /// Overrides the kind default for items of this type; see `effective_default`.
    pub required_defaults: BTreeMap<String, Value>,
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
    /// Type-level default per required attribute, overriding the kind's default.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub required_defaults: BTreeMap<String, Value>,
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
    /// Optional type-level defaults for keys in `required_attributes`.
    #[serde(default)]
    pub required_defaults: Option<BTreeMap<String, Value>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateItemTypeDto {
    pub type_id: i64,
    pub name: Option<String>,
    pub description: Option<String>,
    pub required_attributes: Option<Vec<String>>,
    /// Set (value) or clear (`null`) type-level defaults; keys not listed are unchanged. Each key
    /// must be required by the type after this update.
    #[serde(default)]
    pub required_defaults: Option<BTreeMap<String, Value>>,
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

/// The default a new value of `key` starts from for an item with several types: the type-level
/// defaults of the item's types that require it. None set: the kind default. All agree: that value.
/// Any disagree: the kind default, so there is no type-specific suggestion.
pub fn effective_default(
    type_defaults: &[Attribute],
    kind_default: Option<&Attribute>,
) -> Option<Attribute> {
    match type_defaults.split_first() {
        None => kind_default.cloned(),
        Some((first, rest)) if rest.iter().all(|other| other == first) => Some(first.clone()),
        Some(_) => kind_default.cloned(),
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
            required_defaults: value.required_defaults.clone(),
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attribute::AttributeScalar;

    fn int(n: i64) -> Attribute {
        Attribute::Scalar(AttributeScalar::Integer(n))
    }

    #[test]
    fn no_type_default_uses_the_kind_default() {
        assert_eq!(effective_default(&[], Some(&int(3))), Some(int(3)));
        assert_eq!(effective_default(&[], None), None);
    }

    #[test]
    fn one_type_default_overrides_the_kind() {
        assert_eq!(effective_default(&[int(5)], Some(&int(3))), Some(int(5)));
        assert_eq!(effective_default(&[int(5)], None), Some(int(5)));
    }

    #[test]
    fn agreeing_type_defaults_win() {
        assert_eq!(
            effective_default(&[int(5), int(5)], Some(&int(3))),
            Some(int(5))
        );
    }

    #[test]
    fn conflicting_type_defaults_fall_back_to_the_kind_default() {
        assert_eq!(
            effective_default(&[int(5), int(7)], Some(&int(3))),
            Some(int(3))
        );
        assert_eq!(effective_default(&[int(5), int(7)], None), None);
    }
}
