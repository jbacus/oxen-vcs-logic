use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use tracing::info;

use auxin_server::api;
use auxin_server::auth::AuthService;
use auxin_server::config::Config;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    info!("Starting Auxin Server (Oxen-aligned architecture)...");

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

    // Initialize auth service
    let auth_service = AuthService::new(config.clone());
    info!("Auth service initialized");

    // Start HTTP server
    info!("Starting Actix Web server...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/health", web::get().to(health_check))
            // Public endpoints
            .route("/api/repos", web::get().to(api::list_repositories))
            .route("/api/repos/{namespace}/{name}", web::get().to(api::get_repository))
            // Repository operations
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            .route("/api/repos/{namespace}/{name}/commits", web::get().to(api::get_commits))
            .route("/api/repos/{namespace}/{name}/push", web::post().to(api::push_repository))
            .route("/api/repos/{namespace}/{name}/pull", web::post().to(api::pull_repository))
            .route("/api/repos/{namespace}/{name}/branches", web::get().to(api::list_branches))
            .route("/api/repos/{namespace}/{name}/branches", web::post().to(api::create_branch))
            // Auxin extensions
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::get().to(api::get_metadata))
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::post().to(api::store_metadata))
            .route("/api/repos/{namespace}/{name}/locks/acquire", web::post().to(api::acquire_lock))
            .route("/api/repos/{namespace}/{name}/locks/release", web::post().to(api::release_lock))
            .route("/api/repos/{namespace}/{name}/locks/heartbeat", web::post().to(api::heartbeat_lock))
            .route("/api/repos/{namespace}/{name}/locks/status", web::get().to(api::lock_status))
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}
