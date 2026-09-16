use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{Activity, ActivityType, CreateActivityRequest},
    error::AppError,
};

#[derive(Debug, Serialize)]
pub struct DueActivityView {
    #[serde(flatten)]
    pub activity: Activity,
    pub customer_name: String,
    pub customer_phone: String,
}

#[derive(sqlx::FromRow)]
struct DueActivityRow {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub r#type: ActivityType,
    pub notes: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub staff_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub customer_name: String,
    pub customer_phone: String,
}

pub async fn log_activity(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateActivityRequest>,
) -> Result<Json<Activity>, AppError> {
    let activity = sqlx::query_as::<_, Activity>(
        r#"
        insert into activities (
            customer_id, lead_id, type, notes, due_at, staff_id
        )
        values ($1, $2, $3, $4, $5, $6)
        returning *
        "#,
    )
    .bind(payload.customer_id)
    .bind(payload.lead_id)
    .bind(payload.r#type)
    .bind(payload.notes)
    .bind(payload.due_at)
    .bind(user.id)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(activity))
}

pub async fn list_due_activities(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DueActivityView>>, AppError> {
    let staff_filter = if user.id.is_nil() { None } else { Some(user.id) };

    let rows = sqlx::query_as::<_, DueActivityRow>(
        r#"
        select 
            a.id, a.customer_id, a.lead_id, a.type,
            a.notes, a.due_at, a.completed_at, a.staff_id, a.created_at,
            c.full_name as customer_name, c.phone as customer_phone
        from activities a
        join customers c on a.customer_id = c.id
        where a.due_at is not null
          and a.completed_at is null
          and ($1::uuid is null or a.staff_id = $1 or a.staff_id is null)
        order by a.due_at asc
        limit 50
        "#,
    )
    .bind(staff_filter)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| DueActivityView {
            activity: Activity {
                id: r.id,
                customer_id: r.customer_id,
                lead_id: r.lead_id,
                r#type: r.r#type,
                notes: r.notes,
                due_at: r.due_at,
                completed_at: r.completed_at,
                staff_id: r.staff_id,
                created_at: r.created_at,
            },
            customer_name: r.customer_name,
            customer_phone: r.customer_phone,
        })
        .collect();

    Ok(Json(result))
}
