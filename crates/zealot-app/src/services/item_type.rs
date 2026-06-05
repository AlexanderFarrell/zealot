use std::{collections::HashSet, sync::Arc};

use zealot_domain::{
    common::id::Id,
    item_type::{AddItemTypeDto, ItemType, ItemTypeSummary, UpdateItemTypeDto},
};

use crate::repos::{common::RepoError, item_type::ItemTypeRepo};

#[derive(Debug, Clone)]
pub struct ItemTypeService {
    repo: Arc<dyn ItemTypeRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum ItemTypeServiceError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    ReadOnly(String),
    #[error("{0}")]
    InUse(String),
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl ItemTypeService {
    pub fn new(repo: &Arc<dyn ItemTypeRepo>) -> Self {
        Self { repo: repo.clone() }
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
        self.repo
            .add_item_type(dto, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn update_item_type(
        &self,
        dto: &UpdateItemTypeDto,
        account_id: &Id,
    ) -> Result<Option<ItemType>, ItemTypeServiceError> {
        let type_id = Id::try_from(dto.type_id).map_err(|e| {
            ItemTypeServiceError::Repo(RepoError::DatabaseError { err: e.to_string() })
        })?;
        let current = self.get_mutable_item_type(&type_id, account_id)?;

        let Some(current) = current else {
            return Ok(None);
        };

        let updated = self
            .repo
            .update_item_type(dto, account_id)
            .map_err(ItemTypeServiceError::Repo)?;

        if let Some(required_attributes) = &dto.required_attributes {
            let current_required: HashSet<String> =
                current.required_attributes.into_iter().collect();
            let desired_required: HashSet<String> = required_attributes.iter().cloned().collect();

            let to_add: Vec<String> = desired_required
                .difference(&current_required)
                .cloned()
                .collect();
            let to_remove: Vec<String> = current_required
                .difference(&desired_required)
                .cloned()
                .collect();

            if !to_add.is_empty() {
                self.repo
                    .add_attr_kinds_to_item_type(&to_add, &type_id, account_id)
                    .map_err(ItemTypeServiceError::Repo)?;
            }

            if !to_remove.is_empty() {
                self.repo
                    .remove_attr_kinds_from_item_type(&to_remove, &type_id, account_id)
                    .map_err(ItemTypeServiceError::Repo)?;
            }

            return self.get_item_type(&type_id, account_id);
        }

        Ok(updated)
    }

    pub fn add_attr_kinds_to_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<(), ItemTypeServiceError> {
        let current = self.get_mutable_item_type(type_id, account_id)?;
        if current.is_none() {
            return Err(ItemTypeServiceError::NotFound);
        }
        self.repo
            .add_attr_kinds_to_item_type(attr_kind_keys, type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)
    }

    pub fn remove_attr_kinds_from_item_type(
        &self,
        attr_kind_keys: &Vec<String>,
        type_id: &Id,
        account_id: &Id,
    ) -> Result<(), ItemTypeServiceError> {
        let current = self.get_mutable_item_type(type_id, account_id)?;
        if current.is_none() {
            return Err(ItemTypeServiceError::NotFound);
        }
        self.repo
            .remove_attr_kinds_from_item_type(attr_kind_keys, type_id, account_id)
            .map_err(ItemTypeServiceError::Repo)
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

#[cfg(test)]
mod tests {
    use super::*;
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
    }

    impl MockItemTypeRepo {
        fn with_type(item_type: ItemType, item_count: i64) -> Self {
            Self {
                item_type: Some(item_type),
                item_count,
                deleted: AtomicBool::new(false),
            }
        }
        fn empty() -> Self {
            Self {
                item_type: None,
                item_count: 0,
                deleted: AtomicBool::new(false),
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
}
