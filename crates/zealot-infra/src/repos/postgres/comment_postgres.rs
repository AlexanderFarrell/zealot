use sqlx::PgPool;
use zealot_app::repos::comment::CommentRepo;

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
        day: &sqlx::types::chrono::NaiveDate,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Vec<zealot_domain::comment::Comment>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn get_for_item(
        &self,
        item_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Vec<zealot_domain::comment::Comment>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn add_comment(
        &self,
        dto: &zealot_domain::comment::AddCommentDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::comment::Comment>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn update_comment(
        &self,
        dto: &zealot_domain::comment::UpdateCommentDto,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<Option<zealot_domain::comment::Comment>, zealot_app::repos::common::RepoError> {
        todo!()
    }

    fn delete_comment(
        &self,
        comment_id: &zealot_domain::common::id::Id,
        account_id: &zealot_domain::common::id::Id,
    ) -> Result<(), zealot_app::repos::common::RepoError> {
        todo!()
    }
}
