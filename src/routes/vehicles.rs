use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{
        AddPhotoRequest, CreateVehicleRequest, UpdateVehicleRequest, Vehicle,
        VehicleCondition, VehicleResponse, VehicleStatus,
    },
    error::AppError,
};

#[derive(Debug, Deserialize)]
pub struct VehicleFilter {
    pub status: Option<VehicleStatus>,
    pub model_id: Option<Uuid>,
    pub colour: Option<String>,
    pub condition: Option<VehicleCondition>,
}

#[derive(sqlx::FromRow)]
struct VehicleListRow {
    pub id: Uuid,
    pub model_id: Uuid,
    pub vin: String,
    pub motor_no: Option<String>,
    pub battery_serial: Option<String>,
    pub colour: String,
    pub manufacture_year: Option<i32>,
    pub condition: VehicleCondition,
    pub status: VehicleStatus,
    pub purchase_price: Option<Decimal>,
    pub asking_price: Decimal,
    pub odometer_km: Option<i32>,
    pub location: Option<String>,
    pub arrival_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub brand: String,
    pub model_name: String,
    pub variant: Option<String>,
    pub primary_photo_url: Option<String>,
}

pub async fn list_vehicles(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<VehicleFilter>,
) -> Result<Json<Vec<VehicleResponse>>, AppError> {
    let can_see_cost = user.can_see_purchase_price();

    let rows = sqlx::query_as::<_, VehicleListRow>(
        r#"
        select 
            v.id, v.model_id, v.vin, v.motor_no, v.battery_serial, v.colour,
            v.manufacture_year, v.condition,
            v.status, v.purchase_price, v.asking_price,
            v.odometer_km, v.location, v.arrival_date, v.notes, v.created_at,
            m.brand, m.model_name, m.variant,
            p.storage_path as primary_photo_url
        from vehicles v
        join vehicle_models m on v.model_id = m.id
        left join vehicle_photos p on v.id = p.vehicle_id and p.is_primary = true
        where ($1::vehicle_status is null or v.status = $1)
          and ($2::uuid is null or v.model_id = $2)
          and ($3::text is null or v.colour ilike '%' || $3 || '%')
          and ($4::vehicle_condition is null or v.condition = $4)
        order by v.created_at desc
        "#,
    )
    .bind(filter.status)
    .bind(filter.model_id)
    .bind(filter.colour)
    .bind(filter.condition)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| VehicleResponse {
            id: r.id,
            model_id: r.model_id,
            vin: r.vin,
            motor_no: r.motor_no,
            battery_serial: r.battery_serial,
            colour: r.colour,
            manufacture_year: r.manufacture_year,
            condition: r.condition,
            status: r.status,
            purchase_price: if can_see_cost { r.purchase_price } else { None },
            asking_price: r.asking_price,
            odometer_km: r.odometer_km,
            location: r.location,
            arrival_date: r.arrival_date,
            notes: r.notes,
            brand: Some(r.brand),
            model_name: Some(r.model_name),
            variant: r.variant,
            primary_photo_url: r.primary_photo_url,
            created_at: r.created_at,
        })
        .collect();

    Ok(Json(result))
}

pub async fn create_vehicle(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateVehicleRequest>,
) -> Result<Json<Vehicle>, AppError> {
    user.require_admin_or_sales()?;

    let condition = payload.condition.unwrap_or(VehicleCondition::New);
    let status = payload.status.unwrap_or(VehicleStatus::InTransit);

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        insert into vehicles (
            model_id, vin, motor_no, battery_serial, colour, manufacture_year,
            condition, status, purchase_price, asking_price, odometer_km,
            location, arrival_date, notes
        )
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        returning *
        "#,
    )
    .bind(payload.model_id)
    .bind(payload.vin)
    .bind(payload.motor_no)
    .bind(payload.battery_serial)
    .bind(payload.colour)
    .bind(payload.manufacture_year)
    .bind(condition)
    .bind(status)
    .bind(payload.purchase_price)
    .bind(payload.asking_price)
    .bind(payload.odometer_km.unwrap_or(0))
    .bind(payload.location)
    .bind(payload.arrival_date)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(vehicle))
}

pub async fn get_vehicle(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<VehicleResponse>, AppError> {
    let can_see_cost = user.can_see_purchase_price();

    let r = sqlx::query_as::<_, VehicleListRow>(
        r#"
        select 
            v.id, v.model_id, v.vin, v.motor_no, v.battery_serial, v.colour,
            v.manufacture_year, v.condition,
            v.status, v.purchase_price, v.asking_price,
            v.odometer_km, v.location, v.arrival_date, v.notes, v.created_at,
            m.brand, m.model_name, m.variant,
            p.storage_path as primary_photo_url
        from vehicles v
        join vehicle_models m on v.model_id = m.id
        left join vehicle_photos p on v.id = p.vehicle_id and p.is_primary = true
        where v.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    Ok(Json(VehicleResponse {
        id: r.id,
        model_id: r.model_id,
        vin: r.vin,
        motor_no: r.motor_no,
        battery_serial: r.battery_serial,
        colour: r.colour,
        manufacture_year: r.manufacture_year,
        condition: r.condition,
        status: r.status,
        purchase_price: if can_see_cost { r.purchase_price } else { None },
        asking_price: r.asking_price,
        odometer_km: r.odometer_km,
        location: r.location,
        arrival_date: r.arrival_date,
        notes: r.notes,
        brand: Some(r.brand),
        model_name: Some(r.model_name),
        variant: r.variant,
        primary_photo_url: r.primary_photo_url,
        created_at: r.created_at,
    }))
}

pub async fn update_vehicle(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateVehicleRequest>,
) -> Result<Json<Vehicle>, AppError> {
    user.require_admin_or_sales()?;

    let vehicle = sqlx::query_as::<_, Vehicle>(
        r#"
        update vehicles
        set
            colour = coalesce($2, colour),
            condition = coalesce($3, condition),
            status = coalesce($4, status),
            purchase_price = coalesce($5, purchase_price),
            asking_price = coalesce($6, asking_price),
            odometer_km = coalesce($7, odometer_km),
            location = coalesce($8, location),
            notes = coalesce($9, notes),
            updated_at = now()
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(payload.colour)
    .bind(payload.condition)
    .bind(payload.status)
    .bind(if user.can_see_purchase_price() { payload.purchase_price } else { None })
    .bind(payload.asking_price)
    .bind(payload.odometer_km)
    .bind(payload.location)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(vehicle))
}

pub async fn add_vehicle_photo(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<AddPhotoRequest>,
) -> Result<StatusCode, AppError> {
    user.require_admin_or_sales()?;

    let is_primary = payload.is_primary.unwrap_or(false);

    if is_primary {
        sqlx::query("update vehicle_photos set is_primary = false where vehicle_id = $1")
            .bind(id)
            .execute(&state.pool)
            .await
            .map_err(AppError::from)?;
    }

    sqlx::query(
        "insert into vehicle_photos (vehicle_id, storage_path, is_primary) values ($1, $2, $3)",
    )
    .bind(id)
    .bind(payload.storage_path)
    .bind(is_primary)
    .execute(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(StatusCode::CREATED)
}

pub async fn delete_vehicle_photo(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path((_id, photo_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, AppError> {
    user.require_admin_or_sales()?;

    sqlx::query("delete from vehicle_photos where id = $1")
        .bind(photo_id)
        .execute(&state.pool)
        .await
        .map_err(AppError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
