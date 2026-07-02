use chrono::NaiveDate;
use zealot_domain::time_block::{CreateTimeBlockDto, TimeBlockDto, UpdateTimeBlockDto};

use crate::{ApiError, ZealotClient};

impl ZealotClient {
    pub async fn time_blocks_for_day(
        &self,
        date: NaiveDate,
    ) -> Result<Vec<TimeBlockDto>, ApiError> {
        self.get(&format!("/time_block/day/{}", date.format("%Y-%m-%d")))
            .await
    }

    pub async fn time_blocks_for_range(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<Vec<TimeBlockDto>, ApiError> {
        self.get(&format!(
            "/time_block/range?start={}&end={}",
            start.format("%Y-%m-%d"),
            end.format("%Y-%m-%d")
        ))
        .await
    }

    pub async fn time_blocks_for_item(&self, item_id: i64) -> Result<Vec<TimeBlockDto>, ApiError> {
        self.get(&format!("/time_block/item/{item_id}")).await
    }

    pub async fn create_time_block(
        &self,
        dto: &CreateTimeBlockDto,
    ) -> Result<TimeBlockDto, ApiError> {
        self.post("/time_block/", dto).await
    }

    pub async fn update_time_block(&self, dto: &UpdateTimeBlockDto) -> Result<(), ApiError> {
        self.patch_no_response(&format!("/time_block/{}", dto.block_id), dto)
            .await
    }

    pub async fn delete_time_block(&self, block_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/time_block/{block_id}")).await
    }
}
