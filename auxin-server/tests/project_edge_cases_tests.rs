#[cfg(feature = "web-ui")]
mod project_edge_cases {
    use auxin_server::models::{CreateProjectRequest, Project, UpdateProjectRequest};
    use sqlx::SqlitePool;

    async fn setup_test_db() -> SqlitePool {
        let database_url = "sqlite::memory:";
        auxin_server::db::init_database(database_url)
            .await
            .expect("Failed to initialize test database")
    }

    #[tokio::test]
    async fn test_update_repository_path() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();
        assert!(project.repository_path.is_none());

        // Update repository path
        Project::update_repository_path(&pool, &project.id, "/path/to/repo")
            .await
            .unwrap();

        // Verify it was updated
        let updated = Project::get_by_id(&pool, &project.id).await.unwrap();
        assert_eq!(updated.repository_path, Some("/path/to/repo".to_string()));
    }

    #[tokio::test]
    async fn test_update_repository_path_nonexistent_project() {
        let pool = setup_test_db().await;

        let result =
            Project::update_repository_path(&pool, "nonexistent-id", "/path/to/repo").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_partial_update_name_only() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "original-name".to_string(),
            description: Some("Original description".to_string()),
        };

        let project = Project::create(&pool, request).await.unwrap();

        // Update only name
        let update = UpdateProjectRequest {
            name: Some("new-name".to_string()),
            description: None,
        };

        let updated = Project::update(&pool, &project.id, update).await.unwrap();
        assert_eq!(updated.name, "new-name");
        assert_eq!(
            updated.description,
            Some("Original description".to_string())
        );
    }

    #[tokio::test]
    async fn test_partial_update_description_only() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "original-name".to_string(),
            description: Some("Original description".to_string()),
        };

        let project = Project::create(&pool, request).await.unwrap();

        // Update only description
        let update = UpdateProjectRequest {
            name: None,
            description: Some("New description".to_string()),
        };

        let updated = Project::update(&pool, &project.id, update).await.unwrap();
        assert_eq!(updated.name, "original-name");
        assert_eq!(updated.description, Some("New description".to_string()));
    }

    #[tokio::test]
    async fn test_update_clear_description() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: Some("Has description".to_string()),
        };

        let project = Project::create(&pool, request).await.unwrap();

        // Clear description by setting to None
        let update = UpdateProjectRequest {
            name: None,
            description: None,
        };

        let updated = Project::update(&pool, &project.id, update).await.unwrap();
        assert_eq!(updated.description, Some("Has description".to_string())); // Description stays if not provided
    }

    #[tokio::test]
    async fn test_update_nonexistent_project() {
        let pool = setup_test_db().await;

        let update = UpdateProjectRequest {
            name: Some("new-name".to_string()),
            description: None,
        };

        let result = Project::update(&pool, "nonexistent-id", update).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_namespace_with_slash() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test/namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let result = Project::create(&pool, request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_name_with_slash() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test/project".to_string(),
            description: None,
        };

        let result = Project::create(&pool, request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_namespace_with_double_dots() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test..namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let result = Project::create(&pool, request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_create_project_null_description() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();
        assert!(project.description.is_none());
    }

    #[tokio::test]
    async fn test_list_empty_projects() {
        let pool = setup_test_db().await;

        let projects = Project::list(&pool).await.unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_get_nonexistent_by_namespace_and_name() {
        let pool = setup_test_db().await;

        let result = Project::get_by_namespace_and_name(&pool, "nonexistent", "project").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_created_at_and_updated_at() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();

        // Verify timestamps are set
        assert!(!project.created_at.is_empty());
        assert!(!project.updated_at.is_empty());
        assert_eq!(project.created_at, project.updated_at);

        // Wait a tiny bit and update
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let update = UpdateProjectRequest {
            name: Some("updated-name".to_string()),
            description: None,
        };

        let updated = Project::update(&pool, &project.id, update).await.unwrap();

        // created_at should be the same, updated_at should be different
        assert_eq!(updated.created_at, project.created_at);
        assert_ne!(updated.updated_at, project.updated_at);
    }

    #[tokio::test]
    async fn test_project_id_is_uuid() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();

        // Verify ID is a valid UUID format
        use uuid::Uuid;
        assert!(Uuid::parse_str(&project.id).is_ok());
    }

    #[tokio::test]
    async fn test_multiple_projects_same_namespace() {
        let pool = setup_test_db().await;

        // Create multiple projects in the same namespace
        let req1 = CreateProjectRequest {
            namespace: "shared-ns".to_string(),
            name: "project1".to_string(),
            description: None,
        };
        Project::create(&pool, req1).await.unwrap();

        let req2 = CreateProjectRequest {
            namespace: "shared-ns".to_string(),
            name: "project2".to_string(),
            description: None,
        };
        Project::create(&pool, req2).await.unwrap();

        let req3 = CreateProjectRequest {
            namespace: "shared-ns".to_string(),
            name: "project3".to_string(),
            description: None,
        };
        Project::create(&pool, req3).await.unwrap();

        // All should succeed
        let projects = Project::list(&pool).await.unwrap();
        assert_eq!(projects.len(), 3);
    }

    #[tokio::test]
    async fn test_multiple_projects_same_name_different_namespace() {
        let pool = setup_test_db().await;

        // Create multiple projects with the same name in different namespaces
        let req1 = CreateProjectRequest {
            namespace: "ns1".to_string(),
            name: "shared-name".to_string(),
            description: None,
        };
        Project::create(&pool, req1).await.unwrap();

        let req2 = CreateProjectRequest {
            namespace: "ns2".to_string(),
            name: "shared-name".to_string(),
            description: None,
        };
        Project::create(&pool, req2).await.unwrap();

        // Both should succeed
        let projects = Project::list(&pool).await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn test_special_characters_in_namespace() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns_123".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();
        assert_eq!(project.namespace, "test-ns_123");
    }

    #[tokio::test]
    async fn test_special_characters_in_name() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project_v1.2".to_string(),
            description: None,
        };

        let project = Project::create(&pool, request).await.unwrap();
        assert_eq!(project.name, "test-project_v1.2");
    }

    #[tokio::test]
    async fn test_long_description() {
        let pool = setup_test_db().await;

        let long_desc = "A".repeat(1000); // Very long description
        let request = CreateProjectRequest {
            namespace: "test-ns".to_string(),
            name: "test-project".to_string(),
            description: Some(long_desc.clone()),
        };

        let project = Project::create(&pool, request).await.unwrap();
        assert_eq!(project.description, Some(long_desc));
    }
}
