use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

mod api;
mod auth;
mod config;
mod db;
mod error;
mod models;

use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting OxVCS Server...");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded");

    // Connect to database
    let db_pool = db::create_pool(&config.database_url).await?;
    info!("Database connection established");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    info!("Database migrations complete");

    // Build application state
    let state = api::AppState {
        db: db_pool,
        config: config.clone(),
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/auth/register", post(api::auth::register))
        .route("/api/v1/auth/login", post(api::auth::login))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    info!("Server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
