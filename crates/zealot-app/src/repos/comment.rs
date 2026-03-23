use std::fmt::Debug;

use chrono::NaiveDate;
use zealot_domain::{
    comment::{AddCommentDto, Comment, UpdateCommentDto},
    common::id::Id,
};

use crate::repos::common::RepoError;

pub trait CommentRepo: Debug + Send + Sync {
    fn get_for_day(&self, day: &NaiveDate, account_id: &Id) -> Result<Vec<Comment>, RepoError>;
    fn get_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<Comment>, RepoError>;
    fn add_comment(
        &self,
        dto: &AddCommentDto,
        account_id: &Id,
    ) -> Result<Option<Comment>, RepoError>;
    fn update_comment(
        &self,
        dto: &UpdateCommentDto,
        account_id: &Id,
    ) -> Result<Option<Comment>, RepoError>;
    fn delete_comment(&self, comment_id: &Id, account_id: &Id) -> Result<(), RepoError>;
}
