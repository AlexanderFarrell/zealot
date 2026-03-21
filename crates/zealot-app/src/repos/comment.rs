use chrono::NaiveDate;
use zealot_domain::{comment::{AddCommentDto, Comment, CommentError, UpdateCommentDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait CommentRepo {
    fn get_for_day(&self, day: &NaiveDate, account_id: &Id) -> Result<Vec<Comment>, RepoError<CommentError>>;
    fn get_for_item(&self, item_id: &Id, account_id: &Id) -> Result<Vec<Comment>, RepoError<CommentError>>;
    fn add_comment(&self, dto: &AddCommentDto, account_id: &Id) -> Result<Option<Comment>, RepoError<CommentError>>;
    fn update_comment(&self, dto: &UpdateCommentDto, account_id: &Id) -> Result<Option<Comment>, RepoError<CommentError>>;
    fn delete_comment(&self, comment_id: &Id, account_id: &Id) -> Result<(), RepoError<CommentError>>;
}
