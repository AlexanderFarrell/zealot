use std::sync::Arc;

use zealot_domain::{
    attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto},
    common::id::Id,
};

use crate::repos::{attribute::AttributeRepo, common::RepoError};

#[derive(Debug)]
pub struct AttributeService {
    repo: Arc<dyn AttributeRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum AttributeServiceError {
    #[error("not found")]
    NotFound,
    #[error("{0}")]
    InUse(String),
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl AttributeService {
    pub fn new(repo: &Arc<dyn AttributeRepo>) -> Self {
        Self { repo: repo.clone() }
    }

    pub fn get_kind_by_key(
        &self,
        key: &str,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo
            .get_attribute_kind(key, account_id)
            .map_err(AttributeServiceError::Repo)
    }

    pub fn get_kind_by_id(
        &self,
        kind_id: &Id,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        self.repo
            .get_attribute_kind_by_id(kind_id, account_id)
            .map_err(AttributeServiceError::Repo)
    }

    pub fn get_kinds_for_user(
        &self,
        account_id: &Id,
    ) -> Result<Vec<AttributeKind>, AttributeServiceError> {
        self.repo
            .get_attribute_kinds_for_user(account_id)
            .map(|m| m.into_values().collect())
            .map_err(AttributeServiceError::Repo)
    }

    pub fn add_attribute_kind(
        &self,
        dto: &AddAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        let kind = self
            .repo
            .add_attribute_kind(dto, account_id)
            .map_err(AttributeServiceError::Repo)?;
        if let Some(ref k) = kind {
            tracing::info!(account_id = ?account_id, key = %k.key, "attribute kind created");
        }
        Ok(kind)
    }

    pub fn update_attribute_kind(
        &self,
        dto: &UpdateAttributeKindDto,
        account_id: &Id,
    ) -> Result<Option<AttributeKind>, AttributeServiceError> {
        let kind = self
            .repo
            .update_attribute_kind(dto, account_id)
            .map_err(AttributeServiceError::Repo)?;
        if let Some(ref k) = kind {
            tracing::info!(account_id = ?account_id, key = %k.key, "attribute kind updated");
        }
        Ok(kind)
    }

    pub fn delete_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
        force: bool,
    ) -> Result<(), AttributeServiceError> {
        if !force {
            let count = self.repo.count_attribute_values_for_kind(key, account_id)?;
            if count > 0 {
                return Err(AttributeServiceError::InUse(format!(
                    "Attribute kind '{}' is used by {} item(s). Delete those values first or pass force=true.",
                    key, count
                )));
            }
        } else {
            self.repo
                .delete_attribute_values_for_kind(key, account_id)?;
        }
        self.repo
            .delete_attribute_kind(key, account_id)
            .map_err(AttributeServiceError::Repo)?;
        tracing::info!(account_id = ?account_id, %key, force, "attribute kind deleted");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use zealot_domain::attribute::{AddAttributeKindDto, AttributeKind, UpdateAttributeKindDto};

    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug)]
    struct MockAttributeRepo {
        value_count: i64,
        values_deleted: AtomicBool,
        kind_deleted: AtomicBool,
    }

    impl MockAttributeRepo {
        fn new(value_count: i64) -> Self {
            Self {
                value_count,
                values_deleted: AtomicBool::new(false),
                kind_deleted: AtomicBool::new(false),
            }
        }
    }

    impl AttributeRepo for MockAttributeRepo {
        fn get_attribute_kind(
            &self,
            _key: &str,
            _account_id: &Id,
        ) -> Result<Option<AttributeKind>, RepoError> {
            Ok(None)
        }
        fn get_attribute_kind_by_id(
            &self,
            _id: &Id,
            _account_id: &Id,
        ) -> Result<Option<AttributeKind>, RepoError> {
            Ok(None)
        }
        fn get_attribute_kinds_for_user(
            &self,
            _account_id: &Id,
        ) -> Result<HashMap<String, AttributeKind>, RepoError> {
            Ok(HashMap::new())
        }
        fn add_attribute_kind(
            &self,
            _dto: &AddAttributeKindDto,
            _account_id: &Id,
        ) -> Result<Option<AttributeKind>, RepoError> {
            Ok(None)
        }
        fn update_attribute_kind(
            &self,
            _dto: &UpdateAttributeKindDto,
            _account_id: &Id,
        ) -> Result<Option<AttributeKind>, RepoError> {
            Ok(None)
        }
        fn count_attribute_values_for_kind(
            &self,
            _key: &str,
            _account_id: &Id,
        ) -> Result<i64, RepoError> {
            Ok(self.value_count)
        }
        fn delete_attribute_values_for_kind(
            &self,
            _key: &str,
            _account_id: &Id,
        ) -> Result<(), RepoError> {
            self.values_deleted.store(true, Ordering::Relaxed);
            Ok(())
        }
        fn delete_attribute_kind(&self, _key: &str, _account_id: &Id) -> Result<(), RepoError> {
            self.kind_deleted.store(true, Ordering::Relaxed);
            Ok(())
        }
    }

    fn account_id() -> Id {
        Id::try_from(1i64).unwrap()
    }

    #[test]
    fn delete_unused_kind_succeeds() {
        let repo = Arc::new(MockAttributeRepo::new(0));
        let svc = AttributeService::new(&(repo.clone() as Arc<dyn AttributeRepo>));
        let result = svc.delete_attribute_kind("Status", &account_id(), false);
        assert!(result.is_ok());
        assert!(repo.kind_deleted.load(Ordering::Relaxed));
        assert!(!repo.values_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_kind_without_force_returns_in_use_error() {
        let repo = Arc::new(MockAttributeRepo::new(14));
        let svc = AttributeService::new(&(repo.clone() as Arc<dyn AttributeRepo>));
        let result = svc.delete_attribute_kind("Status", &account_id(), false);
        assert!(matches!(result, Err(AttributeServiceError::InUse(_))));
        if let Err(AttributeServiceError::InUse(msg)) = result {
            assert!(msg.contains("14"));
            assert!(msg.contains("Status"));
        }
        assert!(!repo.kind_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_kind_with_force_purges_values_and_deletes_kind() {
        let repo = Arc::new(MockAttributeRepo::new(14));
        let svc = AttributeService::new(&(repo.clone() as Arc<dyn AttributeRepo>));
        let result = svc.delete_attribute_kind("Status", &account_id(), true);
        assert!(result.is_ok());
        assert!(repo.values_deleted.load(Ordering::Relaxed));
        assert!(repo.kind_deleted.load(Ordering::Relaxed));
    }
}
