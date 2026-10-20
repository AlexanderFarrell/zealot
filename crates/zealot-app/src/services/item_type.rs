use std::{
    collections::{BTreeMap, HashSet},
    sync::Arc,
};

use serde_json::{Value, json};

use zealot_domain::{
    attribute::{Attribute, AttributeScalar},
    common::id::Id,
    item_type::{
        AddItemTypeDto, ItemType, ItemTypeSummary, NON_COMPLIANT_SAMPLE, NonCompliantPageDto,
        NonCompliantSummary, RequiredChangeReport, UpdateItemTypeDto, effective_default,
    },
};

use crate::repos::{
    attribute::AttributeRepo,
    common::RepoError,
    item_type::{Backfill, ItemTypeRepo, Offender, RequiredChange},
};

/// Where defaults for a required attribute come from beyond the types themselves: the attribute
/// kind's default, and the kind's rules for validating a type-level default.
pub trait RequirementDefaults: std::fmt::Debug + Send + Sync {
    fn kind_default(&self, account_id: &Id, key: &str) -> Option<Attribute>;
    /// Parse a type-level default for `key` with the kind's own rules.
    fn parse_override(&self, account_id: &Id, key: &str, raw: &Value) -> Result<Attribute, String>;
    /// Whether `key` names an attribute kind the account can use. Implementations without a
    /// kind store accept every key.
    fn kind_exists(&self, _account_id: &Id, _key: &str) -> Result<bool, String> {
        Ok(true)
    }
}

/// No kind has a default and any type-level default is accepted as given. For tests.
#[derive(Debug)]
pub struct NoDefaults;

impl RequirementDefaults for NoDefaults {
    fn kind_default(&self, _: &Id, _: &str) -> Option<Attribute> {
        None
    }

    fn parse_override(&self, _: &Id, key: &str, _: &Value) -> Result<Attribute, String> {
        Err(format!("no attribute kind '{key}' to validate against"))
    }
}

/// Defaults from the attribute kind (`AttributeKind.default`, Z45) and its validation rules.
/// Type-level overrides are stored on the type and resolved by `change_required`.
#[derive(Debug)]
pub struct KindDefaults {
    attributes: Arc<dyn AttributeRepo>,
}

impl KindDefaults {
    pub fn new(attributes: &Arc<dyn AttributeRepo>) -> Self {
        Self {
            attributes: attributes.clone(),
        }
    }
}

impl RequirementDefaults for KindDefaults {
    fn kind_default(&self, account_id: &Id, key: &str) -> Option<Attribute> {
        match self.attributes.get_attribute_kind(key, account_id) {
            Ok(kind) => kind.and_then(|kind| kind.default),
            Err(err) => {
                // Treated as no default, so the change is rejected rather than half-filled.
                tracing::error!(%key, "attribute kind lookup failed: {err}");
                None
            }
        }
    }

    fn kind_exists(&self, account_id: &Id, key: &str) -> Result<bool, String> {
        self.attributes
            .get_attribute_kind(key, account_id)
            .map(|kind| kind.is_some())
            .map_err(|err| format!("attribute kind lookup failed: {err}"))
    }

    fn parse_override(&self, account_id: &Id, key: &str, raw: &Value) -> Result<Attribute, String> {
        let kind = self
            .attributes
            .get_attribute_kind(key, account_id)
            .map_err(|err| format!("attribute kind lookup failed: {err}"))?
            .ok_or_else(|| format!("unknown attribute kind '{key}'"))?;
        let attribute = kind
            .parse_default(&json!({ "default": raw }))
            .map_err(|err| format!("invalid default for '{key}': {err}"))?
            .ok_or_else(|| format!("default for '{key}' must be a value; send null to clear"))?;
        if usable_default(&attribute) {
            Ok(attribute)
        } else {
            Err(format!(
                "'{key}' cannot have a default: relationships and empty lists have none"
            ))
        }
    }
}

/// An item type after a requirement change, with what the change did.
#[derive(Debug, Clone)]
pub struct ItemTypeChange {
    pub item_type: ItemType,
    pub report: RequiredChangeReport,
}

#[derive(Debug, Clone)]
pub struct ItemTypeService {
    repo: Arc<dyn ItemTypeRepo>,
    defaults: Arc<dyn RequirementDefaults>,
}

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeServiceError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    ReadOnly(String),
    #[error("{0}")]
    InUse(String),
    /// A type-level default that does not fit its attribute kind or the type.
    #[error("{0}")]
    InvalidDefault(String),
    /// A requirement change would leave existing items without a value and has no default to
    /// give them. Nothing was changed.
    /// Required attributes that name no existing attribute kind. Nothing was changed.
    #[error("{0}")]
    UnknownAttributes(String),
    #[error("{message}")]
    WouldInvalidate {
        message: String,
        summary: NonCompliantSummary,
    },
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl ItemTypeService {
    pub fn new(repo: &Arc<dyn ItemTypeRepo>) -> Self {
        Self {
            repo: repo.clone(),
            defaults: Arc::new(NoDefaults),
        }
    }

    pub fn with_defaults(mut self, defaults: Arc<dyn RequirementDefaults>) -> Self {
        self.defaults = defaults;
        self
    }

    pub fn get_item_types(&self, account_id: &Id) -> Result<Vec<ItemType>, ItemTypeServiceError> {
        self.repo
            .get_item_types(account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type_summaries(
        &self,
        account_id: &Id,
    ) -> Result<Vec<ItemTypeSummary>, ItemTypeServiceError> {
        self.repo
            .get_item_type_summaries(account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type_summaries_with_archived(
        &self,
        account_id: &Id,
        include_archived: bool,
    ) -> Result<Vec<ItemTypeSummary>, ItemTypeServiceError> {
        self.repo
            .get_item_type_summaries_with_archived(account_id, include_archived)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type(
        &self,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.repo
            .get_item_type(type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn get_item_type_by_name(
        &self,
        name: &str,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.repo
            .get_item_type_by_name(name, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn add_item_type(
        &self,
        dto: &AddItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        self.ensure_kinds_exist(account_id, &dto.required_attributes)?;
        let overrides = match &dto.required_defaults {
            Some(raw) if !raw.is_empty() => {
                self.parse_overrides(account_id, &dto.required_attributes, raw)?
            }
            _ => BTreeMap::new(),
        };
        let item_type = self
            .repo
            .add_item_type(dto, account_id)
            .map_err(ItemTypeServiceError::Repo)?;
        let Some(created) = item_type else {
            return Ok(None);
        };
        tracing::info!(account_id = ?account_id, type_id = ?created.type_id, name = %created.name, "item type created");
        if overrides.is_empty() {
            // Read back, so the response shows what was stored.
            return self.get_item_type(&created.type_id, account_id);
        }
        // A new type has no members, so only the stored defaults need writing.
        self.repo
            .apply_required_change(
                &created.type_id,
                &RequiredChange {
                    defaults: overrides.into_iter().map(|(k, v)| (k, Some(v.0))).collect(),
                    ..Default::default()
                },
                account_id,
            )
            .map_err(ItemTypeServiceError::Repo)?;
        self.get_item_type(&created.type_id, account_id)
    }

    pub fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemTypeChange>, ItemTypeServiceError> {
        let type_id = Id::try_from(dto.type_id).map_err(|e| {
            ItemTypeServiceError::Repo(RepoError::DatabaseError { err: e.to_string() })
        })?;
        let Some(current) = self.get_mutable_item_type(&type_id, account_id)? else {
            return Ok(None);
        };

        let (add, remove) = match &dto.required_attributes {
            Some(desired) => {
                let current_required: HashSet<&String> =
                    current.required_attributes.iter().collect();
                let desired_set: HashSet<&String> = desired.iter().collect();
                (
                    unique(desired.iter().filter(|key| !current_required.contains(key))),
                    unique(
                        current
                            .required_attributes
                            .iter()
                            .filter(|key| !desired_set.contains(key)),
                    ),
                )
            }
            None => (Vec::new(), Vec::new()),
        };
        let dry_run = dto.dry_run.unwrap_or(false);
        let desired_required: Vec<String> = match &dto.required_attributes {
            Some(desired) => desired.clone(),
            None => current.required_attributes.clone(),
        };
        let override_changes = match &dto.required_defaults {
            Some(raw) => self.parse_override_changes(account_id, &desired_required, raw)?,
            None => BTreeMap::new(),
        };
        let change = self.change_required(
            &current,
            account_id,
            dto.name.clone(),
            dto.description.clone(),
            add,
            remove,
            override_changes,
            dry_run,
        )?;
        tracing::info!(account_id = ?account_id, ?type_id, dry_run, "item type updated");
        Ok(Some(change))
    }

    /// Add required attributes to a type. With `dry_run`, only report what would happen.
    pub fn add_attr_kinds_to_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
        dry_run: bool,
    ) -> Result<RequiredChangeReport, ItemTypeServiceError> {
        let current = self
            .get_mutable_item_type(type_id, account_id)?
            .ok_or(ItemTypeServiceError::NotFound)?;
        let required: HashSet<&String> = current.required_attributes.iter().collect();
        let add = unique(attr_kind_keys.iter().filter(|key| !required.contains(key)));
        Ok(self
            .change_required(
                &current,
                account_id,
                None,
                None,
                add,
                Vec::new(),
                BTreeMap::new(),
                dry_run,
            )?
            .report)
    }

    pub fn remove_attr_kinds_from_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<(), ItemTypeServiceError> {
        let current = self
            .get_mutable_item_type(type_id, account_id)?
            .ok_or(ItemTypeServiceError::NotFound)?;
        self.change_required(
            &current,
            account_id,
            None,
            None,
            Vec::new(),
            unique(attr_kind_keys.iter()),
            BTreeMap::new(),
            false,
        )?;
        Ok(())
    }

    /// Items of the type that lack some of its required attributes.
    pub fn list_non_compliant(
        &self,
        type_id: &Id,
        limit: i64,
        offset: i64,
        account_id: &Id,
    ) -> Result<NonCompliantPageDto, ItemTypeServiceError> {
        self.repo
            .get_item_type(type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)?
            .ok_or(ItemTypeServiceError::NotFound)?;
        self.repo
            .list_non_compliant(type_id, limit.clamp(1, 200), offset.max(0), account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    /// Rejects keys that are not existing attribute kinds, naming all of them.
    fn ensure_kinds_exist(
        &self,
        account_id: &Id,
        keys: &[String],
    ) -> Result<(), ItemTypeServiceError> {
        let mut unknown = Vec::new();
        for key in keys {
            if unknown.contains(key) {
                continue;
            }
            let exists = self
                .defaults
                .kind_exists(account_id, key)
                .map_err(|err| ItemTypeServiceError::Repo(RepoError::DatabaseError { err }))?;
            if !exists {
                unknown.push(key.clone());
            }
        }
        if unknown.is_empty() {
            return Ok(());
        }
        let names: Vec<String> = unknown.iter().map(|key| format!("'{key}'")).collect();
        Err(ItemTypeServiceError::UnknownAttributes(format!(
            "Unknown attribute kind(s): {}. Create the attribute kind first.",
            names.join(", ")
        )))
    }

    /// Validate type-level defaults for a new type: each key must be required, each value must
    /// fit its kind. Returns the validated raw JSON with its parsed form.
    fn parse_overrides(
        &self,
        account_id: &Id,
        required: &[String],
        raw: &BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, (Value, Attribute)>, ItemTypeServiceError> {
        let mut out = BTreeMap::new();
        for (key, value) in raw {
            if value.is_null() {
                continue;
            }
            if !required.contains(key) {
                return Err(ItemTypeServiceError::InvalidDefault(format!(
                    "'{key}' is not a required attribute of this type, so it cannot have a type default"
                )));
            }
            let parsed = self
                .defaults
                .parse_override(account_id, key, value)
                .map_err(ItemTypeServiceError::InvalidDefault)?;
            out.insert(key.clone(), (value.clone(), parsed));
        }
        Ok(out)
    }

    /// As `parse_overrides`, for an update: `null` clears a key.
    fn parse_override_changes(
        &self,
        account_id: &Id,
        required: &[String],
        raw: &BTreeMap<String, Value>,
    ) -> Result<BTreeMap<String, Option<(Value, Attribute)>>, ItemTypeServiceError> {
        let mut out = BTreeMap::new();
        for (key, value) in raw {
            if value.is_null() {
                out.insert(key.clone(), None);
                continue;
            }
            let one = BTreeMap::from([(key.clone(), value.clone())]);
            let mut parsed = self.parse_overrides(account_id, required, &one)?;
            out.insert(key.clone(), parsed.remove(key));
        }
        Ok(out)
    }

    /// The one place a type's requirements change. Existing members are checked first; each
    /// member lacking an added attribute gets the effective default over all of its types (type
    /// defaults that agree, else the kind default). Members with none of those would be left
    /// without a value, so the whole change is rejected. Otherwise the links, rename, type-level
    /// defaults and backfill commit in one transaction. `dry_run` stops before that and reports
    /// the same numbers, including would-be offenders, without an error.
    ///
    /// Backfill is a bulk write and fires no per-item events.
    #[allow(clippy::too_many_arguments)]
    fn change_required(
        &self,
        current: &ItemType,
        account_id: &Id,
        name: Option<String>,
        description: Option<String>,
        add: Vec<String>,
        remove: Vec<String>,
        override_changes: BTreeMap<String, Option<(Value, Attribute)>>,
        dry_run: bool,
    ) -> Result<ItemTypeChange, ItemTypeServiceError> {
        self.ensure_kinds_exist(account_id, &add)?;
        let type_id = i64::from(current.type_id);
        let mut report = RequiredChangeReport {
            dry_run,
            added: add.clone(),
            removed: remove.clone(),
            ..Default::default()
        };
        let mut backfills = Vec::new();
        let mut offenders = Vec::new();

        for key in &add {
            // Type-level defaults for this key across all types, with this change applied to us.
            let mut universe: BTreeMap<i64, Attribute> = BTreeMap::new();
            for (other_type, raw) in self
                .repo
                .overrides_for_key(key, account_id)
                .map_err(ItemTypeServiceError::Repo)?
            {
                if other_type == type_id {
                    continue;
                }
                if let Ok(parsed) = self.defaults.parse_override(account_id, key, &raw) {
                    universe.insert(other_type, parsed);
                }
            }
            match override_changes.get(key) {
                Some(Some((_, parsed))) => {
                    universe.insert(type_id, parsed.clone());
                }
                Some(None) => {}
                None => {
                    if let Some(raw) = current.required_defaults.get(key) {
                        if let Ok(parsed) = self.defaults.parse_override(account_id, key, raw) {
                            universe.insert(type_id, parsed);
                        }
                    }
                }
            }
            let ids: Vec<i64> = universe.keys().copied().collect();
            let kind_default = self
                .defaults
                .kind_default(account_id, key)
                .filter(usable_default);

            let groups = self
                .repo
                .missing_by_override_set(
                    &current.type_id,
                    key,
                    &ids,
                    NON_COMPLIANT_SAMPLE,
                    account_id,
                )
                .map_err(ItemTypeServiceError::Repo)?;
            let mut bad_sets = Vec::new();
            for group in groups.into_iter().filter(|group| group.count > 0) {
                let values: Vec<Attribute> = group
                    .set
                    .iter()
                    .filter_map(|id| universe.get(id).cloned())
                    .collect();
                match effective_default(&values, kind_default.as_ref()) {
                    Some(value) => {
                        *report.backfilled.entry(key.clone()).or_insert(0) += group.count;
                        backfills.push(Backfill {
                            key: key.clone(),
                            universe: ids.clone(),
                            set: group.set,
                            value,
                        });
                    }
                    None => {
                        *report.non_compliant.by_key.entry(key.clone()).or_insert(0) += group.count;
                        let samples = report
                            .non_compliant
                            .sample_ids
                            .entry(key.clone())
                            .or_default();
                        samples.extend(group.sample_ids);
                        samples.sort_unstable();
                        samples.truncate(NON_COMPLIANT_SAMPLE as usize);
                        bad_sets.push(group.set);
                    }
                }
            }
            if !bad_sets.is_empty() {
                offenders.push(Offender {
                    key: key.clone(),
                    universe: ids,
                    sets: bad_sets,
                });
            }
        }
        if !offenders.is_empty() {
            report.non_compliant.total = self
                .repo
                .count_offenders(&current.type_id, &offenders, account_id)
                .map_err(ItemTypeServiceError::Repo)?;
        }

        if dry_run {
            return Ok(ItemTypeChange {
                item_type: current.clone(),
                report,
            });
        }
        if !offenders.is_empty() {
            return Err(ItemTypeServiceError::WouldInvalidate {
                message: would_invalidate_message(&current.name, &report.non_compliant),
                summary: report.non_compliant,
            });
        }

        let mut defaults: BTreeMap<String, Option<Value>> = override_changes
            .into_iter()
            .map(|(key, change)| (key, change.map(|(raw, _)| raw)))
            .collect();
        // A removed requirement takes its type default with it; nothing else to write.
        for key in &remove {
            defaults.remove(key);
        }
        self.repo
            .apply_required_change(
                &current.type_id,
                &RequiredChange {
                    name,
                    description,
                    add,
                    remove,
                    defaults,
                    backfills,
                },
                account_id,
            )
            .map_err(ItemTypeServiceError::Repo)?;
        let item_type = self
            .get_item_type(&current.type_id, account_id)?
            .ok_or(ItemTypeServiceError::NotFound)?;
        Ok(ItemTypeChange { item_type, report })
    }

    pub fn delete_item_type(
        &self,
        type_id: &Id,
        account_id: &Id,
        force: bool,
    ) -> Result<(), ItemTypeServiceError> {
        let current = self.get_mutable_item_type(type_id, account_id)?;
        let current = current.ok_or(ItemTypeServiceError::NotFound)?;

        if !force {
            let count = self
                .repo
                .count_items_for_type(type_id)
                .map_err(ItemTypeServiceError::Repo)?;
            if count > 0 {
                return Err(ItemTypeServiceError::InUse(format!(
                    "Item type '{}' is assigned to {} item(s). Unassign first or pass force=true.",
                    current.name, count
                )));
            }
        }

        let deleted = self
            .repo
            .delete_item_type(type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)?;

        if deleted {
            tracing::info!(account_id = ?account_id, ?type_id, "item type deleted");
            Ok(())
        } else {
            Err(ItemTypeServiceError::NotFound)
        }
    }

    fn get_mutable_item_type(
        &self,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        let current = self
            .repo
            .get_item_type(type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)?;

        if let Some(item_type) = &current {
            if item_type.is_system {
                return Err(ItemTypeServiceError::ReadOnly(
                    "System item types are read-only.".to_string(),
                ));
            }
        }

        Ok(current)
    }
}

fn unique<'a>(keys: impl Iterator<Item = &'a String>) -> Vec<String> {
    let mut seen = HashSet::new();
    keys.filter(|key| seen.insert((*key).clone()))
        .cloned()
        .collect()
}

/// A default can only fill a value in place: not an empty list, and never an item reference
/// (relationships have no default; they also need link syncing).
fn usable_default(attribute: &Attribute) -> bool {
    match attribute {
        Attribute::Scalar(scalar) => !matches!(scalar, AttributeScalar::Item(_)),
        Attribute::List(values) => {
            !values.is_empty() && !values.iter().any(|v| matches!(v, AttributeScalar::Item(_)))
        }
    }
}

fn would_invalidate_message(type_name: &str, summary: &NonCompliantSummary) -> String {
    let keys: Vec<String> = summary
        .by_key
        .iter()
        .map(|(key, count)| {
            let ids: Vec<String> = summary
                .sample_ids
                .get(key)
                .map(|ids| ids.iter().take(5).map(|id| format!("#{id}")).collect())
                .unwrap_or_default();
            format!("'{key}' ({count} item(s), e.g. {})", ids.join(", "))
        })
        .collect();
    format!(
        "Cannot require {} on type '{type_name}': {} existing item(s) lack a value and there is no default to fill it. Nothing was changed. Give the attribute kind a default, or set the value on those items first (dry_run lists them).",
        keys.join(", "),
        summary.total
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repos::item_type::MissingGroup;
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicBool, Ordering};
    use zealot_domain::item_type::{
        AddItemTypeDto, ItemType, ItemTypeRef, ItemTypeSummary, UpdateItemTypeDto,
    };

    #[derive(Debug)]
    struct MockItemTypeRepo {
        item_type: Option<ItemType>,
        item_count: i64,
        deleted: AtomicBool,
        /// Members lacking a key, by key, as the repo would group them.
        missing: HashMap<String, Vec<MissingGroup>>,
        /// Type-level defaults other types set, by key.
        overrides: HashMap<String, Vec<(i64, Value)>>,
        offender_count: i64,
        applied: std::sync::Mutex<Vec<RequiredChange>>,
    }

    impl MockItemTypeRepo {
        fn with_type(item_type: ItemType, item_count: i64) -> Self {
            Self {
                item_type: Some(item_type),
                item_count,
                deleted: AtomicBool::new(false),
                missing: HashMap::new(),
                overrides: HashMap::new(),
                offender_count: 0,
                applied: Default::default(),
            }
        }
        fn empty() -> Self {
            let mut repo = Self::with_type(make_type("Task", false), 0);
            repo.item_type = None;
            repo
        }
    }

    fn make_type(name: &str, is_system: bool) -> ItemType {
        ItemType {
            type_id: Id::try_from(1i64).unwrap(),
            is_system,
            name: name.to_string(),
            description: String::new(),
            required_attributes: vec![],
            required_defaults: BTreeMap::new(),
        }
    }

    fn account_id() -> Id {
        Id::try_from(1i64).unwrap()
    }
    fn type_id() -> Id {
        Id::try_from(1i64).unwrap()
    }

    impl ItemTypeRepo for MockItemTypeRepo {
        fn get_item_types(&self, _: &Id) -> Result<Vec<ItemType>, RepoError> {
            Ok(vec![])
        }
        fn get_item_type_summaries(&self, _: &Id) -> Result<Vec<ItemTypeSummary>, RepoError> {
            Ok(vec![])
        }
        fn get_item_type(&self, _: &Id, _: &Id) -> Result<Option<ItemType>, RepoError> {
            Ok(self.item_type.clone())
        }
        fn get_item_type_by_name(&self, _: &str, _: &Id) -> Result<Option<ItemType>, RepoError> {
            Ok(None)
        }
        fn get_item_type_refs_for_items(
            &self,
            _: &Vec<Id>,
            _: &Id,
        ) -> Result<HashMap<Id, Vec<ItemTypeRef>>, RepoError> {
            Ok(HashMap::new())
        }
        fn get_item_ids_for_type_name(&self, _: &str, _: &Id) -> Result<Vec<Id>, RepoError> {
            Ok(vec![])
        }
        fn add_item_type(&self, _: &AddItemTypeDto, _: &Id) -> Result<Option<ItemType>, RepoError> {
            Ok(self.item_type.clone())
        }
        fn update_item_type(
            &self,
            _: &UpdateItemTypeDto,
            _: &Id,
        ) -> Result<Option<ItemType>, RepoError> {
            Ok(None)
        }
        fn count_items_for_type(&self, _: &Id) -> Result<i64, RepoError> {
            Ok(self.item_count)
        }
        fn delete_item_type(&self, _: &Id, _: &Id) -> Result<bool, RepoError> {
            self.deleted.store(true, Ordering::Relaxed);
            Ok(true)
        }
        fn add_attr_kinds_to_item_type(
            &self,
            _: &Vec<String>,
            _: &Id,
            _: &Id,
        ) -> Result<(), RepoError> {
            Ok(())
        }
        fn remove_attr_kinds_from_item_type(
            &self,
            _: &Vec<String>,
            _: &Id,
            _: &Id,
        ) -> Result<(), RepoError> {
            Ok(())
        }
        fn assign_item_types(&self, _: &Vec<String>, _: &Id, _: &Id) -> Result<(), RepoError> {
            Ok(())
        }
        fn unassign_item_types(&self, _: &Vec<String>, _: &Id, _: &Id) -> Result<(), RepoError> {
            Ok(())
        }
        fn missing_by_override_set(
            &self,
            _: &Id,
            key: &str,
            _: &[i64],
            _: i64,
            _: &Id,
        ) -> Result<Vec<MissingGroup>, RepoError> {
            Ok(self.missing.get(key).cloned().unwrap_or_default())
        }
        fn count_offenders(
            &self,
            _: &Id,
            _: &[crate::repos::item_type::Offender],
            _: &Id,
        ) -> Result<i64, RepoError> {
            Ok(self.offender_count)
        }
        fn overrides_for_key(&self, key: &str, _: &Id) -> Result<Vec<(i64, Value)>, RepoError> {
            Ok(self.overrides.get(key).cloned().unwrap_or_default())
        }
        fn apply_required_change(
            &self,
            _: &Id,
            change: &RequiredChange,
            _: &Id,
        ) -> Result<BTreeMap<String, i64>, RepoError> {
            self.applied.lock().unwrap().push(change.clone());
            Ok(Default::default())
        }
        fn list_non_compliant(
            &self,
            _: &Id,
            _: i64,
            _: i64,
            _: &Id,
        ) -> Result<NonCompliantPageDto, RepoError> {
            Ok(NonCompliantPageDto {
                total: 0,
                items: vec![],
            })
        }
    }

    #[test]
    fn delete_unused_type_succeeds() {
        let repo = Arc::new(MockItemTypeRepo::with_type(make_type("Task", false), 0));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let result = svc.delete_item_type(&type_id(), &account_id(), false);
        assert!(result.is_ok());
        assert!(repo.deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_type_without_force_returns_in_use_error() {
        let repo = Arc::new(MockItemTypeRepo::with_type(make_type("Task", false), 37));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let result = svc.delete_item_type(&type_id(), &account_id(), false);
        assert!(matches!(result, Err(ItemTypeServiceError::InUse(_))));
        if let Err(ItemTypeServiceError::InUse(msg)) = result {
            assert!(msg.contains("37"));
            assert!(msg.contains("Task"));
        }
        assert!(!repo.deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_type_with_force_proceeds() {
        let repo = Arc::new(MockItemTypeRepo::with_type(make_type("Task", false), 37));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let result = svc.delete_item_type(&type_id(), &account_id(), true);
        assert!(result.is_ok());
        assert!(repo.deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_system_type_returns_readonly_regardless_of_force() {
        let repo = Arc::new(MockItemTypeRepo::with_type(make_type("System", true), 0));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let result = svc.delete_item_type(&type_id(), &account_id(), true);
        assert!(matches!(result, Err(ItemTypeServiceError::ReadOnly(_))));
        assert!(!repo.deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_nonexistent_type_returns_not_found() {
        let repo = Arc::new(MockItemTypeRepo::empty());
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let result = svc.delete_item_type(&type_id(), &account_id(), false);
        assert!(matches!(result, Err(ItemTypeServiceError::NotFound)));
    }

    fn int(n: i64) -> Attribute {
        Attribute::Scalar(AttributeScalar::Integer(n))
    }

    /// Kind default and the kind's validation, by key; integers only.
    #[derive(Debug, Default)]
    struct FakeKinds {
        kind_default: Option<Attribute>,
    }
    impl RequirementDefaults for FakeKinds {
        fn kind_default(&self, _: &Id, _: &str) -> Option<Attribute> {
            self.kind_default.clone()
        }
        fn parse_override(&self, _: &Id, key: &str, raw: &Value) -> Result<Attribute, String> {
            match raw.as_i64() {
                Some(n) if (0..=10).contains(&n) => Ok(int(n)),
                _ => Err(format!("invalid default for '{key}'")),
            }
        }
    }

    fn require(keys: &[&str], dry_run: bool) -> UpdateItemTypeDto {
        UpdateItemTypeDto {
            type_id: 1,
            name: None,
            description: None,
            required_attributes: Some(keys.iter().map(|k| k.to_string()).collect()),
            required_defaults: None,
            dry_run: Some(dry_run),
        }
    }

    fn group(set: &[i64], count: i64) -> MissingGroup {
        MissingGroup {
            set: set.to_vec(),
            count,
            sample_ids: vec![7, 9],
        }
    }

    fn service(repo: &Arc<MockItemTypeRepo>, kind_default: Option<Attribute>) -> ItemTypeService {
        ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>))
            .with_defaults(Arc::new(FakeKinds { kind_default }))
    }

    fn priority_repo(groups: Vec<MissingGroup>) -> MockItemTypeRepo {
        let mut repo = MockItemTypeRepo::with_type(make_type("Task", false), 5);
        repo.missing.insert("Priority".into(), groups);
        repo
    }

    #[test]
    fn kind_default_backfills_members_with_no_type_default() {
        let repo = Arc::new(priority_repo(vec![group(&[], 3)]));
        let change = service(&repo, Some(int(3)))
            .update_item_type(&require(&["Priority"], false), &account_id())
            .unwrap()
            .unwrap();
        assert_eq!(change.report.backfilled["Priority"], 3);
        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied[0].backfills.len(), 1);
        assert_eq!(applied[0].backfills[0].value, int(3));
    }

    #[test]
    fn type_default_beats_kind_default_for_its_members() {
        let mut repo = priority_repo(vec![group(&[1], 2)]);
        repo.item_type.as_mut().unwrap().required_defaults =
            BTreeMap::from([("Priority".to_string(), json!(5))]);
        let repo = Arc::new(repo);
        // Our own default is on the stored type; the change re-reads it.
        let mut dto = require(&["Priority"], false);
        dto.required_defaults = Some(BTreeMap::from([("Priority".to_string(), json!(5))]));
        service(&repo, Some(int(3)))
            .update_item_type(&dto, &account_id())
            .unwrap();
        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied[0].backfills[0].value, int(5));
        assert_eq!(applied[0].backfills[0].set, [1]);
        assert_eq!(applied[0].defaults["Priority"], Some(json!(5)));
    }

    #[test]
    fn members_whose_other_type_disagrees_get_the_kind_default() {
        // Types 1 (this one, default 5) and 2 (default 7): one group has both, one only us.
        let mut repo = priority_repo(vec![group(&[1], 2), group(&[1, 2], 4)]);
        repo.overrides
            .insert("Priority".into(), vec![(2, json!(7))]);
        let repo = Arc::new(repo);
        let mut dto = require(&["Priority"], false);
        dto.required_defaults = Some(BTreeMap::from([("Priority".to_string(), json!(5))]));
        service(&repo, Some(int(3)))
            .update_item_type(&dto, &account_id())
            .unwrap();
        let applied = repo.applied.lock().unwrap();
        let by_set: HashMap<Vec<i64>, Attribute> = applied[0]
            .backfills
            .iter()
            .map(|b| (b.set.clone(), b.value.clone()))
            .collect();
        assert_eq!(by_set[&vec![1]], int(5));
        assert_eq!(
            by_set[&vec![1, 2]],
            int(3),
            "conflict falls back to the kind default"
        );
    }

    #[test]
    fn agreeing_type_defaults_win_over_the_kind_default() {
        let mut repo = priority_repo(vec![group(&[1, 2], 4)]);
        repo.overrides
            .insert("Priority".into(), vec![(2, json!(5))]);
        let repo = Arc::new(repo);
        let mut dto = require(&["Priority"], false);
        dto.required_defaults = Some(BTreeMap::from([("Priority".to_string(), json!(5))]));
        service(&repo, Some(int(3)))
            .update_item_type(&dto, &account_id())
            .unwrap();
        assert_eq!(repo.applied.lock().unwrap()[0].backfills[0].value, int(5));
    }

    #[test]
    fn conflict_with_no_kind_default_rejects_the_change() {
        let mut repo = priority_repo(vec![group(&[1, 2], 4)]);
        repo.overrides
            .insert("Priority".into(), vec![(2, json!(7))]);
        repo.offender_count = 4;
        let repo = Arc::new(repo);
        let mut dto = require(&["Priority"], false);
        dto.required_defaults = Some(BTreeMap::from([("Priority".to_string(), json!(5))]));
        let result = service(&repo, None).update_item_type(&dto, &account_id());
        let Err(ItemTypeServiceError::WouldInvalidate { summary, .. }) = result else {
            panic!("expected WouldInvalidate, got {result:?}");
        };
        assert_eq!(summary.total, 4);
        assert_eq!(summary.by_key["Priority"], 4);
        assert!(repo.applied.lock().unwrap().is_empty());
    }

    #[test]
    fn no_default_anywhere_rejects_and_dry_run_reports_without_writing() {
        let mut repo = priority_repo(vec![group(&[], 2)]);
        repo.offender_count = 2;
        let repo = Arc::new(repo);
        let svc = service(&repo, None);
        assert!(matches!(
            svc.update_item_type(&require(&["Priority"], false), &account_id()),
            Err(ItemTypeServiceError::WouldInvalidate { .. })
        ));
        let dry = svc
            .update_item_type(&require(&["Priority"], true), &account_id())
            .unwrap()
            .unwrap();
        assert!(dry.report.dry_run);
        assert_eq!(dry.report.non_compliant.total, 2);
        assert!(repo.applied.lock().unwrap().is_empty());
    }

    #[test]
    fn invalid_or_unrequired_type_defaults_are_rejected_before_any_write() {
        let repo = Arc::new(priority_repo(vec![]));
        let svc = service(&repo, Some(int(3)));
        let mut out_of_range = require(&["Priority"], false);
        out_of_range.required_defaults =
            Some(BTreeMap::from([("Priority".to_string(), json!(99))]));
        assert!(matches!(
            svc.update_item_type(&out_of_range, &account_id()),
            Err(ItemTypeServiceError::InvalidDefault(_))
        ));
        let mut not_required = require(&["Priority"], false);
        not_required.required_defaults = Some(BTreeMap::from([("Other".to_string(), json!(1))]));
        assert!(matches!(
            svc.update_item_type(&not_required, &account_id()),
            Err(ItemTypeServiceError::InvalidDefault(_))
        ));
        assert!(repo.applied.lock().unwrap().is_empty());
    }

    #[test]
    fn null_clears_a_type_default_and_leaves_items_alone() {
        let mut repo = priority_repo(vec![]);
        repo.item_type.as_mut().unwrap().required_attributes = vec!["Priority".into()];
        repo.item_type.as_mut().unwrap().required_defaults =
            BTreeMap::from([("Priority".to_string(), json!(5))]);
        let repo = Arc::new(repo);
        let dto = UpdateItemTypeDto {
            type_id: 1,
            name: None,
            description: None,
            required_attributes: None,
            required_defaults: Some(BTreeMap::from([("Priority".to_string(), Value::Null)])),
            dry_run: None,
        };
        service(&repo, Some(int(3)))
            .update_item_type(&dto, &account_id())
            .unwrap();
        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied[0].defaults["Priority"], None);
        assert!(applied[0].backfills.is_empty());
    }

    #[test]
    fn already_required_keys_are_not_rechecked() {
        let mut repo = priority_repo(vec![group(&[], 2)]);
        repo.item_type.as_mut().unwrap().required_attributes = vec!["Priority".into()];
        let repo = Arc::new(repo);
        let change = service(&repo, None)
            .update_item_type(&require(&["Priority"], false), &account_id())
            .unwrap()
            .unwrap();
        assert!(change.report.added.is_empty());
    }
}
