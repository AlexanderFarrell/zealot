use std::fmt::Debug;

use zealot_domain::{common::id::Id, item::ItemHeading};

use crate::repos::common::RepoError;

pub trait ItemHeadingRepo: Debug + Send + Sync {
    fn get_for_item(&self, item_id: &Id) -> Result<Vec<ItemHeading>, RepoError>;
    fn replace_for_item(
        &self,
        item_id: &Id,
        headings: &[(u8, u32, String)],
    ) -> Result<(), RepoError>;
}
