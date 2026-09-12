use std::fmt::Debug;

use chrono::{DateTime, Utc};
use uuid::Uuid;
use zealot_domain::{account::Account, common::id::Id, statistic::StatisticEntryCore};

use super::common::RepoError;

pub trait StatisticRepo: Debug + Send + Sync {
    fn get_in_scopes(
        &self,
        statistic_entry_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        let _ = (statistic_entry_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic lookup is not implemented by this repository"
                .to_string(),
        })
    }

    fn get(
        &self,
        statistic_entry_id: Id,
        account: &Account,
    ) -> Result<Option<StatisticEntryCore>, RepoError>;

    fn list(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
        account: &Account,
    ) -> Result<Vec<StatisticEntryCore>, RepoError>;

    fn list_in_scopes(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        limit: i64,
        offset: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        let _ = (item_id, start, end, limit, offset, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic list is not implemented by this repository".to_string(),
        })
    }

    fn list_all(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        account: &Account,
    ) -> Result<Vec<StatisticEntryCore>, RepoError>;

    fn list_all_in_scopes(
        &self,
        item_id: Id,
        start: Option<DateTime<Utc>>,
        end: Option<DateTime<Utc>>,
        scope_ids: &[Uuid],
    ) -> Result<Vec<StatisticEntryCore>, RepoError> {
        let _ = (item_id, start, end, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic aggregate list is not implemented by this repository"
                .to_string(),
        })
    }

    fn create(
        &self,
        item_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
    ) -> Result<StatisticEntryCore, RepoError>;

    fn create_in_scopes(
        &self,
        item_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
        scope_ids: &[Uuid],
    ) -> Result<StatisticEntryCore, RepoError> {
        let _ = (
            item_id,
            value,
            occurred_at,
            related_item_id,
            comment,
            account,
            scope_ids,
        );
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic create is not implemented by this repository"
                .to_string(),
        })
    }

    fn update(
        &self,
        statistic_entry_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        account: &Account,
    ) -> Result<Option<StatisticEntryCore>, RepoError>;

    fn update_in_scopes(
        &self,
        statistic_entry_id: Id,
        value: f64,
        occurred_at: DateTime<Utc>,
        related_item_id: Option<Id>,
        comment: Option<String>,
        scope_ids: &[Uuid],
    ) -> Result<Option<StatisticEntryCore>, RepoError> {
        let _ = (
            statistic_entry_id,
            value,
            occurred_at,
            related_item_id,
            comment,
            scope_ids,
        );
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic update is not implemented by this repository"
                .to_string(),
        })
    }

    fn delete(&self, statistic_entry_id: Id, account: &Account) -> Result<bool, RepoError>;

    fn delete_in_scopes(
        &self,
        statistic_entry_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<bool, RepoError> {
        let _ = (statistic_entry_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified statistic delete is not implemented by this repository"
                .to_string(),
        })
    }
}
