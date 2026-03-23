use std::sync::Arc;

use crate::{
    ports::ZealotPorts,
    repos::ZealotRepos,
    services::{account::AccountService, attribute::AttributeService},
};

pub mod account;
pub mod attribute;
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
    pub attribute: Arc<AttributeService>,

    pub ports: ZealotPorts,
    pub repos: ZealotRepos,
}
