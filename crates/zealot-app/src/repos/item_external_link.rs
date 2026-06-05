use std::fmt::Debug;

use zealot_domain::{common::id::Id, item::ItemExternalLink};

use crate::repos::common::RepoError;

pub trait ItemExternalLinkRepo: Debug + Send + Sync {
    fn get_for_item(&self, item_id: &Id) -> Result<Vec<ItemExternalLink>, RepoError>;
    fn replace_for_item(&self, item_id: &Id, urls: &[String]) -> Result<(), RepoError>;
}
