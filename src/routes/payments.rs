use axum::{
    extract::{Path, State},
    Json,
};
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{CreatePaymentRequest, Payment},
    error::AppError,
};

pub async fn list_order_payments(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
) -> Result<Json<Vec<Payment>>, AppError> {
    let payments = sqlx::query_as::<_, Payment>(
        "select * from payments where order_id = $1 order by payment_date asc, created_at asc",
    )
    .bind(order_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(payments))
}

pub async fn record_payment(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(order_id): Path<Uuid>,
    Json(payload): Json<CreatePaymentRequest>,
) -> Result<Json<Payment>, AppError> {
    user.require_admin_or_accounts()?;

    if payload.amount <= Decimal::ZERO {
        return Err(AppError::BadRequest("Payment amount must be greater than zero".to_string()));
    }

    let received_by = if user.id.is_nil() { None } else { Some(user.id) };

    let payment = sqlx::query_as::<_, Payment>(
        r#"
        insert into payments (order_id, amount, method, payment_date, reference_no, received_by, notes)
        values ($1, $2, $3, coalesce($4, current_date), $5, $6, $7)
        returning *
        "#,
    )
    .bind(order_id)
    .bind(payload.amount)
    .bind(payload.method)
    .bind(payload.payment_date)
    .bind(payload.reference_no)
    .bind(received_by)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(payment))
}
