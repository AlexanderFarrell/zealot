use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
use zealot_domain::{
    account::Account,
    common::id::Id,
    repeat::{RepeatEntry, UpdateRepeatEntryDto},
};

use crate::repos::{common::RepoError, repeat::RepeatRepo};

use super::item::ItemService;

#[derive(Debug)]
pub struct RepeatService {
    repo: Arc<dyn RepeatRepo>,
    item_service: Arc<ItemService>,
}

#[derive(Debug, thiserror::Error)]
pub enum RepeatServiceError {
    #[error("not found")]
    NotFound,
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl RepeatService {
    pub fn new(repo: &Arc<dyn RepeatRepo>, item_service: &Arc<ItemService>) -> Self {
        Self { repo: repo.clone(), item_service: item_service.clone() }
    }

    pub async fn get_for_day(
        &self,
        day: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntry>, RepeatServiceError> {
        let cores = self.repo.get_for_day(day, account)?;

        if cores.is_empty() {
            return Ok(Vec::new());
        }

        let item_ids: Vec<Id> = cores.iter().map(|c| c.item_id).collect();
        let items = self
            .item_service
            .get_items_by_ids(&item_ids, account)
            .map_err(|_| RepeatServiceError::NotFound)?;

        let mut items_map: HashMap<Id, _> = items.into_iter().map(|i| (i.item_id, i)).collect();

        let entries = cores
            .into_iter()
            .filter_map(|core| {
                items_map.remove(&core.item_id).map(|item| RepeatEntry {
                    status: core.status,
                    item,
                    date: core.date,
                    comment: core.comment,
                })
            })
            .collect();

        Ok(entries)
    }

    pub async fn set_status(
        &self,
        dto: &UpdateRepeatEntryDto,
        account: &Account,
    ) -> Result<(), RepeatServiceError> {
        self.repo.set_status(dto, account)?;
        Ok(())
    }
}
