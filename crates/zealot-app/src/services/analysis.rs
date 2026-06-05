use std::sync::Arc;

use zealot_domain::{account::Account, common::id::Id, item::ItemCore};

use crate::repos::{item::ItemRepo, item_view::ItemViewRepo};
use crate::repos::common::RepoError;

#[derive(Debug, Clone)]
pub struct AnalysisService {
    item_repo: Arc<dyn ItemRepo>,
    item_view_repo: Arc<dyn ItemViewRepo>,
}

#[derive(Debug, thiserror::Error)]
pub enum AnalysisServiceError {
    #[error("repo error: {0}")]
    Repo(#[from] RepoError),
}

/// Returned by `get_most_viewed_items`: item core data paired with its view count.
#[derive(Debug, Clone)]
pub struct MostViewedItem {
    pub item: ItemCore,
    pub view_count: i64,
}

impl AnalysisService {
    pub fn new(item_repo: &Arc<dyn ItemRepo>, item_view_repo: &Arc<dyn ItemViewRepo>) -> Self {
        Self {
            item_repo: item_repo.clone(),
            item_view_repo: item_view_repo.clone(),
        }
    }

    /// Records a single view for the given item. Errors are non-fatal to callers.
    pub fn record_view(&self, item_id: &Id) -> Result<(), AnalysisServiceError> {
        self.item_view_repo
            .record_view(item_id)
            .map_err(AnalysisServiceError::Repo)
    }

    /// Returns the top `limit` items ordered by descending view count, scoped to the account.
    pub fn get_most_viewed_items(
        &self,
        limit: i64,
        account: &Account,
    ) -> Result<Vec<MostViewedItem>, AnalysisServiceError> {
        let ranked = self
            .item_view_repo
            .get_most_viewed(limit, &account.account_id)
            .map_err(AnalysisServiceError::Repo)?;

        if ranked.is_empty() {
            return Ok(Vec::new());
        }

        let item_ids: Vec<Id> = ranked.iter().map(|(id, _)| *id).collect();
        let items = self
            .item_repo
            .get_items_by_ids(&item_ids, account)
            .map_err(AnalysisServiceError::Repo)?;

        // Preserve the order returned by the view repo (already sorted by count desc).
        let view_count_map: std::collections::HashMap<Id, i64> =
            ranked.into_iter().collect();

        let mut result: Vec<MostViewedItem> = items
            .into_iter()
            .map(|item| {
                let view_count = *view_count_map.get(&item.item_id).unwrap_or(&0);
                MostViewedItem { item, view_count }
            })
            .collect();

        result.sort_by(|a, b| b.view_count.cmp(&a.view_count));

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Debug, sync::Mutex};

    use zealot_domain::{
        account::Account,
        common::id::Id,
        item::{AddItemCoreDto, ItemCore, UpdateItemCoreDto},
    };

    use super::*;
    use crate::repos::{common::RepoError, item::ItemRepo, item_view::ItemViewRepo};

    // --- Mock repos ---

    #[derive(Debug)]
    struct MockItemRepo {
        items: Vec<ItemCore>,
    }

    impl ItemRepo for MockItemRepo {
        fn get_item_by_id(&self, item_id: &Id, _account: &Account) -> Result<Option<ItemCore>, RepoError> {
            Ok(self.items.iter().find(|i| i.item_id == *item_id).cloned())
        }
        fn get_items_by_ids(&self, item_ids: &Vec<Id>, _account: &Account) -> Result<Vec<ItemCore>, RepoError> {
            let found = self.items.iter()
                .filter(|i| item_ids.contains(&i.item_id))
                .cloned()
                .collect();
            Ok(found)
        }
        fn get_items_by_title(&self, _title: &str, _account: &Account) -> Result<Vec<ItemCore>, RepoError> { Ok(vec![]) }
        fn search_items_by_title(&self, _term: &str, _limit: i64, _offset: i64, _account: &Account) -> Result<Vec<ItemCore>, RepoError> { Ok(vec![]) }
        fn search_items_by_content(&self, _term: &str, _limit: i64, _offset: i64, _account: &Account) -> Result<Vec<ItemCore>, RepoError> { Ok(vec![]) }
        fn search_items_by_heading(&self, _term: &str, _limit: i64, _offset: i64, _account: &Account) -> Result<Vec<(ItemCore, String)>, RepoError> { Ok(vec![]) }
        fn regex_items_by_title(&self, _term: &str, _account: &Account) -> Result<Vec<ItemCore>, RepoError> { Ok(vec![]) }
        fn get_recent_items(&self, _limit: i64, _offset: i64, _account: &Account) -> Result<Vec<ItemCore>, RepoError> { Ok(vec![]) }
        fn get_all_item_ids_for_user(&self, _account_id: &Id) -> Result<Vec<Id>, RepoError> { Ok(vec![]) }
        fn add_item(&self, _dto: &AddItemCoreDto, _account: &Account) -> Result<Option<ItemCore>, RepoError> { Ok(None) }
        fn update_item(&self, _dto: &UpdateItemCoreDto, _account: &Account) -> Result<Option<ItemCore>, RepoError> { Ok(None) }
        fn delete_item(&self, _item_id: &Id, _account: &Account) -> Result<(), RepoError> { Ok(()) }
    }

    #[derive(Debug)]
    struct MockItemViewRepo {
        views: Mutex<Vec<(Id, i64)>>,
    }

    impl MockItemViewRepo {
        fn new(views: Vec<(Id, i64)>) -> Self {
            Self { views: Mutex::new(views) }
        }
    }

    impl ItemViewRepo for MockItemViewRepo {
        fn record_view(&self, item_id: &Id) -> Result<(), RepoError> {
            let mut v = self.views.lock().unwrap();
            if let Some(entry) = v.iter_mut().find(|(id, _)| id == item_id) {
                entry.1 += 1;
            } else {
                v.push((*item_id, 1));
            }
            Ok(())
        }

        fn get_most_viewed(&self, limit: i64, _account_id: &Id) -> Result<Vec<(Id, i64)>, RepoError> {
            let mut v = self.views.lock().unwrap().clone();
            v.sort_by(|a, b| b.1.cmp(&a.1));
            Ok(v.into_iter().take(limit as usize).collect())
        }
    }

    fn make_account() -> Account {
        use zealot_domain::common::email::Email;
        Account {
            account_id: Id::try_from(1i64).unwrap(),
            username: "testuser".to_string(),
            email: Email::try_from("test@example.com".to_string()).unwrap(),
            given_name: "Test".to_string(),
            surname: "User".to_string(),
            settings: serde_json::Value::Null,
            has_api_key: false,
        }
    }

    fn make_item(id: i64, title: &str) -> ItemCore {
        ItemCore {
            item_id: Id::try_from(id).unwrap(),
            title: title.to_string(),
            content: String::new(),
        }
    }

    #[test]
    fn get_most_viewed_returns_items_sorted_by_count_desc() {
        let id_a = Id::try_from(1i64).unwrap();
        let id_b = Id::try_from(2i64).unwrap();
        let id_c = Id::try_from(3i64).unwrap();

        let item_repo = Arc::new(MockItemRepo {
            items: vec![
                make_item(1, "Alpha"),
                make_item(2, "Beta"),
                make_item(3, "Gamma"),
            ],
        });
        let view_repo = Arc::new(MockItemViewRepo::new(vec![
            (id_b, 10),
            (id_a, 5),
            (id_c, 20),
        ]));

        let service = AnalysisService::new(&(item_repo as Arc<dyn ItemRepo>), &(view_repo as Arc<dyn ItemViewRepo>));
        let account = make_account();

        let results = service.get_most_viewed_items(10, &account).unwrap();
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].view_count, 20);
        assert_eq!(results[0].item.title, "Gamma");
        assert_eq!(results[1].view_count, 10);
        assert_eq!(results[1].item.title, "Beta");
        assert_eq!(results[2].view_count, 5);
        assert_eq!(results[2].item.title, "Alpha");
    }

    #[test]
    fn get_most_viewed_respects_limit() {
        let item_repo = Arc::new(MockItemRepo {
            items: vec![make_item(1, "A"), make_item(2, "B"), make_item(3, "C")],
        });
        let id_a = Id::try_from(1i64).unwrap();
        let id_b = Id::try_from(2i64).unwrap();
        let id_c = Id::try_from(3i64).unwrap();
        let view_repo = Arc::new(MockItemViewRepo::new(vec![
            (id_a, 3), (id_b, 7), (id_c, 1),
        ]));

        let service = AnalysisService::new(&(item_repo as Arc<dyn ItemRepo>), &(view_repo as Arc<dyn ItemViewRepo>));
        let results = service.get_most_viewed_items(2, &make_account()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].view_count, 7);
        assert_eq!(results[1].view_count, 3);
    }

    #[test]
    fn get_most_viewed_empty_when_no_views() {
        let item_repo = Arc::new(MockItemRepo { items: vec![] });
        let view_repo = Arc::new(MockItemViewRepo::new(vec![]));
        let service = AnalysisService::new(&(item_repo as Arc<dyn ItemRepo>), &(view_repo as Arc<dyn ItemViewRepo>));
        let results = service.get_most_viewed_items(10, &make_account()).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn record_view_increments_count() {
        let item_repo = Arc::new(MockItemRepo { items: vec![make_item(1, "A")] });
        let view_repo = Arc::new(MockItemViewRepo::new(vec![]));
        let service = AnalysisService::new(&(item_repo as Arc<dyn ItemRepo>), &(view_repo as Arc<dyn ItemViewRepo>));
        let id = Id::try_from(1i64).unwrap();

        service.record_view(&id).unwrap();
        service.record_view(&id).unwrap();

        let results = service.get_most_viewed_items(10, &make_account()).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].view_count, 2);
    }
}
