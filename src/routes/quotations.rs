use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{CreateQuotationRequest, Quotation, QuotationStatus, UpdateQuotationStatusRequest},
    error::AppError,
    services::{generate_quote_number, PricingComponents},
};

#[derive(Debug, Deserialize)]
pub struct QuotationFilter {
    pub status: Option<QuotationStatus>,
    pub customer_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct QuotationDetailView {
    #[serde(flatten)]
    pub quotation: Quotation,
    pub customer_name: String,
    pub customer_phone: String,
    pub brand: String,
    pub model_name: String,
    pub variant: Option<String>,
}

#[derive(sqlx::FromRow)]
struct QuotationRow {
    pub id: Uuid,
    pub quote_number: String,
    pub customer_id: Uuid,
    pub model_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub ex_showroom: Decimal,
    pub insurance: Option<Decimal>,
    pub registration: Option<Decimal>,
    pub accessories_total: Option<Decimal>,
    pub handling_charges: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub subsidy_amount: Option<Decimal>,
    pub on_road_total: Decimal,
    pub valid_until: Option<NaiveDate>,
    pub status: QuotationStatus,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub customer_name: String,
    pub customer_phone: String,
    pub brand: String,
    pub model_name: String,
    pub variant: Option<String>,
}

pub async fn list_quotations(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<QuotationFilter>,
) -> Result<Json<Vec<QuotationDetailView>>, AppError> {
    let rows = sqlx::query_as::<_, QuotationRow>(
        r#"
        select 
            q.id, q.quote_number, q.customer_id, q.model_id, q.vehicle_id,
            q.ex_showroom, q.insurance, q.registration, q.accessories_total,
            q.handling_charges, q.discount, q.subsidy_amount, q.on_road_total,
            q.valid_until, q.status, q.created_by, q.created_at,
            c.full_name as customer_name, c.phone as customer_phone,
            m.brand, m.model_name, m.variant
        from quotations q
        join customers c on q.customer_id = c.id
        join vehicle_models m on q.model_id = m.id
        where ($1::quotation_status is null or q.status = $1)
          and ($2::uuid is null or q.customer_id = $2)
        order by q.created_at desc
        "#,
    )
    .bind(filter.status)
    .bind(filter.customer_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| QuotationDetailView {
            quotation: Quotation {
                id: r.id,
                quote_number: r.quote_number,
                customer_id: r.customer_id,
                model_id: r.model_id,
                vehicle_id: r.vehicle_id,
                ex_showroom: r.ex_showroom,
                insurance: r.insurance.unwrap_or(Decimal::ZERO),
                registration: r.registration.unwrap_or(Decimal::ZERO),
                accessories_total: r.accessories_total.unwrap_or(Decimal::ZERO),
                handling_charges: r.handling_charges.unwrap_or(Decimal::ZERO),
                discount: r.discount.unwrap_or(Decimal::ZERO),
                subsidy_amount: r.subsidy_amount.unwrap_or(Decimal::ZERO),
                on_road_total: r.on_road_total,
                valid_until: r.valid_until,
                status: r.status,
                created_by: r.created_by,
                created_at: r.created_at,
            },
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
            brand: r.brand,
            model_name: r.model_name,
            variant: r.variant,
        })
        .collect();

    Ok(Json(result))
}

pub async fn create_quotation(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateQuotationRequest>,
) -> Result<Json<Quotation>, AppError> {
    // 1. Fetch official ex-showroom price from catalogue (server-side verification)
    let ex_showroom = sqlx::query_scalar::<_, Decimal>(
        "select ex_showroom_price from vehicle_models where id = $1 and is_active = true",
    )
    .bind(payload.model_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::BadRequest("Selected vehicle model does not exist or is inactive".to_string()))?;

    let insurance = payload.insurance.unwrap_or(Decimal::ZERO);
    let registration = payload.registration.unwrap_or(Decimal::ZERO);
    let accessories = payload.accessories_total.unwrap_or(Decimal::ZERO);
    let handling = payload.handling_charges.unwrap_or(Decimal::ZERO);
    let discount = payload.discount.unwrap_or(Decimal::ZERO);
    let subsidy = payload.subsidy_amount.unwrap_or(Decimal::ZERO);

    // 2. Compute accurate on-road total on the server
    let pricing = PricingComponents {
        ex_showroom,
        insurance,
        registration,
        accessories_total: accessories,
        handling_charges: handling,
        discount,
        subsidy_amount: subsidy,
    };
    let on_road_total = pricing.calculate_on_road_total();

    // 3. Generate unique sequential quote number (e.g. QT-2026-0001)
    let quote_number = generate_quote_number(&state.pool).await?;

    let created_by = if user.id.is_nil() { None } else { Some(user.id) };

    let quotation = sqlx::query_as::<_, Quotation>(
        r#"
        insert into quotations (
            quote_number, customer_id, model_id, vehicle_id,
            ex_showroom, insurance, registration, accessories_total,
            handling_charges, discount, subsidy_amount, on_road_total,
            valid_until, status, created_by
        )
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'draft', $14)
        returning *
        "#,
    )
    .bind(quote_number)
    .bind(payload.customer_id)
    .bind(payload.model_id)
    .bind(payload.vehicle_id)
    .bind(ex_showroom)
    .bind(insurance)
    .bind(registration)
    .bind(accessories)
    .bind(handling)
    .bind(discount)
    .bind(subsidy)
    .bind(on_road_total)
    .bind(payload.valid_until)
    .bind(created_by)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(quotation))
}

pub async fn get_quotation(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<QuotationDetailView>, AppError> {
    let r = sqlx::query_as::<_, QuotationRow>(
        r#"
        select 
            q.id, q.quote_number, q.customer_id, q.model_id, q.vehicle_id,
            q.ex_showroom, q.insurance, q.registration, q.accessories_total,
            q.handling_charges, q.discount, q.subsidy_amount, q.on_road_total,
            q.valid_until, q.status, q.created_by, q.created_at,
            c.full_name as customer_name, c.phone as customer_phone,
            m.brand, m.model_name, m.variant
        from quotations q
        join customers c on q.customer_id = c.id
        join vehicle_models m on q.model_id = m.id
        where q.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Quotation not found".to_string()))?;

    Ok(Json(QuotationDetailView {
        quotation: Quotation {
            id: r.id,
            quote_number: r.quote_number,
            customer_id: r.customer_id,
            model_id: r.model_id,
            vehicle_id: r.vehicle_id,
            ex_showroom: r.ex_showroom,
            insurance: r.insurance.unwrap_or(Decimal::ZERO),
            registration: r.registration.unwrap_or(Decimal::ZERO),
            accessories_total: r.accessories_total.unwrap_or(Decimal::ZERO),
            handling_charges: r.handling_charges.unwrap_or(Decimal::ZERO),
            discount: r.discount.unwrap_or(Decimal::ZERO),
            subsidy_amount: r.subsidy_amount.unwrap_or(Decimal::ZERO),
            on_road_total: r.on_road_total,
            valid_until: r.valid_until,
            status: r.status,
            created_by: r.created_by,
            created_at: r.created_at,
        },
        customer_name: r.customer_name,
        customer_phone: r.customer_phone,
        brand: r.brand,
        model_name: r.model_name,
        variant: r.variant,
    }))
}

pub async fn update_quotation_status(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateQuotationStatusRequest>,
) -> Result<Json<Quotation>, AppError> {
    let quotation = sqlx::query_as::<_, Quotation>(
        "update quotations set status = $2 where id = $1 returning *",
    )
    .bind(id)
    .bind(payload.status)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(quotation))
}
