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
    #[error("{0}")]
    ReadOnly(String),
    #[error("{0}")]
    InvalidDefault(String),
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
        self.check_default(&dto.key, &dto.base_type, &dto.config)?;
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
        let kind_id = Id::try_from(dto.kind_id).map_err(|_| AttributeServiceError::NotFound)?;
        let current = self.repo.get_attribute_kind_by_id(&kind_id, account_id)?;
        ensure_mutable(current.as_ref())?;
        // A new config replaces the old one whole, so its `default` is checked against the kind it will belong to.
        if let (Some(config), Some(current)) = (&dto.config, &current) {
            let key = dto.key.as_deref().unwrap_or(&current.key);
            let base_type = dto
                .base_type
                .as_deref()
                .unwrap_or_else(|| current.base_type.as_db_str());
            self.check_default(key, base_type, config)?;
        }
        let kind = self
            .repo
            .update_attribute_kind(dto, account_id)
            .map_err(AttributeServiceError::Repo)?;
        if let Some(ref k) = kind {
            tracing::info!(account_id = ?account_id, key = %k.key, "attribute kind updated");
        }
        Ok(kind)
    }

    /// Rejects a kind config whose `default` is not a valid value for the kind.
    fn check_default(
        &self,
        key: &str,
        base_type: &str,
        config: &serde_json::Value,
    ) -> Result<(), AttributeServiceError> {
        let kind = self.repo.preview_attribute_kind(key, base_type, config)?;
        kind.parse_default(config)
            .map(|_| ())
            .map_err(|err| AttributeServiceError::InvalidDefault(format!("Invalid default: {err}")))
    }

    pub fn delete_attribute_kind(
        &self,
        key: &str,
        account_id: &Id,
        force: bool,
    ) -> Result<(), AttributeServiceError> {
        // Check before any value purge: a forced delete must never erase values of a system key.
        let current = self.repo.get_attribute_kind(key, account_id)?;
        ensure_mutable(current.as_ref())?;
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

/// Rejects changes to a system attribute kind. System kinds (`Status`, `Parent`, `Date`, ...) are shared
/// by every account and Zealot features depend on them, so they change only through migrations.
fn ensure_mutable(kind: Option<&AttributeKind>) -> Result<(), AttributeServiceError> {
    match kind {
        Some(kind) if kind.is_system => Err(AttributeServiceError::ReadOnly(format!(
            "System attribute kind '{}' is read-only.",
            kind.key
        ))),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use zealot_domain::attribute::{
        AddAttributeKindDto, AttributeBaseScalarType, AttributeBaseType, AttributeKind,
        AttributeKindSpec, UpdateAttributeKindDto,
    };

    use std::sync::atomic::{AtomicBool, Ordering};

    /// Records which writes the service reached, and serves one stored kind for lookups.
    #[derive(Debug)]
    struct MockAttributeRepo {
        kind: Option<AttributeKind>,
        value_count: i64,
        kind_updated: AtomicBool,
        values_deleted: AtomicBool,
        kind_deleted: AtomicBool,
    }

    impl MockAttributeRepo {
        fn new(kind: Option<AttributeKind>, value_count: i64) -> Self {
            Self {
                kind,
                value_count,
                kind_updated: AtomicBool::new(false),
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
            Ok(self.kind.clone())
        }
        fn get_attribute_kind_by_id(
            &self,
            _id: &Id,
            _account_id: &Id,
        ) -> Result<Option<AttributeKind>, RepoError> {
            Ok(self.kind.clone())
        }
        fn get_attribute_kinds_for_user(
            &self,
            _account_id: &Id,
        ) -> Result<HashMap<String, AttributeKind>, RepoError> {
            Ok(HashMap::new())
        }
        /// Stands in for the repo's config parsing with a 1-10 integer kind.
        fn preview_attribute_kind(
            &self,
            key: &str,
            _base_type: &str,
            _config: &serde_json::Value,
        ) -> Result<AttributeKind, RepoError> {
            let mut kind = make_kind(key, false);
            kind.base_type = AttributeBaseType::Scalar(AttributeBaseScalarType::Integer);
            kind.spec = AttributeKindSpec::Integer {
                min: Some(1),
                max: Some(10),
            };
            Ok(kind)
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
            self.kind_updated.store(true, Ordering::Relaxed);
            Ok(self.kind.clone())
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

    fn make_kind(key: &str, is_system: bool) -> AttributeKind {
        AttributeKind {
            kind_id: Id::try_from(7i64).unwrap(),
            default: None,
            is_system,
            key: key.to_string(),
            description: String::new(),
            base_type: AttributeBaseType::Scalar(AttributeBaseScalarType::Text),
            spec: AttributeKindSpec::Text {
                min_len: None,
                max_len: None,
                pattern: None,
            },
        }
    }

    fn rename_dto() -> UpdateAttributeKindDto {
        UpdateAttributeKindDto {
            kind_id: 7,
            key: Some("Renamed".to_string()),
            description: None,
            base_type: Some("integer".to_string()),
            config: None,
        }
    }

    fn service(repo: &Arc<MockAttributeRepo>) -> AttributeService {
        AttributeService::new(&(repo.clone() as Arc<dyn AttributeRepo>))
    }

    #[test]
    fn delete_unused_kind_succeeds() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Mood", false)), 0));
        let result = service(&repo).delete_attribute_kind("Mood", &account_id(), false);
        assert!(result.is_ok());
        assert!(repo.kind_deleted.load(Ordering::Relaxed));
        assert!(!repo.values_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_kind_without_force_returns_in_use_error() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Mood", false)), 14));
        let result = service(&repo).delete_attribute_kind("Mood", &account_id(), false);
        assert!(matches!(result, Err(AttributeServiceError::InUse(_))));
        if let Err(AttributeServiceError::InUse(msg)) = result {
            assert!(msg.contains("14"));
            assert!(msg.contains("Mood"));
        }
        assert!(!repo.kind_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_in_use_kind_with_force_purges_values_and_deletes_kind() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Mood", false)), 14));
        let result = service(&repo).delete_attribute_kind("Mood", &account_id(), true);
        assert!(result.is_ok());
        assert!(repo.values_deleted.load(Ordering::Relaxed));
        assert!(repo.kind_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn delete_system_kind_is_read_only() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Status", true)), 0));
        let result = service(&repo).delete_attribute_kind("Status", &account_id(), false);
        assert!(
            matches!(&result, Err(AttributeServiceError::ReadOnly(msg)) if msg.contains("Status"))
        );
        assert!(!repo.kind_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn force_delete_system_kind_is_read_only_and_keeps_values() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Parent", true)), 14));
        let result = service(&repo).delete_attribute_kind("Parent", &account_id(), true);
        assert!(matches!(result, Err(AttributeServiceError::ReadOnly(_))));
        assert!(!repo.values_deleted.load(Ordering::Relaxed));
        assert!(!repo.kind_deleted.load(Ordering::Relaxed));
    }

    #[test]
    fn update_system_kind_is_read_only() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Date", true)), 0));
        let result = service(&repo).update_attribute_kind(&rename_dto(), &account_id());
        assert!(
            matches!(&result, Err(AttributeServiceError::ReadOnly(msg)) if msg.contains("Date"))
        );
        assert!(!repo.kind_updated.load(Ordering::Relaxed));
    }

    #[test]
    fn update_user_kind_succeeds() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Mood", false)), 0));
        let result = service(&repo).update_attribute_kind(&rename_dto(), &account_id());
        assert!(matches!(result, Ok(Some(_))));
        assert!(repo.kind_updated.load(Ordering::Relaxed));
    }

    #[test]
    fn update_with_invalid_default_is_rejected_before_writing() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Priority2", false)), 0));
        let mut dto = rename_dto();
        dto.config = Some(serde_json::json!({"min": 1, "max": 10, "default": 11}));
        let err = service(&repo)
            .update_attribute_kind(&dto, &account_id())
            .unwrap_err();
        assert!(matches!(err, AttributeServiceError::InvalidDefault(_)));
        assert!(!repo.kind_updated.load(Ordering::Relaxed));
    }

    #[test]
    fn update_with_valid_default_or_none_reaches_the_repo() {
        let repo = Arc::new(MockAttributeRepo::new(Some(make_kind("Priority2", false)), 0));
        let mut dto = rename_dto();
        dto.config = Some(serde_json::json!({"min": 1, "max": 10, "default": 3}));
        assert!(service(&repo).update_attribute_kind(&dto, &account_id()).is_ok());
        assert!(repo.kind_updated.load(Ordering::Relaxed));
    }
}
