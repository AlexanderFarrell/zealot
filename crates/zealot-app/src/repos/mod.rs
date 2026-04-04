use std::sync::Arc;

use crate::repos::{
    account::AccountRepo, attribute::AttributeRepo, comment::CommentRepo, item::ItemRepo,
    item_attribute_value::ItemAttributeValueRepo, item_link::ItemLinkRepo, item_type::ItemTypeRepo,
    meta::MetaRepo, repeat::RepeatRepo, rule::RuleRepo, scope::ScopeRepo, session::SessionRepo,
};

pub mod account;
pub mod attribute;
pub mod comment;
pub mod common;
pub mod item;
pub mod item_attribute_value;
pub mod item_link;
pub mod item_type;
pub mod meta;
pub mod repeat;
pub mod rule;
pub mod scope;
pub mod session;

#[derive(Debug, Clone)]
pub struct ZealotRepos {
    pub account: Arc<dyn AccountRepo>,
    pub attribute: Arc<dyn AttributeRepo>,
    pub comment: Arc<dyn CommentRepo>,
    pub item: Arc<dyn ItemRepo>,
    pub item_attribute_value: Arc<dyn ItemAttributeValueRepo>,
    pub item_link: Arc<dyn ItemLinkRepo>,
    pub item_type: Arc<dyn ItemTypeRepo>,
    // pub media: Arc<dyn MediaRepo>,
    pub meta: Arc<dyn MetaRepo>,
    pub repeat: Arc<dyn RepeatRepo>,
    pub rule: Arc<dyn RuleRepo>,
    pub scope: Arc<dyn ScopeRepo>,
    pub session: Arc<dyn SessionRepo>,
}
