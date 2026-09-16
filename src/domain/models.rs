use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "vehicle_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    InTransit,
    InStock,
    Reserved,
    Sold,
    Delivered,
    Returned,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "vehicle_condition", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VehicleCondition {
    New,
    Demo,
    Used,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "customer_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CustomerType {
    Individual,
    Business,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "lead_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum LeadStatus {
    New,
    Contacted,
    TestDriveScheduled,
    TestDriveDone,
    Quoted,
    Negotiating,
    Won,
    Lost,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "activity_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ActivityType {
    Call,
    Whatsapp,
    Email,
    Visit,
    Note,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "test_drive_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TestDriveStatus {
    Scheduled,
    Completed,
    NoShow,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "quotation_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum QuotationStatus {
    Draft,
    Sent,
    Accepted,
    Expired,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Booked,
    PaymentPending,
    ReadyForDelivery,
    Delivered,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_mode", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMode {
    Cash,
    Finance,
    Lease,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Upi,
    Card,
    Neft,
    Cheque,
    FinanceDisbursal,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct VehicleModel {
    pub id: Uuid,
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
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Vehicle {
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
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Customer {
    pub id: Uuid,
    pub full_name: String,
    pub phone: String,
    pub email: Option<String>,
    pub r#type: CustomerType,
    pub gst_number: Option<String>,
    pub address_line: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub pincode: Option<String>,
    pub source: Option<String>,
    pub assigned_to: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Lead {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub model_id: Option<Uuid>,
    pub status: LeadStatus,
    pub lost_reason: Option<String>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activity {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub lead_id: Option<Uuid>,
    pub r#type: ActivityType,
    pub notes: Option<String>,
    pub due_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub staff_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct TestDrive {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub model_id: Option<Uuid>,
    pub scheduled_at: DateTime<Utc>,
    pub status: TestDriveStatus,
    pub staff_id: Option<Uuid>,
    pub feedback: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Quotation {
    pub id: Uuid,
    pub quote_number: String,
    pub customer_id: Uuid,
    pub model_id: Uuid,
    pub vehicle_id: Option<Uuid>,
    pub ex_showroom: Decimal,
    pub insurance: Decimal,
    pub registration: Decimal,
    pub accessories_total: Decimal,
    pub handling_charges: Decimal,
    pub discount: Decimal,
    pub subsidy_amount: Decimal,
    pub on_road_total: Decimal,
    pub valid_until: Option<NaiveDate>,
    pub status: QuotationStatus,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
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
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Payment {
    pub id: Uuid,
    pub order_id: Uuid,
    pub amount: Decimal,
    pub method: PaymentMethod,
    pub payment_date: NaiveDate,
    pub reference_no: Option<String>,
    pub received_by: Option<Uuid>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}
