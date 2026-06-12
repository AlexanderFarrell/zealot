use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::{common::id::Id, item::{Item, ItemDto}};

pub struct TimeBlock {
    pub block_id:  Id,
    pub item:      Item,
    pub date:      NaiveDate,
    pub start_min: i32,
    pub end_min:   i32,
    pub note:      String,
}

/// Lightweight time block returned directly by the repo.
/// The service layer hydrates the full Item.
#[derive(Debug, Clone)]
pub struct TimeBlockCore {
    pub block_id:  Id,
    pub item_id:   Id,
    pub date:      NaiveDate,
    pub start_min: i32,
    pub end_min:   i32,
    pub note:      String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimeBlockDto {
    pub block_id:  i64,
    pub item:      ItemDto,
    pub date:      String,
    pub start_min: i32,
    pub end_min:   i32,
    pub note:      String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CreateTimeBlockDto {
    pub item_id:   i64,
    pub date:      String,
    pub start_min: i32,
    pub end_min:   i32,
    pub note:      Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UpdateTimeBlockDto {
    pub block_id:  i64,
    pub date:      Option<String>,
    pub start_min: Option<i32>,
    pub end_min:   Option<i32>,
    pub note:      Option<String>,
}

impl From<TimeBlock> for TimeBlockDto {
    fn from(b: TimeBlock) -> Self {
        Self {
            block_id:  b.block_id.into(),
            item:      ItemDto::from(&b.item),
            date:      b.date.format("%Y-%m-%d").to_string(),
            start_min: b.start_min,
            end_min:   b.end_min,
            note:      b.note,
        }
    }
}
