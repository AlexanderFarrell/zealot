use std::fmt::Debug;

use chrono::NaiveDate;
use uuid::Uuid;
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

    fn get_for_day_in_scopes(
        &self,
        day: &NaiveDate,
        scope_ids: &[Uuid],
    ) -> Result<Vec<CommentCore>, RepoError> {
        let _ = (day, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified comment lookup is not implemented by this repository".into(),
        })
    }
    fn get_for_item_in_scopes(
        &self,
        item_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<Vec<CommentCore>, RepoError> {
        let _ = (item_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified comment lookup is not implemented by this repository".into(),
        })
    }
    fn add_comment_in_scopes(
        &self,
        dto: &AddCommentDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<CommentCore>, RepoError> {
        let _ = (dto, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified comment creation is not implemented by this repository".into(),
        })
    }
    fn update_comment_in_scopes(
        &self,
        dto: &UpdateCommentDto,
        scope_ids: &[Uuid],
    ) -> Result<Option<CommentCore>, RepoError> {
        let _ = (dto, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified comment update is not implemented by this repository".into(),
        })
    }
    fn delete_comment_in_scopes(
        &self,
        comment_id: &Id,
        scope_ids: &[Uuid],
    ) -> Result<(), RepoError> {
        let _ = (comment_id, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified comment deletion is not implemented by this repository".into(),
        })
    }
}
