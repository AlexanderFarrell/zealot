//! Stores and handles comments for items, allowing
//! information to be added in small events, or
//! light information.

use std::sync::Arc;

use zealot_domain::{
    account::Account,
    comment::{AddCommentDto, Comment, CommentCore, UpdateCommentDto},
    common::id::Id,
};

use crate::repos::{comment::CommentRepo, common::RepoError};

use super::item::ItemService;

#[derive(Debug)]
pub struct CommentService {
    repo: Arc<dyn CommentRepo>,
    item_service: Arc<ItemService>,
}

#[derive(Debug, thiserror::Error)]
pub enum CommentServiceError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

impl CommentService {
    pub fn new(repo: &Arc<dyn CommentRepo>, item_service: &Arc<ItemService>) -> Self {
        Self {
            repo: repo.clone(),
            item_service: item_service.clone(),
        }
    }

    fn hydrate(&self, core: CommentCore, account: &Account) -> Result<Comment, CommentServiceError> {
        let item = self
            .item_service
            .get_item_by_id(&core.item_id, account)
            .map_err(|_| CommentServiceError::NotFound)?
            .ok_or(CommentServiceError::NotFound)?;

        Ok(Comment {
            comment_id: core.comment_id,
            item,
            timestamp: core.timestamp,
            content: core.content,
        })
    }

    fn hydrate_many(
        &self,
        cores: Vec<CommentCore>,
        account: &Account,
    ) -> Result<Vec<Comment>, CommentServiceError> {
        cores.into_iter().map(|c| self.hydrate(c, account)).collect()
    }

    /// Gets all comments on a particular day for all items.
    pub async fn get_for_day(
        &self,
        day: &chrono::NaiveDate,
        account: &Account,
    ) -> Result<Vec<Comment>, CommentServiceError> {
        let cores = self.repo.get_for_day(day, &account.account_id)?;
        self.hydrate_many(cores, account)
    }

    /// Gets all comments for a particular item on all days.
    pub async fn get_for_item(
        &self,
        item_id: &Id,
        account: &Account,
    ) -> Result<Vec<Comment>, CommentServiceError> {
        let cores = self.repo.get_for_item(item_id, &account.account_id)?;
        self.hydrate_many(cores, account)
    }

    pub async fn add_comment(
        &self,
        dto: &AddCommentDto,
        account: &Account,
    ) -> Result<Option<Comment>, CommentServiceError> {
        match self.repo.add_comment(dto, &account.account_id)? {
            Some(core) => Ok(Some(self.hydrate(core, account)?)),
            None => Ok(None),
        }
    }

    pub async fn update_comment(
        &self,
        dto: &UpdateCommentDto,
        account: &Account,
    ) -> Result<Option<Comment>, CommentServiceError> {
        match self.repo.update_comment(dto, &account.account_id)? {
            Some(core) => Ok(Some(self.hydrate(core, account)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_comment(
        &self,
        comment_id: &Id,
        account: &Account,
    ) -> Result<(), CommentServiceError> {
        self.repo.delete_comment(comment_id, &account.account_id)?;
        Ok(())
    }
}
