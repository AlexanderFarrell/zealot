use std::sync::Arc;

use crate::services::{account::AccountService, attribute::AttributeService};

pub mod account;
pub mod attribute;
pub mod comment;
pub mod common;
pub mod item_type;
pub mod item;
pub mod media;
pub mod meta;
pub mod planner;
pub mod repeat;
pub mod rule;
pub mod scope;

pub struct ZealotServices {
    pub account: Arc<AccountService>,
    pub attribute: Arc<AttributeService>,
}