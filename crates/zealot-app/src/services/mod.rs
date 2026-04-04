use std::sync::Arc;

use crate::{
    ports::ZealotPorts,
    repos::ZealotRepos,
    services::{
        account::AccountService,
        analysis::AnalysisService,
        attribute::AttributeService,
        auth::AuthService,
        comment::CommentService,
        item::ItemService,
        item_type::ItemTypeService,
        media::MediaService,
        repeat::RepeatService,
    },
};

pub mod account;
pub mod analysis;
pub mod attribute;
pub mod auth;
pub mod comment;
pub mod common;
pub mod item;
pub mod item_type;
pub mod media;
pub mod meta;
pub mod planner;
pub mod repeat;
pub mod rule;
pub mod scope;

#[derive(Debug, Clone)]
pub struct ZealotServices {
    pub account: Arc<AccountService>,
    pub analysis: Arc<AnalysisService>,
    pub attribute: Arc<AttributeService>,
    pub auth: Arc<AuthService>,
    pub comment: Arc<CommentService>,
    pub item: Arc<ItemService>,
    pub item_type: Arc<ItemTypeService>,
    pub media: Arc<MediaService>,
    pub repeat: Arc<RepeatService>,

    pub ports: ZealotPorts,
    pub repos: ZealotRepos,
}

impl ZealotServices {
    pub fn new(ports: ZealotPorts, repos: ZealotRepos) -> Self {
        let item = Arc::new(ItemService::new(
            &repos.item,
            &repos.item_attribute_value,
            &repos.item_link,
            &repos.item_type,
            &repos.attribute,
        ));
        let comment = Arc::new(CommentService::new(&repos.comment, &item));
        let repeat = Arc::new(RepeatService::new(&repos.repeat, &item));

        let media = Arc::new(MediaService::new(&ports.media));

        Self {
            account: Arc::new(AccountService::new(&repos.account)),
            analysis: Arc::new(AnalysisService::new(&repos.item)),
            attribute: Arc::new(AttributeService::new(&repos.attribute)),
            auth: Arc::new(AuthService::new(&repos.account, &ports.password, &repos.session)),
            comment,
            item,
            item_type: Arc::new(ItemTypeService::new(&repos.item_type)),
            media,
            repeat,
            repos,
            ports,
        }
    }
}
