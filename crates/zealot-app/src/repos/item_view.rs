use std::fmt::Debug;

use zealot_domain::common::id::Id;

use crate::repos::common::RepoError;

pub trait ItemViewRepo: Debug + Send + Sync {
    fn record_view(&self, item_id: &Id) -> Result<(), RepoError>;
    fn get_most_viewed(&self, limit: i64, account_id: &Id) -> Result<Vec<(Id, i64)>, RepoError>;
}
