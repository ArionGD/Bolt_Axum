use chrono::Datelike;
use sqlx::PgPool;
use crate::error::AppError;

pub async fn generate_order_number(pool: &PgPool) -> Result<String, AppError> {
    let current_year = chrono::Utc::now().year();
    let count: i64 = sqlx::query_scalar("select count(*) from orders")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok(format!("ORD-{}-{:04}", current_year, count + 1))
}

pub async fn generate_quote_number(pool: &PgPool) -> Result<String, AppError> {
    let current_year = chrono::Utc::now().year();
    let count: i64 = sqlx::query_scalar("select count(*) from quotations")
        .fetch_one(pool)
        .await
        .map_err(AppError::from)?;

    Ok(format!("QT-{}-{:04}", current_year, count + 1))
}
