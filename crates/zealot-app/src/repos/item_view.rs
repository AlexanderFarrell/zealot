use std::fmt::Debug;

use uuid::Uuid;
use zealot_domain::common::id::Id;

use crate::repos::common::RepoError;

pub trait ItemViewRepo: Debug + Send + Sync {
    fn record_view(&self, item_id: &Id) -> Result<(), RepoError>;
    fn get_most_viewed(&self, limit: i64, account_id: &Id) -> Result<Vec<(Id, i64)>, RepoError>;

    fn get_most_viewed_in_scopes(
        &self,
        limit: i64,
        scope_ids: &[Uuid],
    ) -> Result<Vec<(Id, i64)>, RepoError> {
        let _ = (limit, scope_ids);
        Err(RepoError::DatabaseError {
            err: "scope-qualified item view analysis is not implemented by this repository"
                .to_string(),
        })
    }
}
