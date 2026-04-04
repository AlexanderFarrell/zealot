use std::fmt::Debug;

use chrono::NaiveDate;
use zealot_domain::{
    comment::{AddCommentDto, CommentCore, UpdateCommentDto},
    common::id::Id,
};

use crate::repos::common::RepoError;

pub trait CommentRepo: Debug + Send + Sync {
    fn get_for_day(&self, day: &NaiveDate, account_id: &Id) -> Result<Vec<CommentCore>, RepoError>;
    fn get_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<CommentCore>, RepoError>;
    fn add_comment(
        &self,
        dto: &AddCommentDto,
        account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError>;
    fn update_comment(
        &self,
        dto: &UpdateCommentDto,
        account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError>;
    fn delete_comment(&self, comment_id: &Id, account_id: &Id) -> Result<(), RepoError>;
}
