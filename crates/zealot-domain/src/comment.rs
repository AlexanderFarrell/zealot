use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{
    common::id::Id,
    item::{Item, ItemDto, ItemError},
};

/// Lightweight comment as returned directly by the repo.
/// The `item_id` is used by the service layer to hydrate the full `Item`.
#[derive(Debug, Clone)]
pub struct CommentCore {
    pub comment_id: Id,
    pub item_id: Id,
    pub timestamp: NaiveDateTime,
    pub content: String,
}

/// Fully hydrated comment with the associated `Item`.
#[derive(Debug, Clone)]
pub struct Comment {
    pub comment_id: Id,
    pub item: Item,
    pub timestamp: NaiveDateTime,
    pub content: String,
}

// Send DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommentDto {
    pub comment_id: i64,
    pub item: ItemDto,
    pub timestamp: String,
    pub content: String,
}

// Receive DTOs

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AddCommentDto {
    pub item_id: i64,
    pub timestamp: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateCommentDto {
    pub comment_id: i64,
    pub item_id: Option<i64>,
    pub timestamp: Option<String>,
    pub content: Option<String>,
}

// Errors
#[derive(Debug, thiserror::Error)]
pub enum CommentError {
    #[error("error with item: {err:?}")]
    ItemError { err: ItemError },

    #[error("error with date: {err_str:?}")]
    DateError { err_str: String },
}

// Impl

impl From<Comment> for CommentDto {
    fn from(dto: Comment) -> Self {
        Self {
            comment_id: dto.comment_id.into(),
            item: ItemDto::from(&dto.item),
            timestamp: dto.timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
            content: dto.content.clone(),
        }
    }
}
