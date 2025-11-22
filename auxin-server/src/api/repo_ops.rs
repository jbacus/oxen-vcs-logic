use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::auth::{get_optional_user_id_from_request, get_user_id_from_request, AuthService};
use auxin_config::Config;
use crate::error::{AppError, AppResult};
use crate::extensions::{get_activities, log_activity, ActivityType, LogicProMetadata};
use crate::project::ProjectAuth;
use crate::repo::RepositoryOps;
use crate::websocket::WsHub;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct PushRequest {
    pub remote: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PullRequest {
    pub remote: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateBranchRequest {
    pub branch_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockRequest {
    pub user: String,
    pub machine_id: String,
    pub timeout_hours: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReleaseLockRequest {
    pub lock_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeartbeatRequest {
    pub lock_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CloneRequest {
    pub remote_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteBranchRequest {
    pub branch_name: String,
}

/// Get commit history for a repository
pub async fn get_commits(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    query: web::Query<CommitQuery>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting commits for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let repo = RepositoryOps::open(&repo_path)?;
    let commits = repo.log(query.limit)?;

    Ok(HttpResponse::Ok().json(commits))
}

#[derive(Debug, Deserialize)]
pub struct CommitQuery {
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ActivityQuery {
    pub limit: Option<usize>,
}

/// Push to remote repository
pub async fn push_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<PushRequest>,
    ws_hub: web::Data<WsHub>,
    auth_service: web::Data<AuthService>,
    http_req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Pushing repository: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&http_req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    let branch = body.branch.clone().unwrap_or_else(|| "main".to_string());

    repo.push(&body.remote, &branch)?;

    // Log activity
    log_activity(
        &repo_path,
        ActivityType::Push,
        &user_id,
        &format!("Pushed to {} (branch: {})", body.remote, branch),
        Some(serde_json::json!({
            "remote": body.remote,
            "branch": branch
        })),
    )?;

    // Broadcast to WebSocket subscribers
    let _ = ws_hub
        .broadcast_commit(
            &namespace,
            &repo_name,
            "push",
            &format!("Pushed to {} (branch: {})", body.remote, branch),
            &user_id,
        )
        .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("Pushed to {} (branch: {})", body.remote, branch)
    })))
}

/// Pull from remote repository
pub async fn pull_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<PullRequest>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Pulling repository: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    let branch = body.branch.clone().unwrap_or_else(|| "main".to_string());

    repo.pull(&body.remote, &branch)?;

    // Log activity
    log_activity(
        &repo_path,
        ActivityType::Pull,
        &user_id,
        &format!("Pulled from {} (branch: {})", body.remote, branch),
        Some(serde_json::json!({
            "remote": body.remote,
            "branch": branch
        })),
    )?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("Pulled from {} (branch: {})", body.remote, branch)
    })))
}

/// List branches
pub async fn list_branches(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Listing branches for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let repo = RepositoryOps::open(&repo_path)?;
    let branches = repo.list_branches()?;

    Ok(HttpResponse::Ok().json(branches))
}

/// Create a new branch
pub async fn create_branch(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<CreateBranchRequest>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!(
        "Creating branch '{}' for: {}/{}",
        body.branch_name, namespace, repo_name
    );

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    repo.create_branch(&body.branch_name)?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "branch": body.branch_name
    })))
}

/// Restore/rollback to a specific commit
pub async fn restore_commit(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    ws_hub: web::Data<WsHub>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Restoring to commit {} in: {}/{}",
        commit_id, namespace, repo_name
    );

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;

    // Checkout the specific commit
    repo.checkout(&commit_id)?;

    // Log activity
    log_activity(
        &repo_path,
        ActivityType::Restore,
        &user_id,
        &format!("Restored to commit {}", &commit_id[..8]),
        Some(serde_json::json!({
            "commit_id": commit_id
        })),
    )?;

    // Broadcast via WebSocket
    use crate::websocket::WsMessage;
    ws_hub
        .broadcast(
            &format!("{}/{}", namespace, repo_name),
            WsMessage::Activity {
                activity_type: "restore".to_string(),
                user: user_id.clone(),
                message: format!("Restored to commit {}", &commit_id[..8]),
                timestamp: chrono::Utc::now().to_rfc3339(),
            },
        )
        .await?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "commit_id": commit_id,
        "message": "Repository restored to commit"
    })))
}

/// Get Logic Pro metadata for a commit
pub async fn get_metadata(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Getting metadata for commit {} in: {}/{}",
        commit_id, namespace, repo_name
    );

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let repo = RepositoryOps::open(&repo_path)?;
    let metadata = repo.get_metadata(&commit_id)?;

    match metadata {
        Some(md) => Ok(HttpResponse::Ok().json(md)),
        None => Ok(HttpResponse::NotFound().json(serde_json::json!({
            "error": "No metadata found for this commit"
        }))),
    }
}

/// Store Logic Pro metadata for a commit
pub async fn store_metadata(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    metadata: web::Json<LogicProMetadata>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Storing metadata for commit {} in: {}/{}",
        commit_id, namespace, repo_name
    );

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    repo.store_metadata(&commit_id, &metadata)?;

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "commit_id": commit_id
    })))
}

/// Acquire lock for repository
pub async fn acquire_lock(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<LockRequest>,
    ws_hub: web::Data<WsHub>,
    auth_service: web::Data<AuthService>,
    http_req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Acquiring lock for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&http_req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    let timeout = body.timeout_hours.unwrap_or(24);
    let lock = repo.acquire_lock(&body.user, &body.machine_id, timeout)?;

    // Log activity
    log_activity(
        &repo_path,
        ActivityType::LockAcquired,
        &body.user,
        &format!("Acquired lock for {} hours", timeout),
        Some(serde_json::json!({
            "lock_id": lock.lock_id,
            "machine_id": body.machine_id,
            "timeout_hours": timeout
        })),
    )?;

    // Broadcast to WebSocket subscribers
    let _ = ws_hub
        .broadcast_lock_acquired(&namespace, &repo_name, &body.user, &lock.lock_id)
        .await;

    Ok(HttpResponse::Ok().json(lock))
}

/// Release lock for repository
pub async fn release_lock(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<ReleaseLockRequest>,
    ws_hub: web::Data<WsHub>,
    auth_service: web::Data<AuthService>,
    http_req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Releasing lock for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&http_req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;

    // Get lock info before releasing (for activity log)
    let lock_info = repo.lock_status()?;
    let user = lock_info
        .as_ref()
        .map(|l| l.user.clone())
        .unwrap_or_else(|| "unknown".to_string());

    repo.release_lock(&body.lock_id)?;

    // Log activity
    log_activity(
        &repo_path,
        ActivityType::LockReleased,
        &user,
        "Released lock",
        Some(serde_json::json!({
            "lock_id": body.lock_id
        })),
    )?;

    // Broadcast to WebSocket subscribers
    let _ = ws_hub
        .broadcast_lock_released(&namespace, &repo_name, &body.lock_id)
        .await;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "Lock released"
    })))
}

/// Heartbeat for lock
pub async fn heartbeat_lock(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<HeartbeatRequest>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Heartbeat for lock in: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    let lock = repo.heartbeat_lock(&body.lock_id)?;

    Ok(HttpResponse::Ok().json(lock))
}

/// Get lock status
pub async fn lock_status(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting lock status for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let repo = RepositoryOps::open(&repo_path)?;
    let status = repo.lock_status()?;

    match status {
        Some(lock) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "locked": true,
            "lock": lock
        }))),
        None => Ok(HttpResponse::Ok().json(serde_json::json!({
            "locked": false
        }))),
    }
}

/// Get activity feed for repository
pub async fn get_activity(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    query: web::Query<ActivityQuery>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting activity for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let limit = query.limit.unwrap_or(50);
    let activities = get_activities(&repo_path, limit)?;

    Ok(HttpResponse::Ok().json(activities))
}

/// Clone a repository from remote
pub async fn clone_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    body: web::Json<CloneRequest>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Cloning repository to: {}/{}", namespace, repo_name);

    // Require authentication
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    let user = auth_service.get_user_by_token(
        req.headers()
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or_else(|| AppError::Unauthorized("No authorization token".to_string()))?,
    )?;

    // Validate namespace (prevent path traversal)
    if namespace.is_empty() || namespace.contains("..") || namespace.contains('/') {
        return Err(AppError::BadRequest("Invalid namespace".to_string()));
    }

    // Validate repository name
    if repo_name.is_empty() || repo_name.contains("..") || repo_name.contains('/') {
        return Err(AppError::BadRequest("Invalid repository name".to_string()));
    }

    // Validate URL
    if body.remote_url.is_empty() {
        return Err(AppError::BadRequest("Remote URL is required".to_string()));
    }

    // Build destination path
    let dest_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check if repository already exists
    if dest_path.exists() {
        return Err(AppError::BadRequest(
            "Repository already exists at this location".to_string(),
        ));
    }

    // Clone the repository
    let _repo = RepositoryOps::clone(&body.remote_url, &dest_path)?;

    // Create project metadata (cloned repo defaults to public)
    use crate::project::{ProjectMetadata, Visibility};
    let metadata = ProjectMetadata::new(user_id, user.username.clone(), Visibility::Public);
    metadata.save(&dest_path)?;

    info!(
        "Repository cloned successfully: {}/{} (owner: {})",
        namespace, repo_name, user.username
    );

    Ok(HttpResponse::Created().json(serde_json::json!({
        "status": "success",
        "namespace": namespace,
        "name": repo_name,
        "path": dest_path.to_string_lossy(),
        "remote_url": body.remote_url,
        "owner": user.username
    })))
}

/// Delete a branch
pub async fn delete_branch(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, branch_name) = path.into_inner();
    info!(
        "Deleting branch '{}' from: {}/{}",
        branch_name, namespace, repo_name
    );

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    repo.delete_branch(&branch_name)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("Branch '{}' deleted", branch_name)
    })))
}

/// Get repository status
pub async fn get_status(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Getting status for: {}/{}", namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check read access
    let user_id = get_optional_user_id_from_request(&req, &auth_service);
    ProjectAuth::require_read(&repo_path, user_id.as_deref())?;

    let repo = RepositoryOps::open(&repo_path)?;
    let status = repo.status()?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": status
    })))
}

/// Fetch from remote
pub async fn fetch_repository(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    query: web::Query<FetchQuery>,
    auth_service: web::Data<AuthService>,
    req: actix_web::HttpRequest,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    let remote = query.remote.clone().unwrap_or_else(|| "origin".to_string());
    info!("Fetching {} for: {}/{}", remote, namespace, repo_name);

    let repo_path = PathBuf::from(&config.server.sync_dir)
        .join(&namespace)
        .join(&repo_name);

    // Check write access (fetch can modify local refs)
    let user_id = get_user_id_from_request(&req, &auth_service)?;
    ProjectAuth::require_write(&repo_path, &user_id)?;

    let repo = RepositoryOps::open(&repo_path)?;
    repo.fetch(&remote)?;

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": format!("Fetched from {}", remote)
    })))
}

#[derive(Debug, Deserialize)]
pub struct FetchQuery {
    pub remote: Option<String>,
}
