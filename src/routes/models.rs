use axum::{extract::State, Json};
use std::sync::Arc;
use crate::{
    auth::{AppState, AuthUser},
    domain::{CreateModelRequest, VehicleModel},
    error::AppError,
};

pub async fn list_models(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<VehicleModel>>, AppError> {
    let models = sqlx::query_as::<_, VehicleModel>(
        "select * from vehicle_models where is_active = true order by brand, model_name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(models))
}

pub async fn create_model(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateModelRequest>,
) -> Result<Json<VehicleModel>, AppError> {
    user.require_admin()?;

    let model = sqlx::query_as::<_, VehicleModel>(
        r#"
        insert into vehicle_models (
            brand, model_name, variant, body_type, battery_kwh, range_km,
            motor_power_kw, charging_ac_kw, charging_dc_kw, seating_capacity, ex_showroom_price
        )
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        returning *
        "#,
    )
    .bind(payload.brand)
    .bind(payload.model_name)
    .bind(payload.variant)
    .bind(payload.body_type)
    .bind(payload.battery_kwh)
    .bind(payload.range_km)
    .bind(payload.motor_power_kw)
    .bind(payload.charging_ac_kw)
    .bind(payload.charging_dc_kw)
    .bind(payload.seating_capacity)
    .bind(payload.ex_showroom_price)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(model))
}
