pub mod activities;
pub mod admin;
pub mod admin_ui;
pub mod auth;
pub mod customers;
pub mod health;
pub mod leads;
pub mod models;
pub mod orders;
pub mod payments;
pub mod quotations;
pub mod reports;
pub mod test_drives;
pub mod vehicles;

use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use std::sync::Arc;
use crate::auth::AppState;

pub fn create_router(state: Arc<AppState>) -> Router {
    let api_routes = Router::new()
        // Auth Endpoints
        .route("/auth/login", post(auth::login))
        .route("/auth/customer-login", post(auth::customer_login))
        // Superuser Admin Panel Endpoints
        .route("/admin/users", get(admin::list_users))
        .route("/admin/managers", post(admin::create_manager))
        .route("/admin/users/{id}/status", patch(admin::toggle_user_status))
        .route("/admin/users/{id}", delete(admin::delete_user))
        // Vehicle Catalogue
        .route("/models", get(models::list_models).post(models::create_model))
        // Inventory
        .route("/vehicles", get(vehicles::list_vehicles).post(vehicles::create_vehicle))
        .route("/vehicles/{id}", get(vehicles::get_vehicle).patch(vehicles::update_vehicle))
        .route("/vehicles/{id}/photos", post(vehicles::add_vehicle_photo))
        .route("/vehicles/{id}/photos/{photo_id}", delete(vehicles::delete_vehicle_photo))
        // Customers
        .route("/customers", get(customers::list_customers).post(customers::create_customer))
        .route("/customers/{id}", get(customers::get_customer).patch(customers::update_customer))
        .route("/customers/{id}/activities", get(customers::get_customer_activities))
        // Leads & Pipeline
        .route("/leads", get(leads::list_leads).post(leads::create_lead))
        .route("/leads/{id}", patch(leads::update_lead))
        // Activities / Follow-ups
        .route("/activities", post(activities::log_activity))
        .route("/activities/due", get(activities::list_due_activities))
        // Test Drives
        .route("/test-drives", get(test_drives::list_test_drives).post(test_drives::schedule_test_drive))
        .route("/test-drives/{id}", patch(test_drives::update_test_drive))
        // Quotations
        .route("/quotations", get(quotations::list_quotations).post(quotations::create_quotation))
        .route("/quotations/{id}", get(quotations::get_quotation))
        .route("/quotations/{id}/status", patch(quotations::update_quotation_status))
        // Orders & Bookings
        .route("/orders", get(orders::list_orders).post(orders::create_order))
        .route("/orders/{id}", get(orders::get_order).patch(orders::update_order))
        .route("/orders/{id}/payments", get(payments::list_order_payments).post(payments::record_payment))
        // Reports
        .route("/reports/sales-summary", get(reports::sales_summary))
        .route("/reports/stock-ageing", get(reports::stock_ageing))
        .route("/reports/lead-funnel", get(reports::lead_funnel));

    Router::new()
        // Unauthenticated health ping
        .route("/healthz", get(health::healthz))
        // Superuser Admin Panel served directly from Axum on Render
        .route("/admin", get(admin_ui::admin_panel))
        .route("/admin/", get(admin_ui::admin_panel))
        .nest("/api/v1", api_routes)
        .with_state(state)
}
