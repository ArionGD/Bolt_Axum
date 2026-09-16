use axum::{
    extract::{Path, Query, State},
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{CreateTestDriveRequest, TestDrive, TestDriveStatus, UpdateTestDriveRequest},
    error::AppError,
};

#[derive(Debug, Deserialize)]
pub struct TestDriveQuery {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
    pub status: Option<TestDriveStatus>,
}

#[derive(Debug, Serialize)]
pub struct TestDriveDetailView {
    #[serde(flatten)]
    pub test_drive: TestDrive,
    pub customer_name: String,
    pub customer_phone: String,
    pub model_name: Option<String>,
    pub brand: Option<String>,
    pub vin: Option<String>,
}

#[derive(sqlx::FromRow)]
struct TestDriveRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub status: TestDriveStatus,
    pub staff_id: Option<Uuid>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
    pub customer_name: String,
    pub customer_phone: String,
    pub brand: Option<String>,
    pub model_name: Option<String>,
    pub vin: Option<String>,
}

pub async fn list_test_drives(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<TestDriveQuery>,
) -> Result<Json<Vec<TestDriveDetailView>>, AppError> {
    let rows = sqlx::query_as::<_, TestDriveRow>(
        r#"
        select 
            t.id, t.customer_id, t.vehicle_id, t.model_id, t.scheduled_at,
            t.status, t.staff_id, t.feedback, t.created_at,
            c.full_name as customer_name, c.phone as customer_phone,
            m.brand, m.model_name,
            v.vin
        from test_drives t
        join customers c on t.customer_id = c.id
        left join vehicle_models m on t.model_id = m.id
        left join vehicles v on t.vehicle_id = v.id
        where ($1::timestamptz is null or t.scheduled_at >= $1)
          and ($2::timestamptz is null or t.scheduled_at <= $2)
          and ($3::test_drive_status is null or t.status = $3)
        order by t.scheduled_at asc
        "#,
    )
    .bind(query.from)
    .bind(query.to)
    .bind(query.status)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| TestDriveDetailView {
            test_drive: TestDrive {
                id: r.id,
                customer_id: r.customer_id,
                vehicle_id: r.vehicle_id,
                model_id: r.model_id,
                scheduled_at: r.scheduled_at,
                status: r.status,
                staff_id: r.staff_id,
                feedback: r.feedback,
                created_at: r.created_at,
            },
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
            model_name: r.model_name,
            brand: r.brand,
            vin: r.vin,
        })
        .collect();

    Ok(Json(result))
}

pub async fn schedule_test_drive(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateTestDriveRequest>,
) -> Result<Json<TestDrive>, AppError> {
    let staff_id = payload.staff_id.or(if user.id.is_nil() { None } else { Some(user.id) });

    let test_drive = sqlx::query_as::<_, TestDrive>(
        r#"
        insert into test_drives (
            customer_id, vehicle_id, model_id, scheduled_at, staff_id
        )
        values ($1, $2, $3, $4, $5)
        returning *
        "#,
    )
    .bind(payload.customer_id)
    .bind(payload.vehicle_id)
    .bind(payload.model_id)
    .bind(payload.scheduled_at)
    .bind(staff_id)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(test_drive))
}

pub async fn update_test_drive(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateTestDriveRequest>,
) -> Result<Json<TestDrive>, AppError> {
    let test_drive = sqlx::query_as::<_, TestDrive>(
        r#"
        update test_drives
        set
            status = coalesce($2, status),
            feedback = coalesce($3, feedback)
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(payload.status)
    .bind(payload.feedback)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(test_drive))
}
