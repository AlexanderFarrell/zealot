use chrono::NaiveDate;
use zealot_domain::comment::{AddCommentDto, CommentDto, UpdateCommentDto};

use crate::{ApiError, ZealotClient};

impl ZealotClient {
    pub async fn comments_for_day(&self, date: NaiveDate) -> Result<Vec<CommentDto>, ApiError> {
        self.get(&format!("/comment/day/{}", date.format("%Y-%m-%d")))
            .await
    }

    pub async fn comments_for_item(&self, item_id: i64) -> Result<Vec<CommentDto>, ApiError> {
        self.get(&format!("/comment/item/{item_id}")).await
    }

    /// `timestamp` format: `YYYY-MM-DD HH:MM:SS`.
    pub async fn add_comment(&self, dto: &AddCommentDto) -> Result<CommentDto, ApiError> {
        self.post("/comment", dto).await
    }

    pub async fn update_comment(&self, dto: &UpdateCommentDto) -> Result<CommentDto, ApiError> {
        self.patch(&format!("/comment/{}", dto.comment_id), dto)
            .await
    }

    pub async fn delete_comment(&self, comment_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/comment/{comment_id}")).await
    }
}
