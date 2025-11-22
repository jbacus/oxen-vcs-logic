use actix_web::{middleware, web, App, HttpResponse, HttpServer, Result};
use actix_files::{Files, NamedFile};
use std::path::PathBuf;
use tracing::info;

use auxin_server::api;
use auxin_server::auth::{self, AuthService};
use auxin_server::config::Config;
use auxin_server::websocket::{ws_handler, WsHub};

#[cfg(feature = "web-ui")]
use auxin_server::db;

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

    // Initialize WebSocket hub
    let ws_hub = WsHub::new();
    info!("WebSocket hub initialized");

    // Initialize database if web-ui feature is enabled
    #[cfg(feature = "web-ui")]
    let db_pool = if let Some(database_url) = &config.database_url {
        match db::init_database(database_url).await {
            Ok(pool) => {
                info!("Database initialized successfully");
                Some(pool)
            }
            Err(e) => {
                info!("Failed to initialize database: {}. Project CRUD endpoints will not be available.", e);
                None
            }
        }
    } else {
        info!("No DATABASE_URL configured. Project CRUD endpoints will not be available.");
        None
    };

    #[cfg(not(feature = "web-ui"))]
    let db_pool: Option<()> = None;

    // Detect frontend static files directory
    let frontend_dir = PathBuf::from("frontend/dist");
    let serve_frontend = frontend_dir.exists();

    if serve_frontend {
        info!("Frontend static files found at: frontend/dist");
        info!("Web UI will be available at http://{}:{}/", host, port);
    } else {
        info!("Frontend not built. Run 'cd frontend && npm install && npm run build' to enable web UI");
    }

    // Start HTTP server
    info!("Starting Actix Web server...");
    HttpServer::new(move || {
        let mut app = App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(ws_hub.clone()));

        // Add database pool if available
        #[cfg(feature = "web-ui")]
        if let Some(ref pool) = db_pool {
            app = app.app_data(web::Data::new(pool.clone()));
        }

        let mut app = app
            .wrap(middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allow_any_method()
                    .allow_any_header(),
            )
            .route("/health", web::get().to(health_check))
            // Auth endpoints
            .route("/api/auth/register", web::post().to(auth::register))
            .route("/api/auth/login", web::post().to(auth::login))
            .route("/api/auth/logout", web::post().to(auth::logout))
            .route("/api/auth/me", web::get().to(auth::me));

        // Project CRUD endpoints (requires web-ui feature and database)
        #[cfg(feature = "web-ui")]
        if db_pool.is_some() {
            app = app
                .route("/api/projects", web::post().to(api::create_project))
                .route("/api/projects", web::get().to(api::list_projects))
                .route("/api/projects/{id}", web::get().to(api::get_project))
                .route("/api/projects/{namespace}/{name}", web::get().to(api::get_project_by_namespace))
                .route("/api/projects/{id}", web::put().to(api::update_project))
                .route("/api/projects/{id}", web::delete().to(api::delete_project));
        }

        let mut app = app
            // Public endpoints
            .route("/api/repos", web::get().to(api::list_repositories))
            .route("/api/repos/{namespace}/{name}", web::get().to(api::get_repository))
            // Repository operations
            .route("/api/repos/{namespace}/{name}", web::post().to(api::create_repository))
            .route("/api/repos/{namespace}/{name}/clone", web::post().to(api::clone_repository))
            .route("/api/repos/{namespace}/{name}/status", web::get().to(api::get_status))
            .route("/api/repos/{namespace}/{name}/commits", web::get().to(api::get_commits))
            .route("/api/repos/{namespace}/{name}/commits/{commit}/restore", web::post().to(api::restore_commit))
            .route("/api/repos/{namespace}/{name}/push", web::post().to(api::push_repository))
            .route("/api/repos/{namespace}/{name}/pull", web::post().to(api::pull_repository))
            .route("/api/repos/{namespace}/{name}/fetch", web::post().to(api::fetch_repository))
            .route("/api/repos/{namespace}/{name}/branches", web::get().to(api::list_branches))
            .route("/api/repos/{namespace}/{name}/branches", web::post().to(api::create_branch))
            .route("/api/repos/{namespace}/{name}/branches/{branch}", web::delete().to(api::delete_branch))
            // Auxin extensions
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::get().to(api::get_metadata))
            .route("/api/repos/{namespace}/{name}/metadata/{commit}", web::post().to(api::store_metadata))
            .route("/api/repos/{namespace}/{name}/locks/acquire", web::post().to(api::acquire_lock))
            .route("/api/repos/{namespace}/{name}/locks/release", web::post().to(api::release_lock))
            .route("/api/repos/{namespace}/{name}/locks/heartbeat", web::post().to(api::heartbeat_lock))
            .route("/api/repos/{namespace}/{name}/locks/status", web::get().to(api::lock_status))
            .route("/api/repos/{namespace}/{name}/activity", web::get().to(api::get_activity))
            // WebSocket for real-time notifications
            .route("/ws/repos/{namespace}/{name}", web::get().to(ws_handler))
            // Bounce audio endpoints
            .route("/api/repos/{namespace}/{name}/bounces", web::get().to(api::list_bounces))
            .route("/api/repos/{namespace}/{name}/bounces/{commit}", web::get().to(api::get_bounce))
            .route("/api/repos/{namespace}/{name}/bounces/{commit}/audio", web::get().to(api::get_bounce_audio))
            .route("/api/repos/{namespace}/{name}/bounces/{commit}", web::post().to(api::upload_bounce))
            .route("/api/repos/{namespace}/{name}/bounces/{commit}", web::delete().to(api::delete_bounce));

        // Serve frontend static files if available
        if serve_frontend {
            app = app
                .service(Files::new("/assets", "frontend/dist/assets"))
                .default_service(web::get().to(serve_spa));
        }

        app
    })
    .bind((host.as_str(), port))?
    .run()
    .await
}

async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().body("OK"))
}

// Serve SPA for all non-API routes
async fn serve_spa() -> Result<NamedFile> {
    let path = PathBuf::from("frontend/dist/index.html");
    Ok(NamedFile::open(path)?)
}
