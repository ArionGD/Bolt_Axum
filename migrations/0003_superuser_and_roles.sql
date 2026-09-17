-- ============================================================
-- Migration: 0003_superuser_and_roles.sql
-- Description: Adds receive_alerts to customers, defines user roles
-- ============================================================

-- 1. Add receive_alerts to customers table if not exists
alter table customers 
add column if not exists receive_alerts boolean not null default false;

-- 2. Create user_role type if not exists
do $$ begin
  create type user_role as enum ('superuser', 'manager', 'customer');
exception
  when duplicate_object then null;
end $$;

-- 3. Create dealership user accounts table (for managers and customers)
create table if not exists app_accounts (
  id              uuid primary key default gen_random_uuid(),
  email           text unique,
  phone           text unique,
  full_name       text not null,
  role            user_role not null default 'customer',
  password_hash   text,
  receive_alerts  boolean not null default false,
  is_active       boolean not null default true,
  last_login_at   timestamptz,
  created_at      timestamptz not null default now(),
  updated_at      timestamptz not null default now()
);

-- Seed initial Manager 1 account
insert into app_accounts (id, email, phone, full_name, role, password_hash, is_active)
values (
  '11111111-1111-1111-1111-111111111111',
  'manager1@trishamotors.com',
  '+91 98000 11111',
  'Manager 1',
  'manager',
  'manager123',
  true
)
on conflict (id) do nothing;
