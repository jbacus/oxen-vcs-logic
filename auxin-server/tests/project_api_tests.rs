#[cfg(feature = "web-ui")]
mod project_api_tests {
    use actix_web::{test, web, App};
    use auxin_server::api;
    use auxin_server::models::CreateProjectRequest;
    use serde_json::json;

    async fn create_test_pool() -> sqlx::SqlitePool {
        let database_url = "sqlite::memory:";
        auxin_server::db::init_database(database_url)
            .await
            .expect("Failed to initialize test database")
    }

    #[actix_web::test]
    async fn test_api_create_project() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::post().to(api::create_project)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/projects")
            .set_json(json!({
                "namespace": "test-ns",
                "name": "test-project",
                "description": "A test project"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 201);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["namespace"], "test-ns");
        assert_eq!(body["name"], "test-project");
        assert_eq!(body["description"], "A test project");
    }

    #[actix_web::test]
    async fn test_api_list_projects() {
        let pool = create_test_pool().await;

        // Create some projects directly via model
        let req1 = CreateProjectRequest {
            namespace: "ns1".to_string(),
            name: "proj1".to_string(),
            description: Some("Project 1".to_string()),
        };
        auxin_server::models::Project::create(&pool, req1)
            .await
            .unwrap();

        let req2 = CreateProjectRequest {
            namespace: "ns2".to_string(),
            name: "proj2".to_string(),
            description: Some("Project 2".to_string()),
        };
        auxin_server::models::Project::create(&pool, req2)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::get().to(api::list_projects)),
        )
        .await;

        let req = test::TestRequest::get().uri("/api/projects").to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Vec<serde_json::Value> = test::read_body_json(resp).await;
        assert_eq!(body.len(), 2);
    }

    #[actix_web::test]
    async fn test_api_get_project_by_id() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };
        let project = auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::get().to(api::get_project)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri(&format!("/api/projects/{}", project.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["id"], project.id);
        assert_eq!(body["namespace"], "test-ns");
    }

    #[actix_web::test]
    async fn test_api_get_project_by_namespace_and_name() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "my-namespace".to_string(),
            name: "my-project".to_string(),
            description: Some("My Project".to_string()),
        };
        auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(App::new().app_data(web::Data::new(pool.clone())).route(
            "/api/projects/{namespace}/{name}",
            web::get().to(api::get_project_by_namespace),
        ))
        .await;

        let req = test::TestRequest::get()
            .uri("/api/projects/my-namespace/my-project")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["namespace"], "my-namespace");
        assert_eq!(body["name"], "my-project");
    }

    #[actix_web::test]
    async fn test_api_update_project() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: Some("Original".to_string()),
        };
        let project = auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::put().to(api::update_project)),
        )
        .await;

        let req = test::TestRequest::put()
            .uri(&format!("/api/projects/{}", project.id))
            .set_json(json!({
                "name": "updated-project",
                "description": "Updated description"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "updated-project");
        assert_eq!(body["description"], "Updated description");
    }

    #[actix_web::test]
    async fn test_api_partial_update_project() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "original-name".to_string(),
            description: Some("Original description".to_string()),
        };
        let project = auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::put().to(api::update_project)),
        )
        .await;

        // Update only description
        let req = test::TestRequest::put()
            .uri(&format!("/api/projects/{}", project.id))
            .set_json(json!({
                "description": "New description only"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["name"], "original-name"); // Name unchanged
        assert_eq!(body["description"], "New description only");
    }

    #[actix_web::test]
    async fn test_api_delete_project() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };
        let project = auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::delete().to(api::delete_project)),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri(&format!("/api/projects/{}", project.id))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 204);
    }

    #[actix_web::test]
    async fn test_api_get_nonexistent_project() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::get().to(api::get_project)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/api/projects/nonexistent-id")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_api_delete_nonexistent_project() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::delete().to(api::delete_project)),
        )
        .await;

        let req = test::TestRequest::delete()
            .uri("/api/projects/nonexistent-id")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 404);
    }

    #[actix_web::test]
    async fn test_api_create_project_invalid_namespace() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::post().to(api::create_project)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/projects")
            .set_json(json!({
                "namespace": "../invalid",
                "name": "test-project",
                "description": null
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_api_create_duplicate_project() {
        let pool = create_test_pool().await;

        // Create first project
        let req1 = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };
        auxin_server::models::Project::create(&pool, req1)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::post().to(api::create_project)),
        )
        .await;

        // Try to create duplicate
        let req = test::TestRequest::post()
            .uri("/api/projects")
            .set_json(json!({
                "namespace": "test-ns",
                "name": "test-project",
                "description": "Different description"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_api_update_project_invalid_name() {
        let pool = create_test_pool().await;

        // Create a project
        let req = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };
        let project = auxin_server::models::Project::create(&pool, req)
            .await
            .unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects/{id}", web::put().to(api::update_project)),
        )
        .await;

        // Try to update with invalid name
        let req = test::TestRequest::put()
            .uri(&format!("/api/projects/{}", project.id))
            .set_json(json!({
                "name": "../invalid"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_api_create_project_empty_namespace() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::post().to(api::create_project)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/projects")
            .set_json(json!({
                "namespace": "",
                "name": "test-project",
                "description": null
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_api_create_project_empty_name() {
        let pool = create_test_pool().await;
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(pool.clone()))
                .route("/api/projects", web::post().to(api::create_project)),
        )
        .await;

        let req = test::TestRequest::post()
            .uri("/api/projects")
            .set_json(json!({
                "namespace": "test-ns",
                "name": "",
                "description": null
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }
}
