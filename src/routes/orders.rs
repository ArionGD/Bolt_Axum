use axum::{
    extract::{Path, Query, State},
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
        CreateOrderRequest, Order, OrderDetailResponse, OrderStatus, PaymentMethod,
        PaymentMode, UpdateOrderRequest, VehicleStatus,
    },
    error::AppError,
    services::{calculate_balance_due, generate_order_number},
};

#[derive(Debug, Deserialize)]
pub struct OrderFilter {
    pub status: Option<OrderStatus>,
    pub customer_id: Option<Uuid>,
}

#[derive(sqlx::FromRow)]
struct OrderListRow {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub vehicle_id: Uuid,
    pub quotation_id: Option<Uuid>,
    pub booking_date: NaiveDate,
    pub total_amount: Decimal,
    pub booking_amount: Decimal,
    pub status: OrderStatus,
    pub payment_mode: PaymentMode,
    pub finance_partner: Option<String>,
    pub loan_amount: Option<Decimal>,
    pub expected_delivery: Option<NaiveDate>,
    pub actual_delivery: Option<NaiveDate>,
    pub registration_number: Option<String>,
    pub insurance_policy_no: Option<String>,
    pub insurance_expiry: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub customer_name: String,
    pub customer_phone: String,
    pub vin: String,
    pub colour: String,
    pub brand: String,
    pub model_name: String,
    pub total_paid: Decimal,
}

#[derive(sqlx::FromRow)]
#[allow(dead_code)]
struct VehicleLockRow {
    pub id: Uuid,
    pub status: VehicleStatus,
    pub asking_price: Decimal,
    pub vin: String,
    pub colour: String,
    pub brand: String,
    pub model_name: String,
}

pub async fn list_orders(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Query(filter): Query<OrderFilter>,
) -> Result<Json<Vec<OrderDetailResponse>>, AppError> {
    let rows = sqlx::query_as::<_, OrderListRow>(
        r#"
        select 
            o.id, o.order_number, o.customer_id, o.vehicle_id, o.quotation_id,
            o.booking_date, o.total_amount, o.booking_amount,
            o.status,
            o.payment_mode,
            o.finance_partner, o.loan_amount, o.expected_delivery, o.actual_delivery,
            o.registration_number, o.insurance_policy_no, o.insurance_expiry,
            o.created_at,
            c.full_name as customer_name, c.phone as customer_phone,
            v.vin, v.colour,
            m.brand, m.model_name,
            coalesce(sum(p.amount), 0::numeric) as total_paid
        from orders o
        join customers c on o.customer_id = c.id
        join vehicles v on o.vehicle_id = v.id
        join vehicle_models m on v.model_id = m.id
        left join payments p on o.id = p.order_id
        where ($1::order_status is null or o.status = $1)
          and ($2::uuid is null or o.customer_id = $2)
        group by o.id, c.full_name, c.phone, v.vin, v.colour, m.brand, m.model_name
        order by o.created_at desc
        "#,
    )
    .bind(filter.status)
    .bind(filter.customer_id)
    .fetch_all(&state.pool)
    .await
    .map_err(AppError::from)?;

    let result = rows
        .into_iter()
        .map(|r| {
            let balance_due = calculate_balance_due(r.total_amount, r.total_paid);
            OrderDetailResponse {
                id: r.id,
                order_number: r.order_number,
                customer_id: r.customer_id,
                customer_name: r.customer_name,
                customer_phone: r.customer_phone,
                vehicle_id: r.vehicle_id,
                vin: r.vin,
                brand: r.brand,
                model_name: r.model_name,
                colour: r.colour,
                quotation_id: r.quotation_id,
                booking_date: r.booking_date,
                total_amount: r.total_amount,
                booking_amount: r.booking_amount,
                total_paid: r.total_paid,
                balance_due,
                status: r.status,
                payment_mode: r.payment_mode,
                finance_partner: r.finance_partner,
                loan_amount: r.loan_amount,
                expected_delivery: r.expected_delivery,
                actual_delivery: r.actual_delivery,
                registration_number: r.registration_number,
                insurance_policy_no: r.insurance_policy_no,
                insurance_expiry: r.insurance_expiry,
                created_at: r.created_at,
            }
        })
        .collect();

    Ok(Json(result))
}

pub async fn create_order(
    user: AuthUser,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<CreateOrderRequest>,
) -> Result<Json<OrderDetailResponse>, AppError> {
    user.require_admin_or_sales()?;

    // Begin atomic transaction
    let mut tx = state.pool.begin().await.map_err(AppError::from)?;

    // 1. Verify physical vehicle exists and is available
    let vehicle = sqlx::query_as::<_, VehicleLockRow>(
        r#"
        select v.id, v.status, v.asking_price, v.vin, v.colour, m.brand, m.model_name
        from vehicles v
        join vehicle_models m on v.model_id = m.id
        where v.id = $1
        for update
        "#,
    )
    .bind(payload.vehicle_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Vehicle not found".to_string()))?;

    if vehicle.status != VehicleStatus::InStock && vehicle.status != VehicleStatus::InTransit {
        return Err(AppError::Conflict(format!(
            "Vehicle {} is not available for booking (current status: {:?})",
            vehicle.vin, vehicle.status
        )));
    }

    // 2. Determine total amount: if quotation provided, use quotation's verified on-road price
    let total_amount = if let Some(quote_id) = payload.quotation_id {
        sqlx::query_scalar::<_, Decimal>("select on_road_total from quotations where id = $1")
            .bind(quote_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(AppError::from)?
            .ok_or_else(|| AppError::BadRequest("Referenced quotation not found".to_string()))?
    } else {
        vehicle.asking_price
    };

    let order_number = generate_order_number(&state.pool).await?;
    let booking_amount = payload.booking_amount.unwrap_or(Decimal::ZERO);
    let payment_mode = payload.payment_mode.unwrap_or(PaymentMode::Cash);
    let created_by = if user.id.is_nil() { None } else { Some(user.id) };

    // 3. Insert order (enforcing database unique constraint on vehicle_id)
    let order = sqlx::query_as::<_, Order>(
        r#"
        insert into orders (
            order_number, customer_id, vehicle_id, quotation_id,
            total_amount, booking_amount, status, payment_mode,
            finance_partner, loan_amount, expected_delivery, created_by
        )
        values ($1, $2, $3, $4, $5, $6, 'booked', $7, $8, $9, $10, $11)
        returning *
        "#,
    )
    .bind(&order_number)
    .bind(payload.customer_id)
    .bind(payload.vehicle_id)
    .bind(payload.quotation_id)
    .bind(total_amount)
    .bind(booking_amount)
    .bind(payment_mode)
    .bind(payload.finance_partner.as_deref())
    .bind(payload.loan_amount)
    .bind(payload.expected_delivery)
    .bind(created_by)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // 4. Update vehicle status to 'reserved' atomically
    sqlx::query("update vehicles set status = 'reserved', updated_at = now() where id = $1")
        .bind(payload.vehicle_id)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

    // 5. If booking amount > 0, record initial payment automatically
    let mut total_paid = Decimal::ZERO;
    if booking_amount > Decimal::ZERO {
        sqlx::query(
            r#"
            insert into payments (order_id, amount, method, payment_date, notes, received_by)
            values ($1, $2, $3, current_date, 'Initial booking deposit', $4)
            "#,
        )
        .bind(order.id)
        .bind(booking_amount)
        .bind(PaymentMethod::Upi)
        .bind(created_by)
        .execute(&mut *tx)
        .await
        .map_err(AppError::from)?;

        total_paid = booking_amount;
    }

    // Customer details for response
    let (customer_name, customer_phone) = sqlx::query_as::<_, (String, String)>(
        "select full_name, phone from customers where id = $1",
    )
    .bind(payload.customer_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    tx.commit().await.map_err(AppError::from)?;

    let balance_due = calculate_balance_due(total_amount, total_paid);

    Ok(Json(OrderDetailResponse {
        id: order.id,
        order_number,
        customer_id: payload.customer_id,
        customer_name,
        customer_phone,
        vehicle_id: payload.vehicle_id,
        vin: vehicle.vin,
        brand: vehicle.brand,
        model_name: vehicle.model_name,
        colour: vehicle.colour,
        quotation_id: payload.quotation_id,
        booking_date: order.booking_date,
        total_amount,
        booking_amount,
        total_paid,
        balance_due,
        status: OrderStatus::Booked,
        payment_mode,
        finance_partner: payload.finance_partner,
        loan_amount: payload.loan_amount,
        expected_delivery: payload.expected_delivery,
        actual_delivery: None,
        registration_number: None,
        insurance_policy_no: None,
        insurance_expiry: None,
        created_at: order.created_at,
    }))
}

pub async fn get_order(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<OrderDetailResponse>, AppError> {
    let r = sqlx::query_as::<_, OrderListRow>(
        r#"
        select 
            o.id, o.order_number, o.customer_id, o.vehicle_id, o.quotation_id,
            o.booking_date, o.total_amount, o.booking_amount,
            o.status,
            o.payment_mode,
            o.finance_partner, o.loan_amount, o.expected_delivery, o.actual_delivery,
            o.registration_number, o.insurance_policy_no, o.insurance_expiry,
            o.created_at,
            c.full_name as customer_name, c.phone as customer_phone,
            v.vin, v.colour,
            m.brand, m.model_name,
            coalesce(sum(p.amount), 0::numeric) as total_paid
        from orders o
        join customers c on o.customer_id = c.id
        join vehicles v on o.vehicle_id = v.id
        join vehicle_models m on v.model_id = m.id
        left join payments p on o.id = p.order_id
        where o.id = $1
        group by o.id, c.full_name, c.phone, v.vin, v.colour, m.brand, m.model_name
        "#,
    )
    .bind(id)
    .fetch_optional(&state.pool)
    .await
    .map_err(AppError::from)?
    .ok_or_else(|| AppError::NotFound("Order not found".to_string()))?;

    let balance_due = calculate_balance_due(r.total_amount, r.total_paid);

    Ok(Json(OrderDetailResponse {
        id: r.id,
        order_number: r.order_number,
        customer_id: r.customer_id,
        customer_name: r.customer_name,
        customer_phone: r.customer_phone,
        vehicle_id: r.vehicle_id,
        vin: r.vin,
        brand: r.brand,
        model_name: r.model_name,
        colour: r.colour,
        quotation_id: r.quotation_id,
        booking_date: r.booking_date,
        total_amount: r.total_amount,
        booking_amount: r.booking_amount,
        total_paid: r.total_paid,
        balance_due,
        status: r.status,
        payment_mode: r.payment_mode,
        finance_partner: r.finance_partner,
        loan_amount: r.loan_amount,
        expected_delivery: r.expected_delivery,
        actual_delivery: r.actual_delivery,
        registration_number: r.registration_number,
        insurance_policy_no: r.insurance_policy_no,
        insurance_expiry: r.insurance_expiry,
        created_at: r.created_at,
    }))
}

pub async fn update_order(
    _user: AuthUser,
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateOrderRequest>,
) -> Result<Json<Order>, AppError> {
    let mut tx = state.pool.begin().await.map_err(AppError::from)?;

    let order = sqlx::query_as::<_, Order>(
        r#"
        update orders
        set
            status = coalesce($2, status),
            payment_mode = coalesce($3, payment_mode),
            finance_partner = coalesce($4, finance_partner),
            loan_amount = coalesce($5, loan_amount),
            expected_delivery = coalesce($6, expected_delivery),
            actual_delivery = coalesce($7, actual_delivery),
            registration_number = coalesce($8, registration_number),
            insurance_policy_no = coalesce($9, insurance_policy_no),
            insurance_expiry = coalesce($10, insurance_expiry),
            updated_at = now()
        where id = $1
        returning *
        "#,
    )
    .bind(id)
    .bind(payload.status)
    .bind(payload.payment_mode)
    .bind(payload.finance_partner)
    .bind(payload.loan_amount)
    .bind(payload.expected_delivery)
    .bind(payload.actual_delivery)
    .bind(payload.registration_number)
    .bind(payload.insurance_policy_no)
    .bind(payload.insurance_expiry)
    .fetch_one(&mut *tx)
    .await
    .map_err(AppError::from)?;

    // If order is delivered, update vehicle to delivered
    if order.status == OrderStatus::Delivered {
        sqlx::query("update vehicles set status = 'delivered', updated_at = now() where id = $1")
            .bind(order.vehicle_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
    } else if order.status == OrderStatus::Cancelled {
        // If order is cancelled, free up the vehicle back to in_stock
        sqlx::query("update vehicles set status = 'in_stock', updated_at = now() where id = $1")
            .bind(order.vehicle_id)
            .execute(&mut *tx)
            .await
            .map_err(AppError::from)?;
    }

    tx.commit().await.map_err(AppError::from)?;

    Ok(Json(order))
}
