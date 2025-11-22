use actix_web::{web, HttpResponse};
use sqlx::SqlitePool;
use tracing::info;

use crate::error::AppResult;
use crate::models::{CreateProjectRequest, Project, UpdateProjectRequest};

/// Create a new project
pub async fn create_project(
    pool: web::Data<SqlitePool>,
    req: web::Json<CreateProjectRequest>,
) -> AppResult<HttpResponse> {
    info!("Creating project: {}/{}", req.namespace, req.name);

    let project = Project::create(&pool, req.into_inner()).await?;

    Ok(HttpResponse::Created().json(project))
}

/// List all projects
pub async fn list_projects(pool: web::Data<SqlitePool>) -> AppResult<HttpResponse> {
    info!("Listing all projects");

    let projects = Project::list(&pool).await?;

    Ok(HttpResponse::Ok().json(projects))
}

/// Get a project by ID
pub async fn get_project(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    info!("Getting project: {}", id);

    let project = Project::get_by_id(&pool, &id).await?;

    Ok(HttpResponse::Ok().json(project))
}

/// Get a project by namespace and name
pub async fn get_project_by_namespace(
    pool: web::Data<SqlitePool>,
    path: web::Path<(String, String)>,
) -> AppResult<HttpResponse> {
    let (namespace, name) = path.into_inner();
    info!("Getting project: {}/{}", namespace, name);

    let project = Project::get_by_namespace_and_name(&pool, &namespace, &name).await?;

    Ok(HttpResponse::Ok().json(project))
}

/// Update a project
pub async fn update_project(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
    req: web::Json<UpdateProjectRequest>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    info!("Updating project: {}", id);

    let project = Project::update(&pool, &id, req.into_inner()).await?;

    Ok(HttpResponse::Ok().json(project))
}

/// Delete a project
pub async fn delete_project(
    pool: web::Data<SqlitePool>,
    path: web::Path<String>,
) -> AppResult<HttpResponse> {
    let id = path.into_inner();
    info!("Deleting project: {}", id);

    Project::delete(&pool, &id).await?;

    Ok(HttpResponse::NoContent().finish())
}
