use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::json;
use zealot_domain::{
    account::Account,
    attribute::{AttributeFilterDto, Week},
    item::Item,
};

use super::item::{ItemService, ItemServiceError};

#[derive(Debug, Clone)]
pub struct PlannerService {
    item_service: Arc<ItemService>,
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerServiceError {
    #[error("item service error: {0}")]
    Item(#[from] ItemServiceError),
}

impl PlannerService {
    pub fn new(item_service: &Arc<ItemService>) -> Self {
        Self {
            item_service: item_service.clone(),
        }
    }

    pub fn get_for_day(
        &self,
        day: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items(
            vec![AttributeFilterDto {
                key: String::from("Date"),
                op: String::from("eq"),
                value: json!(day.format("%Y-%m-%d").to_string()),
                list_mode: String::from("any"),
            }],
            account,
        )
    }

    pub fn get_for_week(
        &self,
        week: &Week,
        account: &Account,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items(
            vec![AttributeFilterDto {
                key: String::from("Week"),
                op: String::from("eq"),
                value: json!(week.to_string()),
                list_mode: String::from("any"),
            }],
            account,
        )
    }

    pub fn get_for_month(
        &self,
        month: i64,
        year: i64,
        account: &Account,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items(
            vec![
                AttributeFilterDto {
                    key: String::from("Month"),
                    op: String::from("eq"),
                    value: json!(month),
                    list_mode: String::from("any"),
                },
                AttributeFilterDto {
                    key: String::from("Year"),
                    op: String::from("eq"),
                    value: json!(year),
                    list_mode: String::from("any"),
                },
            ],
            account,
        )
    }

    pub fn get_for_year(
        &self,
        year: i64,
        account: &Account,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items(
            vec![AttributeFilterDto {
                key: String::from("Year"),
                op: String::from("eq"),
                value: json!(year),
                list_mode: String::from("any"),
            }],
            account,
        )
    }

    fn filter_items(
        &self,
        filters: Vec<AttributeFilterDto>,
        account: &Account,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.item_service
            .filter_items(&filters, account)
            .map_err(PlannerServiceError::from)
    }
}
