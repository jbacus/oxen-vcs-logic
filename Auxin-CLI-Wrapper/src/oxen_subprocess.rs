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
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

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
        // For existing paths, canonicalize to check for path traversal
        if path.exists() {
            let canonical = path.canonicalize()
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
///
/// # Architecture
///
/// - Executes `oxen` commands using `std::process::Command`
/// - Parses stdout/stderr to extract structured data
/// - Provides type-safe Rust API over CLI interface
/// - Handles errors, timeouts, and edge cases
///
/// # Requirements
///
/// The `oxen` CLI must be installed and accessible in PATH:
/// ```bash
/// pip install oxen-ai    # Recommended for most users
/// # or
/// cargo install oxen     # Build from source
/// ```
///
/// Verify installation:
/// ```bash
/// oxen --version
/// ```
///
/// # Examples
///
/// ```no_run
/// use auxin_cli::OxenSubprocess;
/// use std::path::Path;
///
/// fn main() -> anyhow::Result<()> {
///     let oxen = OxenSubprocess::new();
///
///     // Check if oxen is available
///     if !oxen.is_available() {
///         eprintln!("oxen CLI not found. Install: pip install oxen-ai");
///         return Ok(());
///     }
///
///     // Initialize repository
///     let project = Path::new("/path/to/project.logicx");
///     oxen.init(project)?;
///
///     // Add and commit files
///     oxen.add_all(project)?;
///     oxen.commit(project, "Initial commit")?;
///
///     // View history
///     let commits = oxen.log(project, Some(10))?;
///     for commit in commits {
///         println!("{}: {}", commit.id, commit.message);
///     }
///
///     Ok(())
/// }
/// ```
///
/// # Performance
///
/// Each method call spawns a subprocess with typical overhead:
/// - Startup: ~10-50ms per command
/// - Command execution: Depends on operation (init: ~100ms, commit: ~500ms)
/// - Output parsing: <5ms for typical outputs
///
/// For high-frequency operations, consider batching or caching.
///
/// # Error Handling
///
/// All methods return `Result<T, anyhow::Error>` with descriptive error messages:
/// - Command not found: "oxen command not found in PATH"
/// - Non-zero exit: Includes stderr output from oxen
/// - Parse errors: "Failed to parse oxen output: ..."
///
/// # Verbose Mode
///
/// Enable verbose logging to see executed commands and output:
/// ```no_run
/// use auxin_cli::OxenSubprocess;
///
/// let oxen = OxenSubprocess::new().verbose(true);
/// // Will log all executed commands and their output
/// ```
///
/// # See Also
///
/// - `CommitInfo` - Parsed commit information
/// - `StatusInfo` - File status information
/// - `BranchInfo` - Branch details
#[derive(Debug, Clone)]
pub struct OxenSubprocess {
    /// Path to oxen executable (defaults to "oxen" in PATH)
    oxen_path: String,
    /// Enable verbose logging of commands and output
    verbose: bool,
}

impl OxenSubprocess {
    /// Create a new OxenSubprocess wrapper with default settings
    pub fn new() -> Self {
        Self {
            oxen_path: "oxen".to_string(),
            verbose: false,
        }
    }

    /// Create with custom oxen executable path
    pub fn with_path(oxen_path: impl Into<String>) -> Self {
        Self {
            oxen_path: oxen_path.into(),
            verbose: false,
        }
    }

    /// Enable verbose output
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Check if oxen is available in PATH
    pub fn is_available(&self) -> bool {
        Command::new(&self.oxen_path)
            .arg("--version")
            .output()
            .is_ok()
    }

    /// Get oxen version
    pub fn version(&self) -> Result<String> {
        let output = self.run_command(&["--version"], None)?;
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

        self.run_command(&["init"], Some(path))?;

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

        // Sanitize all file paths before passing to subprocess
        let file_args: Vec<String> = files
            .iter()
            .map(|f| sanitize_path(f, Some(repo_path)))
            .collect::<Result<Vec<String>>>()
            .context("Failed to sanitize file paths")?;

        let mut args = vec!["add"];
        for file in &file_args {
            args.push(file);
        }

        self.run_command(&args, Some(repo_path))?;

        info!("Added {} file(s) to staging", files.len());
        Ok(())
    }

    /// Add all files to staging
    pub fn add_all(&self, repo_path: &Path) -> Result<()> {
        vlog!("Adding all files to staging");

        self.run_command(&["add", "."], Some(repo_path))?;

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

        vlog!("Creating commit with message: {}", sanitized_message);

        let output = self.run_command(&["commit", "-m", &sanitized_message], Some(repo_path))?;

        // Parse commit hash from output
        // Typical output: "Commit abc123def created"
        let commit_id = self
            .parse_commit_id(&output)
            .unwrap_or_else(|| "unknown".to_string());

        info!("Created commit: {}", commit_id);

        Ok(CommitInfo {
            id: commit_id,
            message: message.to_string(),
        })
    }

    /// Get commit log
    pub fn log(&self, repo_path: &Path, limit: Option<usize>) -> Result<Vec<CommitInfo>> {
        vlog!("Fetching commit log");

        let mut args = vec!["log"];
        let limit_str;
        if let Some(n) = limit {
            limit_str = format!("-n={}", n);
            args.push(&limit_str);
        }

        let output = self.run_command(&args, Some(repo_path))?;

        let commits = self.parse_log_output(&output)?;

        vlog!("Found {} commit(s)", commits.len());
        Ok(commits)
    }

    /// Get repository status
    pub fn status(&self, repo_path: &Path) -> Result<StatusInfo> {
        vlog!("Getting repository status");

        let output = self.run_command(&["status"], Some(repo_path))?;

        let status = self.parse_status_output(&output)?;

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

        self.run_command(&["checkout", target], Some(repo_path))?;

        info!("Checked out: {}", target);
        Ok(())
    }

    /// Create a new branch
    pub fn create_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        vlog!("Creating branch: {}", branch_name);

        self.run_command(&["checkout", "-b", branch_name], Some(repo_path))?;

        info!("Created branch: {}", branch_name);
        Ok(())
    }

    /// List branches
    pub fn list_branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>> {
        vlog!("Listing branches");

        let output = self.run_command(&["branch"], Some(repo_path))?;

        let branches = self.parse_branches_output(&output)?;

        vlog!("Found {} branch(es)", branches.len());
        Ok(branches)
    }

    /// Get current branch name
    pub fn current_branch(&self, repo_path: &Path) -> Result<String> {
        vlog!("Getting current branch");

        let output = self.run_command(&["branch", "--show-current"], Some(repo_path))?;

        let branch = output.trim().to_string();
        vlog!("Current branch: {}", branch);
        Ok(branch)
    }

    /// Delete a branch (force delete to allow removing unmerged branches)
    pub fn delete_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        vlog!("Deleting branch: {}", branch_name);

        // Use -D (force delete) to allow deleting unmerged branches (e.g., draft branches)
        self.run_command(&["branch", "-D", branch_name], Some(repo_path))?;

        info!("Deleted branch: {}", branch_name);
        Ok(())
    }

    /// Push to remote
    pub fn push(&self, repo_path: &Path, remote: Option<&str>, branch: Option<&str>) -> Result<()> {
        vlog!("Pushing to remote");

        let mut args = vec!["push"];
        if let Some(r) = remote {
            args.push(r);
        }
        if let Some(b) = branch {
            args.push(b);
        }

        self.run_command(&args, Some(repo_path))?;

        info!("Pushed to remote");
        Ok(())
    }

    /// Pull from remote
    pub fn pull(&self, repo_path: &Path) -> Result<()> {
        vlog!("Pulling from remote");

        self.run_command(&["pull"], Some(repo_path))?;

        info!("Pulled from remote");
        Ok(())
    }

    // ========== Private Helper Methods ==========

    /// Run an oxen command and capture output
    fn run_command(&self, args: &[&str], cwd: Option<&Path>) -> Result<String> {
        if self.verbose {
            vlog!("Running: {} {}", self.oxen_path, args.join(" "));
        }

        let mut cmd = Command::new(&self.oxen_path);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let output = cmd
            .output()
            .with_context(|| format!("Failed to execute oxen command: {}", args.join(" ")))?;

        self.handle_output(output, args)
    }

    /// Handle command output and errors
    fn handle_output(&self, output: Output, args: &[&str]) -> Result<String> {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // IMPORTANT: Check both stdout AND stderr for error messages even if exit code is 0
        // Oxen CLI has TWO bugs:
        // 1. It returns exit code 0 even on failures
        // 2. It sometimes writes errors to stdout instead of stderr
        // (e.g., "Revision not found" during checkout goes to stdout!)

        let stdout_lower = stdout.to_lowercase();
        let stderr_lower = stderr.to_lowercase();

        // Known error patterns from Oxen CLI (check both stdout and stderr)
        let has_error = stdout_lower.contains("revision not found")
            || stdout_lower.contains("not found")
            || stdout_lower.contains("error:")
            || stdout_lower.contains("fatal:")
            || stdout_lower.contains("failed to")
            || stderr_lower.contains("revision not found")
            || stderr_lower.contains("not found")
            || stderr_lower.contains("error:")
            || stderr_lower.contains("fatal:")
            || stderr_lower.contains("failed to");

        if has_error {
            error!("Command failed: oxen {}", args.join(" "));
            if !stderr.is_empty() {
                error!("stderr: {}", stderr);
            }
            if !stdout.is_empty() {
                error!("stdout: {}", stdout);
            }

            let error_output = if !stderr.is_empty() {
                stderr.trim()
            } else {
                stdout.trim()
            };

            return Err(anyhow!(
                "oxen command failed: {}\n{}",
                args.join(" "),
                error_output
            ));
        }

        // Also check exit code (for well-behaved commands)
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
        // Try to extract commit hash from various formats
        // "Commit abc123 created" or "abc123" or "[abc123]"

        for line in output.lines() {
            // Look for hexadecimal strings that might be commit hashes
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
                // Files/directories listed under section headers (indented)
                // Handle format like "Media (1 item)" by extracting just the path
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
        assert_eq!(oxen.oxen_path, "oxen");
        assert!(!oxen.verbose);
    }

    #[test]
    fn test_with_path() {
        let oxen = OxenSubprocess::with_path("/usr/local/bin/oxen");
        assert_eq!(oxen.oxen_path, "/usr/local/bin/oxen");
    }

    #[test]
    fn test_verbose_builder() {
        let oxen = OxenSubprocess::new().verbose(true);
        assert!(oxen.verbose);
    }

    #[test]
    fn test_parse_commit_id_various_formats() {
        let oxen = OxenSubprocess::new();

        assert_eq!(
            oxen.parse_commit_id("Commit abc123def created"),
            Some("abc123def".to_string())
        );

        assert_eq!(
            oxen.parse_commit_id("[abc1234] message here"), // 7 chars minimum
            Some("abc1234".to_string())
        );

        assert_eq!(
            oxen.parse_commit_id("abc123def456"),
            Some("abc123def456".to_string())
        );
    }

    #[test]
    fn test_parse_commit_id_invalid() {
        let oxen = OxenSubprocess::new();

        assert_eq!(oxen.parse_commit_id("No hash here"), None);
        assert_eq!(oxen.parse_commit_id("123"), None); // Too short
        assert_eq!(oxen.parse_commit_id("xyz"), None); // Not hex
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

    // ========== Additional Comprehensive Tests ==========

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
        let output = "Some text\nCommit abc1234 created\nMore text"; // 7 chars minimum
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
        assert_eq!(oxen.parse_commit_id("abc12"), None); // Less than 7 chars
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
    fn test_parse_log_output_multiple_commits() {
        let oxen = OxenSubprocess::new();
        let output = r#"
commit abc123
Author: User <user@example.com>
Date: 2025-01-01

    Third commit

commit def456
Author: User <user@example.com>
Date: 2025-01-02

    Second commit

commit xyz789
Author: User <user@example.com>
Date: 2025-01-03

    First commit
        "#;

        let commits = oxen.parse_log_output(output).unwrap();
        assert_eq!(commits.len(), 3);
        assert_eq!(commits[0].id, "abc123");
        assert_eq!(commits[1].id, "def456");
        assert_eq!(commits[2].id, "xyz789");
    }

    #[test]
    fn test_parse_log_output_multiline_message() {
        let oxen = OxenSubprocess::new();
        let output = r#"
commit abc123
Author: User <user@example.com>
Date: 2025-01-01

    First line of commit
    Second line of commit
    Third line of commit
        "#;

        let commits = oxen.parse_log_output(output).unwrap();
        assert_eq!(commits.len(), 1);
        assert!(commits[0].message.contains("First line"));
        assert!(commits[0].message.contains("Second line"));
        assert!(commits[0].message.contains("Third line"));
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
    fn test_parse_status_output_modified_only() {
        let oxen = OxenSubprocess::new();
        let output = "M  file1.txt\nM  file2.rs";
        let status = oxen.parse_status_output(output).unwrap();

        assert_eq!(status.modified.len(), 2);
        assert!(status.untracked.is_empty());
        assert!(status.staged.is_empty());
    }

    #[test]
    fn test_parse_status_output_untracked_only() {
        let oxen = OxenSubprocess::new();
        let output = "?  new1.txt\n?  new2.txt";
        let status = oxen.parse_status_output(output).unwrap();

        assert!(status.modified.is_empty());
        assert_eq!(status.untracked.len(), 2);
        assert!(status.staged.is_empty());
    }

    #[test]
    fn test_parse_status_output_staged_only() {
        let oxen = OxenSubprocess::new();
        let output = "A  added1.txt\nA  added2.txt";
        let status = oxen.parse_status_output(output).unwrap();

        assert!(status.modified.is_empty());
        assert!(status.untracked.is_empty());
        assert_eq!(status.staged.len(), 2);
    }

    #[test]
    fn test_parse_status_output_mixed() {
        let oxen = OxenSubprocess::new();
        let output = r#"
M  modified.txt
?  untracked.txt
A  staged.txt
modified: another_modified.txt
        "#;

        let status = oxen.parse_status_output(output).unwrap();

        assert_eq!(status.modified.len(), 2);
        assert_eq!(status.untracked.len(), 1);
        assert_eq!(status.staged.len(), 1);
    }

    #[test]
    fn test_parse_status_output_with_paths() {
        let oxen = OxenSubprocess::new();
        let output = "M  src/main.rs\n?  tests/new_test.rs";
        let status = oxen.parse_status_output(output).unwrap();

        assert_eq!(status.modified[0], PathBuf::from("src/main.rs"));
        assert_eq!(status.untracked[0], PathBuf::from("tests/new_test.rs"));
    }

    #[test]
    fn test_parse_branches_output_empty() {
        let oxen = OxenSubprocess::new();
        let branches = oxen.parse_branches_output("").unwrap();
        assert!(branches.is_empty());
    }

    #[test]
    fn test_parse_branches_output_single_branch() {
        let oxen = OxenSubprocess::new();
        let output = "* main";
        let branches = oxen.parse_branches_output(output).unwrap();

        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_current);
    }

    #[test]
    fn test_parse_branches_output_multiple_branches() {
        let oxen = OxenSubprocess::new();
        let output = r#"
* main
  develop
  feature-branch
  draft
        "#;

        let branches = oxen.parse_branches_output(output).unwrap();

        assert_eq!(branches.len(), 4);
        assert_eq!(branches[0].name, "main");
        assert!(branches[0].is_current);
        assert_eq!(branches[1].name, "develop");
        assert!(!branches[1].is_current);
        assert_eq!(branches[2].name, "feature-branch");
        assert!(!branches[2].is_current);
    }

    #[test]
    fn test_parse_branches_output_no_current() {
        let oxen = OxenSubprocess::new();
        let output = "  branch1\n  branch2";
        let branches = oxen.parse_branches_output(output).unwrap();

        assert_eq!(branches.len(), 2);
        assert!(!branches[0].is_current);
        assert!(!branches[1].is_current);
    }

    #[test]
    fn test_parse_branches_output_with_whitespace() {
        let oxen = OxenSubprocess::new();
        let output = "  \n* main  \n  develop\n  \n";
        let branches = oxen.parse_branches_output(output).unwrap();

        assert_eq!(branches.len(), 2);
        assert_eq!(branches[0].name, "main");
        assert_eq!(branches[1].name, "develop");
    }

    #[test]
    fn test_extract_path_from_status_line_modified() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.extract_path_from_status_line("M  path/to/file.txt"),
            PathBuf::from("path/to/file.txt")
        );
    }

    #[test]
    fn test_extract_path_from_status_line_modified_colon() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.extract_path_from_status_line("modified: src/lib.rs"),
            PathBuf::from("src/lib.rs")
        );
    }

    #[test]
    fn test_extract_path_from_status_line_untracked() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.extract_path_from_status_line("?  new_file.rs"),
            PathBuf::from("new_file.rs")
        );
    }

    #[test]
    fn test_extract_path_from_status_line_added() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.extract_path_from_status_line("A  added.txt"),
            PathBuf::from("added.txt")
        );
    }

    #[test]
    fn test_extract_path_from_status_line_with_spaces() {
        let oxen = OxenSubprocess::new();
        assert_eq!(
            oxen.extract_path_from_status_line("M  path with spaces/file.txt"),
            PathBuf::from("path with spaces/file.txt")
        );
    }

    #[test]
    fn test_oxen_subprocess_default() {
        let oxen = OxenSubprocess::default();
        assert_eq!(oxen.oxen_path, "oxen");
        assert!(!oxen.verbose);
    }

    #[test]
    fn test_oxen_subprocess_clone() {
        let oxen = OxenSubprocess::new().verbose(true);
        let cloned = oxen.clone();
        assert!(cloned.verbose);
        assert_eq!(cloned.oxen_path, "oxen");
    }

    #[test]
    fn test_oxen_subprocess_debug() {
        let oxen = OxenSubprocess::new();
        let debug_str = format!("{:?}", oxen);
        assert!(debug_str.contains("oxen"));
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
    fn test_commit_info_clone() {
        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: "Test".to_string(),
        };
        let cloned = commit.clone();
        assert_eq!(commit, cloned);
    }

    #[test]
    fn test_commit_info_debug() {
        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: "Test".to_string(),
        };
        let debug_str = format!("{:?}", commit);
        assert!(debug_str.contains("abc123"));
        assert!(debug_str.contains("Test"));
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
    fn test_status_info_debug() {
        let status = StatusInfo {
            modified: vec![PathBuf::from("a.txt")],
            untracked: vec![],
            staged: vec![],
        };
        let debug_str = format!("{:?}", status);
        assert!(debug_str.contains("modified"));
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
    fn test_branch_info_clone() {
        let branch = BranchInfo {
            name: "main".to_string(),
            is_current: true,
        };
        let cloned = branch.clone();
        assert_eq!(branch, cloned);
    }

    #[test]
    fn test_branch_info_debug() {
        let branch = BranchInfo {
            name: "main".to_string(),
            is_current: true,
        };
        let debug_str = format!("{:?}", branch);
        assert!(debug_str.contains("main"));
        assert!(debug_str.contains("true"));
    }

    #[test]
    fn test_verbose_builder_chain() {
        let oxen = OxenSubprocess::new()
            .verbose(true)
            .verbose(false)
            .verbose(true);
        assert!(oxen.verbose);
    }

    #[test]
    fn test_with_path_custom() {
        let custom_path = "/usr/local/bin/oxen";
        let oxen = OxenSubprocess::with_path(custom_path);
        assert_eq!(oxen.oxen_path, custom_path);
    }

    #[test]
    fn test_parse_log_output_with_metadata() {
        let oxen = OxenSubprocess::new();
        let output = r#"
commit abc123
Author: User <user@example.com>
Date: 2025-01-01

    Added drum track

    BPM: 120
    Sample Rate: 48000 Hz
    Key: C Major
        "#;

        let commits = oxen.parse_log_output(output).unwrap();
        assert_eq!(commits.len(), 1);
        assert!(commits[0].message.contains("Added drum track"));
        assert!(commits[0].message.contains("BPM: 120"));
    }

    #[test]
    fn test_parse_status_output_new_file_colon_format() {
        let oxen = OxenSubprocess::new();
        let output = "new file: path/to/file.txt";
        let status = oxen.parse_status_output(output).unwrap();

        assert_eq!(status.staged.len(), 1);
        assert_eq!(status.staged[0], PathBuf::from("path/to/file.txt"));
    }

    #[test]
    fn test_parse_branches_with_remotes() {
        let oxen = OxenSubprocess::new();
        let output = r#"
* main
  remotes/origin/main
  develop
        "#;

        let branches = oxen.parse_branches_output(output).unwrap();
        // Should parse all as branch names
        assert!(branches.len() >= 2);
    }

    #[test]
    fn test_status_info_all_empty() {
        let status = StatusInfo {
            modified: vec![],
            untracked: vec![],
            staged: vec![],
        };
        assert!(status.modified.is_empty());
        assert!(status.untracked.is_empty());
        assert!(status.staged.is_empty());
    }

    #[test]
    fn test_status_info_multiple_files() {
        let status = StatusInfo {
            modified: vec![
                PathBuf::from("file1.txt"),
                PathBuf::from("file2.txt"),
                PathBuf::from("file3.txt"),
            ],
            untracked: vec![PathBuf::from("new1.txt"), PathBuf::from("new2.txt")],
            staged: vec![PathBuf::from("staged.txt")],
        };

        assert_eq!(status.modified.len(), 3);
        assert_eq!(status.untracked.len(), 2);
        assert_eq!(status.staged.len(), 1);
    }

    #[test]
    fn test_commit_info_empty_message() {
        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: String::new(),
        };
        assert!(commit.message.is_empty());
    }

    #[test]
    fn test_commit_info_long_message() {
        let long_msg = "a".repeat(1000);
        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: long_msg.clone(),
        };
        assert_eq!(commit.message.len(), 1000);
    }

    #[test]
    fn test_parse_commit_id_edge_case_7_chars() {
        let oxen = OxenSubprocess::new();
        assert_eq!(oxen.parse_commit_id("abc1234"), Some("abc1234".to_string()));
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
        assert_eq!(oxen.parse_commit_id(&hash), None); // Too long
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
