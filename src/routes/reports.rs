use axum::{
    extract::{Query, State},
    Json,
};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    auth::{AppState, AuthUser},
    domain::{LeadFunnelStage, SalesSummaryReport, StockAgeingBucket},
    error::AppError,
    services::calculate_balance_due,
};

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

#[derive(sqlx::FromRow)]
struct SalesSummaryRow {
    pub total_orders: i64,
    pub total_delivered: i64,
    pub total_booked: i64,
    pub total_sales_value: Decimal,
    pub total_cash_collected: Decimal,
}

#[derive(sqlx::FromRow)]
struct StockAgeingRow {
    pub under_30: Option<i64>,
    pub days_30_60: Option<i64>,
    pub days_60_90: Option<i64>,
    pub over_90: Option<i64>,
}

#[derive(sqlx::FromRow)]
struct LeadFunnelRow {
    pub stage: String,
    pub count: i64,
}

pub async fn sales_summary(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(range): Query<DateRangeQuery>,
) -> Result<Json<SalesSummaryReport>, AppError> {
    user.require_admin_or_accounts()?;

    let row = sqlx::query_as::<_, SalesSummaryRow>(
        r#"
        select 
            count(distinct o.id) as total_orders,
            count(distinct o.id) filter (where o.status = 'delivered') as total_delivered,
            count(distinct o.id) filter (where o.status = 'booked') as total_booked,
            coalesce(sum(distinct o.total_amount) filter (where o.status != 'cancelled'), 0::numeric) as total_sales_value,
            coalesce(sum(p.amount), 0::numeric) as total_cash_collected
        from orders o
        left join payments p on o.id = p.order_id
        where ($1::date is null or o.booking_date >= $1)
          and ($2::date is null or o.booking_date <= $2)
        "#,
    )
    .bind(range.from)
    .bind(range.to)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    let total_sales_value = row.total_sales_value;
    let total_cash_collected = row.total_cash_collected;
    let total_outstanding_balance = calculate_balance_due(total_sales_value, total_cash_collected);

    Ok(Json(SalesSummaryReport {
        total_orders: row.total_orders,
        total_delivered: row.total_delivered,
        total_booked: row.total_booked,
        total_sales_value,
        total_cash_collected,
        total_outstanding_balance,
    }))
}

pub async fn stock_ageing(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<StockAgeingBucket>, AppError> {
    let row = sqlx::query_as::<_, StockAgeingRow>(
        r#"
        select 
            count(*) filter (where current_date - coalesce(arrival_date, created_at::date) < 30) as under_30,
            count(*) filter (where current_date - coalesce(arrival_date, created_at::date) between 30 and 59) as days_30_60,
            count(*) filter (where current_date - coalesce(arrival_date, created_at::date) between 60 and 89) as days_60_90,
            count(*) filter (where current_date - coalesce(arrival_date, created_at::date) >= 90) as over_90
        from vehicles
        where status in ('in_stock', 'in_transit')
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(StockAgeingBucket {
        under_30_days: row.under_30.unwrap_or(0),
        days_30_to_60: row.days_30_60.unwrap_or(0),
        days_60_to_90: row.days_60_90.unwrap_or(0),
        over_90_days: row.over_90.unwrap_or(0),
    }))
}

pub async fn lead_funnel(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<LeadFunnelStage>>, AppError> {
    let rows = sqlx::query_as::<_, LeadFunnelRow>(
        r#"
        select 
            status::text as stage,
            count(*) as count
        from leads
        group by status
        order by count(*) desc
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| LeadFunnelStage {
            stage: r.stage,
            count: r.count,
        })
        .collect();

    Ok(Json(result))
}
