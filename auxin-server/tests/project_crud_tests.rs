#[cfg(feature = "web-ui")]
mod project_crud_tests {
    use auxin_server::models::{CreateProjectRequest, Project, UpdateProjectRequest};
    use sqlx::SqlitePool;
    use tempfile::tempdir;

    async fn setup_test_db() -> SqlitePool {
        // Use in-memory SQLite database for testing
        let database_url = "sqlite::memory:";

        let pool = auxin_server::db::init_database(database_url)
            .await
            .expect("Failed to initialize test database");

        pool
    }

    #[tokio::test]
    async fn test_create_project() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: Some("A test project".to_string()),
        };

        let project = Project::create(&pool, request).await.unwrap();

        assert_eq!(project.namespace, "test-namespace");
        assert_eq!(project.name, "test-project");
        assert_eq!(project.description, Some("A test project".to_string()));
        assert!(project.repository_path.is_none());
    }

    #[tokio::test]
    async fn test_list_projects() {
        let pool = setup_test_db().await;

        // Create multiple projects
        let request1 = CreateProjectRequest {
            namespace: "namespace1".to_string(),
            name: "project1".to_string(),
            description: Some("Project 1".to_string()),
        };
        Project::create(&pool, request1).await.unwrap();

        let request2 = CreateProjectRequest {
            namespace: "namespace2".to_string(),
            name: "project2".to_string(),
            description: Some("Project 2".to_string()),
        };
        Project::create(&pool, request2).await.unwrap();

        let projects = Project::list(&pool).await.unwrap();
        assert_eq!(projects.len(), 2);
    }

    #[tokio::test]
    async fn test_get_project_by_id() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let created_project = Project::create(&pool, request).await.unwrap();
        let fetched_project = Project::get_by_id(&pool, &created_project.id)
            .await
            .unwrap();

        assert_eq!(created_project.id, fetched_project.id);
        assert_eq!(created_project.namespace, fetched_project.namespace);
        assert_eq!(created_project.name, fetched_project.name);
    }

    #[tokio::test]
    async fn test_get_project_by_namespace_and_name() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let created_project = Project::create(&pool, request).await.unwrap();
        let fetched_project =
            Project::get_by_namespace_and_name(&pool, "test-namespace", "test-project")
                .await
                .unwrap();

        assert_eq!(created_project.id, fetched_project.id);
    }

    #[tokio::test]
    async fn test_update_project() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: Some("Original description".to_string()),
        };

        let created_project = Project::create(&pool, request).await.unwrap();

        let update_request = UpdateProjectRequest {
            name: Some("updated-project".to_string()),
            description: Some("Updated description".to_string()),
        };

        let updated_project = Project::update(&pool, &created_project.id, update_request)
            .await
            .unwrap();

        assert_eq!(updated_project.name, "updated-project");
        assert_eq!(
            updated_project.description,
            Some("Updated description".to_string())
        );
    }

    #[tokio::test]
    async fn test_delete_project() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let created_project = Project::create(&pool, request).await.unwrap();

        // Delete the project
        Project::delete(&pool, &created_project.id).await.unwrap();

        // Verify it's deleted
        let result = Project::get_by_id(&pool, &created_project.id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_duplicate_namespace_and_name() {
        let pool = setup_test_db().await;

        let request1 = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: None,
        };
        Project::create(&pool, request1).await.unwrap();

        // Try to create another project with same namespace and name
        let request2 = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "test-project".to_string(),
            description: Some("Different description".to_string()),
        };

        let result = Project::create(&pool, request2).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_namespace() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "../invalid".to_string(),
            name: "test-project".to_string(),
            description: None,
        };

        let result = Project::create(&pool, request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_invalid_name() {
        let pool = setup_test_db().await;

        let request = CreateProjectRequest {
            namespace: "test-namespace".to_string(),
            name: "../invalid".to_string(),
            description: None,
        };

        let result = Project::create(&pool, request).await;
        assert!(result.is_err());
    }
}
