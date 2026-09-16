-- ============================================================
-- EV DEALERSHIP CRM + INVENTORY SYSTEM — DATABASE SCHEMA (v1.0)
-- Target: Supabase (PostgreSQL 15+)
-- ============================================================

-- ============================================================
-- 0. Keep-alive table (UptimeRobot ping target)
-- ============================================================
create table if not exists heartbeat (
  id          int primary key default 1,
  pinged_at   timestamptz not null default now()
);

insert into heartbeat (id) values (1)
on conflict (id) do update set pinged_at = now();

-- ============================================================
-- 1. Staff profiles (linked to Supabase Auth users)
-- ============================================================
do $$ begin
  create type staff_role as enum ('admin', 'sales', 'accounts');
exception
  when duplicate_object then null;
end $$;

create table if not exists profiles (
  id          uuid primary key references auth.users(id) on delete cascade,
  full_name   text not null,
  phone       text,
  role        staff_role not null default 'sales',
  is_active   boolean not null default true,
  created_at  timestamptz not null default now()
);

-- ============================================================
-- 2. Vehicle catalogue — the models sold
-- ============================================================
create table if not exists vehicle_models (
  id                uuid primary key default gen_random_uuid(),
  brand             text not null,
  model_name        text not null,
  variant           text,
  body_type         text,
  battery_kwh       numeric(5,2),
  range_km          int,
  motor_power_kw    numeric(6,2),
  charging_ac_kw    numeric(5,2),
  charging_dc_kw    numeric(6,2),
  seating_capacity  int,
  ex_showroom_price numeric(12,2) not null,
  is_active         boolean not null default true,
  created_at        timestamptz not null default now(),
  unique (brand, model_name, variant)
);

-- ============================================================
-- 3. Inventory — physical vehicles in stock
-- ============================================================
do $$ begin
  create type vehicle_status as enum
    ('in_transit', 'in_stock', 'reserved', 'sold', 'delivered', 'returned');
exception
  when duplicate_object then null;
end $$;

do $$ begin
  create type vehicle_condition as enum ('new', 'demo', 'used');
exception
  when duplicate_object then null;
end $$;

create table if not exists vehicles (
  id                uuid primary key default gen_random_uuid(),
  model_id          uuid not null references vehicle_models(id),
  vin               text not null unique,      -- chassis number
  motor_no          text,
  battery_serial    text,
  colour            text not null,
  manufacture_year  int,
  condition         vehicle_condition not null default 'new',
  status            vehicle_status not null default 'in_transit',
  purchase_price    numeric(12,2),             -- dealer cost, admin only
  asking_price      numeric(12,2) not null,
  odometer_km       int default 0,
  location          text,                      -- showroom / yard / workshop
  arrival_date      date,
  notes             text,
  created_at        timestamptz not null default now(),
  updated_at        timestamptz not null default now()
);

create index if not exists idx_vehicles_status on vehicles (status);
create index if not exists idx_vehicles_model_id on vehicles (model_id);

create table if not exists vehicle_photos (
  id            uuid primary key default gen_random_uuid(),
  vehicle_id    uuid not null references vehicles(id) on delete cascade,
  storage_path  text not null,                 -- path inside Supabase Storage
  is_primary    boolean not null default false,
  uploaded_at   timestamptz not null default now()
);

-- ============================================================
-- 4. Customers
-- ============================================================
do $$ begin
  create type customer_type as enum ('individual', 'business');
exception
  when duplicate_object then null;
end $$;

create table if not exists customers (
  id            uuid primary key default gen_random_uuid(),
  full_name     text not null,
  phone         text not null,
  email         text,
  type          customer_type not null default 'individual',
  gst_number    text,                          -- business buyers
  address_line  text,
  city          text,
  state         text,
  pincode       text,
  source        text,                          -- walk-in / website / referral / event
  assigned_to   uuid references profiles(id),
  notes         text,
  created_at    timestamptz not null default now(),
  updated_at    timestamptz not null default now()
);

create index if not exists idx_customers_phone on customers (phone);
create index if not exists idx_customers_assigned_to on customers (assigned_to);

-- ============================================================
-- 5. Enquiries / leads pipeline
-- ============================================================
do $$ begin
  create type lead_status as enum
    ('new', 'contacted', 'test_drive_scheduled', 'test_drive_done',
     'quoted', 'negotiating', 'won', 'lost');
exception
  when duplicate_object then null;
end $$;

create table if not exists leads (
  id                  uuid primary key default gen_random_uuid(),
  customer_id         uuid not null references customers(id) on delete cascade,
  model_id            uuid references vehicle_models(id),
  status              lead_status not null default 'new',
  lost_reason         text,
  expected_close_date date,
  assigned_to         uuid references profiles(id),
  created_at          timestamptz not null default now(),
  updated_at          timestamptz not null default now()
);

create index if not exists idx_leads_status on leads (status);

-- ============================================================
-- 6. Follow-ups / activity log
-- ============================================================
do $$ begin
  create type activity_type as enum ('call', 'whatsapp', 'email', 'visit', 'note');
exception
  when duplicate_object then null;
end $$;

create table if not exists activities (
  id            uuid primary key default gen_random_uuid(),
  customer_id   uuid not null references customers(id) on delete cascade,
  lead_id       uuid references leads(id) on delete cascade,
  type          activity_type not null,
  notes         text,
  due_at        timestamptz,                   -- set = it's a reminder
  completed_at  timestamptz,
  staff_id      uuid references profiles(id),
  created_at    timestamptz not null default now()
);

create index if not exists idx_activities_due on activities (due_at) where completed_at is null;

-- ============================================================
-- 7. Test drives
-- ============================================================
do $$ begin
  create type test_drive_status as enum
    ('scheduled', 'completed', 'no_show', 'cancelled');
exception
  when duplicate_object then null;
end $$;

create table if not exists test_drives (
  id            uuid primary key default gen_random_uuid(),
  customer_id   uuid not null references customers(id),
  vehicle_id    uuid references vehicles(id),
  model_id      uuid references vehicle_models(id),
  scheduled_at  timestamptz not null,
  status        test_drive_status not null default 'scheduled',
  staff_id      uuid references profiles(id),
  feedback      text,
  created_at    timestamptz not null default now()
);

-- ============================================================
-- 8. Quotations
-- ============================================================
do $$ begin
  create type quotation_status as enum ('draft', 'sent', 'accepted', 'expired');
exception
  when duplicate_object then null;
end $$;

create table if not exists quotations (
  id                uuid primary key default gen_random_uuid(),
  quote_number      text not null unique,
  customer_id       uuid not null references customers(id),
  model_id          uuid not null references vehicle_models(id),
  vehicle_id        uuid references vehicles(id),
  ex_showroom       numeric(12,2) not null,
  insurance         numeric(12,2) default 0,
  registration      numeric(12,2) default 0,
  accessories_total numeric(12,2) default 0,
  handling_charges  numeric(12,2) default 0,
  discount          numeric(12,2) default 0,
  subsidy_amount    numeric(12,2) default 0,
  on_road_total     numeric(12,2) not null,
  valid_until       date,
  status            quotation_status not null default 'draft',
  created_by        uuid references profiles(id),
  created_at        timestamptz not null default now()
);

-- ============================================================
-- 9. Orders (bookings and sales)
-- ============================================================
do $$ begin
  create type order_status as enum
    ('booked', 'payment_pending', 'ready_for_delivery',
     'delivered', 'cancelled');
exception
  when duplicate_object then null;
end $$;

do $$ begin
  create type payment_mode as enum ('cash', 'finance', 'lease');
exception
  when duplicate_object then null;
end $$;

create table if not exists orders (
  id                    uuid primary key default gen_random_uuid(),
  order_number          text not null unique,   -- e.g. ORD-2026-0001
  customer_id           uuid not null references customers(id),
  vehicle_id            uuid not null references vehicles(id),
  quotation_id          uuid references quotations(id),
  booking_date          date not null default current_date,
  total_amount          numeric(12,2) not null,
  booking_amount        numeric(12,2) not null default 0,
  status                order_status not null default 'booked',
  payment_mode          payment_mode not null default 'cash',
  finance_partner       text,
  loan_amount           numeric(12,2),
  expected_delivery     date,
  actual_delivery       date,
  registration_number   text,                   -- filled after RTO
  insurance_policy_no   text,
  insurance_expiry      date,
  created_by            uuid references profiles(id),
  created_at            timestamptz not null default now(),
  updated_at            timestamptz not null default now(),
  unique (vehicle_id)                           -- one live order per vehicle (double-booking lock)
);

create index if not exists idx_orders_status on orders (status);

-- ============================================================
-- 10. Payments against orders
-- ============================================================
do $$ begin
  create type payment_method as enum
    ('cash', 'upi', 'card', 'neft', 'cheque', 'finance_disbursal');
exception
  when duplicate_object then null;
end $$;

create table if not exists payments (
  id            uuid primary key default gen_random_uuid(),
  order_id      uuid not null references orders(id) on delete cascade,
  amount        numeric(12,2) not null check (amount > 0),
  method        payment_method not null,
  payment_date  date not null default current_date,
  reference_no  text,
  received_by   uuid references profiles(id),
  notes         text,
  created_at    timestamptz not null default now()
);

create index if not exists idx_payments_order_id on payments (order_id);

-- ============================================================
-- 11. Lock everything down (Row Level Security)
-- ============================================================
alter table profiles        enable row level security;
alter table vehicle_models  enable row level security;
alter table vehicles        enable row level security;
alter table vehicle_photos  enable row level security;
alter table customers       enable row level security;
alter table leads           enable row level security;
alter table activities      enable row level security;
alter table test_drives     enable row level security;
alter table quotations      enable row level security;
alter table orders          enable row level security;
alter table payments        enable row level security;
alter table heartbeat       enable row level security;
-- No public policies are created on purpose: only the backend's
-- service role connects with bypass capabilities, preventing public REST leakage.
