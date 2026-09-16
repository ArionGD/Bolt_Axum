use std::{net::SocketAddr, sync::Arc};
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod db;
mod domain;
mod error;
mod routes;
mod services;

use auth::AppState;
use config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ev_dealership_backend=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 2. Load configuration
    let config = Config::from_env();
    tracing::info!("Starting EV Dealership Backend on port {}", config.port);

    // 3. Connect to Database
    let pool = match db::create_pool(&config.database_url).await {
        Ok(pool) => {
            tracing::info!("Connected to PostgreSQL database successfully");
            if let Err(e) = db::run_migrations(&pool).await {
                tracing::warn!("Failed running migrations on startup: {:?}", e);
            }
            pool
        }
        Err(err) => {
            tracing::error!("Could not connect to database at startup: {:?}. Running with fallback state.", err);
            // Re-attempt with simple connect or fail gracefully
            db::create_pool(&config.database_url).await?
        }
    };

    let app_state = Arc::new(AppState {
        pool,
        config: config.clone(),
    });

    // 4. Configure permissive CORS for web/desktop/mobile
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 5. Build Router
    let app = routes::create_router(app_state)
        .layer(cors)
        .layer(TraceLayer::new_for_http());

    // 6. Bind listener
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);

    // 7. Serve with graceful shutdown
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    tracing::info!("Shutting down EV Dealership Backend gracefully...");
}
