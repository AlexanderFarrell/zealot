use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
use zealot_domain::{
    account::Account,
    common::id::Id,
    item::Item,
    repeat::{RepeatEntry, UpdateRepeatEntryDto},
};

use crate::repos::{common::RepoError, repeat::RepeatRepo};
use crate::services::scope::ScopeAccess;

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
        Self {
            repo: repo.clone(),
            item_service: item_service.clone(),
        }
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

    pub async fn get_for_day_in_scopes(
        &self,
        day: &NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<RepeatEntry>, RepeatServiceError> {
        let scope_ids: Vec<_> = access.scopes.iter().map(|scope| scope.scope_id).collect();
        let cores = self.repo.get_for_day_in_scopes(day, &scope_ids)?;
        if cores.is_empty() {
            return Ok(Vec::new());
        }

        let item_ids: Vec<Id> = cores.iter().map(|core| core.item_id).collect();
        let items: Vec<Item> = item_ids
            .iter()
            .filter_map(|item_id| {
                self.item_service
                    .get_item_by_id_in_scopes(item_id, access)
                    .transpose()
            })
            .collect::<Result<_, _>>()
            .map_err(|_| RepeatServiceError::NotFound)?;
        let mut items_map: HashMap<Id, _> =
            items.into_iter().map(|item| (item.item_id, item)).collect();

        Ok(cores
            .into_iter()
            .filter_map(|core| {
                items_map.remove(&core.item_id).map(|item| RepeatEntry {
                    status: core.status,
                    item,
                    date: core.date,
                    comment: core.comment,
                })
            })
            .collect())
    }

    pub fn get_all_repeat_items(&self, account: &Account) -> Result<Vec<Item>, RepeatServiceError> {
        self.item_service
            .get_items_by_type("Repeat", account)
            .map_err(|_| RepeatServiceError::NotFound)
    }

    pub fn get_all_repeat_items_in_scopes(
        &self,
        access: &ScopeAccess,
    ) -> Result<Vec<Item>, RepeatServiceError> {
        self.item_service
            .get_items_by_type_in_scopes("Repeat", access)
            .map_err(|_| RepeatServiceError::NotFound)
    }

    pub async fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<RepeatEntry>, RepeatServiceError> {
        let cores = self.repo.get_for_range(start, end, account)?;

        if cores.is_empty() {
            return Ok(Vec::new());
        }

        let mut seen = std::collections::HashSet::new();
        let item_ids: Vec<Id> = cores
            .iter()
            .map(|c| c.item_id)
            .filter(|id| seen.insert(*id))
            .collect();
        let items = self
            .item_service
            .get_items_by_ids(&item_ids, account)
            .map_err(|_| RepeatServiceError::NotFound)?;

        let items_map: HashMap<Id, _> = items.into_iter().map(|i| (i.item_id, i)).collect();

        let entries = cores
            .into_iter()
            .filter_map(|core| {
                items_map.get(&core.item_id).map(|item| RepeatEntry {
                    status: core.status,
                    item: item.clone(),
                    date: core.date,
                    comment: core.comment,
                })
            })
            .collect();

        Ok(entries)
    }

    pub async fn get_for_range_in_scopes(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<RepeatEntry>, RepeatServiceError> {
        let scope_ids: Vec<_> = access.scopes.iter().map(|scope| scope.scope_id).collect();
        let cores = self.repo.get_for_range_in_scopes(start, end, &scope_ids)?;
        if cores.is_empty() {
            return Ok(Vec::new());
        }

        let mut seen = std::collections::HashSet::new();
        let item_ids: Vec<Id> = cores
            .iter()
            .map(|core| core.item_id)
            .filter(|item_id| seen.insert(*item_id))
            .collect();
        let items: Vec<Item> = item_ids
            .iter()
            .filter_map(|item_id| {
                self.item_service
                    .get_item_by_id_in_scopes(item_id, access)
                    .transpose()
            })
            .collect::<Result<_, _>>()
            .map_err(|_| RepeatServiceError::NotFound)?;
        let items_map: HashMap<Id, _> =
            items.into_iter().map(|item| (item.item_id, item)).collect();

        Ok(cores
            .into_iter()
            .filter_map(|core| {
                items_map.get(&core.item_id).map(|item| RepeatEntry {
                    status: core.status,
                    item: item.clone(),
                    date: core.date,
                    comment: core.comment,
                })
            })
            .collect())
    }

    pub async fn set_status(
        &self,
        dto: &UpdateRepeatEntryDto,
        account: &Account,
    ) -> Result<(), RepeatServiceError> {
        self.repo.set_status(dto, account)?;
        tracing::info!(account_id = ?account.account_id, item_id = ?dto.item_id, date = %dto.date, status = ?dto.status, "repeat status set");
        Ok(())
    }

    pub async fn set_status_in_scopes(
        &self,
        dto: &UpdateRepeatEntryDto,
        access: &ScopeAccess,
    ) -> Result<(), RepeatServiceError> {
        let scope_ids: Vec<_> = access.scopes.iter().map(|scope| scope.scope_id).collect();
        self.repo.set_status_in_scopes(dto, &scope_ids)?;
        tracing::info!(
            principal_id = %access.principal_id,
            item_id = ?dto.item_id,
            date = %dto.date,
            status = ?dto.status,
            "repeat status set"
        );
        Ok(())
    }
}
