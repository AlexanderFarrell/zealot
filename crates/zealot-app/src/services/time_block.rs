use std::{collections::HashMap, sync::Arc};

use chrono::NaiveDate;
use uuid::Uuid;
use zealot_domain::{
    account::Account,
    common::id::Id,
    time_block::{CreateTimeBlockDto, TimeBlock, TimeBlockCore, UpdateTimeBlockDto},
};

use crate::repos::{common::RepoError, time_block::TimeBlockRepo};

use super::item::ItemService;
use super::scope::ScopeAccess;

#[derive(Debug)]
pub struct TimeBlockService {
    repo: Arc<dyn TimeBlockRepo>,
    item_service: Arc<ItemService>,
}

#[derive(Debug, thiserror::Error)]
pub enum TimeBlockServiceError {
    #[error("not found")]
    NotFound,
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl TimeBlockService {
    pub fn new(repo: &Arc<dyn TimeBlockRepo>, item_service: &Arc<ItemService>) -> Self {
        Self {
            repo: repo.clone(),
            item_service: item_service.clone(),
        }
    }

    pub async fn get_for_day(
        &self,
        date: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        let cores = self.repo.get_for_day(date, account)?;
        self.hydrate(cores, account)
    }

    pub async fn get_for_range(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        account: &Account,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        let cores = self.repo.get_for_range(start, end, account)?;
        self.hydrate(cores, account)
    }

    pub async fn get_for_item(
        &self,
        item_id: Id,
        account: &Account,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        let cores = self.repo.get_for_item(item_id, account)?;
        self.hydrate(cores, account)
    }

    pub async fn create(
        &self,
        dto: &CreateTimeBlockDto,
        account: &Account,
    ) -> Result<TimeBlock, TimeBlockServiceError> {
        let core = self.repo.create(dto, account)?;
        let item_id = core.item_id;
        let ids = vec![item_id];
        let items = self
            .item_service
            .get_items_by_ids(&ids, account)
            .map_err(|_| TimeBlockServiceError::NotFound)?;
        let item = items
            .into_iter()
            .next()
            .ok_or(TimeBlockServiceError::NotFound)?;
        let block = TimeBlock {
            block_id: core.block_id,
            item,
            date: core.date,
            start_min: core.start_min,
            end_min: core.end_min,
            note: core.note,
        };
        tracing::info!(account_id = ?account.account_id, block_id = ?block.block_id, item_id = ?block.item.item_id, date = %block.date, "time block created");
        Ok(block)
    }

    pub async fn update(
        &self,
        dto: &UpdateTimeBlockDto,
        account: &Account,
    ) -> Result<(), TimeBlockServiceError> {
        self.repo.update(dto, account)?;
        tracing::info!(account_id = ?account.account_id, block_id = ?dto.block_id, "time block updated");
        Ok(())
    }

    pub async fn delete(
        &self,
        block_id: Id,
        account: &Account,
    ) -> Result<(), TimeBlockServiceError> {
        self.repo.delete(block_id, account)?;
        tracing::info!(account_id = ?account.account_id, ?block_id, "time block deleted");
        Ok(())
    }

    fn hydrate(
        &self,
        cores: Vec<TimeBlockCore>,
        account: &Account,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
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
            .map_err(|_| TimeBlockServiceError::NotFound)?;

        let items_map: HashMap<Id, _> = items.into_iter().map(|i| (i.item_id, i)).collect();

        let blocks = cores
            .into_iter()
            .filter_map(|core| {
                items_map.get(&core.item_id).map(|item| TimeBlock {
                    block_id: core.block_id,
                    item: item.clone(),
                    date: core.date,
                    start_min: core.start_min,
                    end_min: core.end_min,
                    note: core.note,
                })
            })
            .collect();

        Ok(blocks)
    }

    fn scope_ids(access: &ScopeAccess) -> Vec<Uuid> {
        access.scopes.iter().map(|scope| scope.scope_id).collect()
    }

    fn hydrate_scoped(
        &self,
        cores: Vec<TimeBlockCore>,
        access: &ScopeAccess,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        let mut blocks = Vec::with_capacity(cores.len());
        for core in cores {
            let item = self
                .item_service
                .get_item_by_id_in_scopes(&core.item_id, access)
                .map_err(|_| TimeBlockServiceError::NotFound)?
                .ok_or(TimeBlockServiceError::NotFound)?;
            blocks.push(TimeBlock {
                block_id: core.block_id,
                item,
                date: core.date,
                start_min: core.start_min,
                end_min: core.end_min,
                note: core.note,
            });
        }
        Ok(blocks)
    }

    pub async fn get_for_day_in_scopes(
        &self,
        date: &NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        self.hydrate_scoped(
            self.repo
                .get_for_day_in_scopes(date, &Self::scope_ids(access))?,
            access,
        )
    }

    pub async fn get_for_range_in_scopes(
        &self,
        start: &NaiveDate,
        end: &NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        self.hydrate_scoped(
            self.repo
                .get_for_range_in_scopes(start, end, &Self::scope_ids(access))?,
            access,
        )
    }

    pub async fn get_for_item_in_scopes(
        &self,
        item_id: Id,
        access: &ScopeAccess,
    ) -> Result<Vec<TimeBlock>, TimeBlockServiceError> {
        if self
            .item_service
            .get_item_by_id_in_scopes(&item_id, access)
            .map_err(|_| TimeBlockServiceError::NotFound)?
            .is_none()
        {
            return Err(TimeBlockServiceError::NotFound);
        }
        self.hydrate_scoped(
            self.repo
                .get_for_item_in_scopes(item_id, &Self::scope_ids(access))?,
            access,
        )
    }

    pub async fn create_in_scopes(
        &self,
        dto: &CreateTimeBlockDto,
        access: &ScopeAccess,
        owner_account: &Account,
    ) -> Result<TimeBlock, TimeBlockServiceError> {
        let core = self
            .repo
            .create_in_scopes(dto, owner_account, &Self::scope_ids(access))?;
        self.hydrate_scoped(vec![core], access)?
            .into_iter()
            .next()
            .ok_or(TimeBlockServiceError::NotFound)
    }

    pub async fn update_in_scopes(
        &self,
        dto: &UpdateTimeBlockDto,
        access: &ScopeAccess,
    ) -> Result<(), TimeBlockServiceError> {
        self.repo.update_in_scopes(dto, &Self::scope_ids(access))?;
        Ok(())
    }

    pub async fn delete_in_scopes(
        &self,
        block_id: Id,
        access: &ScopeAccess,
    ) -> Result<(), TimeBlockServiceError> {
        self.repo
            .delete_in_scopes(block_id, &Self::scope_ids(access))?;
        Ok(())
    }
}
