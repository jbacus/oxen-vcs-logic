use actix_web::{test, web, App};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

// Import server modules
use auxin_server::{api, auth::AuthService, config::Config};

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

    // Test with path traversal
    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/../../etc/passwd")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}
