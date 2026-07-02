use chrono::NaiveDate;
use zealot_domain::item::ItemDto;

use crate::{ApiError, ZealotClient};

impl ZealotClient {
    pub async fn planner_day(&self, date: NaiveDate) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/planner/day/{}", date.format("%Y-%m-%d")))
            .await
    }

    /// `week` uses ISO week format, e.g. `2026-W27`.
    pub async fn planner_week(&self, week: &str) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/planner/week/{week}")).await
    }

    pub async fn planner_month(&self, month: u32, year: i32) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/planner/month/{month}/year/{year}"))
            .await
    }

    pub async fn planner_year(&self, year: i32) -> Result<Vec<ItemDto>, ApiError> {
        self.get(&format!("/planner/year/{year}")).await
    }
}
