use std::env;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Config {
    pub database_url: String,
    pub supabase_jwt_secret: String,
    pub supabase_url: String,
    pub supabase_service_key: String,
    pub port: u16,
    pub app_env: String,
    pub superuser_email: String,
    pub superuser_password: String,
    pub superuser_name: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://postgres:postgres@localhost:5432/ev_dealership".to_string()
        });

        let supabase_jwt_secret = env::var("SUPABASE_JWT_SECRET").unwrap_or_else(|_| {
            "default-secret-key-for-development-mode-change-in-production".to_string()
        });

        let supabase_url = env::var("SUPABASE_URL").unwrap_or_else(|_| {
            "https://placeholder.supabase.co".to_string()
        });

        let supabase_service_key = env::var("SUPABASE_SERVICE_KEY").unwrap_or_default();

        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);

        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let superuser_email = env::var("SUPERUSER_EMAIL")
            .unwrap_or_else(|_| "superuser@voltdealership.com".to_string());

        let superuser_password = env::var("SUPERUSER_PASSWORD")
            .unwrap_or_else(|_| "SuperAdminSecret2026!".to_string());

        let superuser_name = env::var("SUPERUSER_NAME")
            .unwrap_or_else(|_| "Developer Superuser".to_string());

        Self {
            database_url,
            supabase_jwt_secret,
            supabase_url,
            supabase_service_key,
            port,
            app_env,
            superuser_email,
            superuser_password,
            superuser_name,
        }
    }
}
