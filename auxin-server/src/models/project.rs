use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use uuid::Uuid;
use chrono::Utc;

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Project {
    pub id: String,
    pub namespace: String,
    pub name: String,
    pub description: Option<String>,
    pub repository_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateProjectRequest {
    pub namespace: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl Project {
    /// Create a new project
    pub async fn create(pool: &SqlitePool, req: CreateProjectRequest) -> AppResult<Project> {
        // Validate namespace
        if req.namespace.is_empty() || req.namespace.contains("..") || req.namespace.contains('/') {
            return Err(AppError::BadRequest("Invalid namespace".to_string()));
        }

        // Validate name
        if req.name.is_empty() || req.name.contains("..") || req.name.contains('/') {
            return Err(AppError::BadRequest("Invalid project name".to_string()));
        }

        let id = Uuid::new_v4().to_string();
        let now = Utc::now().to_rfc3339();

        let project = sqlx::query_as::<_, Project>(
            r#"
            INSERT INTO projects (id, namespace, name, description, repository_path, created_at, updated_at)
            VALUES (?, ?, ?, ?, NULL, ?, ?)
            RETURNING *
            "#
        )
        .bind(&id)
        .bind(&req.namespace)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&now)
        .bind(&now)
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("UNIQUE constraint failed") {
                AppError::BadRequest("Project with this namespace and name already exists".to_string())
            } else {
                AppError::Database(e.to_string())
            }
        })?;

        Ok(project)
    }

    /// List all projects
    pub async fn list(pool: &SqlitePool) -> AppResult<Vec<Project>> {
        let projects = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects ORDER BY created_at DESC"
        )
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(projects)
    }

    /// Get a project by ID
    pub async fn get_by_id(pool: &SqlitePool, id: &str) -> AppResult<Project> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        Ok(project)
    }

    /// Get a project by namespace and name
    pub async fn get_by_namespace_and_name(
        pool: &SqlitePool,
        namespace: &str,
        name: &str,
    ) -> AppResult<Project> {
        let project = sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE namespace = ? AND name = ?"
        )
        .bind(namespace)
        .bind(name)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?
        .ok_or_else(|| AppError::NotFound("Project not found".to_string()))?;

        Ok(project)
    }

    /// Update a project
    pub async fn update(
        pool: &SqlitePool,
        id: &str,
        req: UpdateProjectRequest,
    ) -> AppResult<Project> {
        // First check if project exists
        let existing = Self::get_by_id(pool, id).await?;

        let name = req.name.unwrap_or(existing.name);
        let description = req.description.or(existing.description);
        let now = Utc::now().to_rfc3339();

        // Validate name if provided
        if name.is_empty() || name.contains("..") || name.contains('/') {
            return Err(AppError::BadRequest("Invalid project name".to_string()));
        }

        let project = sqlx::query_as::<_, Project>(
            r#"
            UPDATE projects
            SET name = ?, description = ?, updated_at = ?
            WHERE id = ?
            RETURNING *
            "#
        )
        .bind(&name)
        .bind(&description)
        .bind(&now)
        .bind(id)
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(project)
    }

    /// Delete a project
    pub async fn delete(pool: &SqlitePool, id: &str) -> AppResult<()> {
        let result = sqlx::query("DELETE FROM projects WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Project not found".to_string()));
        }

        Ok(())
    }

    /// Update repository path
    pub async fn update_repository_path(
        pool: &SqlitePool,
        id: &str,
        repository_path: &str,
    ) -> AppResult<()> {
        let now = Utc::now().to_rfc3339();

        let result = sqlx::query(
            "UPDATE projects SET repository_path = ?, updated_at = ? WHERE id = ?"
        )
        .bind(repository_path)
        .bind(&now)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("Project not found".to_string()));
        }

        Ok(())
    }
}
