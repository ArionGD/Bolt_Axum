use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;
use crate::error::AppError;

pub type DbPool = PgPool;

pub async fn create_pool(database_url: &str) -> Result<DbPool, AppError> {
    PgPoolOptions::new()
        .max_connections(10)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(5))
        .idle_timeout(Duration::from_secs(600))
        .connect(database_url)
        .await
        .map_err(AppError::from)
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), AppError> {
    // Run initial schema migration
    let init_sql = include_str!("../migrations/0001_init.sql");
    sqlx::raw_sql(init_sql)
        .execute(pool)
        .await
        .map_err(AppError::from)?;

    // Run models seed migration
    let seed_sql = include_str!("../migrations/0002_seed_models.sql");
    sqlx::raw_sql(seed_sql)
        .execute(pool)
        .await
        .map_err(AppError::from)?;

    // Run superuser & roles migration
    let roles_sql = include_str!("../migrations/0003_superuser_and_roles.sql");
    sqlx::raw_sql(roles_sql)
        .execute(pool)
        .await
        .map_err(AppError::from)?;

    tracing::info!("Database schema and seed migrations applied successfully");
    Ok(())
}
