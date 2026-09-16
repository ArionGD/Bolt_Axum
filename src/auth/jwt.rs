use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error::AppError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SupabaseClaims {
    pub sub: String,            // Supabase auth user UUID
    pub email: Option<String>,
    pub exp: usize,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
#[serde(rename_all = "lowercase")]
pub enum StaffRole {
    Superuser,
    Manager,
    Customer,
    Admin,
    Sales,
    Accounts,
}

impl StaffRole {
    #[allow(dead_code)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "superuser" => Some(StaffRole::Superuser),
            "manager" => Some(StaffRole::Manager),
            "customer" => Some(StaffRole::Customer),
            "admin" => Some(StaffRole::Admin),
            "sales" => Some(StaffRole::Sales),
            "accounts" => Some(StaffRole::Accounts),
            _ => None,
        }
    }

    pub fn is_superuser(&self) -> bool {
        matches!(self, StaffRole::Superuser | StaffRole::Admin)
    }

    pub fn is_manager_or_above(&self) -> bool {
        matches!(
            self,
            StaffRole::Superuser
                | StaffRole::Manager
                | StaffRole::Admin
                | StaffRole::Sales
                | StaffRole::Accounts
        )
    }
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<Uuid, AppError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let token_data = decode::<SupabaseClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|e| AppError::AuthError(format!("Invalid token: {}", e)))?;

    Uuid::parse_str(&token_data.claims.sub)
        .map_err(|_| AppError::AuthError("Malformed user ID in token claims".to_string()))
}
