use std::sync::Arc;

use crate::repos::{
    account::AccountRepo, attribute::AttributeRepo, comment::CommentRepo, item::ItemRepo,
    item_attribute_value::ItemAttributeValueRepo, item_external_link::ItemExternalLinkRepo,
    item_heading::ItemHeadingRepo, item_link::ItemLinkRepo, item_type::ItemTypeRepo,
    item_view::ItemViewRepo, meta::MetaRepo, repeat::RepeatRepo, rule::RuleRepo, scope::ScopeRepo,
    session::SessionRepo, statistic::StatisticRepo, time_block::TimeBlockRepo,
};

pub mod account;
pub mod attribute;
pub mod comment;
pub mod common;
pub mod item;
pub mod item_attribute_value;
pub mod item_external_link;
pub mod item_heading;
pub mod item_link;
pub mod item_type;
pub mod item_view;
pub mod meta;
pub mod repeat;
pub mod rule;
pub mod scope;
pub mod session;
pub mod statistic;
pub mod time_block;

#[derive(Debug, Clone)]
pub struct ZealotRepos {
    pub account: Arc<dyn AccountRepo>,
    pub attribute: Arc<dyn AttributeRepo>,
    pub comment: Arc<dyn CommentRepo>,
    pub item: Arc<dyn ItemRepo>,
    pub item_attribute_value: Arc<dyn ItemAttributeValueRepo>,
    pub item_external_link: Arc<dyn ItemExternalLinkRepo>,
    pub item_heading: Arc<dyn ItemHeadingRepo>,
    pub item_link: Arc<dyn ItemLinkRepo>,
    pub item_type: Arc<dyn ItemTypeRepo>,
    pub item_view: Arc<dyn ItemViewRepo>,
    // pub media: Arc<dyn MediaRepo>,
    pub meta: Arc<dyn MetaRepo>,
    pub repeat: Arc<dyn RepeatRepo>,
    pub rule: Arc<dyn RuleRepo>,
    pub scope: Arc<dyn ScopeRepo>,
    pub session: Arc<dyn SessionRepo>,
    pub statistic: Arc<dyn StatisticRepo>,
    pub time_block: Arc<dyn TimeBlockRepo>,
}
