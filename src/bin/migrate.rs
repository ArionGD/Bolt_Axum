use dotenvy::dotenv;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let safe_host = database_url.split('@').last().unwrap_or("");
    println!("Attempting connection to PostgreSQL host: {}", safe_host);

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(15))
        .connect(&database_url)
        .await?;

    println!("✓ Connected to PostgreSQL database successfully!");

    println!("Applying 0001_init.sql...");
    let init_sql = include_str!("../../migrations/0001_init.sql");
    sqlx::raw_sql(init_sql).execute(&pool).await?;
    println!("✓ 0001_init.sql applied!");

    println!("Applying 0002_seed_models.sql...");
    let seed_sql = include_str!("../../migrations/0002_seed_models.sql");
    sqlx::raw_sql(seed_sql).execute(&pool).await?;
    println!("✓ 0002_seed_models.sql applied!");

    println!("Applying 0003_superuser_and_roles.sql...");
    let roles_sql = include_str!("../../migrations/0003_superuser_and_roles.sql");
    sqlx::raw_sql(roles_sql).execute(&pool).await?;
    println!("✓ 0003_superuser_and_roles.sql applied!");

    // Clean up any old duplicate brand rows
    sqlx::query("DELETE FROM vehicle_models WHERE brand IN ('EV Scooty', 'EV Rickshaw')")
        .execute(&pool)
        .await?;
    sqlx::query("UPDATE app_accounts SET email = 'manager1@trishamotors.com' WHERE email = 'manager1@voltdealership.com'")
        .execute(&pool)
        .await?;
    println!("✓ Live database updated to Trisha Motors branding!");

    let models_count: (i64,) = sqlx::query_as("SELECT count(*) FROM vehicle_models")
        .fetch_one(&pool)
        .await?;
    println!("✓ vehicle_models seeded count: {}", models_count.0);

    let acct_count: (i64,) = sqlx::query_as("SELECT count(*) FROM app_accounts")
        .fetch_one(&pool)
        .await?;
    println!("✓ app_accounts seeded count: {}", acct_count.0);

    println!("🎉 All migrations successfully verified against Supabase!");
    Ok(())
}
