use chrono::NaiveDate;
use zealot_domain::{
    item::ItemDto,
    repeat::{RepeatEntryDto, UpdateRepeatEntryDto},
};

use crate::{ApiError, ZealotClient};

impl ZealotClient {
    /// All items that have repeat tracking enabled.
    pub async fn repeat_items(&self) -> Result<Vec<ItemDto>, ApiError> {
        self.get("/repeat/items").await
    }

    pub async fn repeats_for_day(&self, date: NaiveDate) -> Result<Vec<RepeatEntryDto>, ApiError> {
        self.get(&format!("/repeat/day/{}", date.format("%Y-%m-%d")))
            .await
    }

    pub async fn repeats_for_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<RepeatEntryDto>, ApiError> {
        self.get(&format!(
            "/repeat/range?start={}&end={}",
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d")
        ))
        .await
    }

    /// Set a repeat status. `status` is one of `Complete`, `Skip`, `Alternate`,
    /// `Not Complete` (send `None` to clear back to not-complete server-side).
    pub async fn set_repeat_status(&self, dto: &UpdateRepeatEntryDto) -> Result<(), ApiError> {
        self.put_no_response("/repeat/status", dto).await
    }
}
