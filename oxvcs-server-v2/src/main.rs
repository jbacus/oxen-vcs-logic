use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use std::env;
use tracing::info;

mod api;
mod config;
mod error;
mod extensions;

use config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting OxVCS Server (Oxen-aligned architecture)...");

    // Load configuration
    let config = Config::from_env().expect("Failed to load configuration");
    info!("Configuration loaded");
    info!("SYNC_DIR: {}", config.sync_dir);
    info!("Server will listen on {}:{}", config.host, config.port);

    // Ensure SYNC_DIR exists
    std::fs::create_dir_all(&config.sync_dir)
        .expect("Failed to create SYNC_DIR");

    let host = config.host.clone();
    let port = config.port;

    // Start HTTP server
    info!("Starting Actix Web server...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/health", web::get().to(health_check))
            .route("/api/repos", web::get().to(api::list_repositories))
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            .route("/api/repos/{namespace}/{name}", web::get().to(api::get_repository))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}
