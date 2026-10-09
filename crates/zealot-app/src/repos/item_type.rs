use std::fmt::Debug;

use std::collections::HashMap;

use zealot_domain::{
    attribute::Attribute,
    common::id::Id,
    item_type::{
        AddItemTypeDto, ItemType, ItemTypeRef, ItemTypeSummary, NonCompliantPageDto,
        UpdateItemTypeDto,
    },
};

use crate::repos::common::RepoError;

/// Members of a type that lack one attribute key and share the same set of "override types":
/// the item's types, among a given universe, that set a type-level default for that key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingGroup {
    /// Type ids from the universe that the members have (empty: no type-level default applies).
    pub set: Vec<i64>,
    pub count: i64,
    /// Up to the requested sample size of item ids, ascending.
    pub sample_ids: Vec<i64>,
}

/// Members of a type that need a value for `key` but would get none: lacking it, with exactly the
/// override types in one of `sets` (among `universe`), and no kind default to fall back on.
#[derive(Debug, Clone)]
pub struct Offender {
    pub key: String,
    pub universe: Vec<i64>,
    pub sets: Vec<Vec<i64>>,
}

/// Write `value` to members of the type that lack `key` and whose override types, among
/// `universe`, are exactly `set`.
#[derive(Debug, Clone)]
pub struct Backfill {
    pub key: String,
    pub universe: Vec<i64>,
    pub set: Vec<i64>,
    pub value: Attribute,
}

/// One atomic change to a type: optional rename, requirement links added and removed, type-level
/// defaults set (`Some`) or cleared (`None`), and the default values written onto members that
/// lack an added key.
#[derive(Debug, Clone, Default)]
pub struct RequiredChange {
    pub name: Option<String>,
    pub description: Option<String>,
    pub add: Vec<String>,
    pub remove: Vec<String>,
    pub defaults: std::collections::BTreeMap<String, Option<serde_json::Value>>,
    pub backfills: Vec<Backfill>,
}

pub trait ItemTypeRepo: Debug + Send + Sync {
    fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, RepoError>;
    fn get_item_type_summaries_with_archived(
        &self,
        account_id: &Id,
        _include_archived: bool,
    ) -> Result<Vec<ItemTypeSummary>, RepoError> {
        self.get_item_type_summaries(account_id)
    }
    fn get_item_type_summaries(&self, account_id: &Id) -> Result<Vec<ItemTypeSummary>, RepoError>;
    fn get_item_type(
        &self,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn get_item_type_refs_for_items(
        &self,
        item_ids: &Vec<Id>,
        account_id: &Id,
    ) -> Result<HashMap<Id, Vec<ItemTypeRef>>, RepoError>;
    fn get_item_ids_for_type_name(&self, name: &str, account_id: &Id)
    -> Result<Vec<Id>, RepoError>;
    fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, RepoError>;
    fn count_items_for_type(&self, type_id: &Id) -> Result<i64, RepoError>;
    fn delete_item_type(&self, item_type_id: &Id, account_id: &Id) -> Result<bool, RepoError>;
    fn add_attr_kinds_to_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
    fn remove_attr_kinds_from_item_type(
        &self,
        attr_kinds: &Vec<String>,
        item_type_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
    fn assign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
    fn unassign_item_types(
        &self,
        type_names: &Vec<String>,
        item_id: &Id,
        account_id: &Id,
    ) -> Result<(), RepoError>;
    /// Members of the type lacking `key`, grouped by which of `universe` (type ids with a type-level
    /// default for `key`) they have. Empty when `key` names no attribute kind, matching
    /// `add_attr_kinds_to_item_type`.
    fn missing_by_override_set(
        &self,
        type_id: &Id,
        key: &str,
        universe: &[i64],
        sample: i64,
        account_id: &Id,
    ) -> Result<Vec<MissingGroup>, RepoError>;
    /// Distinct members of the type that fall in any of `offenders`.
    fn count_offenders(
        &self,
        type_id: &Id,
        offenders: &[Offender],
        account_id: &Id,
    ) -> Result<i64, RepoError>;
    /// Every type (of the account or system) that sets a type-level default for `key`.
    fn overrides_for_key(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Vec<(i64, serde_json::Value)>, RepoError>;
    /// Apply `change` in one transaction. Returns the number of members given each backfilled
    /// key. A failure leaves the type and its items untouched.
    fn apply_required_change(
        &self,
        type_id: &Id,
        change: &RequiredChange,
        account_id: &Id,
    ) -> Result<std::collections::BTreeMap<String, i64>, RepoError>;
    /// Members that lack any currently required key, with the keys each lacks.
    fn list_non_compliant(
        &self,
        type_id: &Id,
        limit: i64,
        offset: i64,
        account_id: &Id,
    ) -> Result<NonCompliantPageDto, RepoError>;
}
