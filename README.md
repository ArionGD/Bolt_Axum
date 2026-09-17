# Trisha Motors — Axum Backend & Database

High-performance Rust Axum backend powering the Trisha Motors EV Dealership management system, integrated with PostgreSQL (Supabase) and serving the standalone Superuser Admin Panel.

## Features
- **Axum High-Performance API**: RESTful endpoints for Inventory, Leads, Test Drives, Quotations, and Orders with physical VIN locking.
- **Integrated Superuser Panel (`/admin`)**: Standalone developer admin interface served directly by the Axum binary via HTML/CSS.
- **Automated PostgreSQL Migrations**: Executes schema and seed migrations on startup (`migrations/`).
- **Strict Role-Based Access Control**:
  - `superuser`: Authenticated against server `.env` credentials.
  - `manager`: Operational showroom staff.
  - `customer`: Public buyers with phone-based access.

## Tech Stack
- **Language**: Rust (edition 2021)
- **Framework**: Axum 0.7, Tokio, Tower-HTTP (CORS, Trace)
- **Database**: PostgreSQL (SQLx runtime queries with connection pooling)
- **Authentication**: JWT & Argon2 password hashing

## Database Setup & Migrations
Migrations are stored in `migrations/`:
- `0001_init.sql` — Schema definition for vehicles, customers, leads, test drives, quotes, orders, payments.
- `0002_seed_models.sql` — Seed data for Scooty models (Models 1, 2, Pro) and E-Rickshaw Model 1.
- `0003_superuser_and_roles.sql` — Superuser system and role management.

## Environment Variables (.env)
```env
PORT=8080
DATABASE_URL=postgresql://postgres:[password]@db.[ref].supabase.co:5432/postgres?sslmode=require
SUPABASE_JWT_SECRET=your-32-char-jwt-secret-key-here
SUPERUSER_EMAIL=superuser@voltdealership.com
SUPERUSER_PASSWORD=YourSuperSecretPassword2026!
SUPERUSER_NAME=Developer Superuser
APP_ENV=production
```

## Running Locally
```bash
cargo run
```

## Deploying to Render.com (Web Service)
1. Create a new **Web Service** on Render connected to `https://github.com/ArionGD/Bolt_Axum.git`.
2. Configure settings:
   - **Environment**: Rust
   - **Build Command**: `cargo build --release`
   - **Start Command**: `./target/release/ev-dealership-backend`
3. Add environment variables under the **Environment** tab:
   - `DATABASE_URL`: Your Supabase connection string.
   - `SUPERUSER_EMAIL`: Developer email.
   - `SUPERUSER_PASSWORD`: Master password.
   - `SUPABASE_JWT_SECRET`: Secret token for JWT verification.
