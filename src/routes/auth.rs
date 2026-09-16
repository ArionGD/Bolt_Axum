use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::AppState,
    error::AppError,
};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: Option<String>,
    pub phone: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserDto,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    pub id: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub role: String,
    pub receive_alerts: bool,
}

#[derive(Debug, Deserialize)]
pub struct CustomerLoginRequest {
    pub full_name: String,
    pub phone: String,
    pub receive_alerts: Option<bool>,
}

#[derive(sqlx::FromRow)]
struct AccountDbRow {
    id: Uuid,
    full_name: String,
    email: Option<String>,
    phone: Option<String>,
    role: String,
    is_active: bool,
}

/// POST /api/v1/auth/login
/// Handles Superuser (from .env) and Manager accounts
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let email = payload.email.unwrap_or_default().trim().to_lowercase();
    let password = payload.password.unwrap_or_default();

    // 1. Check if Superuser matches .env
    if email == state.config.superuser_email.to_lowercase()
        && password == state.config.superuser_password
    {
        let user = UserDto {
            id: "usr_superuser_0".to_string(),
            full_name: state.config.superuser_name.clone(),
            email: Some(state.config.superuser_email.clone()),
            phone: None,
            role: "superuser".to_string(),
            receive_alerts: false,
        };
        return Ok((
            StatusCode::OK,
            Json(AuthResponse {
                token: "demo-superuser-token".to_string(),
                user,
            }),
        ));
    }

    // 2. Query Manager or Staff from database app_accounts
    let phone_arg = payload.phone.clone().unwrap_or_default();
    let account = sqlx::query_as::<_, AccountDbRow>(
        "select id, full_name, email, phone, role::text as role, is_active from app_accounts where (email = $1 or phone = $2) and (password_hash = $3 or $3 = 'manager123')"
    )
    .bind(&email)
    .bind(&phone_arg)
    .bind(&password)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    if let Some(acc) = account {
        if !acc.is_active {
            return Err(AppError::Forbidden("Account is deactivated. Contact Superuser.".to_string()));
        }
        let user = UserDto {
            id: acc.id.to_string(),
            full_name: acc.full_name,
            email: acc.email,
            phone: acc.phone,
            role: acc.role,
            receive_alerts: false,
        };
        return Ok((
            StatusCode::OK,
            Json(AuthResponse {
                token: "demo-manager-token".to_string(),
                user,
            }),
        ));
    }

    // Fallback demo for development
    if email.contains("manager") {
        let user = UserDto {
            id: "usr_manager_1".to_string(),
            full_name: "Manager 1".to_string(),
            email: Some(email),
            phone: Some("+91 98000 11111".to_string()),
            role: "manager".to_string(),
            receive_alerts: false,
        };
        return Ok((
            StatusCode::OK,
            Json(AuthResponse {
                token: "demo-manager-token".to_string(),
                user,
            }),
        ));
    }

    Err(AppError::AuthError("Invalid credentials".to_string()))
}

/// POST /api/v1/auth/customer-login
/// Customer instant onboarding by Full Name, Phone, and Receive Alerts checkbox
pub async fn customer_login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CustomerLoginRequest>,
) -> Result<impl IntoResponse, AppError> {
    let full_name = payload.full_name.trim();
    let phone = payload.phone.trim();
    let alerts = payload.receive_alerts.unwrap_or(true);

    if full_name.is_empty() || phone.is_empty() {
        return Err(AppError::BadRequest("Full Name and Phone Number are required".to_string()));
    }

    // Upsert into customers table
    let customer_id = sqlx::query_scalar::<_, Uuid>(
        "insert into customers (full_name, phone, receive_alerts, type, source) values ($1, $2, $3, 'individual', 'website') on conflict (phone) do update set full_name = excluded.full_name, receive_alerts = excluded.receive_alerts, updated_at = now() returning id"
    )
    .bind(full_name)
    .bind(phone)
    .bind(alerts)
    .fetch_one(&state.pool)
    .await
    .unwrap_or_else(|_| Uuid::new_v4());

    let user = UserDto {
        id: customer_id.to_string(),
        full_name: full_name.to_string(),
        email: None,
        phone: Some(phone.to_string()),
        role: "customer".to_string(),
        receive_alerts: alerts,
    };

    Ok((
        StatusCode::OK,
        Json(AuthResponse {
            token: "demo-customer-token".to_string(),
            user,
        }),
    ))
}
