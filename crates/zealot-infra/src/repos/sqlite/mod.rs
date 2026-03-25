use std::sync::Arc;

use sqlx::SqlitePool;
use zealot_app::repos::ZealotRepos;

use crate::repos::sqlite::{
    account_sqlite::AccountSqliteRepo, attribute_sqlite::AttributeSqliteRepo,
    comment_sqlite::CommentSqliteRepo, item_sqlite::ItemSqliteRepo,
    item_type_sqlite::ItemTypeSqliteRepo, meta_sqlite::MetaSqliteRepo,
    repeat_sqlite::RepeatSqliteRepo, rule_sqlite::RuleSqliteRepo, scope_sqlite::ScopeSqliteRepo,
};

pub mod account_sqlite;
pub mod attribute_sqlite;
pub mod comment_sqlite;
pub mod item_sqlite;
pub mod item_type_sqlite;
pub mod meta_sqlite;
pub mod repeat_sqlite;
pub mod rule_sqlite;
pub mod scope_sqlite;

pub fn get_sqlite_repos(pool: SqlitePool) -> ZealotRepos {
    ZealotRepos {
        account: Arc::new(AccountSqliteRepo::new(pool.clone())),
        attribute: Arc::new(AttributeSqliteRepo::new(pool.clone())),
        comment: Arc::new(CommentSqliteRepo::new(pool.clone())),
        item: Arc::new(ItemSqliteRepo::new(pool.clone())),
        item_type: Arc::new(ItemTypeSqliteRepo::new(pool.clone())),
        meta: Arc::new(MetaSqliteRepo::new(pool.clone())),
        repeat: Arc::new(RepeatSqliteRepo::new(pool.clone())),
        rule: Arc::new(RuleSqliteRepo::new(pool.clone())),
        scope: Arc::new(ScopeSqliteRepo::new(pool.clone())),
    }
}
