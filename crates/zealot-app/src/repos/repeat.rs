use chrono::NaiveDate;
use zealot_domain::{account::Account, repeat::{RepeatEntry, RepeatEntryDto, RepeatError}};

use crate::repos::common::RepoError;

pub trait RepeatRepo {
    fn get_for_day(&self, day: &NaiveDate, account: &Account) -> Result<Vec<RepeatEntry>, RepoError<RepeatError>>;
    fn set_status(&self, dto: &RepeatEntryDto, account: &Account) -> Result<(), RepoError<RepeatError>>;
}
