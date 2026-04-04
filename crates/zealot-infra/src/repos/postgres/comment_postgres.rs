use sqlx::PgPool;
use zealot_app::repos::{comment::CommentRepo, common::RepoError};
use zealot_domain::comment::{AddCommentDto, CommentCore, UpdateCommentDto};
use zealot_domain::common::id::Id;

#[derive(Debug)]
pub struct CommentPostgresRepo {
    pool: PgPool,
}

impl CommentPostgresRepo {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl CommentRepo for CommentPostgresRepo {
    fn get_for_day(
        &self,
        _day: &sqlx::types::chrono::NaiveDate,
        _account_id: &Id,
    ) -> Result<Vec<CommentCore>, RepoError> {
        todo!()
    }

    fn get_for_item(
        &self,
        _item_id: &Id,
        _account_id: &Id,
    ) -> Result<Vec<CommentCore>, RepoError> {
        todo!()
    }

    fn add_comment(
        &self,
        _dto: &AddCommentDto,
        _account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError> {
        todo!()
    }

    fn update_comment(
        &self,
        _dto: &UpdateCommentDto,
        _account_id: &Id,
    ) -> Result<Option<CommentCore>, RepoError> {
        todo!()
    }

    fn delete_comment(
        &self,
        _comment_id: &Id,
        _account_id: &Id,
    ) -> Result<(), RepoError> {
        todo!()
    }
}
