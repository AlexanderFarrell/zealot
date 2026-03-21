use crate::repos::{account::AccountRepo, 
    attribute::AttributeRepo, comment::CommentRepo, item::ItemRepo, item_type::ItemTypeRepo, media::MediaRepo, meta::MetaRepo, repeat::RepeatRepo, rule::RuleRepo, scope::ScopeRepo};

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

pub struct ZealotRepos {
    pub account: Box<dyn AccountRepo>,
    pub attribute: Box<dyn AttributeRepo>,
    pub comment: Box<dyn CommentRepo>,
    pub item: Box<dyn ItemRepo>,
    pub item_type: Box<dyn ItemTypeRepo>,
    pub media: Box<dyn MediaRepo>,
    pub meta: Box<dyn MetaRepo>,
    pub repeat: Box<dyn RepeatRepo>,
    pub rule: Box<dyn RuleRepo>,
    pub scope: Box<dyn ScopeRepo>,
}