use chrono::NaiveDate;
use zealot_domain::{comment::{AddCommentDto, Comment, CommentError, UpdateCommentDto}, common::id::Id};

use crate::repos::common::RepoError;

pub trait CommentRepo {
    fn get_for_day(day: &NaiveDate, account_id: &Id) -> Result<Vec<Comment>, RepoError<CommentError>>;
    fn get_for_item(item_id: &Id, account_id: &Id) -> Result<Vec<Comment>, RepoError<CommentError>>;
    fn add_comment(dto: &AddCommentDto, account_id: &Id) -> Result<Option<Comment>, RepoError<CommentError>>;
    fn update_comment(dto: &UpdateCommentDto, account_id: &Id) -> Result<Option<Comment>, RepoError<CommentError>>;
    fn delete_comment(comment_id: &Id, account_id: &Id) -> Result<(), RepoError<CommentError>>;
}