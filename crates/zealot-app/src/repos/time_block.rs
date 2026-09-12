use std::fmt::Debug;

use chrono::NaiveDate;
use uuid::Uuid;
use zealot_domain::{
    account::Account,
    common::id::Id,
    time_block::{CreateTimeBlockDto, TimeBlockCore, UpdateTimeBlockDto},
};

use crate::repos::common::RepoError;

pub trait TimeBlockRepo: Debug + Send + Sync {
    fn get_for_day(
        &self,
        date: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError>;

    fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError>;

    fn get_for_item(&self, item_id: Id, account: &Account)
        -> Result<Vec<TimeBlockCore>, RepoError>;

    fn create(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
    ) -> Result<TimeBlockCore, RepoError>;

    fn update(&self, dto: &UpdateTimeBlockDto, account: &Account) -> Result<(), RepoError>;

    fn delete(&self, block_id: Id, account: &Account) -> Result<(), RepoError>;

    fn get_for_day_in_scopes(
        &self,
        date: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let _ = (date, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block lookup is not implemented by this repository".into(),
        })
    }
    fn get_for_range_in_scopes(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let _ = (start, end, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block lookup is not implemented by this repository".into(),
        })
    }
    fn get_for_item_in_scopes(
        &self,
        item_id: Id,
        scope_ids: &[Uuid],
    ) -> Result<Vec<TimeBlockCore>, RepoError> {
        let _ = (item_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block lookup is not implemented by this repository".into(),
        })
    }
    fn create_in_scopes(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
        scope_ids: &[Uuid],
    ) -> Result<TimeBlockCore, RepoError> {
        let _ = (dto, account, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block creation is not implemented by this repository".into(),
        })
    }
    fn update_in_scopes(
        &self,
        dto: &UpdateTimeBlockDto,
        scope_ids: &[Uuid],
    ) -> Result<(), RepoError> {
        let _ = (dto, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block update is not implemented by this repository".into(),
        })
    }
    fn delete_in_scopes(&self, block_id: Id, scope_ids: &[Uuid]) -> Result<(), RepoError> {
        let _ = (block_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified time-block deletion is not implemented by this repository".into(),
        })
    }
}
