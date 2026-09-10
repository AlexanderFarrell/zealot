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

async fn reconcile_schema_migration_version_sqlite(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let version: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("SQLite schema version lookup failed: {e}"))?;
    sqlx::query("UPDATE server SET schema_migration_version = ?")
        .bind(version)
        .execute(pool)
        .await
        .map_err(|e| format!("SQLite schema version update failed: {e}"))?;
    Ok(())
}

async fn reconcile_schema_migration_version_postgres(pool: &sqlx::PgPool) -> Result<(), String> {
    let version: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(version), 0) FROM _sqlx_migrations")
        .fetch_one(pool)
        .await
        .map_err(|e| format!("Postgres schema version lookup failed: {e}"))?;
    sqlx::query("UPDATE server SET schema_migration_version = $1")
        .bind(version)
        .execute(pool)
        .await
        .map_err(|e| format!("Postgres schema version update failed: {e}"))?;
    Ok(())
}

pub async fn get_repo_from_config(config: &ZealotConfig) -> Result<ZealotRepos, String> {
    match config.database.as_str() {
        "postgres" => {
            let mut options = PgConnectOptions::new()
                .host(&config.db_host.clone().unwrap_or(String::from("localhost")));
            if let Some(ref username) = config.db_username {
                options = options.username(username);
            }
            if let Some(ref password) = config.db_password {
                options = options.password(password);
            }
            if let Some(ref database) = config.db_database {
                options = options.database(database);
            }

            match PgPool::connect_with(options).await {
                Ok(pool) => {
                    sqlx::migrate!("migrations/postgres")
                        .run(&pool)
                        .await
                        .map_err(|e| format!("Postgres migration failed: {}", e))?;
                    reconcile_schema_migration_version_postgres(&pool).await?;
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
                    reconcile_schema_migration_version_sqlite(&pool).await?;
                    Ok(get_sqlite_repos(pool))
                }
                Err(err) => Err(format!("Error connecting to sqlite: {}", err)),
            }
        }
        value => Err(format!("{} is not a supported database", value)),
    }
}
