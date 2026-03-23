use sqlx::{PgPool, postgres::PgConnectOptions};
use zealot_app::{config::ZealotConfig, repos::ZealotRepos};

use crate::repos::postgres::get_postgres_repos;

pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub async fn get_repo_from_config(config: &ZealotConfig) -> Result<ZealotRepos, String> {
    match config.database.as_str() {
        "postgres" => {
            let options = PgConnectOptions::new()
                .host(&config.db_host.clone().unwrap_or(String::from("localhost")));

            match PgPool::connect_with(options).await {
                Ok(pool) => Ok(get_postgres_repos(pool)),
                Err(err) => Err(format!("Error connecting to postgres: {}", err)),
            }
        }
        _ => Err(String::from("Must specify a valid database")),
    }
}
