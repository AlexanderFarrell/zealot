use std::sync::Arc;

use crate::{
    ports::ZealotPorts,
    repos::ZealotRepos,
    services::{
        account::AccountService,
        analysis::AnalysisService,
        attribute::AttributeService,
        auth::AuthService,
        item::ItemService,
        item_type::ItemTypeService,
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
    pub item: Arc<ItemService>,
    pub item_type: Arc<ItemTypeService>,

    pub ports: ZealotPorts,
    pub repos: ZealotRepos,
}

impl ZealotServices {
    pub fn new(ports: ZealotPorts, repos: ZealotRepos) -> Self {
        Self {
            account: Arc::new(AccountService::new(&repos.account)),
            analysis: Arc::new(AnalysisService::new(&repos.item)),
            attribute: Arc::new(AttributeService::new(&repos.attribute)),
            auth: Arc::new(AuthService::new(&repos.account, &ports.password, &repos.session)),
            item: Arc::new(ItemService::new(
                &repos.item,
                &repos.item_attribute_value,
                &repos.item_link,
                &repos.item_type,
                &repos.attribute,
            )),
            item_type: Arc::new(ItemTypeService::new(&repos.item_type)),
            repos,
            ports,
        }
    }
}
