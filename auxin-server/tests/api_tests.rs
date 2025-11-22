use actix_web::{test, web, App};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

// Import server modules
use auxin_server::{api, auth::{self, AuthService}, config::Config, websocket::WsHub};

fn test_config(temp_dir: &TempDir) -> Config {
    Config {
        sync_dir: temp_dir.path().to_string_lossy().to_string(),
        host: "127.0.0.1".to_string(),
        port: 3000,
        auth_token_secret: "test_secret".to_string(),
        auth_token_expiry_hours: 24,
        enable_redis_locks: false,
        enable_web_ui: false,
        redis_url: None,
        database_url: None,
    }
}

#[actix_web::test]
async fn test_health_check() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/health", web::get().to(|| async { "OK" })),
    )
    .await;

    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_create_repository() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register a user and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "description": "Test repository"
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    // Verify .oxen directory was created
    let repo_path = temp_dir.path().join("testuser/testrepo/.oxen");
    assert!(repo_path.exists());
    assert!(repo_path.join("metadata").exists());
    assert!(repo_path.join("locks").exists());
}

#[actix_web::test]
async fn test_list_repositories() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Create test repository manually
    let repo_path = temp_dir.path().join("testuser/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/repos", web::get().to(api::list_repositories)),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/repos").to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["namespace"], "testuser");
    assert_eq!(body[0]["name"], "testrepo");
}

#[actix_web::test]
async fn test_get_repository() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Create test repository manually
    let repo_path = temp_dir.path().join("testuser/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}",
                web::get().to(api::get_repository),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/repos/testuser/testrepo")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["namespace"], "testuser");
    assert_eq!(body["name"], "testrepo");
}

#[actix_web::test]
async fn test_get_nonexistent_repository() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}",
                web::get().to(api::get_repository),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/repos/nonexistent/repo")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_create_duplicate_repository() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register a user and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    // Create repository manually
    let repo_path = temp_dir.path().join("testuser/testrepo");
    fs::create_dir_all(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "description": "Test repository"
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_invalid_repository_name() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register a user and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "description": "Test repository"
    });

    // Test with path traversal in namespace
    let req = test::TestRequest::post()
        .uri("/api/repos/..%2F..%2Fetc/passwd")  // URL-encoded ../../etc
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "Should reject path traversal in namespace");

    // Test with path traversal in repo name
    let req2 = test::TestRequest::post()
        .uri("/api/repos/testuser/..%2F..%2Fetc")  // URL-encoded ../../etc
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), 400, "Should reject path traversal in repo name");
}

// Auth endpoint tests

#[actix_web::test]
async fn test_auth_register() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/auth/register", web::post().to(auth::register)),
    )
    .await;

    let payload = json!({
        "username": "testuser",
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/register")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].as_str().unwrap().starts_with("auxin_"));
    assert_eq!(body["user"]["username"], "testuser");
    assert_eq!(body["user"]["email"], "test@example.com");
}

#[actix_web::test]
async fn test_auth_login() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register user first
    auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/auth/login", web::post().to(auth::login)),
    )
    .await;

    let payload = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].as_str().unwrap().starts_with("auxin_"));
    assert_eq!(body["user"]["username"], "testuser");
}

#[actix_web::test]
async fn test_auth_login_wrong_password() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register user first
    auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/auth/login", web::post().to(auth::login)),
    )
    .await;

    let payload = json!({
        "email": "test@example.com",
        "password": "wrongpassword"
    });

    let req = test::TestRequest::post()
        .uri("/api/auth/login")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_auth_me() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/auth/me", web::get().to(auth::me)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["username"], "testuser");
    assert_eq!(body["email"], "test@example.com");
}

#[actix_web::test]
async fn test_auth_me_no_token() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route("/api/auth/me", web::get().to(auth::me)),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/auth/me")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

// Activity endpoint tests

#[actix_web::test]
async fn test_get_activity_empty() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    // Register a user and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    // Create test repository with project metadata
    let repo_path = temp_dir.path().join("testuser/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();

    use auxin_server::project::{ProjectMetadata, Visibility};
    let metadata = ProjectMetadata::new(user.id.clone(), "testuser".to_string(), Visibility::Public);
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .route(
                "/api/repos/{namespace}/{name}/activity",
                web::get().to(api::get_activity),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/repos/testuser/testrepo/activity")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert!(body.is_empty());
}

#[actix_web::test]
async fn test_lock_creates_activity() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());
    let ws_hub = WsHub::new();

    // Register a user and get token
    let user = auth_service
        .register("testuser", "test@example.com", "password123")
        .unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();

    // Create test repository with project metadata
    let repo_path = temp_dir.path().join("testuser/testrepo");
    fs::create_dir_all(repo_path.join(".oxen/locks")).unwrap();
    fs::create_dir_all(repo_path.join(".oxen/metadata")).unwrap();

    use auxin_server::project::{ProjectMetadata, Visibility};
    let metadata = ProjectMetadata::new(user.id.clone(), "testuser".to_string(), Visibility::Public);
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service))
            .app_data(web::Data::new(ws_hub))
            .route(
                "/api/repos/{namespace}/{name}/locks/acquire",
                web::post().to(api::acquire_lock),
            )
            .route(
                "/api/repos/{namespace}/{name}/activity",
                web::get().to(api::get_activity),
            ),
    )
    .await;

    // Acquire lock
    let lock_payload = json!({
        "user": "testuser",
        "machine_id": "test-machine",
        "timeout_hours": 24
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo/locks/acquire")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&lock_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Check activity was created
    let req = test::TestRequest::get()
        .uri("/api/repos/testuser/testrepo/activity")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["activity_type"], "lock_acquired");
    assert_eq!(body[0]["user"], "testuser");
}
