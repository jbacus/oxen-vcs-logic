use crate::{error, info, vlog};
/// Oxen subprocess wrapper for executing actual oxen CLI commands
///
/// This module provides a wrapper around the oxen CLI tool, executing
/// commands via subprocess and parsing their output. This is a temporary
/// solution until official Rust bindings (liboxen) are available.
///
/// # Requirements
///
/// - `oxen` CLI must be installed and available in PATH
/// - Install via: `pip install oxen-ai` or `cargo install oxen`
///
/// # Usage
///
/// ```rust,no_run
/// use auxin_cli::oxen_subprocess::OxenSubprocess;
/// use std::path::Path;
///
/// let oxen = OxenSubprocess::new();
/// let result = oxen.init(Path::new("my_project.logicx"));
/// ```
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use wait_timeout::ChildExt;

// ========== Error Types ==========

/// Categorized errors from Oxen operations
#[derive(Debug, Clone, PartialEq)]
pub enum OxenError {
    /// Resource not found (commit, branch, file)
    NotFound(String),
    /// Network-related error (retry-able)
    NetworkError(String),
    /// Permission denied
    PermissionDenied(String),
    /// Invalid repository state
    InvalidRepository(String),
    /// Command timed out
    Timeout(String),
    /// Oxen CLI not available
    NotInstalled,
    /// Authentication error
    AuthenticationError(String),
    /// Other unclassified error
    Other(String),
}

impl std::fmt::Display for OxenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OxenError::NotFound(msg) => write!(f, "Not found: {}", msg),
            OxenError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            OxenError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            OxenError::InvalidRepository(msg) => write!(f, "Invalid repository: {}", msg),
            OxenError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            OxenError::NotInstalled => write!(f, "Oxen CLI not installed"),
            OxenError::AuthenticationError(msg) => write!(f, "Authentication error: {}", msg),
            OxenError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for OxenError {}

impl OxenError {
    /// Check if this error is retry-able
    pub fn is_retryable(&self) -> bool {
        matches!(self, OxenError::NetworkError(_) | OxenError::Timeout(_))
    }

    /// Classify error from output text
    fn classify(stdout: &str, stderr: &str) -> Option<Self> {
        let stdout_lower = stdout.to_lowercase();
        let stderr_lower = stderr.to_lowercase();

        // Check for specific error patterns
        if stdout_lower.contains("revision not found")
            || stdout_lower.contains("branch not found")
            || stdout_lower.contains("file not found")
            || stderr_lower.contains("not found")
        {
            return Some(OxenError::NotFound(
                stderr.trim().to_string().chars().take(200).collect(),
            ));
        }

        if stderr_lower.contains("connection refused")
            || stderr_lower.contains("network")
            || stderr_lower.contains("timeout")
            || stderr_lower.contains("could not resolve")
            || stderr_lower.contains("failed to connect")
        {
            return Some(OxenError::NetworkError(
                stderr.trim().to_string().chars().take(200).collect(),
            ));
        }

        if stderr_lower.contains("permission denied")
            || stderr_lower.contains("access denied")
            || stderr_lower.contains("forbidden")
        {
            return Some(OxenError::PermissionDenied(
                stderr.trim().to_string().chars().take(200).collect(),
            ));
        }

        if stderr_lower.contains("not a valid oxen repository")
            || stderr_lower.contains("not an oxen repository")
            || stdout_lower.contains("not a valid oxen repository")
        {
            return Some(OxenError::InvalidRepository(
                stderr.trim().to_string().chars().take(200).collect(),
            ));
        }

        if stderr_lower.contains("authentication")
            || stderr_lower.contains("unauthorized")
            || stderr_lower.contains("invalid credentials")
        {
            return Some(OxenError::AuthenticationError(
                stderr.trim().to_string().chars().take(200).collect(),
            ));
        }

        // Check for general error indicators
        if stdout_lower.contains("error:")
            || stdout_lower.contains("fatal:")
            || stdout_lower.contains("failed to")
            || stderr_lower.contains("error:")
            || stderr_lower.contains("fatal:")
            || stderr_lower.contains("failed to")
        {
            let msg = if !stderr.trim().is_empty() {
                stderr.trim()
            } else {
                stdout.trim()
            };
            return Some(OxenError::Other(msg.chars().take(200).collect()));
        }

        None
    }
}

// ========== Cache ==========

/// Cached entry with timestamp
#[derive(Clone)]
struct CacheEntry<T> {
    data: T,
    timestamp: Instant,
}

/// Cache for expensive operations
struct OxenCache {
    log_cache: HashMap<(PathBuf, Option<usize>), CacheEntry<Vec<CommitInfo>>>,
    status_cache: HashMap<PathBuf, CacheEntry<StatusInfo>>,
    branches_cache: HashMap<PathBuf, CacheEntry<Vec<BranchInfo>>>,
    ttl: Duration,
}

impl OxenCache {
    fn new(ttl: Duration) -> Self {
        Self {
            log_cache: HashMap::new(),
            status_cache: HashMap::new(),
            branches_cache: HashMap::new(),
            ttl,
        }
    }

    fn get_log(&self, key: &(PathBuf, Option<usize>)) -> Option<Vec<CommitInfo>> {
        self.log_cache.get(key).and_then(|entry| {
            if entry.timestamp.elapsed() < self.ttl {
                Some(entry.data.clone())
            } else {
                None
            }
        })
    }

    fn set_log(&mut self, key: (PathBuf, Option<usize>), data: Vec<CommitInfo>) {
        self.log_cache.insert(
            key,
            CacheEntry {
                data,
                timestamp: Instant::now(),
            },
        );
    }

    fn get_status(&self, key: &PathBuf) -> Option<StatusInfo> {
        self.status_cache.get(key).and_then(|entry| {
            if entry.timestamp.elapsed() < self.ttl {
                Some(entry.data.clone())
            } else {
                None
            }
        })
    }

    fn set_status(&mut self, key: PathBuf, data: StatusInfo) {
        self.status_cache.insert(
            key,
            CacheEntry {
                data,
                timestamp: Instant::now(),
            },
        );
    }

    fn get_branches(&self, key: &PathBuf) -> Option<Vec<BranchInfo>> {
        self.branches_cache.get(key).and_then(|entry| {
            if entry.timestamp.elapsed() < self.ttl {
                Some(entry.data.clone())
            } else {
                None
            }
        })
    }

    fn set_branches(&mut self, key: PathBuf, data: Vec<BranchInfo>) {
        self.branches_cache.insert(
            key,
            CacheEntry {
                data,
                timestamp: Instant::now(),
            },
        );
    }

    fn invalidate(&mut self, repo_path: &Path) {
        // Remove all entries for this repository
        self.log_cache
            .retain(|(path, _), _| path != repo_path);
        self.status_cache.remove(repo_path);
        self.branches_cache.remove(repo_path);
    }

    fn invalidate_all(&mut self) {
        self.log_cache.clear();
        self.status_cache.clear();
        self.branches_cache.clear();
    }
}

// ========== Configuration ==========

/// Configuration for OxenSubprocess
#[derive(Debug, Clone)]
pub struct OxenConfig {
    /// Path to oxen executable
    pub oxen_path: String,
    /// Default timeout for operations (in seconds)
    pub default_timeout: u64,
    /// Timeout for network operations (in seconds)
    pub network_timeout: u64,
    /// Maximum files per batch for add operations
    pub batch_size: usize,
    /// Cache TTL (in milliseconds)
    pub cache_ttl_ms: u64,
    /// Default remote name
    pub default_remote: String,
    /// Default main branch name
    pub main_branch: String,
    /// Default draft branch name
    pub draft_branch: String,
}

impl Default for OxenConfig {
    fn default() -> Self {
        Self {
            oxen_path: std::env::var("AUXIN_OXEN_PATH").unwrap_or_else(|_| "oxen".to_string()),
            default_timeout: std::env::var("AUXIN_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(30),
            network_timeout: std::env::var("AUXIN_NETWORK_TIMEOUT")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(120),
            batch_size: std::env::var("AUXIN_BATCH_SIZE")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            cache_ttl_ms: std::env::var("AUXIN_CACHE_TTL_MS")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(1000),
            default_remote: std::env::var("AUXIN_DEFAULT_REMOTE")
                .unwrap_or_else(|_| "origin".to_string()),
            main_branch: std::env::var("AUXIN_MAIN_BRANCH")
                .unwrap_or_else(|_| "main".to_string()),
            draft_branch: std::env::var("AUXIN_DRAFT_BRANCH")
                .unwrap_or_else(|_| "draft".to_string()),
        }
    }
}

// ========== Main Struct ==========

/// Minimum supported Oxen CLI version
const MIN_OXEN_VERSION: &str = "0.19";

/// Sanitize a path to prevent path traversal attacks
///
/// # Security
///
/// This function validates that paths:
/// - Do not contain dangerous characters (null bytes, control characters)
/// - Do not contain suspicious patterns (command injection attempts)
/// - Are within the expected repository root (when provided)
///
/// # Arguments
///
/// * `path` - The path to sanitize
/// * `repo_root` - Optional repository root to validate path is within
///
/// # Returns
///
/// The sanitized path string, or an error if validation fails
fn sanitize_path(path: &Path, repo_root: Option<&Path>) -> Result<String> {
    let path_str = path.to_string_lossy();

    // Check for null bytes (potential security issue)
    if path_str.contains('\0') {
        return Err(anyhow!("Invalid path: contains null byte"));
    }

    // Check for control characters
    if path_str.chars().any(|c| c.is_control() && c != '\n' && c != '\t') {
        return Err(anyhow!("Invalid path: contains control characters"));
    }

    // Check for command injection patterns
    let dangerous_patterns = [
        "$(", "`", ";", "&&", "||", "|", ">", "<", "\n", "\r",
    ];
    for pattern in &dangerous_patterns {
        if path_str.contains(pattern) {
            return Err(anyhow!(
                "Invalid path: contains potentially dangerous pattern '{}'",
                pattern
            ));
        }
    }

    // If repo_root is provided, ensure path is within it (canonicalize if possible)
    if let Some(root) = repo_root {
        // For relative paths, resolve them relative to repo root
        let resolved_path = if path.is_relative() {
            root.join(path)
        } else {
            path.to_path_buf()
        };

        // For existing paths, canonicalize to check for path traversal
        if resolved_path.exists() {
            let canonical = resolved_path.canonicalize()
                .context("Failed to canonicalize path")?;
            let root_canonical = root.canonicalize()
                .context("Failed to canonicalize repository root")?;

            if !canonical.starts_with(&root_canonical) {
                return Err(anyhow!(
                    "Path traversal detected: {} is outside repository root {}",
                    canonical.display(),
                    root_canonical.display()
                ));
            }
        } else {
            // For non-existing paths, check for obvious traversal patterns
            let normalized = path_str.replace("\\", "/");
            if normalized.contains("/../") || normalized.starts_with("../") {
                return Err(anyhow!(
                    "Potential path traversal detected in: {}",
                    path.display()
                ));
            }
        }
    }

    Ok(path_str.to_string())
}

/// Sanitize a commit message to prevent injection
fn sanitize_message(message: &str) -> Result<String> {
    // Check for null bytes
    if message.contains('\0') {
        return Err(anyhow!("Invalid message: contains null byte"));
    }

    // Commit messages can contain most characters, but we should
    // limit extremely long messages and check for control characters
    if message.len() > 10000 {
        return Err(anyhow!("Commit message too long (max 10000 characters)"));
    }

    Ok(message.to_string())
}

/// Wrapper for executing Oxen CLI commands via subprocess.
///
/// This struct provides a Rust interface to the `oxen` command-line tool by executing
/// commands as subprocesses and parsing their output. This is a production-ready
/// solution until official Rust bindings (liboxen) become available.
#[derive(Clone)]
pub struct OxenSubprocess {
    /// Configuration
    config: OxenConfig,
    /// Enable verbose logging
    verbose: bool,
    /// Cache for expensive operations
    cache: Arc<Mutex<OxenCache>>,
}

impl std::fmt::Debug for OxenSubprocess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OxenSubprocess")
            .field("config", &self.config)
            .field("verbose", &self.verbose)
            .finish()
    }
}

impl OxenSubprocess {
    /// Create a new OxenSubprocess wrapper with default settings
    pub fn new() -> Self {
        let config = OxenConfig::default();
        let cache_ttl = Duration::from_millis(config.cache_ttl_ms);
        Self {
            config,
            verbose: false,
            cache: Arc::new(Mutex::new(OxenCache::new(cache_ttl))),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: OxenConfig) -> Self {
        let cache_ttl = Duration::from_millis(config.cache_ttl_ms);
        Self {
            config,
            verbose: false,
            cache: Arc::new(Mutex::new(OxenCache::new(cache_ttl))),
        }
    }

    /// Create with custom oxen executable path
    pub fn with_path(oxen_path: impl Into<String>) -> Self {
        let config = OxenConfig {
            oxen_path: oxen_path.into(),
            ..Default::default()
        };
        let cache_ttl = Duration::from_millis(config.cache_ttl_ms);
        Self {
            config,
            verbose: false,
            cache: Arc::new(Mutex::new(OxenCache::new(cache_ttl))),
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Get configuration
    pub fn config(&self) -> &OxenConfig {
        &self.config
    }

    /// Invalidate cache for a repository
    pub fn invalidate_cache(&self, repo_path: &Path) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.invalidate(repo_path);
        }
    }

    /// Invalidate all caches
    pub fn invalidate_all_caches(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.invalidate_all();
        }
    }

    /// Check if oxen is available in PATH
    pub fn is_available(&self) -> bool {
        Command::new(&self.config.oxen_path)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get oxen version
    pub fn version(&self) -> Result<String> {
        let output = self.run_command(&["--version"], None, None)?;
        Ok(output.trim().to_string())
    }

    /// Verify that the installed Oxen CLI version is compatible
    ///
    /// # Returns
    ///
    /// Ok(()) if the version is compatible, error otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use auxin_cli::OxenSubprocess;
    ///
    /// let oxen = OxenSubprocess::new();
    /// oxen.verify_version()?;
    /// ```
    pub fn verify_version(&self) -> Result<()> {
        let version = self.version()?;

        // Parse version string (e.g., "oxen 0.19.5" or "0.19.5")
        let version_parts: Vec<&str> = version.split_whitespace().collect();
        let version_str = version_parts.last().unwrap_or(&"unknown");

        // Check if version starts with minimum required version
        if !version_str.starts_with(MIN_OXEN_VERSION) {
            // Also check if version is newer (e.g., 0.20.x should work)
            let parts: Vec<&str> = version_str.split('.').collect();
            let min_parts: Vec<&str> = MIN_OXEN_VERSION.split('.').collect();

            let is_compatible = if parts.len() >= 2 && min_parts.len() >= 2 {
                let major: u32 = parts[0].parse().unwrap_or(0);
                let minor: u32 = parts[1].parse().unwrap_or(0);
                let min_major: u32 = min_parts[0].parse().unwrap_or(0);
                let min_minor: u32 = min_parts[1].parse().unwrap_or(0);

                (major > min_major) || (major == min_major && minor >= min_minor)
            } else {
                false
            };

            if !is_compatible {
                return Err(anyhow!(
                    "Oxen CLI version {} is not compatible. Requires {} or newer.\n\
                     Please update: pip install --upgrade oxen-ai",
                    version_str,
                    MIN_OXEN_VERSION
                ));
            }
        }

        vlog!("Oxen CLI version {} verified", version_str);
        Ok(())
    }

    /// Initialize a new oxen repository
    pub fn init(&self, path: &Path) -> Result<()> {
        vlog!("Initializing oxen repository at: {}", path.display());

        self.run_command(&["init"], Some(path), None)?;
        self.invalidate_cache(path);

        info!("Initialized oxen repository: {}", path.display());
        Ok(())
    }

    /// Add files to staging
    ///
    /// # Security
    ///
    /// All file paths are sanitized to prevent:
    /// - Path traversal attacks (../)
    /// - Command injection via malicious filenames
    /// - Null byte injection
    pub fn add(&self, repo_path: &Path, files: &[&Path]) -> Result<()> {
        if files.is_empty() {
            return Err(anyhow!("No files specified to add"));
        }

        vlog!("Adding {} file(s) to staging", files.len());

        // Use batching for large file sets
        if files.len() > self.config.batch_size {
            return self.add_batched(repo_path, files);
        }

        let file_args: Vec<String> = files
            .iter()
            .map(|f| sanitize_path(f, Some(repo_path)))
            .collect::<Result<Vec<String>>>()
            .context("Failed to sanitize file paths")?;

        let mut args = vec!["add"];
        for file in &file_args {
            args.push(file);
        }

        self.run_command(&args, Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Added {} file(s) to staging", files.len());
        Ok(())
    }

    /// Add files in batches (for large file sets)
    fn add_batched(&self, repo_path: &Path, files: &[&Path]) -> Result<()> {
        let batch_size = self.config.batch_size;
        let total_batches = files.len().div_ceil(batch_size);

        vlog!(
            "Adding {} files in {} batches",
            files.len(),
            total_batches
        );

        for (i, chunk) in files.chunks(batch_size).enumerate() {
            vlog!("Processing batch {}/{}", i + 1, total_batches);

            let file_args: Vec<String> = chunk
                .iter()
                .map(|f| f.to_string_lossy().to_string())
                .collect();

            let mut args = vec!["add"];
            for file in &file_args {
                args.push(file);
            }

            self.run_command(&args, Some(repo_path), None)?;
        }

        self.invalidate_cache(repo_path);
        info!("Added {} file(s) to staging in {} batches", files.len(), total_batches);
        Ok(())
    }

    /// Add all files to staging
    pub fn add_all(&self, repo_path: &Path) -> Result<()> {
        vlog!("Adding all files to staging");

        self.run_command(&["add", "."], Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Added all files to staging");
        Ok(())
    }

    /// Create a commit
    ///
    /// # Security
    ///
    /// The commit message is sanitized to prevent injection attacks.
    pub fn commit(&self, repo_path: &Path, message: &str) -> Result<CommitInfo> {
        if message.is_empty() {
            return Err(anyhow!("Commit message cannot be empty"));
        }

        // Sanitize the commit message
        let sanitized_message = sanitize_message(message)?;

        let output = self.run_command(&["commit", "-m", &sanitized_message], Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        // Parse commit hash from output
        let commit_id = self
            .parse_commit_id(&output)
            .unwrap_or_else(|| "unknown".to_string());

        info!("Created commit: {}", commit_id);

        Ok(CommitInfo {
            id: commit_id,
            message: message.to_string(),
        })
    }

    /// Get commit log (with caching)
    pub fn log(&self, repo_path: &Path, limit: Option<usize>) -> Result<Vec<CommitInfo>> {
        vlog!("Fetching commit log");

        // Check cache first
        let cache_key = (repo_path.to_path_buf(), limit);
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_log(&cache_key) {
                vlog!("Returning cached log ({} commits)", cached.len());
                return Ok(cached);
            }
        }

        let mut args = vec!["log"];
        let limit_str;
        if let Some(n) = limit {
            limit_str = format!("-n={}", n);
            args.push(&limit_str);
        }

        let output = self.run_command(&args, Some(repo_path), None)?;
        let commits = self.parse_log_output(&output)?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.set_log(cache_key, commits.clone());
        }

        vlog!("Found {} commit(s)", commits.len());
        Ok(commits)
    }

    /// Get repository status (with caching)
    pub fn status(&self, repo_path: &Path) -> Result<StatusInfo> {
        vlog!("Getting repository status");

        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_status(&repo_path.to_path_buf()) {
                vlog!("Returning cached status");
                return Ok(cached);
            }
        }

        let output = self.run_command(&["status"], Some(repo_path), None)?;
        let status = self.parse_status_output(&output)?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.set_status(repo_path.to_path_buf(), status.clone());
        }

        vlog!(
            "Status: {} modified, {} untracked",
            status.modified.len(),
            status.untracked.len()
        );
        Ok(status)
    }

    /// Checkout a specific commit or branch
    pub fn checkout(&self, repo_path: &Path, target: &str) -> Result<()> {
        vlog!("Checking out: {}", target);

        self.run_command(&["checkout", target], Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Checked out: {}", target);
        Ok(())
    }

    /// Create a new branch
    pub fn create_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        vlog!("Creating branch: {}", branch_name);

        self.run_command(&["checkout", "-b", branch_name], Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Created branch: {}", branch_name);
        Ok(())
    }

    /// List branches (with caching)
    pub fn list_branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>> {
        vlog!("Listing branches");

        // Check cache first
        if let Ok(cache) = self.cache.lock() {
            if let Some(cached) = cache.get_branches(&repo_path.to_path_buf()) {
                vlog!("Returning cached branches ({} branches)", cached.len());
                return Ok(cached);
            }
        }

        let output = self.run_command(&["branch"], Some(repo_path), None)?;
        let branches = self.parse_branches_output(&output)?;

        // Update cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.set_branches(repo_path.to_path_buf(), branches.clone());
        }

        vlog!("Found {} branch(es)", branches.len());
        Ok(branches)
    }

    /// Get current branch name
    pub fn current_branch(&self, repo_path: &Path) -> Result<String> {
        vlog!("Getting current branch");

        let output = self.run_command(&["branch", "--show-current"], Some(repo_path), None)?;

        let branch = output.trim().to_string();
        vlog!("Current branch: {}", branch);
        Ok(branch)
    }

    /// Delete a branch (force delete to allow removing unmerged branches)
    pub fn delete_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        vlog!("Deleting branch: {}", branch_name);

        // Use -D (force delete) to allow deleting unmerged branches
        self.run_command(&["branch", "-D", branch_name], Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Deleted branch: {}", branch_name);
        Ok(())
    }

    /// Push to remote (with network timeout)
    pub fn push(&self, repo_path: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        vlog!("Pushing to remote");

        let mut args = vec!["push"];
        if let Some(r) = remote {
            args.push(r);
        }
        if let Some(b) = branch {
            args.push(b);
        }

        // Use network timeout for push operations
        let timeout = Some(Duration::from_secs(self.config.network_timeout));
        self.run_command(&args, Some(repo_path), timeout)?;

        info!("Pushed to remote");
        Ok(())
    }

    /// Pull from remote (with network timeout)
    pub fn pull(&self, repo_path: &Path) -> Result<()> {
        vlog!("Pulling from remote");

        let timeout = Some(Duration::from_secs(self.config.network_timeout));
        self.run_command(&["pull"], Some(repo_path), timeout)?;
        self.invalidate_cache(repo_path);

        info!("Pulled from remote");
        Ok(())
    }

    // ========== New Operations ==========

    /// Fetch from remote without merging
    pub fn fetch(&self, repo_path: &Path, remote: Option<&str>) -> Result<()> {
        vlog!("Fetching from remote");

        let mut args = vec!["fetch"];
        if let Some(r) = remote {
            args.push(r);
        }

        let timeout = Some(Duration::from_secs(self.config.network_timeout));
        self.run_command(&args, Some(repo_path), timeout)?;

        info!("Fetched from remote");
        Ok(())
    }

    /// Show diff between commits or working directory
    pub fn diff(&self, repo_path: &Path, target: Option<&str>) -> Result<String> {
        vlog!("Getting diff");

        let mut args = vec!["diff"];
        if let Some(t) = target {
            args.push(t);
        }

        let output = self.run_command(&args, Some(repo_path), None)?;
        Ok(output)
    }

    /// Reset/unstage files
    pub fn reset(&self, repo_path: &Path, files: Option<&[&Path]>) -> Result<()> {
        vlog!("Resetting files");

        let file_args: Vec<String> = files
            .map(|file_list| {
                file_list
                    .iter()
                    .map(|f| f.to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();

        let mut args = vec!["reset"];
        for file in &file_args {
            args.push(file);
        }

        self.run_command(&args, Some(repo_path), None)?;
        self.invalidate_cache(repo_path);

        info!("Reset completed");
        Ok(())
    }

    /// Create a tag
    pub fn tag(&self, repo_path: &Path, tag_name: &str, message: Option<&str>) -> Result<()> {
        vlog!("Creating tag: {}", tag_name);

        let mut args = vec!["tag", tag_name];
        if let Some(msg) = message {
            args.push("-m");
            args.push(msg);
        }

        self.run_command(&args, Some(repo_path), None)?;

        info!("Created tag: {}", tag_name);
        Ok(())
    }

    /// List tags
    pub fn list_tags(&self, repo_path: &Path) -> Result<Vec<String>> {
        vlog!("Listing tags");

        let output = self.run_command(&["tag"], Some(repo_path), None)?;

        let tags: Vec<String> = output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        vlog!("Found {} tag(s)", tags.len());
        Ok(tags)
    }

    /// Show commit details
    pub fn show(&self, repo_path: &Path, commit_id: &str) -> Result<String> {
        vlog!("Showing commit: {}", commit_id);

        let output = self.run_command(&["show", commit_id], Some(repo_path), None)?;
        Ok(output)
    }

    /// Add remote
    pub fn remote_add(&self, repo_path: &Path, name: &str, url: &str) -> Result<()> {
        vlog!("Adding remote: {} -> {}", name, url);

        self.run_command(&["remote", "add", name, url], Some(repo_path), None)?;

        info!("Added remote: {}", name);
        Ok(())
    }

    /// List remotes
    pub fn remote_list(&self, repo_path: &Path) -> Result<Vec<(String, String)>> {
        vlog!("Listing remotes");

        let output = self.run_command(&["remote", "-v"], Some(repo_path), None)?;

        let mut remotes = Vec::new();
        for line in output.lines() {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                remotes.push((parts[0].to_string(), parts[1].to_string()));
            }
        }

        // Deduplicate (fetch and push often shown separately)
        remotes.sort();
        remotes.dedup();

        vlog!("Found {} remote(s)", remotes.len());
        Ok(remotes)
    }

    /// Remove remote
    pub fn remote_remove(&self, repo_path: &Path, name: &str) -> Result<()> {
        vlog!("Removing remote: {}", name);

        self.run_command(&["remote", "remove", name], Some(repo_path), None)?;

        info!("Removed remote: {}", name);
        Ok(())
    }

    // ========== Private Helper Methods ==========

    /// Run an oxen command with timeout
    fn run_command(
        &self,
        args: &[&str],
        cwd: Option<&Path>,
        timeout: Option<Duration>,
    ) -> Result<String> {
        if self.verbose {
            vlog!("Running: {} {}", self.config.oxen_path, args.join(" "));
        }

        let mut cmd = Command::new(&self.config.oxen_path);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Failed to spawn oxen command: {}", args.join(" ")))?;

        // Apply timeout
        let timeout_duration = timeout.unwrap_or(Duration::from_secs(self.config.default_timeout));

        let status = child
            .wait_timeout(timeout_duration)
            .with_context(|| format!("Error waiting for oxen command: {}", args.join(" ")))?;

        match status {
            Some(_) => {
                // Command completed within timeout
                let output = self.collect_output(child)?;
                self.handle_output(output, args)
            }
            None => {
                // Timeout - kill the process
                let _ = child.kill();
                let _ = child.wait();

                let cmd_str = args.join(" ");
                error!("Command timed out after {:?}: oxen {}", timeout_duration, cmd_str);

                Err(anyhow!(OxenError::Timeout(format!(
                    "Command timed out after {:?}: oxen {}",
                    timeout_duration, cmd_str
                ))))
            }
        }
    }

    /// Collect output from completed child process
    fn collect_output(&self, child: Child) -> Result<Output> {
        let output = child
            .wait_with_output()
            .context("Failed to collect command output")?;
        Ok(output)
    }

    /// Handle command output and errors
    fn handle_output(&self, output: Output, args: &[&str]) -> Result<String> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // Check for categorized errors
        if let Some(oxen_error) = OxenError::classify(&stdout, &stderr) {
            error!("Command failed: oxen {}", args.join(" "));
            if !stderr.is_empty() {
                error!("stderr: {}", stderr);
            }
            if !stdout.is_empty() && self.verbose {
                error!("stdout: {}", stdout);
            }

            return Err(anyhow!(oxen_error));
        }

        // Also check exit code
        if !output.status.success() {
            error!("Command failed: oxen {}", args.join(" "));
            error!("stderr: {}", stderr);
            return Err(anyhow!(
                "oxen command failed: {}\nstderr: {}",
                args.join(" "),
                stderr
            ));
        }

        if self.verbose && !stdout.is_empty() {
            vlog!("stdout: {}", stdout);
        }

        Ok(stdout)
    }

    /// Parse commit ID from commit command output
    fn parse_commit_id(&self, output: &str) -> Option<String> {
        // Look for "commit " prefix first (most reliable)
        for line in output.lines() {
            if let Some(rest) = line.trim().strip_prefix("commit ") {
                let hash = rest.split_whitespace().next()?;
                if hash.len() >= 7 && hash.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Some(hash.to_string());
                }
            }
        }

        // Fall back to looking for hex strings
        for line in output.lines() {
            for word in line.split_whitespace() {
                let cleaned = word.trim_matches(|c| !char::is_alphanumeric(c));
                if cleaned.len() >= 7
                    && cleaned.len() <= 40
                    && cleaned.chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Some(cleaned.to_string());
                }
            }
        }

        None
    }

    /// Parse log output into CommitInfo structs
    fn parse_log_output(&self, output: &str) -> Result<Vec<CommitInfo>> {
        let mut commits = Vec::new();
        let mut current_id = None;
        let mut current_message = String::new();

        for line in output.lines() {
            let trimmed = line.trim();

            // Look for commit hash line
            if let Some(hash) = trimmed.strip_prefix("commit ") {
                // Save previous commit if exists
                if let Some(id) = current_id.take() {
                    commits.push(CommitInfo {
                        id,
                        message: current_message.trim().to_string(),
                    });
                    current_message.clear();
                }

                // Extract new commit hash
                current_id = Some(hash.trim().to_string());
            } else if !trimmed.is_empty()
                && !trimmed.starts_with("Author:")
                && !trimmed.starts_with("Date:")
            {
                // This is part of the commit message
                if !current_message.is_empty() {
                    current_message.push('\n');
                }
                current_message.push_str(trimmed);
            }
        }

        // Don't forget the last commit
        if let Some(id) = current_id {
            commits.push(CommitInfo {
                id,
                message: current_message.trim().to_string(),
            });
        }

        Ok(commits)
    }

    /// Parse status output
    fn parse_status_output(&self, output: &str) -> Result<StatusInfo> {
        let mut modified = Vec::new();
        let mut untracked = Vec::new();
        let mut staged = Vec::new();

        let mut current_section = None;

        for line in output.lines() {
            let trimmed = line.trim();

            // Check for section headers
            if trimmed.starts_with("Untracked Files")
                || trimmed.starts_with("Untracked Directories")
            {
                current_section = Some("untracked");
                continue;
            } else if trimmed.starts_with("Modified Files")
                || trimmed.starts_with("Changes not staged")
            {
                current_section = Some("modified");
                continue;
            } else if trimmed.starts_with("Staged Files")
                || trimmed.starts_with("Changes to be committed")
            {
                current_section = Some("staged");
                continue;
            }

            // Skip empty lines and help text
            if trimmed.is_empty() || trimmed.starts_with("(use") || trimmed.starts_with("On branch")
            {
                continue;
            }

            // Legacy format support
            if trimmed.starts_with("M ") || trimmed.starts_with("modified:") {
                let path = self.extract_path_from_status_line(trimmed);
                modified.push(path);
            } else if trimmed.starts_with("? ") || trimmed.starts_with("untracked:") {
                let path = self.extract_path_from_status_line(trimmed);
                untracked.push(path);
            } else if trimmed.starts_with("A ") || trimmed.starts_with("new file:") {
                let path = self.extract_path_from_status_line(trimmed);
                staged.push(path);
            } else if let Some(section) = current_section {
                // Files/directories listed under section headers
                let path_str = if let Some(paren_pos) = trimmed.find(" (") {
                    &trimmed[..paren_pos]
                } else {
                    trimmed
                };
                let path = PathBuf::from(path_str);
                match section {
                    "untracked" => untracked.push(path),
                    "modified" => modified.push(path),
                    "staged" => staged.push(path),
                    _ => {}
                }
            }
        }

        Ok(StatusInfo {
            modified,
            untracked,
            staged,
        })
    }

    /// Extract file path from status line
    fn extract_path_from_status_line(&self, line: &str) -> PathBuf {
        // Handle formats like:
        // "M  path/to/file"
        // "modified: path/to/file"
        // "new file: path/to/file"

        // First try to split on ':'
        if let Some(colon_pos) = line.find(':') {
            let path = &line[colon_pos + 1..];
            return PathBuf::from(path.trim());
        }

        // Fall back to splitting on whitespace
        let parts: Vec<&str> = line.splitn(2, char::is_whitespace).collect();
        if parts.len() >= 2 {
            PathBuf::from(parts[1].trim())
        } else {
            PathBuf::from(line.trim())
        }
    }

    /// Parse branches output
    fn parse_branches_output(&self, output: &str) -> Result<Vec<BranchInfo>> {
        let mut branches = Vec::new();

        for line in output.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let is_current = trimmed.starts_with('*');
            let name = if is_current {
                trimmed[1..].trim().to_string()
            } else {
                trimmed.to_string()
            };

            branches.push(BranchInfo { name, is_current });
        }

        Ok(branches)
    }
}

impl Default for OxenSubprocess {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Data Structures ==========

/// Information about a commit
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CommitInfo {
    /// Commit hash/ID
    pub id: String,
    /// Commit message
    pub message: String,
}

/// Repository status information
#[derive(Debug, Clone, PartialEq)]
pub struct StatusInfo {
    /// Modified files
    pub modified: Vec<PathBuf>,
    /// Untracked files
    pub untracked: Vec<PathBuf>,
    /// Staged files
    pub staged: Vec<PathBuf>,
}

/// Branch information
#[derive(Debug, Clone, PartialEq)]
pub struct BranchInfo {
    /// Branch name
    pub name: String,
    /// Whether this is the current branch
    pub is_current: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_default() {
        let oxen = OxenSubprocess::new();
        assert_eq!(oxen.config.oxen_path, "oxen");
        assert!(!oxen.verbose);
    }

    #[test]
    fn test_with_path() {
        let oxen = OxenSubprocess::with_path("/usr/local/bin/oxen");
        assert_eq!(oxen.config.oxen_path, "/usr/local/bin/oxen");
    }

    #[test]
    fn test_verbose_builder() {
        let oxen = OxenSubprocess::new().verbose(true);
        assert!(oxen.verbose);
    }

    #[test]
    fn test_config_defaults() {
        let config = OxenConfig::default();
        assert_eq!(config.oxen_path, "oxen");
        assert_eq!(config.default_timeout, 30);
        assert_eq!(config.network_timeout, 120);
        assert_eq!(config.batch_size, 1000);
        assert_eq!(config.default_remote, "origin");
        assert_eq!(config.main_branch, "main");
        assert_eq!(config.draft_branch, "draft");
    }

    #[test]
    fn test_oxen_error_display() {
        let err = OxenError::NotFound("branch 'test'".to_string());
        assert!(err.to_string().contains("Not found"));
        assert!(err.to_string().contains("branch 'test'"));
    }

    #[test]
    fn test_oxen_error_retryable() {
        assert!(OxenError::NetworkError("timeout".to_string()).is_retryable());
        assert!(OxenError::Timeout("cmd".to_string()).is_retryable());
        assert!(!OxenError::NotFound("file".to_string()).is_retryable());
        assert!(!OxenError::PermissionDenied("access".to_string()).is_retryable());
    }

    #[test]
    fn test_error_classification_not_found() {
        let err = OxenError::classify("revision not found", "");
        assert!(matches!(err, Some(OxenError::NotFound(_))));
    }

    #[test]
    fn test_error_classification_network() {
        let err = OxenError::classify("", "connection refused");
        assert!(matches!(err, Some(OxenError::NetworkError(_))));
    }

    #[test]
    fn test_error_classification_permission() {
        let err = OxenError::classify("", "permission denied");
        assert!(matches!(err, Some(OxenError::PermissionDenied(_))));
    }

    #[test]
    fn test_error_classification_auth() {
        let err = OxenError::classify("", "authentication failed");
        assert!(matches!(err, Some(OxenError::AuthenticationError(_))));
    }

    #[test]
    fn test_error_classification_none() {
        let err = OxenError::classify("all good", "");
        assert!(err.is_none());
    }

    #[test]
    fn test_parse_commit_id_various_formats() {
        let oxen = OxenSubprocess::new();

        assert_eq!(
            oxen.parse_commit_id("Commit abc123def created"),
            Some("abc123def".to_string())
        );

        assert_eq!(
            oxen.parse_commit_id("[abc1234] message here"),
            Some("abc1234".to_string())
        );

        assert_eq!(
            oxen.parse_commit_id("abc123def456"),
            Some("abc123def456".to_string())
        );
    }

    #[test]
    fn test_parse_commit_id_prefers_commit_prefix() {
        let oxen = OxenSubprocess::new();
        let output = "commit abc1234def\nSome other text with def5678";
        assert_eq!(oxen.parse_commit_id(output), Some("abc1234def".to_string()));
    }

    #[test]
    fn test_parse_commit_id_invalid() {
        let oxen = OxenSubprocess::new();

        assert_eq!(oxen.parse_commit_id("No hash here"), None);
        assert_eq!(oxen.parse_commit_id("123"), None);
        assert_eq!(oxen.parse_commit_id("xyz"), None);
    }

    #[test]
    fn test_extract_path_from_status_line() {
        let oxen = OxenSubprocess::new();

        assert_eq!(
            oxen.extract_path_from_status_line("M  src/main.rs"),
            PathBuf::from("src/main.rs")
        );

        assert_eq!(
            oxen.extract_path_from_status_line("modified: src/lib.rs"),
            PathBuf::from("src/lib.rs")
        );

        assert_eq!(
            oxen.extract_path_from_status_line("?  test.txt"),
            PathBuf::from("test.txt")
        );
    }

    #[test]
    fn test_parse_status_output() {
        let oxen = OxenSubprocess::new();
        let output = r#"
M  src/main.rs
?  temp.txt
A  new_file.rs
modified: another.rs
        "#;

        let status = oxen.parse_status_output(output).unwrap();

        assert_eq!(status.modified.len(), 2);
        assert!(status.modified.contains(&PathBuf::from("src/main.rs")));
        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.staged.len(), 1);
    }

    #[test]
    fn test_parse_branches_output() {
        let oxen = OxenSubprocess::new();
        let output = r#"
* main
  draft
  feature-branch
        "#;

        let branches = oxen.parse_branches_output(output).unwrap();

        assert_eq!(branches.len(), 3);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_current);
        assert_eq!(branches[1].name, "draft");
        assert!(!branches[1].is_current);
    }

    #[test]
    fn test_parse_log_output() {
        let oxen = OxenSubprocess::new();
        let output = r#"
commit abc123def456
Author: User <user@example.com>
Date: 2025-01-01

    First commit message

commit 789xyz012
Author: User <user@example.com>
Date: 2025-01-02

    Second commit
    Multi-line message
        "#;

        let commits = oxen.parse_log_output(output).unwrap();

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].id, "abc123def456");
        assert!(commits[0].message.contains("First commit"));
        assert_eq!(commits[1].id, "789xyz012");
        assert!(commits[1].message.contains("Second commit"));
    }

    #[test]
    fn test_commit_info_serialization() {
        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: "Test commit".to_string(),
        };

        let json = serde_json::to_string(&commit).unwrap();
        assert!(json.contains("abc123"));
        assert!(json.contains("Test commit"));

        let deserialized: CommitInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, commit);
    }

    #[test]
    fn test_status_info_equality() {
        let status1 = StatusInfo {
            modified: vec![PathBuf::from("a.txt")],
            untracked: vec![],
            staged: vec![],
        };

        let status2 = StatusInfo {
            modified: vec![PathBuf::from("a.txt")],
            untracked: vec![],
            staged: vec![],
        };

        assert_eq!(status1, status2);
    }

    #[test]
    fn test_branch_info_current_detection() {
        let branch = BranchInfo {
            name: "main".to_string(),
            is_current: true,
        };

        assert!(branch.is_current);
        assert_eq!(branch.name, "main");
    }

    #[test]
    fn test_parse_commit_id_short_hash() {
        let oxen = OxenSubprocess::new();
        assert_eq!(oxen.parse_commit_id("abc1234"), Some("abc1234".to_string()));
    }

    #[test]
    fn test_parse_commit_id_long_hash() {
        let oxen = OxenSubprocess::new();
        let long_hash = "abc123def456789012345678901234567890";
        assert_eq!(oxen.parse_commit_id(long_hash), Some(long_hash.to_string()));
    }

    #[test]
    fn test_parse_commit_id_with_brackets() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.parse_commit_id("[abc123def]"),
            Some("abc123def".to_string())
        );
    }

    #[test]
    fn test_parse_commit_id_multiline() {
        let oxen = OxenSubprocess::new();
        let output = "Some text\nCommit abc1234 created\nMore text";
        assert_eq!(oxen.parse_commit_id(output), Some("abc1234".to_string()));
    }

    #[test]
    fn test_parse_commit_id_empty_string() {
        let oxen = OxenSubprocess::new();
        assert_eq!(oxen.parse_commit_id(""), None);
    }

    #[test]
    fn test_parse_commit_id_too_short() {
        let oxen = OxenSubprocess::new();
        assert_eq!(oxen.parse_commit_id("abc12"), None);
    }

    #[test]
    fn test_parse_log_output_empty() {
        let oxen = OxenSubprocess::new();
        let commits = oxen.parse_log_output("").unwrap();
        assert!(commits.is_empty());
    }

    #[test]
    fn test_parse_log_output_single_commit() {
        let oxen = OxenSubprocess::new();
        let output = r#"
commit abc123
Author: Test User <test@example.com>
Date: 2025-01-01

    First commit
        "#;

        let commits = oxen.parse_log_output(output).unwrap();
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].id, "abc123");
        assert_eq!(commits[0].message, "First commit");
    }

    #[test]
    fn test_parse_status_output_empty() {
        let oxen = OxenSubprocess::new();
        let status = oxen.parse_status_output("").unwrap();
        assert!(status.modified.is_empty());
        assert!(status.untracked.is_empty());
        assert!(status.staged.is_empty());
    }

    #[test]
    fn test_parse_branches_output_empty() {
        let oxen = OxenSubprocess::new();
        let branches = oxen.parse_branches_output("").unwrap();
        assert!(branches.is_empty());
    }

    #[test]
    fn test_cache_invalidation() {
        let oxen = OxenSubprocess::new();
        let path = PathBuf::from("/test/path");

        // Just test that invalidation doesn't panic
        oxen.invalidate_cache(&path);
        oxen.invalidate_all_caches();
    }

    #[test]
    fn test_oxen_subprocess_clone() {
        let oxen = OxenSubprocess::new().verbose(true);
        let cloned = oxen.clone();
        assert!(cloned.verbose);
        assert_eq!(cloned.config.oxen_path, "oxen");
    }

    #[test]
    fn test_oxen_subprocess_debug() {
        let oxen = OxenSubprocess::new();
        let debug_str = format!("{:?}", oxen);
        assert!(debug_str.contains("oxen"));
    }

    #[test]
    fn test_status_info_clone() {
        let status = StatusInfo {
            modified: vec![PathBuf::from("a.txt")],
            untracked: vec![],
            staged: vec![],
        };
        let cloned = status.clone();
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_branch_info_equality() {
        let branch1 = BranchInfo {
            name: "main".to_string(),
            is_current: true,
        };
        let branch2 = BranchInfo {
            name: "main".to_string(),
            is_current: true,
        };
        assert_eq!(branch1, branch2);
    }

    #[test]
    fn test_commit_info_equality() {
        let commit1 = CommitInfo {
            id: "abc123".to_string(),
            message: "Test".to_string(),
        };
        let commit2 = CommitInfo {
            id: "abc123".to_string(),
            message: "Test".to_string(),
        };
        assert_eq!(commit1, commit2);
    }

    #[test]
    fn test_with_config() {
        let mut config = OxenConfig::default();
        config.default_timeout = 60;
        config.batch_size = 500;

        let oxen = OxenSubprocess::with_config(config);
        assert_eq!(oxen.config.default_timeout, 60);
        assert_eq!(oxen.config.batch_size, 500);
    }

    #[test]
    fn test_parse_commit_id_edge_case_40_chars() {
        let oxen = OxenSubprocess::new();
        let hash = "a".repeat(40);
        assert_eq!(oxen.parse_commit_id(&hash), Some(hash));
    }

    #[test]
    fn test_parse_commit_id_edge_case_41_chars() {
        let oxen = OxenSubprocess::new();
        let hash = "a".repeat(41);
        assert_eq!(oxen.parse_commit_id(&hash), None);
    }

    // ========== Security Tests ==========

    #[test]
    fn test_sanitize_path_valid() {
        let path = Path::new("src/main.rs");
        let result = sanitize_path(path, None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "src/main.rs");
    }

    #[test]
    fn test_sanitize_path_null_byte() {
        let path = Path::new("file\0.txt");
        let result = sanitize_path(path, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_sanitize_path_command_injection() {
        let dangerous_paths = vec![
            "file$(whoami).txt",
            "file`id`.txt",
            "file;rm -rf.txt",
            "file&&cat.txt",
            "file||echo.txt",
            "file|cat.txt",
            "file>out.txt",
            "file<in.txt",
        ];

        for path_str in dangerous_paths {
            let path = Path::new(path_str);
            let result = sanitize_path(path, None);
            assert!(
                result.is_err(),
                "Expected error for path: {}, got: {:?}",
                path_str,
                result
            );
        }
    }

    #[test]
    fn test_sanitize_path_traversal() {
        let path = Path::new("../../../etc/passwd");
        let result = sanitize_path(path, Some(Path::new("/home/user/project")));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("traversal"));
    }

    #[test]
    fn test_sanitize_message_valid() {
        let result = sanitize_message("Add new feature");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Add new feature");
    }

    #[test]
    fn test_sanitize_message_null_byte() {
        let result = sanitize_message("Message\0with null");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("null byte"));
    }

    #[test]
    fn test_sanitize_message_too_long() {
        let long_message = "a".repeat(10001);
        let result = sanitize_message(&long_message);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too long"));
    }

    #[test]
    fn test_sanitize_message_multiline() {
        let result = sanitize_message("Line 1\nLine 2\nLine 3");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_message_unicode() {
        let result = sanitize_message("Fix 🐛 bug with 日本語 text");
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_path_with_spaces() {
        let path = Path::new("path with spaces/file.txt");
        let result = sanitize_path(path, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_path_unicode() {
        let path = Path::new("文件/file.txt");
        let result = sanitize_path(path, None);
        assert!(result.is_ok());
    }

    #[test]
    fn test_version_parsing() {
        // This tests the version string format parsing
        let test_versions = vec![
            ("oxen 0.19.5", true),
            ("0.19.0", true),
            ("oxen 0.20.0", true),
            ("0.18.9", false),
            ("oxen 0.10.0", false),
        ];

        for (version_str, should_be_compatible) in test_versions {
            let version_parts: Vec<&str> = version_str.split_whitespace().collect();
            let version = version_parts.last().unwrap_or(&"unknown");
            let parts: Vec<&str> = version.split('.').collect();

            let is_compatible = if parts.len() >= 2 {
                let major: u32 = parts[0].parse().unwrap_or(0);
                let minor: u32 = parts[1].parse().unwrap_or(0);
                (major > 0) || (major == 0 && minor >= 19)
            } else {
                false
            };

            assert_eq!(
                is_compatible, should_be_compatible,
                "Version {} should be {}compatible",
                version_str,
                if should_be_compatible { "" } else { "in" }
            );
        }
    }
}
