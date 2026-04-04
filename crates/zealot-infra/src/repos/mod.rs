use sqlx::{
    PgPool,
    postgres::PgConnectOptions,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use zealot_app::{config::ZealotConfig, repos::ZealotRepos};

use crate::repos::{postgres::get_postgres_repos, sqlite::get_sqlite_repos};

pub mod mysql;
pub mod postgres;
pub mod sqlite;

pub async fn get_repo_from_config(config: &ZealotConfig) -> Result<ZealotRepos, String> {
    match config.database.as_str() {
        "postgres" => {
            let options = PgConnectOptions::new()
                .host(&config.db_host.clone().unwrap_or(String::from("localhost")));

            match PgPool::connect_with(options).await {
                Ok(pool) => {
                    sqlx::migrate!("migrations/postgres")
                        .run(&pool)
                        .await
                        .map_err(|e| format!("Postgres migration failed: {}", e))?;
                    Ok(get_postgres_repos(pool))
                }
                Err(err) => Err(format!("Error connecting to postgres: {}", err)),
            }
        }
        "sqlite" => {
            let options = SqliteConnectOptions::new()
                .filename(&config.db_filename.clone())
                .create_if_missing(true)
                .foreign_keys(true);

            match SqlitePoolOptions::new()
                .max_connections(5)
                .connect_with(options)
                .await
            {
                Ok(pool) => {
                    sqlx::migrate!("migrations/sqlite")
                        .run(&pool)
                        .await
                        .map_err(|e| format!("SQLite migration failed: {}", e))?;
                    Ok(get_sqlite_repos(pool))
                }
                Err(err) => Err(format!("Error connecting to sqlite: {}", err)),
            }
        }
        value => Err(format!("{} is not a supported database", value)),
    }
}
