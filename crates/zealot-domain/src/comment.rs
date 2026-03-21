use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use crate::{common::id::Id, item::{Item, ItemDto, ItemError}};


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
    ItemError{err: ItemError},

    #[error("error with date: {err_str:?}")]
    DateError{err_str: String},
}

// Impl

