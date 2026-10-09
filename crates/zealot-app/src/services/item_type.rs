use std::{collections::HashSet, sync::Arc};

use zealot_domain::{
    attribute::{Attribute, AttributeScalar},
    common::id::Id,
    item_type::{
        AddItemTypeDto, ItemType, ItemTypeSummary, NON_COMPLIANT_SAMPLE, NonCompliantPageDto,
        NonCompliantSummary, RequiredChangeReport, UpdateItemTypeDto,
    },
};

use crate::repos::{
    attribute::AttributeRepo,
    common::RepoError,
    item_type::{ItemTypeRepo, RequiredChange},
};

/// Where a default value for a newly required attribute comes from. The type is passed so a
/// later type-level override can resolve as `type default ?? kind default`.
pub trait RequirementDefaults: std::fmt::Debug + Send + Sync {
    fn default_for(&self, account_id: &Id, item_type: &ItemType, key: &str) -> Option<Attribute>;
}

/// No attribute has a default, so every requirement change that leaves items without a value
/// is rejected. Used until attribute kind defaults (Z45) are wired in.
#[derive(Debug)]
pub struct NoDefaults;

impl RequirementDefaults for NoDefaults {
    fn default_for(&self, _: &Id, _: &ItemType, _: &str) -> Option<Attribute> {
        None
    }
}

/// Defaults from the attribute kind (`AttributeKind.default`). Resolution is `type default ??
/// kind default`; type-level defaults (Z218) slot in at `type_default`.
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

    /// No type-level defaults exist yet.
    fn type_default(&self, _item_type: &ItemType, _key: &str) -> Option<Attribute> {
        None
    }
}

impl RequirementDefaults for KindDefaults {
    fn default_for(&self, account_id: &Id, item_type: &ItemType, key: &str) -> Option<Attribute> {
        self.type_default(item_type, key).or_else(|| {
            match self.attributes.get_attribute_kind(key, account_id) {
                Ok(kind) => kind.and_then(|kind| kind.default),
                Err(err) => {
                    // Treated as no default, so the change is rejected rather than half-filled.
                    tracing::error!(%key, "attribute kind lookup failed: {err}");
                    None
                }
            }
        })
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
    /// A requirement change would leave existing items without a value and has no default to
    /// give them. Nothing was changed.
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
        let item_type = self
            .repo
            .add_item_type(dto, account_id)
            .map_err(ItemTypeServiceError::Repo)?;
        if let Some(ref t) = item_type {
            tracing::info!(account_id = ?account_id, type_id = ?t.type_id, name = %t.name, "item type created");
        }
        Ok(item_type)
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
        let change = self.change_required(
            &current,
            account_id,
            dto.name.clone(),
            dto.description.clone(),
            add,
            remove,
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
            .change_required(&current, account_id, None, None, add, Vec::new(), dry_run)?
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

    /// The one place a type's requirements change. Existing members are checked first; an added
    /// attribute without a default that some members lack rejects the whole change. Otherwise the
    /// links, rename and default backfill commit in one transaction. `dry_run` stops before that
    /// and reports the same numbers, including would-be offenders, without an error.
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
        dry_run: bool,
    ) -> Result<ItemTypeChange, ItemTypeServiceError> {
        let missing = if add.is_empty() {
            Vec::new()
        } else {
            self.repo
                .count_missing_required(&current.type_id, &add, NON_COMPLIANT_SAMPLE, account_id)
                .map_err(ItemTypeServiceError::Repo)?
        };

        let mut report = RequiredChangeReport {
            dry_run,
            added: add.clone(),
            removed: remove.clone(),
            ..Default::default()
        };
        let mut backfills = Vec::new();
        let mut offending = Vec::new();
        for entry in missing.into_iter().filter(|entry| entry.count > 0) {
            match self
                .defaults
                .default_for(account_id, current, &entry.key)
                .filter(usable_default)
            {
                Some(default) => {
                    report.backfilled.insert(entry.key.clone(), entry.count);
                    backfills.push((entry.key, default));
                }
                None => {
                    report
                        .non_compliant
                        .by_key
                        .insert(entry.key.clone(), entry.count);
                    report
                        .non_compliant
                        .sample_ids
                        .insert(entry.key.clone(), entry.sample_ids);
                    offending.push(entry.key);
                }
            }
        }
        if !offending.is_empty() {
            report.non_compliant.total = self
                .repo
                .count_items_missing_any(&current.type_id, &offending, account_id)
                .map_err(ItemTypeServiceError::Repo)?;
        }

        if dry_run {
            return Ok(ItemTypeChange {
                item_type: current.clone(),
                report,
            });
        }
        if !offending.is_empty() {
            return Err(ItemTypeServiceError::WouldInvalidate {
                message: would_invalidate_message(&current.name, &report.non_compliant),
                summary: report.non_compliant,
            });
        }

        self.repo
            .apply_required_change(
                &current.type_id,
                &RequiredChange {
                    name,
                    description,
                    add,
                    remove,
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
    use crate::repos::item_type::MissingRequired;
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
        missing: HashMap<String, MissingRequired>,
        distinct_missing: i64,
        applied: std::sync::Mutex<Vec<RequiredChange>>,
    }

    impl MockItemTypeRepo {
        fn with_type(item_type: ItemType, item_count: i64) -> Self {
            Self {
                item_type: Some(item_type),
                item_count,
                deleted: AtomicBool::new(false),
                missing: HashMap::new(),
                distinct_missing: 0,
                applied: Default::default(),
            }
        }
        fn empty() -> Self {
            Self {
                item_type: None,
                item_count: 0,
                deleted: AtomicBool::new(false),
                missing: HashMap::new(),
                distinct_missing: 0,
                applied: Default::default(),
            }
        }
    }

    fn make_type(name: &str, is_system: bool) -> ItemType {
        ItemType {
            type_id: Id::try_from(1i64).unwrap(),
            is_system,
            name: name.to_string(),
            description: String::new(),
            required_attributes: vec![],
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
            Ok(None)
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
        fn count_missing_required(
            &self,
            _: &Id,
            keys: &[String],
            _: i64,
            _: &Id,
        ) -> Result<Vec<MissingRequired>, RepoError> {
            Ok(keys
                .iter()
                .filter_map(|key| self.missing.get(key))
                .cloned()
                .collect())
        }
        fn count_items_missing_any(&self, _: &Id, _: &[String], _: &Id) -> Result<i64, RepoError> {
            Ok(self.distinct_missing)
        }
        fn apply_required_change(
            &self,
            _: &Id,
            change: &RequiredChange,
            _: &Id,
        ) -> Result<std::collections::BTreeMap<String, i64>, RepoError> {
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

    fn require_update(keys: &[&str], dry_run: bool) -> UpdateItemTypeDto {
        UpdateItemTypeDto {
            type_id: 1,
            name: None,
            description: None,
            required_attributes: Some(keys.iter().map(|k| k.to_string()).collect()),
            dry_run: Some(dry_run),
        }
    }

    fn with_missing(key: &str, count: i64) -> MockItemTypeRepo {
        let mut repo = MockItemTypeRepo::with_type(make_type("Task", false), count);
        repo.missing.insert(
            key.to_string(),
            MissingRequired {
                key: key.to_string(),
                count,
                sample_ids: vec![7, 9],
            },
        );
        repo.distinct_missing = count;
        repo
    }

    #[derive(Debug)]
    struct FixedDefault(Attribute);
    impl RequirementDefaults for FixedDefault {
        fn default_for(&self, _: &Id, _: &ItemType, _: &str) -> Option<Attribute> {
            Some(self.0.clone())
        }
    }

    #[test]
    fn default_that_is_an_item_reference_or_empty_list_does_not_count() {
        for default in [
            Attribute::Scalar(AttributeScalar::Item(Id::try_from(3i64).unwrap())),
            Attribute::List(vec![]),
        ] {
            let repo = Arc::new(with_missing("Owner", 2));
            let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>))
                .with_defaults(Arc::new(FixedDefault(default)));
            let result = svc.update_item_type(&require_update(&["Owner"], false), &account_id());
            assert!(matches!(
                result,
                Err(ItemTypeServiceError::WouldInvalidate { .. })
            ));
            assert!(repo.applied.lock().unwrap().is_empty());
        }
    }

    #[test]
    fn usable_default_backfills_in_one_repo_call() {
        let repo = Arc::new(with_missing("Owner", 2));
        let default = Attribute::Scalar(AttributeScalar::Text("me".into()));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>))
            .with_defaults(Arc::new(FixedDefault(default.clone())));
        let change = svc
            .update_item_type(&require_update(&["Owner"], false), &account_id())
            .unwrap()
            .unwrap();
        assert_eq!(change.report.backfilled["Owner"], 2);
        let applied = repo.applied.lock().unwrap();
        assert_eq!(applied.len(), 1);
        assert_eq!(applied[0].add, ["Owner"]);
        assert_eq!(applied[0].backfills, [("Owner".to_string(), default)]);
    }

    #[test]
    fn dry_run_never_calls_the_write_path() {
        let repo = Arc::new(with_missing("Owner", 2));
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let change = svc
            .update_item_type(&require_update(&["Owner"], true), &account_id())
            .unwrap()
            .unwrap();
        assert!(change.report.dry_run);
        assert_eq!(change.report.non_compliant.total, 2);
        assert!(repo.applied.lock().unwrap().is_empty());
    }

    #[test]
    fn already_required_keys_are_not_rechecked() {
        let mut repo = with_missing("Owner", 2);
        repo.item_type.as_mut().unwrap().required_attributes = vec!["Owner".into()];
        let repo = Arc::new(repo);
        let svc = ItemTypeService::new(&(repo.clone() as Arc<dyn ItemTypeRepo>));
        let change = svc
            .update_item_type(&require_update(&["Owner"], false), &account_id())
            .unwrap()
            .unwrap();
        assert!(change.report.added.is_empty());
    }
}
