//! Stores and handles comments for items, allowing
//! information to be added in small events, or
//! light information.

use std::sync::Arc;

use uuid::Uuid;
use zealot_domain::{
    account::Account,
    comment::{AddCommentDto, Comment, CommentCore, UpdateCommentDto},
    common::id::Id,
};

use crate::{
    ports::events::{EventPort, ZealotEvent},
    repos::{comment::CommentRepo, common::RepoError},
};

use super::item::ItemService;
use super::scope::ScopeAccess;

#[derive(Debug)]
pub struct CommentService {
    repo: Arc<dyn CommentRepo>,
    item_service: Arc<ItemService>,
    event_port: Arc<dyn EventPort>,
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
    pub fn new(
        repo: &Arc<dyn CommentRepo>,
        item_service: &Arc<ItemService>,
        event_port: &Arc<dyn EventPort>,
    ) -> Self {
        Self {
            repo: repo.clone(),
            item_service: item_service.clone(),
            event_port: event_port.clone(),
        }
    }

    fn hydrate(
        &self,
        core: CommentCore,
        account: &Account,
    ) -> Result<Comment, CommentServiceError> {
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
        cores
            .into_iter()
            .map(|c| self.hydrate(c, account))
            .collect()
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
            Some(core) => {
                let comment = self.hydrate(core, account)?;
                tracing::info!(account_id = ?account.account_id, comment_id = ?comment.comment_id, item_id = ?comment.item.item_id, "comment added");
                self.event_port.emit(ZealotEvent::CommentAdded {
                    account_id: account.account_id,
                    item_id: comment.item.item_id,
                    comment: comment.clone(),
                });
                Ok(Some(comment))
            }
            None => Ok(None),
        }
    }

    pub async fn update_comment(
        &self,
        dto: &UpdateCommentDto,
        account: &Account,
    ) -> Result<Option<Comment>, CommentServiceError> {
        match self.repo.update_comment(dto, &account.account_id)? {
            Some(core) => {
                let comment = self.hydrate(core, account)?;
                tracing::info!(account_id = ?account.account_id, comment_id = ?comment.comment_id, "comment updated");
                Ok(Some(comment))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_comment(
        &self,
        comment_id: &Id,
        account: &Account,
    ) -> Result<(), CommentServiceError> {
        self.repo.delete_comment(comment_id, &account.account_id)?;
        tracing::info!(account_id = ?account.account_id, ?comment_id, "comment deleted");
        Ok(())
    }

    fn scope_ids(access: &ScopeAccess) -> Vec<Uuid> {
        access.scopes.iter().map(|scope| scope.scope_id).collect()
    }

    fn hydrate_scoped(
        &self,
        core: CommentCore,
        access: &ScopeAccess,
    ) -> Result<Comment, CommentServiceError> {
        let item = self
            .item_service
            .get_item_by_id_in_scopes(&core.item_id, access)
            .map_err(|_| CommentServiceError::NotFound)?
            .ok_or(CommentServiceError::NotFound)?;
        Ok(Comment {
            comment_id: core.comment_id,
            item,
            timestamp: core.timestamp,
            content: core.content,
        })
    }

    pub async fn get_for_day_in_scopes(
        &self,
        day: &chrono::NaiveDate,
        access: &ScopeAccess,
    ) -> Result<Vec<Comment>, CommentServiceError> {
        let cores = self
            .repo
            .get_for_day_in_scopes(day, &Self::scope_ids(access))?;
        cores
            .into_iter()
            .map(|core| self.hydrate_scoped(core, access))
            .collect()
    }

    pub async fn get_for_item_in_scopes(
        &self,
        item_id: &Id,
        access: &ScopeAccess,
    ) -> Result<Vec<Comment>, CommentServiceError> {
        let item = self
            .item_service
            .get_item_by_id_in_scopes(item_id, access)
            .map_err(|_| CommentServiceError::NotFound)?
            .ok_or(CommentServiceError::NotFound)?;
        let cores = self
            .repo
            .get_for_item_in_scopes(item_id, &Self::scope_ids(access))?;
        cores
            .into_iter()
            .map(|core| {
                Ok(Comment {
                    comment_id: core.comment_id,
                    item: item.clone(),
                    timestamp: core.timestamp,
                    content: core.content,
                })
            })
            .collect()
    }

    pub async fn add_comment_in_scopes(
        &self,
        dto: &AddCommentDto,
        access: &ScopeAccess,
    ) -> Result<Option<Comment>, CommentServiceError> {
        match self
            .repo
            .add_comment_in_scopes(dto, &Self::scope_ids(access))?
        {
            Some(core) => {
                let comment = self.hydrate_scoped(core, access)?;
                let account_id = self
                    .item_service
                    .get_item_owner_account_id_in_scopes(&comment.item.item_id, access)
                    .map_err(|_| CommentServiceError::NotFound)?
                    .ok_or(CommentServiceError::NotFound)?;
                self.event_port.emit(ZealotEvent::CommentAdded {
                    account_id,
                    item_id: comment.item.item_id,
                    comment: comment.clone(),
                });
                Ok(Some(comment))
            }
            None => Ok(None),
        }
    }

    pub async fn update_comment_in_scopes(
        &self,
        dto: &UpdateCommentDto,
        access: &ScopeAccess,
    ) -> Result<Option<Comment>, CommentServiceError> {
        match self
            .repo
            .update_comment_in_scopes(dto, &Self::scope_ids(access))?
        {
            Some(core) => Ok(Some(self.hydrate_scoped(core, access)?)),
            None => Ok(None),
        }
    }

    pub async fn delete_comment_in_scopes(
        &self,
        comment_id: &Id,
        access: &ScopeAccess,
    ) -> Result<(), CommentServiceError> {
        self.repo
            .delete_comment_in_scopes(comment_id, &Self::scope_ids(access))?;
        Ok(())
    }
}
