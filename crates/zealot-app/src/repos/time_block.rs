use std::fmt::Debug;

use chrono::NaiveDate;
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

    fn get_for_item(
        &self,
        item_id: Id,
        account: &Account,
    ) -> Result<Vec<TimeBlockCore>, RepoError>;

    fn create(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
    ) -> Result<TimeBlockCore, RepoError>;

    fn update(
        &self,
        dto: &UpdateTimeBlockDto,
        account: &Account,
    ) -> Result<(), RepoError>;

    fn delete(
        &self,
        block_id: Id,
        account: &Account,
    ) -> Result<(), RepoError>;
}
