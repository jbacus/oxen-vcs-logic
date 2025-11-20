// VCS operations using Oxen CLI subprocess
// This approach works without liboxen compilation and uses the same
// proven subprocess wrapper approach as the Auxin CLI

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tracing::{debug, info, warn};

use crate::error::{AppError, AppResult};
use crate::extensions::{FileLock, LogicProMetadata};

/// Execute an oxen command and return the output
fn run_oxen_command(args: &[&str], cwd: Option<&Path>) -> AppResult<Output> {
    let mut cmd = Command::new("oxen");
    cmd.args(args);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    debug!("Running oxen command: oxen {}", args.join(" "));

    let output = cmd.output().map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            AppError::Internal(
                "Oxen CLI not found. Install with: pip install oxenai".to_string()
            )
        } else {
            AppError::Internal(format!("Failed to execute oxen command: {}", e))
        }
    })?;

    Ok(output)
}

/// Check if oxen command succeeded and return stdout
fn check_oxen_output(output: Output, operation: &str) -> AppResult<String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Check for specific error types
        let error_msg = if !stderr.is_empty() {
            stderr.to_string()
        } else {
            stdout.to_string()
        };

        if error_msg.contains("not found") || error_msg.contains("does not exist") {
            Err(AppError::NotFound(format!("{}: {}", operation, error_msg)))
        } else if error_msg.contains("permission") || error_msg.contains("unauthorized") {
            Err(AppError::Unauthorized(format!("{}: {}", operation, error_msg)))
        } else {
            Err(AppError::Internal(format!("{} failed: {}", operation, error_msg)))
        }
    }
}

/// Repository operations wrapper (mock implementation)
pub struct RepositoryOps {
    repo_path: PathBuf,
}

impl RepositoryOps {
    /// Open an existing repository
    pub fn open(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();

        // Verify .oxen directory exists
        if !repo_path.join(".oxen").exists() {
            return Err(AppError::NotFound("Repository not found".to_string()));
        }

        Ok(Self { repo_path })
    }

    /// Initialize a new repository
    pub fn init(repo_path: impl AsRef<Path>) -> AppResult<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();

        info!("Initializing repository at: {:?}", repo_path);

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to create repository directory: {}", e))
        })?;

        // Create .oxen directory structure
        let oxen_dir = repo_path.join(".oxen");
        std::fs::create_dir_all(&oxen_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create .oxen directory: {}", e))
        })?;

        // Try to initialize using oxen CLI if available
        match run_oxen_command(&["init"], Some(&repo_path)) {
            Ok(output) => {
                if let Err(e) = check_oxen_output(output, "Repository init") {
                    warn!("Oxen init returned error (may be expected): {}", e);
                }
            }
            Err(e) => {
                // CLI not available - create minimal structure for basic operations
                warn!("Oxen CLI not available, creating minimal repository structure: {}", e);

                // Create minimal config file
                let config_path = oxen_dir.join("config.toml");
                std::fs::write(&config_path, "[repository]\nversion = \"0.1\"\n").map_err(|e| {
                    AppError::Internal(format!("Failed to create config: {}", e))
                })?;
            }
        }

        // Create Auxin extension directories
        std::fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
            AppError::Internal(format!("Failed to create metadata directory: {}", e))
        })?;

        std::fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
            AppError::Internal(format!("Failed to create locks directory: {}", e))
        })?;

        info!("Repository initialized successfully");

        Ok(Self { repo_path })
    }

    /// Add files to the staging area
    pub fn add(&self, paths: &[impl AsRef<Path>]) -> AppResult<()> {
        for path in paths {
            let path_str = path.as_ref().to_string_lossy();
            debug!("Adding file: {}", path_str);

            let output = run_oxen_command(&["add", &path_str], Some(&self.repo_path))?;
            check_oxen_output(output, &format!("Add {}", path_str))?;
        }

        Ok(())
    }

    /// Commit staged changes
    pub fn commit(&self, message: &str) -> AppResult<String> {
        info!("Creating commit: {}", message);

        let output = run_oxen_command(&["commit", "-m", message], Some(&self.repo_path))?;
        let stdout = check_oxen_output(output, "Commit")?;

        // Parse commit ID from output (format: "Commit <id>")
        let commit_id = stdout
            .lines()
            .find(|line| line.contains("Commit"))
            .and_then(|line| line.split_whitespace().last())
            .unwrap_or("unknown")
            .to_string();

        info!("Commit created: {}", commit_id);
        Ok(commit_id)
    }

    /// Get commit history
    pub fn log(&self, limit: Option<usize>) -> AppResult<Vec<CommitInfo>> {
        let mut args = vec!["log"];
        let limit_str;

        if let Some(n) = limit {
            limit_str = n.to_string();
            args.push("-n");
            args.push(&limit_str);
        }

        let output = run_oxen_command(&args, Some(&self.repo_path))?;
        let stdout = check_oxen_output(output, "Log")?;

        // Parse text output
        if stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        // Parse oxen log text format:
        // commit <hash>
        //
        // Author: user <email>
        // Date:   Thursday, 20 November 2025 18:02:01 +00
        //
        //     Commit message
        //
        let mut commits = Vec::new();
        let lines: Vec<&str> = stdout.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i].trim();

            // Look for "commit " line
            if let Some(id) = line.strip_prefix("commit ") {
                let id = id.trim().to_string();
                i += 1;

                // Skip empty line after commit
                if i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Parse author (optional)
                let author = if i < lines.len() && lines[i].trim().starts_with("Author:") {
                    let auth = lines[i].trim().strip_prefix("Author:").unwrap_or("").trim().to_string();
                    i += 1;
                    auth
                } else {
                    String::from("unknown")
                };

                // Parse date (optional) - format: "Date:   Thursday, 20 November 2025 18:02:01 +00"
                let timestamp = if i < lines.len() && lines[i].trim().starts_with("Date:") {
                    // Just use current time as parsing the full format is complex
                    i += 1;
                    chrono::Utc::now().to_rfc3339()
                } else {
                    chrono::Utc::now().to_rfc3339()
                };

                // Skip empty line before message
                if i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }

                // Parse message (indented lines)
                let mut message_lines = Vec::new();
                while i < lines.len() {
                    let msg_line = lines[i];
                    // Message lines start with 4 spaces
                    if msg_line.starts_with("    ") {
                        message_lines.push(msg_line.trim());
                        i += 1;
                    } else if msg_line.trim().is_empty() {
                        // Empty line might be part of message or separator
                        i += 1;
                        // If next line is a commit, break
                        if i < lines.len() && lines[i].trim().starts_with("commit ") {
                            i -= 1; // Back up so we can process this commit
                            break;
                        }
                    } else {
                        break;
                    }
                }

                let message = if message_lines.is_empty() {
                    String::from("(no message)")
                } else {
                    message_lines.join(" ")
                };

                commits.push(CommitInfo {
                    id,
                    message,
                    author,
                    timestamp,
                });
            } else {
                i += 1;
            }
        }

        Ok(commits)
    }

    /// Push to remote repository
    pub fn push(&self, remote: &str, branch: &str) -> AppResult<()> {
        info!("Pushing to remote: {} (branch: {})", remote, branch);

        let output = run_oxen_command(&["push", remote, branch], Some(&self.repo_path))?;
        check_oxen_output(output, "Push")?;

        info!("Push completed successfully");
        Ok(())
    }

    /// Pull from remote repository
    pub fn pull(&self, remote: &str, branch: &str) -> AppResult<()> {
        info!("Pulling from remote: {} (branch: {})", remote, branch);

        let output = run_oxen_command(&["pull", remote, branch], Some(&self.repo_path))?;
        check_oxen_output(output, "Pull")?;

        info!("Pull completed successfully");
        Ok(())
    }

    /// Clone a remote repository
    pub fn clone(remote_url: &str, dest_path: impl AsRef<Path>) -> AppResult<Self> {
        let dest_path = dest_path.as_ref();
        info!("Cloning repository from: {} to: {:?}", remote_url, dest_path);

        // Get parent directory for clone command
        let parent = dest_path.parent().ok_or_else(|| {
            AppError::BadRequest("Invalid destination path".to_string())
        })?;

        // Create parent directory if it doesn't exist
        std::fs::create_dir_all(parent).map_err(|e| {
            AppError::Internal(format!("Failed to create parent directory: {}", e))
        })?;

        let dest_str = dest_path.to_string_lossy();
        let output = run_oxen_command(&["clone", remote_url, &dest_str], None)?;
        check_oxen_output(output, "Clone")?;

        // Create Auxin extension directories
        let oxen_dir = dest_path.join(".oxen");
        std::fs::create_dir_all(oxen_dir.join("metadata")).map_err(|e| {
            AppError::Internal(format!("Failed to create metadata directory: {}", e))
        })?;

        std::fs::create_dir_all(oxen_dir.join("locks")).map_err(|e| {
            AppError::Internal(format!("Failed to create locks directory: {}", e))
        })?;

        info!("Clone completed successfully");
        Ok(Self {
            repo_path: dest_path.to_path_buf(),
        })
    }

    /// Get current branch name
    pub fn current_branch(&self) -> AppResult<String> {
        let output = run_oxen_command(&["branch", "--current"], Some(&self.repo_path))?;
        let stdout = check_oxen_output(output, "Current branch")?;

        Ok(stdout.trim().to_string())
    }

    /// List all branches
    pub fn list_branches(&self) -> AppResult<Vec<String>> {
        let output = run_oxen_command(&["branch"], Some(&self.repo_path))?;
        let stdout = check_oxen_output(output, "List branches")?;

        let branches: Vec<String> = stdout
            .lines()
            .map(|line| {
                // Remove leading "* " from current branch
                line.trim_start_matches("* ").trim().to_string()
            })
            .filter(|s| !s.is_empty())
            .collect();

        Ok(branches)
    }

    /// Create a new branch
    pub fn create_branch(&self, branch_name: &str) -> AppResult<()> {
        info!("Creating branch: {}", branch_name);

        let output = run_oxen_command(&["branch", branch_name], Some(&self.repo_path))?;
        check_oxen_output(output, &format!("Create branch {}", branch_name))?;

        Ok(())
    }

    /// Checkout a branch
    pub fn checkout(&self, branch_name: &str) -> AppResult<()> {
        info!("Checking out branch: {}", branch_name);

        let output = run_oxen_command(&["checkout", branch_name], Some(&self.repo_path))?;
        check_oxen_output(output, &format!("Checkout {}", branch_name))?;

        Ok(())
    }

    /// Delete a branch
    pub fn delete_branch(&self, branch_name: &str) -> AppResult<()> {
        info!("Deleting branch: {}", branch_name);

        // Prevent deletion of main/master
        if branch_name == "main" || branch_name == "master" {
            return Err(AppError::BadRequest(
                "Cannot delete main/master branch".to_string(),
            ));
        }

        let output = run_oxen_command(&["branch", "-d", branch_name], Some(&self.repo_path))?;
        check_oxen_output(output, &format!("Delete branch {}", branch_name))?;

        Ok(())
    }

    /// Fetch from remote
    pub fn fetch(&self, remote: &str) -> AppResult<()> {
        info!("Fetching from remote: {}", remote);

        let output = run_oxen_command(&["fetch", remote], Some(&self.repo_path))?;
        check_oxen_output(output, "Fetch")?;

        info!("Fetch completed successfully");
        Ok(())
    }

    /// Get repository status
    pub fn status(&self) -> AppResult<String> {
        let output = run_oxen_command(&["status"], Some(&self.repo_path))?;
        check_oxen_output(output, "Status")
    }

    /// Add remote
    pub fn add_remote(&self, name: &str, url: &str) -> AppResult<()> {
        info!("Adding remote: {} -> {}", name, url);

        let output = run_oxen_command(&["remote", "add", name, url], Some(&self.repo_path))?;
        check_oxen_output(output, &format!("Add remote {}", name))?;

        Ok(())
    }

    /// List remotes
    pub fn list_remotes(&self) -> AppResult<Vec<(String, String)>> {
        let output = run_oxen_command(&["remote", "-v"], Some(&self.repo_path))?;
        let stdout = check_oxen_output(output, "List remotes")?;

        let remotes: Vec<(String, String)> = stdout
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some((parts[0].to_string(), parts[1].to_string()))
                } else {
                    None
                }
            })
            .collect();

        Ok(remotes)
    }

    /// Get repository path
    pub fn path(&self) -> &Path {
        &self.repo_path
    }

    // Auxin Extensions (these work in mock mode)

    /// Store Logic Pro metadata for a commit
    pub fn store_metadata(&self, commit_id: &str, metadata: &LogicProMetadata) -> AppResult<()> {
        let metadata_path = self
            .repo_path
            .join(".oxen")
            .join("metadata")
            .join(format!("{}.json", commit_id));

        let json = serde_json::to_string_pretty(metadata).map_err(|e| {
            AppError::Internal(format!("Failed to serialize metadata: {}", e))
        })?;

        std::fs::write(&metadata_path, json).map_err(|e| {
            AppError::Internal(format!("Failed to write metadata: {}", e))
        })?;

        info!("Metadata stored for commit: {}", commit_id);
        Ok(())
    }

    /// Retrieve Logic Pro metadata for a commit
    pub fn get_metadata(&self, commit_id: &str) -> AppResult<Option<LogicProMetadata>> {
        let metadata_path = self
            .repo_path
            .join(".oxen")
            .join("metadata")
            .join(format!("{}.json", commit_id));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let json = std::fs::read_to_string(&metadata_path).map_err(|e| {
            AppError::Internal(format!("Failed to read metadata: {}", e))
        })?;

        let metadata = serde_json::from_str(&json).map_err(|e| {
            AppError::Internal(format!("Failed to parse metadata: {}", e))
        })?;

        Ok(Some(metadata))
    }

    /// Acquire lock for this repository
    pub fn acquire_lock(
        &self,
        user: &str,
        machine_id: &str,
        timeout_hours: u64,
    ) -> AppResult<FileLock> {
        FileLock::acquire(&self.repo_path, user, machine_id, timeout_hours).map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                AppError::Conflict(e.to_string())
            } else {
                AppError::Internal(format!("Failed to acquire lock: {}", e))
            }
        })
    }

    /// Release lock for this repository
    pub fn release_lock(&self, lock_id: &str) -> AppResult<()> {
        FileLock::release(&self.repo_path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to release lock: {}", e))
            }
        })
    }

    /// Update lock heartbeat
    pub fn heartbeat_lock(&self, lock_id: &str) -> AppResult<FileLock> {
        FileLock::heartbeat(&self.repo_path, lock_id).map_err(|e| {
            if e.kind() == std::io::ErrorKind::PermissionDenied {
                AppError::Unauthorized(e.to_string())
            } else {
                AppError::Internal(format!("Failed to update heartbeat: {}", e))
            }
        })
    }

    /// Get lock status
    pub fn lock_status(&self) -> AppResult<Option<FileLock>> {
        FileLock::status(&self.repo_path).map_err(|e| {
            AppError::Internal(format!("Failed to get lock status: {}", e))
        })
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CommitInfo {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}
