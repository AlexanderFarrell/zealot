use sqlx::PgPool;
use std::sync::Arc;
use zealot_api::http;
use zealot_app::{
    app::AppState,
    ports::ZealotPorts,
    repos::ZealotRepos,
    services::{
        ZealotServices,
        account::AccountService,
        attribute::{self, AttributeService},
    },
};
use zealot_infra::repos::postgres::{
    account_postgres::AccountPostgresRepo, attribute_postgres::AttributePostgresRepo,
    get_postgres_repos,
};

#[tokio::main]
async fn main() -> Result<(), ()> {
    let db: PgPool = PgPool::connect("postgres://...").await.unwrap();

    let repos = get_postgres_repos(db);

    let state = AppState {
        services: ZealotServices {
            account: Arc::new(AccountService::new(&repos.account)),
            attribute: Arc::new(AttributeService::new(&repos.attribute)),
            repos: repos,
            ports: ZealotPorts {},
        },
    };

    let http_server = http::build_router(state);
}
