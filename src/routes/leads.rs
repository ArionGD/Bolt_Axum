use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{CreateLeadRequest, Lead, LeadStatus, UpdateLeadRequest},
    error::AppError,
};

#[derive(Debug, Deserialize)]
pub struct LeadFilter {
    pub status: Option<LeadStatus>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct LeadDetailView {
    #[serde(flatten)]
    pub lead: Lead,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub model_name: Option<String>,
    pub brand: Option<String>,
}

#[derive(sqlx::FromRow)]
struct LeadDetailRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub model_id: Option<Uuid>,
    pub status: LeadStatus,
    pub lost_reason: Option<String>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub customer_name: String,
    pub customer_phone: String,
    pub customer_email: Option<String>,
    pub brand: Option<String>,
    pub model_name: Option<String>,
}

pub async fn list_leads(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<LeadFilter>,
) -> Result<Json<Vec<LeadDetailView>>, AppError> {
    let rows = sqlx::query_as::<_, LeadDetailRow>(
        r#"
        select 
            l.id, l.customer_id, l.model_id, l.status,
            l.lost_reason, l.expected_close_date, l.assigned_to, l.created_at, l.updated_at,
            c.full_name as customer_name, c.phone as customer_phone, c.email as customer_email,
            m.brand, m.model_name
        from leads l
        join customers c on l.customer_id = c.id
        left join vehicle_models m on l.model_id = m.id
        where ($1::lead_status is null or l.status = $1)
          and ($2::uuid is null or l.assigned_to = $2)
        order by l.created_at desc
        "#,
    )
    .bind(filter.status)
    .bind(filter.assigned_to)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| LeadDetailView {
            lead: Lead {
                id: r.id,
                customer_id: r.customer_id,
                model_id: r.model_id,
                status: r.status,
                lost_reason: r.lost_reason,
                expected_close_date: r.expected_close_date,
                assigned_to: r.assigned_to,
                created_at: r.created_at,
                updated_at: r.updated_at,
            },
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
            customer_email: r.customer_email,
            model_name: r.model_name,
            brand: r.brand,
        })
        .collect();

    Ok(Json(result))
}

pub async fn create_lead(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateLeadRequest>,
) -> Result<Json<Lead>, AppError> {
    let status = payload.status.unwrap_or(LeadStatus::New);

    let lead = sqlx::query_as::<_, Lead>(
        r#"
        insert into leads (
            customer_id, model_id, status, expected_close_date, assigned_to
        )
        values ($1, $2, $3, $4, $5)
        returning *
        "#,
    )
    .bind(payload.customer_id)
    .bind(payload.model_id)
    .bind(status)
    .bind(payload.expected_close_date)
    .bind(payload.assigned_to)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(lead))
}

pub async fn update_lead(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateLeadRequest>,
) -> Result<Json<Lead>, AppError> {
    let lead = sqlx::query_as::<_, Lead>(
        r#"
        update leads
        set
            status = coalesce($2, status),
            lost_reason = coalesce($3, lost_reason),
            expected_close_date = coalesce($4, expected_close_date),
            assigned_to = coalesce($5, assigned_to),
            updated_at = now()
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(payload.status)
    .bind(payload.lost_reason)
    .bind(payload.expected_close_date)
    .bind(payload.assigned_to)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(lead))
}
