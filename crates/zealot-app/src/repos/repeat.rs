use std::fmt::Debug;

use chrono::NaiveDate;
use zealot_domain::{account::Account, repeat::{RepeatEntry, RepeatEntryDto}};

use crate::repos::common::RepoError;

pub trait RepeatRepo: Debug + Send + Sync {
    fn get_for_day(&self, day: &NaiveDate, account: &Account) -> Result<Vec<RepeatEntry>, RepoError>;
    fn set_status(&self, dto: &RepeatEntryDto, account: &Account) -> Result<(), RepoError>;
}
