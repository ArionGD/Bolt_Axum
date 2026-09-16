use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::jwt::{verify_jwt, StaffRole},
    config::Config,
    db::DbPool,
    error::AppError,
};

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: StaffRole,
}

impl AuthUser {
    pub fn require_superuser(&self) -> Result<(), AppError> {
        if self.role.is_superuser() {
            Ok(())
        } else {
            Err(AppError::Forbidden("Action restricted to developer superuser".to_string()))
        }
    }

    pub fn require_admin(&self) -> Result<(), AppError> {
        if self.role.is_manager_or_above() {
            Ok(())
        } else {
            Err(AppError::Forbidden("Action restricted to showroom managers and administrators".to_string()))
        }
    }

    pub fn require_admin_or_sales(&self) -> Result<(), AppError> {
        if self.role.is_manager_or_above() {
            Ok(())
        } else {
            Err(AppError::Forbidden("Action restricted to sales or manager staff".to_string()))
        }
    }

    pub fn require_admin_or_accounts(&self) -> Result<(), AppError> {
        if self.role.is_manager_or_above() {
            Ok(())
        } else {
            Err(AppError::Forbidden("Action restricted to accounts or manager staff".to_string()))
        }
    }

    pub fn can_see_purchase_price(&self) -> bool {
        self.role.is_superuser() || self.role == StaffRole::Manager || self.role == StaffRole::Admin
    }
}

pub struct AppState {
    pub pool: DbPool,
    pub config: Config,
}

impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    Arc<AppState>: axum::extract::FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state: Arc<AppState> = axum::extract::FromRef::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok());

        let token = match auth_header {
            Some(header) if header.starts_with("Bearer ") => &header[7..],
            _ => {
                return Err((
                    StatusCode::UNAUTHORIZED,
                    Json(json!({"error": "Missing or malformed Authorization header", "code": "UNAUTHORIZED"})),
                )
                    .into_response());
            }
        };

        // Development/Demo fallback tokens for testing without active Supabase instance
        if app_state.config.app_env == "development" {
            if token == "demo-superuser-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Superuser,
                });
            } else if token == "demo-manager-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Manager,
                });
            } else if token == "demo-customer-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Customer,
                });
            } else if token == "demo-admin-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Admin,
                });
            } else if token == "demo-sales-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Sales,
                });
            } else if token == "demo-accounts-token" {
                return Ok(AuthUser {
                    id: Uuid::nil(),
                    role: StaffRole::Accounts,
                });
            }
        }

        // Verify Supabase JWT
        let user_id = match verify_jwt(token, &app_state.config.supabase_jwt_secret) {
            Ok(id) => id,
            Err(err) => return Err(err.into_response()),
        };

        // Query staff role from database profiles table
        let profile_role = sqlx::query_scalar::<_, StaffRole>(
            "select role from profiles where id = $1 and is_active = true",
        )
        .bind(user_id)
        .fetch_optional(&app_state.pool)
        .await
        .map_err(|e| AppError::DatabaseError(e).into_response())?;

        let role = profile_role.unwrap_or(StaffRole::Sales);

        Ok(AuthUser { id: user_id, role })
    }
}
