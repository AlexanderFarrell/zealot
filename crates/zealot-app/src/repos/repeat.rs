use std::fmt::Debug;

use chrono::NaiveDate;
use uuid::Uuid;
use zealot_domain::{
    account::Account,
    repeat::{RepeatEntryCore, UpdateRepeatEntryDto},
};

use crate::repos::common::RepoError;

pub trait RepeatRepo: Debug + Send + Sync {
    fn get_for_day_in_scopes(
        &self,
        day: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<RepeatEntryCore>, RepoError> {
        let _ = (day, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified repeat day lookup is not implemented by this repository"
                .to_string(),
        })
    }

    // Get the status for each repeat for a day.
    fn get_for_day(
        &self,
        day: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntryCore>, RepoError>;

    // Get the status for each repeat across an inclusive date range.
    fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntryCore>, RepoError>;

    fn get_for_range_in_scopes(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<RepeatEntryCore>, RepoError> {
        let _ = (start, end, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified repeat range lookup is not implemented by this repository"
                .to_string(),
        })
    }

    // For the item with a repeat type, set what occurred
    // on that day (complete, skip, etc.).
    fn set_status(&self, dto: &UpdateRepeatEntryDto, account: &Account) -> Result<(), RepoError>;

    fn set_status_in_scopes(
        &self,
        dto: &UpdateRepeatEntryDto,
        scope_ids: &[Uuid],
    ) -> Result<(), RepoError> {
        let _ = (dto, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified repeat status mutation is not implemented by this repository"
                .to_string(),
        })
    }
}
