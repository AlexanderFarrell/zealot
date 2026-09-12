use std::sync::Arc;

use chrono::NaiveDate;
use serde_json::json;
use zealot_domain::{
    account::Account,
    attribute::{AttributeFilterDto, Week},
    item::Item,
};

use super::item::{ItemService, ItemServiceError};
use super::scope::ScopeAccess;

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
            .map_err(|e| {
                tracing::error!(account_id = ?account.account_id, %e, "planner filter failed");
                PlannerServiceError::from(e)
            })
    }

    pub fn get_for_day_in_scopes(
        &self,
        day: &NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items_in_scopes(
            vec![AttributeFilterDto {
                key: String::from("Date"),
                op: String::from("eq"),
                value: json!(day.format("%Y-%m-%d").to_string()),
                list_mode: String::from("any"),
            }],
            access,
        )
    }

    pub fn get_for_week_in_scopes(
        &self,
        week: &Week,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items_in_scopes(
            vec![AttributeFilterDto {
                key: String::from("Week"),
                op: String::from("eq"),
                value: json!(week.to_string()),
                list_mode: String::from("any"),
            }],
            access,
        )
    }

    pub fn get_for_month_in_scopes(
        &self,
        month: i64,
        year: i64,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items_in_scopes(
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
            access,
        )
    }

    pub fn get_for_year_in_scopes(
        &self,
        year: i64,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.filter_items_in_scopes(
            vec![AttributeFilterDto {
                key: String::from("Year"),
                op: String::from("eq"),
                value: json!(year),
                list_mode: String::from("any"),
            }],
            access,
        )
    }

    fn filter_items_in_scopes(
        &self,
        filters: Vec<AttributeFilterDto>,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, PlannerServiceError> {
        self.item_service
            .filter_items_in_scopes(&filters, access)
            .map_err(PlannerServiceError::from)
    }
}
