use std::fmt::Debug;

use chrono::NaiveDate;
use zealot_domain::{
    account::Account,
    repeat::{RepeatEntryCore, UpdateRepeatEntryDto},
};

use crate::repos::common::RepoError;

pub trait RepeatRepo: Debug + Send + Sync {
    // Get the status for each repeat for a day.
    fn get_for_day(
        &self,
        day: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntryCore>, RepoError>;

    // For the item with a repeat type, set what occurred
    // on that day (complete, skip, etc.).
    fn set_status(&self, dto: &UpdateRepeatEntryDto, account: &Account) -> Result<(), RepoError>;
}
