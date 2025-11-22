use actix_web::{test, web, App};
use serde_json::json;
use std::fs;
use tempfile::TempDir;

use auxin_server::{
    api, auth::{self, AuthService}, config::Config, project::{ProjectMetadata, Visibility},
};

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

/// Helper to register a user and get a token
async fn register_user(
    auth_service: &AuthService,
    username: &str,
    email: &str,
    password: &str,
) -> (String, String) {
    let user = auth_service.register(username, email, password).unwrap();
    let token = auth_service
        .generate_token(&user.id, &user.username)
        .unwrap();
    (user.id, token)
}

#[actix_web::test]
async fn test_create_repository_requires_auth() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "description": "Test repository",
        "visibility": "public"
    });

    // Try without authentication
    let req = test::TestRequest::post()
        .uri("/api/repos/testuser/testrepo")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401, "Should require authentication");
}

#[actix_web::test]
async fn test_create_public_repository_with_auth() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (user_id, token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "description": "Test repository",
        "visibility": "public"
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/alice/testrepo")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["owner"], "alice");
    assert_eq!(body["visibility"], "public");

    // Verify project metadata was created
    let repo_path = temp_dir.path().join("alice/testrepo");
    let metadata = ProjectMetadata::load(&repo_path).unwrap();
    assert_eq!(metadata.owner_id, user_id);
    assert_eq!(metadata.owner_username, "alice");
    assert_eq!(metadata.visibility, Visibility::Public);
}

#[actix_web::test]
async fn test_create_private_repository() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (_, token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}",
                web::post().to(api::create_repository),
            ),
    )
    .await;

    let payload = json!({
        "visibility": "private"
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/alice/private-repo")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 201);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["visibility"], "private");
}

#[actix_web::test]
async fn test_list_repositories_filters_by_visibility() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, alice_token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (_, _bob_token) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's public repo
    let public_repo_path = temp_dir.path().join("alice/public-repo");
    fs::create_dir_all(public_repo_path.join(".oxen")).unwrap();
    let public_metadata = ProjectMetadata::new(alice_id.clone(), "alice".to_string(), Visibility::Public);
    public_metadata.save(&public_repo_path).unwrap();

    // Create alice's private repo
    let private_repo_path = temp_dir.path().join("alice/private-repo");
    fs::create_dir_all(private_repo_path.join(".oxen")).unwrap();
    let private_metadata = ProjectMetadata::new(alice_id.clone(), "alice".to_string(), Visibility::Private);
    private_metadata.save(&private_repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route("/api/repos", web::get().to(api::list_repositories)),
    )
    .await;

    // Anonymous user should only see public repos
    let req = test::TestRequest::get().uri("/api/repos").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1, "Anonymous should only see public repo");
    assert_eq!(body[0]["name"], "public-repo");

    // Alice should see both repos
    let req = test::TestRequest::get()
        .uri("/api/repos")
        .insert_header(("Authorization", format!("Bearer {}", alice_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 2, "Owner should see all their repos");
}

#[actix_web::test]
async fn test_private_repository_access_denied() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, _) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (_, bob_token) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's private repo
    let private_repo_path = temp_dir.path().join("alice/private-repo");
    fs::create_dir_all(private_repo_path.join(".oxen")).unwrap();
    let private_metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Private);
    private_metadata.save(&private_repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}",
                web::get().to(api::get_repository),
            ),
    )
    .await;

    // Bob should not be able to access alice's private repo
    let req = test::TestRequest::get()
        .uri("/api/repos/alice/private-repo")
        .insert_header(("Authorization", format!("Bearer {}", bob_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Should deny access to private repo");
}

#[actix_web::test]
async fn test_add_collaborator() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, alice_token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, _) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's repo
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Private);
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/collaborators",
                web::post().to(api::add_collaborator),
            ),
    )
    .await;

    let payload = json!({
        "user_id": bob_id
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/alice/testrepo/collaborators")
        .insert_header(("Authorization", format!("Bearer {}", alice_token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify collaborator was added
    let updated_metadata = ProjectMetadata::load(&repo_path).unwrap();
    assert_eq!(updated_metadata.collaborators.len(), 1);
    assert!(updated_metadata.collaborators.contains(&bob_id));
}

#[actix_web::test]
async fn test_add_collaborator_requires_owner() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, _) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, bob_token) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;
    let (charlie_id, _) = register_user(&auth_service, "charlie", "charlie@example.com", "password123").await;

    // Create alice's repo with bob as collaborator
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let mut metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Private);
    metadata.add_collaborator(bob_id).unwrap();
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/collaborators",
                web::post().to(api::add_collaborator),
            ),
    )
    .await;

    // Bob (collaborator) should NOT be able to add charlie
    let payload = json!({
        "user_id": charlie_id
    });

    let req = test::TestRequest::post()
        .uri("/api/repos/alice/testrepo/collaborators")
        .insert_header(("Authorization", format!("Bearer {}", bob_token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Only owner can add collaborators");
}

#[actix_web::test]
async fn test_collaborator_can_access_private_repo() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, _) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, bob_token) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's private repo with bob as collaborator
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let mut metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Private);
    metadata.add_collaborator(bob_id).unwrap();
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}",
                web::get().to(api::get_repository),
            ),
    )
    .await;

    // Bob should be able to access the repo
    let req = test::TestRequest::get()
        .uri("/api/repos/alice/testrepo")
        .insert_header(("Authorization", format!("Bearer {}", bob_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200, "Collaborator should have access");

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["owner"], "alice");
}

#[actix_web::test]
async fn test_remove_collaborator() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, alice_token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, _) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's repo with bob as collaborator
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let mut metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Private);
    metadata.add_collaborator(bob_id.clone()).unwrap();
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/collaborators/{user_id}",
                web::delete().to(api::remove_collaborator),
            ),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri(&format!("/api/repos/alice/testrepo/collaborators/{}", bob_id))
        .insert_header(("Authorization", format!("Bearer {}", alice_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify collaborator was removed
    let updated_metadata = ProjectMetadata::load(&repo_path).unwrap();
    assert_eq!(updated_metadata.collaborators.len(), 0);
}

#[actix_web::test]
async fn test_update_visibility() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, alice_token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;

    // Create alice's public repo
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Public);
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/visibility",
                web::put().to(api::update_visibility),
            ),
    )
    .await;

    let payload = json!({
        "visibility": "private"
    });

    let req = test::TestRequest::put()
        .uri("/api/repos/alice/testrepo/visibility")
        .insert_header(("Authorization", format!("Bearer {}", alice_token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // Verify visibility was updated
    let updated_metadata = ProjectMetadata::load(&repo_path).unwrap();
    assert_eq!(updated_metadata.visibility, Visibility::Private);
}

#[actix_web::test]
async fn test_update_visibility_requires_owner() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, _) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, bob_token) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;

    // Create alice's repo with bob as collaborator
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let mut metadata = ProjectMetadata::new(alice_id, "alice".to_string(), Visibility::Public);
    metadata.add_collaborator(bob_id).unwrap();
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/visibility",
                web::put().to(api::update_visibility),
            ),
    )
    .await;

    let payload = json!({
        "visibility": "private"
    });

    // Bob (collaborator) should NOT be able to change visibility
    let req = test::TestRequest::put()
        .uri("/api/repos/alice/testrepo/visibility")
        .insert_header(("Authorization", format!("Bearer {}", bob_token)))
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "Only owner can change visibility");
}

#[actix_web::test]
async fn test_list_collaborators() {
    let temp_dir = TempDir::new().unwrap();
    let config = test_config(&temp_dir);
    let auth_service = AuthService::new(config.clone());

    let (alice_id, alice_token) = register_user(&auth_service, "alice", "alice@example.com", "password123").await;
    let (bob_id, _) = register_user(&auth_service, "bob", "bob@example.com", "password123").await;
    let (charlie_id, _) = register_user(&auth_service, "charlie", "charlie@example.com", "password123").await;

    // Create alice's repo with bob and charlie as collaborators
    let repo_path = temp_dir.path().join("alice/testrepo");
    fs::create_dir_all(repo_path.join(".oxen")).unwrap();
    let mut metadata = ProjectMetadata::new(alice_id.clone(), "alice".to_string(), Visibility::Private);
    metadata.add_collaborator(bob_id.clone()).unwrap();
    metadata.add_collaborator(charlie_id.clone()).unwrap();
    metadata.save(&repo_path).unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(config))
            .app_data(web::Data::new(auth_service.clone()))
            .route(
                "/api/repos/{namespace}/{name}/collaborators",
                web::get().to(api::list_collaborators),
            ),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/repos/alice/testrepo/collaborators")
        .insert_header(("Authorization", format!("Bearer {}", alice_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["owner_id"], alice_id);
    assert_eq!(body["owner_username"], "alice");
    assert_eq!(body["collaborators"].as_array().unwrap().len(), 2);
}
