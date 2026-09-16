use axum::{
    extract::{Path, Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::{
    auth::{AppState, AuthUser},
    domain::{
        Activity, CreateCustomerRequest, Customer, CustomerType, Lead, Order,
        UpdateCustomerRequest,
    },
    error::AppError,
};

#[derive(Debug, Deserialize)]
pub struct CustomerListQuery {
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct CustomerDetailResponse {
    pub customer: Customer,
    pub leads: Vec<Lead>,
    pub orders: Vec<Order>,
}

pub async fn list_customers(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(query): Query<CustomerListQuery>,
) -> Result<Json<Vec<Customer>>, AppError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 100);
    let offset = query.offset.unwrap_or(0).max(0);

    let customers = sqlx::query_as::<_, Customer>(
        r#"
        select * from customers
        where ($1::text is null 
            or full_name ilike '%' || $1 || '%' 
            or phone ilike '%' || $1 || '%'
            or email ilike '%' || $1 || '%')
        order by created_at desc
        limit $2 offset $3
        "#,
    )
    .bind(query.search)
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(customers))
}

pub async fn create_customer(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Json<Customer>, AppError> {
    let customer_type = payload.r#type.unwrap_or(CustomerType::Individual);

    let customer = sqlx::query_as::<_, Customer>(
        r#"
        insert into customers (
            full_name, phone, email, type, gst_number, address_line,
            city, state, pincode, source, assigned_to, notes
        )
        values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        returning *
        "#,
    )
    .bind(payload.full_name)
    .bind(payload.phone)
    .bind(payload.email)
    .bind(customer_type)
    .bind(payload.gst_number)
    .bind(payload.address_line)
    .bind(payload.city)
    .bind(payload.state)
    .bind(payload.pincode)
    .bind(payload.source)
    .bind(payload.assigned_to)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(customer))
}

pub async fn get_customer(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<CustomerDetailResponse>, AppError> {
    let customer = sqlx::query_as::<_, Customer>("select * from customers where id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(AppError::from)?
        .ok_or_else(|| AppError::NotFound("Customer not found".to_string()))?;

    let leads = sqlx::query_as::<_, Lead>(
        "select * from leads where customer_id = $1 order by created_at desc",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let orders = sqlx::query_as::<_, Order>(
        "select * from orders where customer_id = $1 order by created_at desc",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(CustomerDetailResponse {
        customer,
        leads,
        orders,
    }))
}

pub async fn update_customer(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateCustomerRequest>,
) -> Result<Json<Customer>, AppError> {
    let customer = sqlx::query_as::<_, Customer>(
        r#"
        update customers
        set
            full_name = coalesce($2, full_name),
            phone = coalesce($3, phone),
            email = coalesce($4, email),
            type = coalesce($5, type),
            gst_number = coalesce($6, gst_number),
            address_line = coalesce($7, address_line),
            city = coalesce($8, city),
            state = coalesce($9, state),
            pincode = coalesce($10, pincode),
            source = coalesce($11, source),
            assigned_to = coalesce($12, assigned_to),
            notes = coalesce($13, notes),
            updated_at = now()
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(payload.full_name)
    .bind(payload.phone)
    .bind(payload.email)
    .bind(payload.r#type)
    .bind(payload.gst_number)
    .bind(payload.address_line)
    .bind(payload.city)
    .bind(payload.state)
    .bind(payload.pincode)
    .bind(payload.source)
    .bind(payload.assigned_to)
    .bind(payload.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(customer))
}

pub async fn get_customer_activities(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<Activity>>, AppError> {
    let activities = sqlx::query_as::<_, Activity>(
        "select * from activities where customer_id = $1 order by created_at desc",
    )
    .bind(id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    Ok(Json(activities))
}
