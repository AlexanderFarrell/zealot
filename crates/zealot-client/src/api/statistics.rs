use zealot_domain::{
    item::ItemDto,
    statistic::{
        CreateStatisticEntryDto, StatisticDailyPointDto, StatisticEntryDto, StatisticEntryPageDto,
        StatisticSummaryDto, UpdateStatisticEntryDto,
    },
};

use crate::{ApiError, ZealotClient};

fn range_query(start: Option<&str>, end: Option<&str>) -> String {
    let mut params = Vec::new();
    if let Some(start) = start {
        params.push(format!("start={}", urlencoding::encode(start)));
    }
    if let Some(end) = end {
        params.push(format!("end={}", urlencoding::encode(end)));
    }
    if params.is_empty() {
        String::new()
    } else {
        format!("?{}", params.join("&"))
    }
}

impl ZealotClient {
    pub async fn statistic_items(&self, parent_id: Option<i64>) -> Result<Vec<ItemDto>, ApiError> {
        let query = parent_id
            .map(|id| format!("?parent_id={id}"))
            .unwrap_or_default();
        self.get(&format!("/statistic/items{query}")).await
    }

    pub async fn statistic_entries(
        &self,
        item_id: i64,
        start: Option<&str>,
        end: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> Result<StatisticEntryPageDto, ApiError> {
        let mut query = range_query(start, end);
        query.push(if query.is_empty() { '?' } else { '&' });
        query.push_str(&format!("limit={limit}&offset={offset}"));
        self.get(&format!("/statistic/{item_id}/entries{query}"))
            .await
    }

    pub async fn statistic_daily(
        &self,
        item_id: i64,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<Vec<StatisticDailyPointDto>, ApiError> {
        self.get(&format!(
            "/statistic/{item_id}/daily{}",
            range_query(start, end)
        ))
        .await
    }

    pub async fn statistic_summary(
        &self,
        item_id: i64,
        start: Option<&str>,
        end: Option<&str>,
    ) -> Result<StatisticSummaryDto, ApiError> {
        self.get(&format!(
            "/statistic/{item_id}/summary{}",
            range_query(start, end)
        ))
        .await
    }

    pub async fn create_statistic_entry(
        &self,
        item_id: i64,
        dto: &CreateStatisticEntryDto,
    ) -> Result<StatisticEntryDto, ApiError> {
        self.post(&format!("/statistic/{item_id}/entries"), dto)
            .await
    }

    pub async fn update_statistic_entry(
        &self,
        statistic_entry_id: i64,
        dto: &UpdateStatisticEntryDto,
    ) -> Result<StatisticEntryDto, ApiError> {
        self.patch(&format!("/statistic/entries/{statistic_entry_id}"), dto)
            .await
    }

    pub async fn delete_statistic_entry(&self, statistic_entry_id: i64) -> Result<(), ApiError> {
        self.delete(&format!("/statistic/entries/{statistic_entry_id}"))
            .await
    }
}
