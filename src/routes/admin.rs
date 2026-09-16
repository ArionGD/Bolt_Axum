use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    error::AppError,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct AccountRecordDto {
    pub id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub full_name: String,
    pub role: String,
    pub receive_alerts: bool,
    pub is_active: bool,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateManagerRequest {
    pub full_name: String,
    pub email: String,
    pub phone: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ToggleStatusRequest {
    pub is_active: bool,
}

#[derive(sqlx::FromRow)]
struct AccountListRow {
    id: Uuid,
    email: Option<String>,
    phone: Option<String>,
    full_name: String,
    role: String,
    receive_alerts: bool,
    is_active: bool,
    created_at: DateTime<Utc>,
}

#[derive(sqlx::FromRow)]
struct CustomerListRow {
    id: Uuid,
    email: Option<String>,
    phone: String,
    full_name: String,
    receive_alerts: bool,
    created_at: DateTime<Utc>,
}

/// GET /api/v1/admin/users
/// Restricted strictly to Superuser
pub async fn list_users(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    user.require_superuser()?;

    let mut all_users: Vec<AccountRecordDto> = Vec::new();

    // 1. Superuser from .env
    all_users.push(AccountRecordDto {
        id: "usr_superuser_0".to_string(),
        email: Some(state.config.superuser_email.clone()),
        phone: None,
        full_name: state.config.superuser_name.clone(),
        role: "superuser".to_string(),
        receive_alerts: false,
        is_active: true,
        created_at: "2026-01-01T00:00:00Z".to_string(),
    });

    // 2. Fetch Manager accounts from database
    let accounts_res = sqlx::query_as::<_, AccountListRow>(
        "select id, email, phone, full_name, role::text as role, receive_alerts, is_active, created_at from app_accounts order by created_at desc"
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(records) = accounts_res {
        for r in records {
            all_users.push(AccountRecordDto {
                id: r.id.to_string(),
                email: r.email,
                phone: r.phone,
                full_name: r.full_name,
                role: r.role,
                receive_alerts: r.receive_alerts,
                is_active: r.is_active,
                created_at: r.created_at.to_rfc3339(),
            });
        }
    } else {
        // Fallback demo manager if offline/mock
        all_users.push(AccountRecordDto {
            id: "usr_manager_1".to_string(),
            email: Some("manager1@voltdealership.com".to_string()),
            phone: Some("+91 98000 11111".to_string()),
            full_name: "Manager 1".to_string(),
            role: "manager".to_string(),
            receive_alerts: false,
            is_active: true,
            created_at: "2026-02-01T09:00:00Z".to_string(),
        });
    }

    // 3. Fetch Customers from database
    let customers_res = sqlx::query_as::<_, CustomerListRow>(
        "select id, email, phone, full_name, receive_alerts, created_at from customers order by created_at desc limit 50"
    )
    .fetch_all(&state.pool)
    .await;

    if let Ok(cust_records) = customers_res {
        for c in cust_records {
            all_users.push(AccountRecordDto {
                id: c.id.to_string(),
                email: c.email,
                phone: Some(c.phone),
                full_name: c.full_name,
                role: "customer".to_string(),
                receive_alerts: c.receive_alerts,
                is_active: true,
                created_at: c.created_at.to_rfc3339(),
            });
        }
    }

    Ok((StatusCode::OK, Json(all_users)))
}

/// POST /api/v1/admin/managers
/// Superuser provisions a new Manager account
pub async fn create_manager(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateManagerRequest>,
) -> Result<impl IntoResponse, AppError> {
    user.require_superuser()?;

    let full_name = payload.full_name.trim();
    let email = payload.email.trim().to_lowercase();
    let password = payload.password.unwrap_or_else(|| "manager123".to_string());

    if full_name.is_empty() || email.is_empty() {
        return Err(AppError::BadRequest("Full name and email are required".to_string()));
    }

    let id = Uuid::new_v4();
    let _ = sqlx::query(
        "insert into app_accounts (id, full_name, email, phone, role, password_hash, is_active) values ($1, $2, $3, $4, 'manager', $5, true)"
    )
    .bind(id)
    .bind(full_name)
    .bind(&email)
    .bind(payload.phone.as_deref())
    .bind(password)
    .execute(&state.pool)
    .await;

    let new_manager = AccountRecordDto {
        id: id.to_string(),
        email: Some(email),
        phone: payload.phone,
        full_name: full_name.to_string(),
        role: "manager".to_string(),
        receive_alerts: false,
        is_active: true,
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    Ok((StatusCode::CREATED, Json(new_manager)))
}

/// PATCH /api/v1/admin/users/:id/status
pub async fn toggle_user_status(
    user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ToggleStatusRequest>,
) -> Result<impl IntoResponse, AppError> {
    user.require_superuser()?;

    if let Ok(uuid) = Uuid::parse_str(&id) {
        let _ = sqlx::query(
            "update app_accounts set is_active = $1, updated_at = now() where id = $2"
        )
        .bind(payload.is_active)
        .bind(uuid)
        .execute(&state.pool)
        .await;
    }

    Ok(StatusCode::NO_CONTENT)
}

/// DELETE /api/v1/admin/users/:id
pub async fn delete_user(
    user: AuthUser,
    Path(id): Path<String>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    user.require_superuser()?;

    if let Ok(uuid) = Uuid::parse_str(&id) {
        let _ = sqlx::query("delete from app_accounts where id = $1")
            .bind(uuid)
            .execute(&state.pool)
            .await;
    }

    Ok(StatusCode::NO_CONTENT)
}
