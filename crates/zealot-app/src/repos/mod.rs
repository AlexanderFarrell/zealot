use std::sync::Arc;

use crate::repos::{
    account::AccountRepo, attribute::AttributeRepo, comment::CommentRepo, item::ItemRepo,
    item_type::ItemTypeRepo, media::MediaRepo, meta::MetaRepo, repeat::RepeatRepo, rule::RuleRepo,
    scope::ScopeRepo,
};

pub mod account;
pub mod attribute;
pub mod comment;
pub mod common;
pub mod item;
pub mod item_type;
pub mod media;
pub mod meta;
pub mod repeat;
pub mod rule;
pub mod scope;

#[derive(Debug, Clone)]
pub struct ZealotRepos {
    pub account: Arc<dyn AccountRepo>,
    pub attribute: Arc<dyn AttributeRepo>,
    pub comment: Arc<dyn CommentRepo>,
    pub item: Arc<dyn ItemRepo>,
    pub item_type: Arc<dyn ItemTypeRepo>,
    // pub media: Arc<dyn MediaRepo>,
    pub meta: Arc<dyn MetaRepo>,
    pub repeat: Arc<dyn RepeatRepo>,
    pub rule: Arc<dyn RuleRepo>,
    pub scope: Arc<dyn ScopeRepo>,
}
