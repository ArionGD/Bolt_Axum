use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::models::{
    ActivityType, CustomerType, LeadStatus, OrderStatus, PaymentMethod, PaymentMode,
    QuotationStatus, TestDriveStatus, VehicleCondition, VehicleStatus,
};

// --- Models DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub brand: String,
    pub model_name: String,
    pub variant: Option<String>,
    pub body_type: Option<String>,
    pub battery_kwh: Option<Decimal>,
    pub range_km: Option<i32>,
    pub motor_power_kw: Option<Decimal>,
    pub charging_ac_kw: Option<Decimal>,
    pub charging_dc_kw: Option<Decimal>,
    pub seating_capacity: Option<i32>,
    pub ex_showroom_price: Decimal,
}

// --- Vehicles DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateVehicleRequest {
    pub model_id: Uuid,
    pub vin: String,
    pub motor_no: Option<String>,
    pub battery_serial: Option<String>,
    pub colour: String,
    pub manufacture_year: Option<i32>,
    pub condition: Option<VehicleCondition>,
    pub status: Option<VehicleStatus>,
    pub purchase_price: Option<Decimal>,
    pub asking_price: Decimal,
    pub odometer_km: Option<i32>,
    pub location: Option<String>,
    pub arrival_date: Option<NaiveDate>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateVehicleRequest {
    pub colour: Option<String>,
    pub condition: Option<VehicleCondition>,
    pub status: Option<VehicleStatus>,
    pub purchase_price: Option<Decimal>,
    pub asking_price: Option<Decimal>,
    pub odometer_km: Option<i32>,
    pub location: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VehicleResponse {
    pub id: Uuid,
    pub model_id: Uuid,
    pub vin: String,
    pub motor_no: Option<String>,
    pub battery_serial: Option<String>,
    pub colour: String,
    pub manufacture_year: Option<i32>,
    pub condition: VehicleCondition,
    pub status: VehicleStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purchase_price: Option<Decimal>,
    pub asking_price: Decimal,
    pub odometer_km: Option<i32>,
    pub location: Option<String>,
    pub arrival_date: Option<NaiveDate>,
    pub notes: Option<String>,
    pub brand: Option<String>,
    pub model_name: Option<String>,
    pub variant: Option<String>,
    pub primary_photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddPhotoRequest {
    pub storage_path: String,
    pub is_primary: Option<bool>,
}

// --- Customers DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateCustomerRequest {
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub r#type: Option<CustomerType>,
    pub gst_number: Option<String>,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pincode: Option<String>,
    pub source: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCustomerRequest {
    pub full_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub r#type: Option<CustomerType>,
    pub gst_number: Option<String>,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pincode: Option<String>,
    pub source: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub notes: Option<String>,
}

// --- Leads DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateLeadRequest {
    pub customer_id: Uuid,
    pub model_id: Option<Uuid>,
    pub status: Option<LeadStatus>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateLeadRequest {
    pub status: Option<LeadStatus>,
    pub lost_reason: Option<String>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
}

// --- Activities DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateActivityRequest {
    pub customer_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub r#type: ActivityType,
    pub notes: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
}

// --- Test Drives DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateTestDriveRequest {
    pub customer_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub staff_id: Option<Uuid>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestDriveRequest {
    pub status: Option<TestDriveStatus>,
    pub feedback: Option<String>,
}

// --- Quotations DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateQuotationRequest {
    pub customer_id: Uuid,
    pub model_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub insurance: Option<Decimal>,
    pub registration: Option<Decimal>,
    pub accessories_total: Option<Decimal>,
    pub handling_charges: Option<Decimal>,
    pub discount: Option<Decimal>,
    pub subsidy_amount: Option<Decimal>,
    pub valid_until: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateQuotationStatusRequest {
    pub status: QuotationStatus,
}

// --- Orders DTO ---
#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub vehicle_id: Uuid,
    pub quotation_id: Option<Uuid>,
    pub booking_amount: Option<Decimal>,
    pub payment_mode: Option<PaymentMode>,
    pub finance_partner: Option<String>,
    pub loan_amount: Option<Decimal>,
    pub expected_delivery: Option<NaiveDate>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateOrderRequest {
    pub status: Option<OrderStatus>,
    pub payment_mode: Option<PaymentMode>,
    pub finance_partner: Option<String>,
    pub loan_amount: Option<Decimal>,
    pub expected_delivery: Option<NaiveDate>,
    pub actual_delivery: Option<NaiveDate>,
    pub registration_number: Option<String>,
    pub insurance_policy_no: Option<String>,
    pub insurance_expiry: Option<NaiveDate>,
}

#[derive(Debug, Serialize)]
pub struct OrderDetailResponse {
    pub id: Uuid,
    pub order_number: String,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_phone: String,
    pub vehicle_id: Uuid,
    pub vin: String,
    pub brand: String,
    pub model_name: String,
    pub colour: String,
    pub quotation_id: Option<Uuid>,
    pub booking_date: NaiveDate,
    pub total_amount: Decimal,
    pub booking_amount: Decimal,
    pub total_paid: Decimal,
    pub balance_due: Decimal,
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
}

// --- Payments DTO ---
#[derive(Debug, Deserialize)]
pub struct CreatePaymentRequest {
    pub amount: Decimal,
    pub method: PaymentMethod,
    pub payment_date: Option<NaiveDate>,
    pub reference_no: Option<String>,
    pub notes: Option<String>,
}

// --- Reports DTO ---
#[derive(Debug, Serialize)]
pub struct SalesSummaryReport {
    pub total_orders: i64,
    pub total_delivered: i64,
    pub total_booked: i64,
    pub total_sales_value: Decimal,
    pub total_cash_collected: Decimal,
    pub total_outstanding_balance: Decimal,
}

#[derive(Debug, Serialize)]
pub struct StockAgeingBucket {
    pub under_30_days: i64,
    pub days_30_to_60: i64,
    pub days_60_to_90: i64,
    pub over_90_days: i64,
}

#[derive(Debug, Serialize)]
pub struct LeadFunnelStage {
    pub stage: String,
    pub count: i64,
}
