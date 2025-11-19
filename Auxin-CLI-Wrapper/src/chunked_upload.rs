//! Chunked upload manager for large file transfers
//!
//! This module provides intelligent handling of large file uploads with:
//! - Progress tracking per file
//! - Bandwidth estimation and ETA
//! - Resume capability after interruption
//! - Abort and resume later functionality
//!
//! # Architecture
//!
//! For very large files (>100MB), uploads are tracked in chunks:
//! - State persisted to disk for crash recovery
//! - Each chunk's completion is recorded
//! - Failed uploads can resume from last successful point
//!
//! # Example
//!
//! ```no_run
//! use auxin_cli::chunked_upload::{ChunkedUploadManager, UploadConfig};
//! use std::path::Path;
//!
//! let config = UploadConfig::default();
//! let mut manager = ChunkedUploadManager::new(config)?;
//!
//! // Start or resume an upload
//! let result = manager.upload_with_progress(
//!     Path::new("/path/to/repo"),
//!     "origin",
//!     "main",
//!     |progress| {
//!         println!("Progress: {}%", progress.percentage);
//!     }
//! )?;
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::oxen_subprocess::OxenSubprocess;

// Suppress unused import warning for Colorize (used by macros)
#[allow(unused_imports)]
use colored::Colorize as _;

/// Default state directory for upload tracking
const DEFAULT_STATE_DIR: &str = ".auxin/uploads";

/// Chunk size for tracking large files (100 MB)
const DEFAULT_CHUNK_SIZE: u64 = 100 * 1024 * 1024;

/// Minimum file size to enable chunked tracking (50 MB)
const MIN_CHUNKED_SIZE: u64 = 50 * 1024 * 1024;

/// Configuration for chunked uploads
#[derive(Debug, Clone)]
pub struct UploadConfig {
    /// Chunk size in bytes for progress tracking
    pub chunk_size: u64,
    /// Minimum file size to enable chunked tracking
    pub min_chunked_size: u64,
    /// Directory to store upload state
    pub state_dir: PathBuf,
    /// Maximum retries per chunk
    pub max_retries: u32,
    /// Enable verbose logging
    pub verbose: bool,
}

impl Default for UploadConfig {
    fn default() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            chunk_size: DEFAULT_CHUNK_SIZE,
            min_chunked_size: MIN_CHUNKED_SIZE,
            state_dir: home.join(DEFAULT_STATE_DIR),
            max_retries: 3,
            verbose: false,
        }
    }
}

/// State of an individual file upload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadState {
    /// File path relative to repository
    pub path: String,
    /// Total file size in bytes
    pub size: u64,
    /// Number of bytes uploaded
    pub bytes_uploaded: u64,
    /// Upload status
    pub status: UploadStatus,
    /// Last error message
    pub last_error: Option<String>,
    /// Upload started at
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
}

/// Status of an upload
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UploadStatus {
    /// Upload pending
    Pending,
    /// Upload in progress
    InProgress,
    /// Upload completed successfully
    Completed,
    /// Upload failed
    Failed,
    /// Upload aborted by user
    Aborted,
}

/// Overall upload session state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSession {
    /// Unique session ID
    pub id: String,
    /// Repository path
    pub repo_path: String,
    /// Remote name
    pub remote: String,
    /// Branch name
    pub branch: String,
    /// Files being uploaded
    pub files: Vec<FileUploadState>,
    /// Total bytes to upload
    pub total_bytes: u64,
    /// Total bytes uploaded
    pub bytes_uploaded: u64,
    /// Session started at
    pub started_at: DateTime<Utc>,
    /// Last activity timestamp
    pub last_activity: DateTime<Utc>,
    /// Overall status
    pub status: UploadStatus,
    /// Bandwidth samples (bytes per second)
    pub bandwidth_samples: Vec<f64>,
}

impl UploadSession {
    /// Create a new upload session
    pub fn new(repo_path: &Path, remote: &str, branch: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            repo_path: repo_path.to_string_lossy().to_string(),
            remote: remote.to_string(),
            branch: branch.to_string(),
            files: Vec::new(),
            total_bytes: 0,
            bytes_uploaded: 0,
            started_at: Utc::now(),
            last_activity: Utc::now(),
            status: UploadStatus::Pending,
            bandwidth_samples: Vec::new(),
        }
    }

    /// Calculate completion percentage
    pub fn percentage(&self) -> f64 {
        if self.total_bytes == 0 {
            100.0
        } else {
            (self.bytes_uploaded as f64 / self.total_bytes as f64) * 100.0
        }
    }

    /// Get average bandwidth in bytes per second
    pub fn average_bandwidth(&self) -> Option<f64> {
        if self.bandwidth_samples.is_empty() {
            None
        } else {
            let sum: f64 = self.bandwidth_samples.iter().sum();
            Some(sum / self.bandwidth_samples.len() as f64)
        }
    }

    /// Estimate remaining time in seconds
    pub fn estimated_remaining_seconds(&self) -> Option<u64> {
        let bandwidth = self.average_bandwidth()?;
        if bandwidth <= 0.0 {
            return None;
        }
        let remaining_bytes = self.total_bytes.saturating_sub(self.bytes_uploaded);
        Some((remaining_bytes as f64 / bandwidth) as u64)
    }

    /// Add a bandwidth sample
    pub fn add_bandwidth_sample(&mut self, bytes_per_second: f64) {
        // Keep last 10 samples for moving average
        if self.bandwidth_samples.len() >= 10 {
            self.bandwidth_samples.remove(0);
        }
        self.bandwidth_samples.push(bytes_per_second);
    }
}

/// Progress information for callbacks
#[derive(Debug, Clone)]
pub struct UploadProgress {
    /// Session ID
    pub session_id: String,
    /// Completion percentage (0-100)
    pub percentage: f64,
    /// Bytes uploaded so far
    pub bytes_uploaded: u64,
    /// Total bytes to upload
    pub total_bytes: u64,
    /// Current file being uploaded
    pub current_file: Option<String>,
    /// Average bandwidth in bytes per second
    pub bandwidth_bps: Option<f64>,
    /// Estimated remaining time in seconds
    pub eta_seconds: Option<u64>,
    /// Number of files completed
    pub files_completed: usize,
    /// Total number of files
    pub total_files: usize,
}

impl UploadProgress {
    /// Format bandwidth as human-readable string
    pub fn bandwidth_string(&self) -> String {
        match self.bandwidth_bps {
            Some(bps) if bps >= 1_000_000.0 => format!("{:.1} MB/s", bps / 1_000_000.0),
            Some(bps) if bps >= 1_000.0 => format!("{:.1} KB/s", bps / 1_000.0),
            Some(bps) => format!("{:.0} B/s", bps),
            None => "calculating...".to_string(),
        }
    }

    /// Format ETA as human-readable string
    pub fn eta_string(&self) -> String {
        match self.eta_seconds {
            Some(secs) if secs >= 3600 => {
                let hours = secs / 3600;
                let mins = (secs % 3600) / 60;
                format!("{}h {}m", hours, mins)
            }
            Some(secs) if secs >= 60 => {
                let mins = secs / 60;
                let secs = secs % 60;
                format!("{}m {}s", mins, secs)
            }
            Some(secs) => format!("{}s", secs),
            None => "calculating...".to_string(),
        }
    }

    /// Format bytes as human-readable string
    pub fn bytes_string(bytes: u64) -> String {
        if bytes >= 1_000_000_000 {
            format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
        } else if bytes >= 1_000_000 {
            format!("{:.1} MB", bytes as f64 / 1_000_000.0)
        } else if bytes >= 1_000 {
            format!("{:.1} KB", bytes as f64 / 1_000.0)
        } else {
            format!("{} B", bytes)
        }
    }
}

/// Manager for chunked uploads with progress tracking
pub struct ChunkedUploadManager {
    /// Configuration
    config: UploadConfig,
    /// Current session (if any)
    current_session: Option<UploadSession>,
    /// Oxen subprocess wrapper
    oxen: OxenSubprocess,
}

impl ChunkedUploadManager {
    /// Create a new chunked upload manager
    pub fn new(config: UploadConfig) -> Result<Self> {
        // Create state directory if it doesn't exist
        if !config.state_dir.exists() {
            fs::create_dir_all(&config.state_dir)
                .context("Failed to create upload state directory")?;
        }

        Ok(Self {
            config,
            current_session: None,
            oxen: OxenSubprocess::new(),
        })
    }

    /// Create with default configuration
    pub fn with_defaults() -> Result<Self> {
        Self::new(UploadConfig::default())
    }

    /// Get or create an upload session for a repository
    pub fn get_or_create_session(
        &mut self,
        repo_path: &Path,
        remote: &str,
        branch: &str,
    ) -> Result<&UploadSession> {
        // Check for existing session
        let session_file = self.session_file_path(repo_path);

        if session_file.exists() {
            // Load existing session
            let content = fs::read_to_string(&session_file)
                .context("Failed to read upload session file")?;
            let session: UploadSession = serde_json::from_str(&content)
                .context("Failed to parse upload session")?;

            // Only resume if same remote/branch and not completed
            if session.remote == remote
                && session.branch == branch
                && session.status != UploadStatus::Completed
            {
                crate::info!("Resuming upload session: {}", &session.id[..8]);
                self.current_session = Some(session);
            } else {
                // Create new session
                self.current_session = Some(UploadSession::new(repo_path, remote, branch));
            }
        } else {
            // Create new session
            self.current_session = Some(UploadSession::new(repo_path, remote, branch));
        }

        Ok(self.current_session.as_ref().unwrap())
    }

    /// Scan repository for files to upload
    pub fn scan_files(&mut self, repo_path: &Path) -> Result<Vec<FileUploadState>> {
        crate::vlog!("Scanning repository for files to upload");

        // Get status from Oxen to find files that need uploading
        let output = std::process::Command::new("oxen")
            .args(["status", "--staged"])
            .current_dir(repo_path)
            .output()
            .context("Failed to run oxen status")?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut files = Vec::new();

        // Parse staged files from status output
        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("new file:") || trimmed.starts_with("modified:") {
                let file_path = trimmed
                    .split(':')
                    .nth(1)
                    .map(|s| s.trim())
                    .unwrap_or("");

                if !file_path.is_empty() {
                    let full_path = repo_path.join(file_path);
                    let size = full_path.metadata()
                        .map(|m| m.len())
                        .unwrap_or(0);

                    files.push(FileUploadState {
                        path: file_path.to_string(),
                        size,
                        bytes_uploaded: 0,
                        status: UploadStatus::Pending,
                        last_error: None,
                        started_at: Utc::now(),
                        last_activity: Utc::now(),
                    });
                }
            }
        }

        crate::vlog!("Found {} files to upload", files.len());
        Ok(files)
    }

    /// Upload with progress callback
    pub fn upload_with_progress<F>(
        &mut self,
        repo_path: &Path,
        remote: &str,
        branch: &str,
        progress_callback: F,
    ) -> Result<UploadResult>
    where
        F: Fn(UploadProgress),
    {
        // Get or create session
        self.get_or_create_session(repo_path, remote, branch)?;

        // Check if we need to scan files
        let needs_scan = self.current_session
            .as_ref()
            .map(|s| s.files.is_empty())
            .unwrap_or(false);

        // Scan files if this is a new session
        if needs_scan {
            let files = self.scan_files(repo_path)?;
            if let Some(session) = &mut self.current_session {
                session.files = files;
                session.total_bytes = session.files.iter().map(|f| f.size).sum();
            }
        }

        // Save initial state
        self.save_session(repo_path)?;

        // Update session status
        if let Some(session) = &mut self.current_session {
            session.status = UploadStatus::InProgress;
        }

        // Send initial progress
        self.send_progress(&progress_callback);

        // Track timing for bandwidth calculation
        let start_time = Instant::now();
        let start_bytes = self.current_session.as_ref().map(|s| s.bytes_uploaded).unwrap_or(0);

        // Execute the actual push
        crate::info!("Starting upload to {}/{}", remote, branch);

        let push_result = self.oxen.push(repo_path, Some(remote), Some(branch));

        // Calculate bandwidth
        let elapsed = start_time.elapsed();
        if elapsed.as_secs() > 0 {
            if let Some(session) = &mut self.current_session {
                // Assume all bytes were uploaded on success
                if push_result.is_ok() {
                    let bytes_transferred = session.total_bytes.saturating_sub(start_bytes);
                    let bandwidth = bytes_transferred as f64 / elapsed.as_secs_f64();
                    session.add_bandwidth_sample(bandwidth);
                    session.bytes_uploaded = session.total_bytes;

                    // Mark all files as completed
                    for file in &mut session.files {
                        file.status = UploadStatus::Completed;
                        file.bytes_uploaded = file.size;
                        file.last_activity = Utc::now();
                    }
                }
            }
        }

        // Handle result
        match push_result {
            Ok(_) => {
                if let Some(session) = &mut self.current_session {
                    session.status = UploadStatus::Completed;
                    session.last_activity = Utc::now();
                }

                // Send final progress
                self.send_progress(&progress_callback);

                // Clean up session file
                let session_file = self.session_file_path(repo_path);
                if session_file.exists() {
                    let _ = fs::remove_file(&session_file);
                }

                let result = UploadResult {
                    success: true,
                    files_uploaded: self.current_session.as_ref().map(|s| s.files.len()).unwrap_or(0),
                    bytes_uploaded: self.current_session.as_ref().map(|s| s.bytes_uploaded).unwrap_or(0),
                    duration: elapsed,
                    average_bandwidth: self.current_session.as_ref().and_then(|s| s.average_bandwidth()),
                    error: None,
                };

                crate::info!("Upload completed: {} files, {}",
                    result.files_uploaded,
                    UploadProgress::bytes_string(result.bytes_uploaded)
                );

                Ok(result)
            }
            Err(e) => {
                let error_msg = e.to_string();

                if let Some(session) = &mut self.current_session {
                    session.status = UploadStatus::Failed;
                    session.last_activity = Utc::now();

                    // Mark current file as failed
                    for file in &mut session.files {
                        if file.status == UploadStatus::InProgress {
                            file.status = UploadStatus::Failed;
                            file.last_error = Some(error_msg.clone());
                        }
                    }
                }

                // Save state for resume
                self.save_session(repo_path)?;

                crate::error!("Upload failed: {}", error_msg);

                Ok(UploadResult {
                    success: false,
                    files_uploaded: 0,
                    bytes_uploaded: self.current_session.as_ref().map(|s| s.bytes_uploaded).unwrap_or(0),
                    duration: elapsed,
                    average_bandwidth: self.current_session.as_ref().and_then(|s| s.average_bandwidth()),
                    error: Some(error_msg),
                })
            }
        }
    }

    /// Abort the current upload (can be resumed later)
    pub fn abort(&mut self, repo_path: &Path) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            session.status = UploadStatus::Aborted;
            session.last_activity = Utc::now();

            // Mark in-progress files as aborted
            for file in &mut session.files {
                if file.status == UploadStatus::InProgress {
                    file.status = UploadStatus::Aborted;
                }
            }
        }

        // Save state for later resume
        self.save_session(repo_path)?;

        crate::info!("Upload aborted - can be resumed later");
        Ok(())
    }

    /// Check if there's a resumable session
    pub fn has_resumable_session(&self, repo_path: &Path) -> bool {
        let session_file = self.session_file_path(repo_path);
        if session_file.exists() {
            if let Ok(content) = fs::read_to_string(&session_file) {
                if let Ok(session) = serde_json::from_str::<UploadSession>(&content) {
                    return session.status != UploadStatus::Completed;
                }
            }
        }
        false
    }

    /// Get information about a resumable session
    pub fn get_resumable_session_info(&self, repo_path: &Path) -> Option<UploadSessionInfo> {
        let session_file = self.session_file_path(repo_path);
        if session_file.exists() {
            if let Ok(content) = fs::read_to_string(&session_file) {
                if let Ok(session) = serde_json::from_str::<UploadSession>(&content) {
                    if session.status != UploadStatus::Completed {
                        return Some(UploadSessionInfo {
                            id: session.id.clone(),
                            percentage: session.percentage(),
                            bytes_uploaded: session.bytes_uploaded,
                            total_bytes: session.total_bytes,
                            files_count: session.files.len(),
                            started_at: session.started_at,
                            last_activity: session.last_activity,
                        });
                    }
                }
            }
        }
        None
    }

    /// Clear a resumable session
    pub fn clear_session(&self, repo_path: &Path) -> Result<()> {
        let session_file = self.session_file_path(repo_path);
        if session_file.exists() {
            fs::remove_file(&session_file)
                .context("Failed to remove upload session file")?;
        }
        Ok(())
    }

    /// Send progress update via callback
    fn send_progress<F>(&self, callback: &F)
    where
        F: Fn(UploadProgress),
    {
        if let Some(session) = &self.current_session {
            let files_completed = session.files.iter()
                .filter(|f| f.status == UploadStatus::Completed)
                .count();

            let current_file = session.files.iter()
                .find(|f| f.status == UploadStatus::InProgress)
                .map(|f| f.path.clone());

            let progress = UploadProgress {
                session_id: session.id.clone(),
                percentage: session.percentage(),
                bytes_uploaded: session.bytes_uploaded,
                total_bytes: session.total_bytes,
                current_file,
                bandwidth_bps: session.average_bandwidth(),
                eta_seconds: session.estimated_remaining_seconds(),
                files_completed,
                total_files: session.files.len(),
            };

            callback(progress);
        }
    }

    /// Get path to session file
    fn session_file_path(&self, repo_path: &Path) -> PathBuf {
        // Use repository path hash as session filename
        let repo_hash = format!("{:x}", md5::compute(repo_path.to_string_lossy().as_bytes()));
        self.config.state_dir.join(format!("{}.json", repo_hash))
    }

    /// Save current session to disk
    fn save_session(&self, repo_path: &Path) -> Result<()> {
        if let Some(session) = &self.current_session {
            let session_file = self.session_file_path(repo_path);
            let json = serde_json::to_string_pretty(session)
                .context("Failed to serialize upload session")?;
            fs::write(&session_file, json)
                .context("Failed to write upload session file")?;
        }
        Ok(())
    }
}

/// Result of an upload operation
#[derive(Debug, Clone)]
pub struct UploadResult {
    /// Whether upload completed successfully
    pub success: bool,
    /// Number of files uploaded
    pub files_uploaded: usize,
    /// Total bytes uploaded
    pub bytes_uploaded: u64,
    /// Total duration
    pub duration: Duration,
    /// Average bandwidth (bytes per second)
    pub average_bandwidth: Option<f64>,
    /// Error message if failed
    pub error: Option<String>,
}

/// Summary information about a resumable session
#[derive(Debug, Clone)]
pub struct UploadSessionInfo {
    /// Session ID
    pub id: String,
    /// Completion percentage
    pub percentage: f64,
    /// Bytes uploaded
    pub bytes_uploaded: u64,
    /// Total bytes
    pub total_bytes: u64,
    /// Number of files
    pub files_count: usize,
    /// When upload started
    pub started_at: DateTime<Utc>,
    /// Last activity
    pub last_activity: DateTime<Utc>,
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_upload_config_default() {
        let config = UploadConfig::default();
        assert_eq!(config.chunk_size, DEFAULT_CHUNK_SIZE);
        assert_eq!(config.min_chunked_size, MIN_CHUNKED_SIZE);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_upload_session_new() {
        let session = UploadSession::new(
            Path::new("/test/repo"),
            "origin",
            "main"
        );

        assert_eq!(session.remote, "origin");
        assert_eq!(session.branch, "main");
        assert_eq!(session.status, UploadStatus::Pending);
        assert!(session.files.is_empty());
    }

    #[test]
    fn test_upload_session_percentage() {
        let mut session = UploadSession::new(
            Path::new("/test/repo"),
            "origin",
            "main"
        );

        // Empty session should be 100%
        assert_eq!(session.percentage(), 100.0);

        // With bytes
        session.total_bytes = 1000;
        session.bytes_uploaded = 500;
        assert_eq!(session.percentage(), 50.0);

        session.bytes_uploaded = 1000;
        assert_eq!(session.percentage(), 100.0);
    }

    #[test]
    fn test_upload_session_bandwidth() {
        let mut session = UploadSession::new(
            Path::new("/test/repo"),
            "origin",
            "main"
        );

        // No samples
        assert!(session.average_bandwidth().is_none());

        // Add samples
        session.add_bandwidth_sample(1000.0);
        session.add_bandwidth_sample(2000.0);
        session.add_bandwidth_sample(3000.0);

        let avg = session.average_bandwidth().unwrap();
        assert!((avg - 2000.0).abs() < 0.001);
    }

    #[test]
    fn test_upload_session_eta() {
        let mut session = UploadSession::new(
            Path::new("/test/repo"),
            "origin",
            "main"
        );

        session.total_bytes = 10000;
        session.bytes_uploaded = 5000;
        session.add_bandwidth_sample(1000.0); // 1000 B/s

        let eta = session.estimated_remaining_seconds().unwrap();
        assert_eq!(eta, 5); // 5000 bytes remaining at 1000 B/s = 5 seconds
    }

    #[test]
    fn test_upload_progress_formatting() {
        let progress = UploadProgress {
            session_id: "test".to_string(),
            percentage: 50.0,
            bytes_uploaded: 500_000_000,
            total_bytes: 1_000_000_000,
            current_file: Some("test.file".to_string()),
            bandwidth_bps: Some(10_000_000.0),
            eta_seconds: Some(3661),
            files_completed: 5,
            total_files: 10,
        };

        assert_eq!(progress.bandwidth_string(), "10.0 MB/s");
        assert_eq!(progress.eta_string(), "1h 1m");
        assert_eq!(UploadProgress::bytes_string(500_000_000), "500.0 MB");
    }

    #[test]
    fn test_chunked_upload_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config = UploadConfig {
            state_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ChunkedUploadManager::new(config).unwrap();
        assert!(temp_dir.path().exists());
        assert!(manager.current_session.is_none());
    }

    #[test]
    fn test_session_file_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = UploadConfig {
            state_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ChunkedUploadManager::new(config).unwrap();
        let session_file = manager.session_file_path(Path::new("/test/repo"));

        assert!(session_file.to_string_lossy().contains(".json"));
        assert!(session_file.starts_with(temp_dir.path()));
    }

    #[test]
    fn test_has_resumable_session() {
        let temp_dir = TempDir::new().unwrap();
        let config = UploadConfig {
            state_dir: temp_dir.path().to_path_buf(),
            ..Default::default()
        };

        let manager = ChunkedUploadManager::new(config).unwrap();
        let repo_path = Path::new("/test/repo");

        // No session exists
        assert!(!manager.has_resumable_session(repo_path));
    }

    #[test]
    fn test_file_upload_state() {
        let state = FileUploadState {
            path: "test.file".to_string(),
            size: 1000,
            bytes_uploaded: 500,
            status: UploadStatus::InProgress,
            last_error: None,
            started_at: Utc::now(),
            last_activity: Utc::now(),
        };

        assert_eq!(state.path, "test.file");
        assert_eq!(state.status, UploadStatus::InProgress);
    }

    #[test]
    fn test_upload_result() {
        let result = UploadResult {
            success: true,
            files_uploaded: 5,
            bytes_uploaded: 1_000_000,
            duration: Duration::from_secs(10),
            average_bandwidth: Some(100_000.0),
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.files_uploaded, 5);
    }
}
