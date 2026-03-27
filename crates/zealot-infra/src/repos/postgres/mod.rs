use std::sync::Arc;

use sqlx::PgPool;
use zealot_app::repos::ZealotRepos;

use crate::repos::postgres::{
    account_postgres::AccountPostgresRepo, attribute_postgres::AttributePostgresRepo,
    comment_postgres::CommentPostgresRepo, item_postgres::ItemPostgresRepo,
    item_type_postgres::ItemTypePostgresRepo, meta_postgres::MetaPostgresRepo,
    repeat_postgres::RepeatPostgresRepo, rule_postgres::RulePostgresRepo,
    scope_postgres::ScopePostgresRepo, session_postgres::SessionPostgresRepo,
};

pub mod account_postgres;
pub mod attribute_postgres;
pub mod comment_postgres;
pub mod item_postgres;
pub mod item_type_postgres;
pub mod meta_postgres;
pub mod repeat_postgres;
pub mod rule_postgres;
pub mod scope_postgres;
pub mod session_postgres;

pub fn get_postgres_repos(db: PgPool) -> ZealotRepos {
    ZealotRepos {
        account: Arc::new(AccountPostgresRepo::new(db.clone())),
        attribute: Arc::new(AttributePostgresRepo::new(db.clone())),
        comment: Arc::new(CommentPostgresRepo::new(db.clone())),
        item: Arc::new(ItemPostgresRepo::new(db.clone())),
        item_type: Arc::new(ItemTypePostgresRepo::new(db.clone())),
        meta: Arc::new(MetaPostgresRepo::new(db.clone())),
        repeat: Arc::new(RepeatPostgresRepo::new(db.clone())),
        rule: Arc::new(RulePostgresRepo::new(db.clone())),
        scope: Arc::new(ScopePostgresRepo::new(db.clone())),
        session: Arc::new(SessionPostgresRepo::new(db.clone())),
    }
}
