# EV Dealership CRM — API Specification (v1.0)

Base URL: `https://<your-render-service>.onrender.com/api/v1` (or `http://localhost:8080/api/v1` locally)

All endpoints (except `/healthz`) require:
```http
Authorization: Bearer <Supabase_JWT_Token>
```

---

## 1. Health & Keep-Alive
### `GET /healthz`
- **Auth**: None
- **Purpose**: Pings PostgreSQL database (reads count from `heartbeat`, upserts current timestamp into `heartbeat`).
- **Response**: `200 OK` (`"ok"`) or `500 Internal Server Error` (`"db error"`).

---

## 2. Staff & Authentication
Staff authentication is managed by Supabase Auth (`/auth/v1/token?grant_type=password`).  
On every API request, the Axum middleware verifies:
1. Valid JWT signature against `SUPABASE_JWT_SECRET`.
2. Valid expiration date.
3. User profile from the `profiles` table to extract `staff_role` (`admin`, `sales`, `accounts`).

---

## 3. Vehicle Models (Catalogue)
### `GET /models`
- **Roles**: All
- **Response**: `[ { "id": "...", "brand": "Tata", "model_name": "Nexon EV", "variant": "Empowered+ 45", "battery_kwh": 45.0, "range_km": 489, "motor_power_kw": 106.4, "charging_ac_kw": 7.2, "charging_dc_kw": 50.0, "seating_capacity": 5, "ex_showroom_price": 1699000.00, "is_active": true } ]`

### `POST /models`
- **Roles**: `admin`
- **Request Body**: Same fields as above.

---

## 4. Vehicle Inventory (Physical Stock)
### `GET /vehicles`
- **Query Params**: `status`, `model_id`, `colour`, `condition`
- **Roles**: All (Non-admin users will have `purchase_price` omitted or set to `null`).
- **Response**: List of physical vehicles with linked model information and primary photo.

### `POST /vehicles`
- **Roles**: `admin`, `sales`
- **Request Body**:
```json
{
  "model_id": "uuid",
  "vin": "MAT612345N2P12345",
  "motor_no": "MOT-99882",
  "battery_serial": "BAT-44512",
  "colour": "Intense Teal",
  "manufacture_year": 2026,
  "condition": "new",
  "status": "in_stock",
  "purchase_price": 1520000.00,
  "asking_price": 1725000.00,
  "odometer_km": 15,
  "location": "Showroom Floor",
  "arrival_date": "2026-09-01",
  "notes": "Display unit"
}
```

### `PATCH /vehicles/:id`
- **Roles**: `admin`, `sales`
- **Request Body**: Partial update for `status`, `asking_price`, `location`, `odometer_km`, `notes`.

### `POST /vehicles/:id/photos`
- **Roles**: `admin`, `sales`
- **Request Body**: `{ "storage_path": "vehicles/vin-123/front.jpg", "is_primary": true }`

### `DELETE /vehicles/:id/photos/:photoId`
- **Roles**: `admin`, `sales`

---

## 5. Customers
### `GET /customers`
- **Query Params**: `search` (matches full_name or phone), `limit`, `offset`
- **Roles**: All

### `POST /customers`
- **Request Body**:
```json
{
  "full_name": "Rahul Verma",
  "phone": "+91 98765 43210",
  "email": "rahul.verma@example.com",
  "type": "individual",
  "gst_number": null,
  "address_line": "Flat 402, Green Meadows",
  "city": "Bengaluru",
  "state": "Karnataka",
  "pincode": "560102",
  "source": "walk-in",
  "assigned_to": "uuid",
  "notes": "Looking for 400km+ range EV for daily city commute"
}
```

### `GET /customers/:id`
- Returns customer record with active leads and purchase history.

### `PATCH /customers/:id`
- Partial update customer details.

---

## 6. Leads & Pipeline
### `GET /leads`
- **Query Params**: `status`, `assigned_to`
- **Status values**: `new`, `contacted`, `test_drive_scheduled`, `test_drive_done`, `quoted`, `negotiating`, `won`, `lost`.

### `POST /leads`
- **Request Body**:
```json
{
  "customer_id": "uuid",
  "model_id": "uuid",
  "status": "new",
  "expected_close_date": "2026-09-30",
  "assigned_to": "uuid"
}
```

### `PATCH /leads/:id`
- **Request Body**: `{ "status": "won", "lost_reason": null, "expected_close_date": "...", "assigned_to": "..." }`

---

## 7. Activities & Follow-ups
### `POST /activities`
- **Request Body**:
```json
{
  "customer_id": "uuid",
  "lead_id": "uuid",
  "type": "call",
  "notes": "Discussed subsidy and home charger installation",
  "due_at": "2026-09-18T10:00:00Z"
}
```

### `GET /activities/due`
- Returns upcoming and overdue reminders for the logged-in staff member.

---

## 8. Test Drives
### `GET /test-drives`
- **Query Params**: `from`, `to`, `status`

### `POST /test-drives`
- **Request Body**:
```json
{
  "customer_id": "uuid",
  "model_id": "uuid",
  "vehicle_id": "uuid",
  "scheduled_at": "2026-09-17T14:30:00Z"
}
```

### `PATCH /test-drives/:id`
- **Request Body**: `{ "status": "completed", "feedback": "Customer loved the regenerative braking and acceleration." }`

---

## 9. Quotations
### `GET /quotations`
- List quotes with filter by status (`draft`, `sent`, `accepted`, `expired`).

### `POST /quotations`
- **Server Rule**: `on_road_total` is computed strictly on the backend:
  `ex_showroom + insurance + registration + accessories_total + handling_charges - discount - subsidy_amount`
- Generates unique `quote_number` (e.g. `QT-2026-0001`).

---

## 10. Orders & Vehicle Booking
### `GET /orders`
- **Query Params**: `status` (`booked`, `payment_pending`, `ready_for_delivery`, `delivered`, `cancelled`)

### `POST /orders`
- **Atomic Booking**:
  - Checks if vehicle is available (`in_stock`).
  - Sets vehicle status to `reserved`.
  - Creates order linked to `vehicle_id`.
  - The `unique (vehicle_id)` constraint prevents concurrent duplicate bookings.
- **Request Body**:
```json
{
  "customer_id": "uuid",
  "vehicle_id": "uuid",
  "quotation_id": "uuid",
  "booking_amount": 25000.00,
  "payment_mode": "finance",
  "finance_partner": "HDFC Auto Finance",
  "loan_amount": 1400000.00,
  "expected_delivery": "2026-09-25"
}
```

### `GET /orders/:id`
- Returns order details, customer info, vehicle specs, list of all payments received, and dynamically calculated `balance_due`:
  `balance_due = order.total_amount - sum(payments.amount)`

### `PATCH /orders/:id`
- Update order status, registration number (after RTO), insurance policy, delivery date.

---

## 11. Payments
### `POST /orders/:id/payments`
- **Roles**: `admin`, `accounts`
- **Request Body**:
```json
{
  "amount": 50000.00,
  "method": "neft",
  "reference_no": "NEFT-HDFC-9918231",
  "payment_date": "2026-09-16",
  "notes": "Second installment towards margin money"
}
```

---

## 12. Reports & Analytics
### `GET /reports/sales-summary?from=YYYY-MM-DD&to=YYYY-MM-DD`
- Total cars booked, delivered, total sales value, total cash collected, outstanding balance.

### `GET /reports/stock-ageing`
- Inventory broken down into ageing tiers:
  - `< 30 days`
  - `30–60 days`
  - `60–90 days`
  - `> 90 days` (flagged as slow moving)

### `GET /reports/lead-funnel`
- Count of active leads grouped across each pipeline stage (`new` -> `won`/`lost`).
