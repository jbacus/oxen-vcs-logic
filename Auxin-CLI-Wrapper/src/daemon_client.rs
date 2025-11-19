/// Daemon client for communicating with Auxin LaunchAgent
///
/// Provides CLI commands to control and query the background daemon service.
/// Uses launchctl for daemon lifecycle management and status checks.
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;

/// LaunchAgent service identifier
const LAUNCH_AGENT_LABEL: &str = "com.auxin.agent";

/// Daemon status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaemonStatus {
    /// Whether the daemon process is running
    pub is_running: bool,
    /// Process ID if running
    pub pid: Option<u32>,
    /// Number of monitored projects (if available)
    pub project_count: Option<usize>,
    /// Daemon version (if available)
    pub version: Option<String>,
    /// Uptime in seconds (if available)
    pub uptime: Option<f64>,
}

/// Daemon client for lifecycle management
pub struct DaemonClient;

impl DaemonClient {
    /// Create a new daemon client
    pub fn new() -> Self {
        Self
    }

    /// Get the status of the daemon service
    pub fn status(&self) -> Result<DaemonStatus> {
        // Use launchctl to check if the daemon is loaded and running
        let output = Command::new("launchctl")
            .args(["list", LAUNCH_AGENT_LABEL])
            .output()
            .context("Failed to execute launchctl")?;

        if !output.status.success() {
            // Service not loaded
            return Ok(DaemonStatus {
                is_running: false,
                pid: None,
                project_count: None,
                version: None,
                uptime: None,
            });
        }

        // Parse launchctl output
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract PID from output
        // Format: "PID" = <number>;
        let pid = extract_pid_from_launchctl_output(&stdout);

        Ok(DaemonStatus {
            is_running: pid.is_some(),
            pid,
            project_count: None, // Would need XPC/socket connection to get this
            version: None,       // Would need XPC/socket connection to get this
            uptime: None,        // Would need XPC/socket connection to get this
        })
    }

    /// Start the daemon service
    pub fn start(&self) -> Result<()> {
        // Construct path to the LaunchAgent plist
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let plist_path = format!(
            "{}/Library/LaunchAgents/{}.plist",
            home, LAUNCH_AGENT_LABEL
        );

        // Check if plist exists
        if !std::path::Path::new(&plist_path).exists() {
            anyhow::bail!(
                "LaunchAgent plist not found at: {}\nPlease run the installer first.",
                plist_path
            );
        }

        // Load the LaunchAgent
        let output = Command::new("launchctl")
            .args(["load", &plist_path])
            .output()
            .context("Failed to execute launchctl load")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start daemon: {}", stderr);
        }

        // Give it a moment to start
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Verify it started
        let status = self.status()?;
        if !status.is_running {
            anyhow::bail!("Daemon loaded but not running. Check system logs for errors.");
        }

        Ok(())
    }

    /// Stop the daemon service
    pub fn stop(&self) -> Result<()> {
        // Construct path to the LaunchAgent plist
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        let plist_path = format!(
            "{}/Library/LaunchAgents/{}.plist",
            home, LAUNCH_AGENT_LABEL
        );

        // Unload the LaunchAgent
        let output = Command::new("launchctl")
            .args(["unload", &plist_path])
            .output()
            .context("Failed to execute launchctl unload")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Check if it's "Could not find specified service" - that's OK
            if stderr.contains("Could not find") {
                return Ok(()); // Already stopped
            }

            anyhow::bail!("Failed to stop daemon: {}", stderr);
        }

        Ok(())
    }

    /// Restart the daemon service
    pub fn restart(&self) -> Result<()> {
        // Try to stop (ignore errors if not running)
        let _ = self.stop();

        // Wait a moment for clean shutdown
        std::thread::sleep(std::time::Duration::from_millis(500));

        // Start
        self.start()
    }

    /// Check if the daemon is installed
    pub fn is_installed(&self) -> bool {
        let home = match std::env::var("HOME") {
            Ok(h) => h,
            Err(_) => return false,
        };

        let plist_path = format!(
            "{}/Library/LaunchAgents/{}.plist",
            home, LAUNCH_AGENT_LABEL
        );

        std::path::Path::new(&plist_path).exists()
    }

    /// Get the path to the daemon's log file
    pub fn log_path(&self) -> Result<String> {
        let home = std::env::var("HOME").context("HOME environment variable not set")?;
        Ok(format!("{}/Library/Logs/Auxin/daemon.log", home))
    }

    /// Tail the daemon logs (returns last N lines)
    pub fn tail_logs(&self, lines: usize) -> Result<Vec<String>> {
        let log_path = self.log_path()?;

        if !std::path::Path::new(&log_path).exists() {
            return Ok(vec![format!("Log file not found: {}", log_path)]);
        }

        let output = Command::new("tail")
            .args(["-n", &lines.to_string(), &log_path])
            .output()
            .context("Failed to execute tail command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to read logs: {}", stderr);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.lines().map(|s| s.to_string()).collect())
    }
}

impl Default for DaemonClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract PID from launchctl list output
fn extract_pid_from_launchctl_output(output: &str) -> Option<u32> {
    // Look for line like: "PID" = <number>;
    for line in output.lines() {
        if line.contains("\"PID\"") {
            // Extract number between = and ;
            if let Some(start) = line.find('=') {
                if let Some(end) = line.find(';') {
                    let pid_str = line[start + 1..end].trim();
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_pid() {
        let output = r#"{
    "Label" = "com.auxin.agent";
    "LimitLoadToSessionType" = "Aqua";
    "OnDemand" = false;
    "LastExitStatus" = 0;
    "PID" = 12345;
    "Program" = "/usr/local/bin/auxin-daemon";
};"#;

        let pid = extract_pid_from_launchctl_output(output);
        assert_eq!(pid, Some(12345));
    }

    #[test]
    fn test_extract_pid_not_running() {
        let output = r#"{
    "Label" = "com.auxin.agent";
    "LimitLoadToSessionType" = "Aqua";
    "OnDemand" = false;
    "LastExitStatus" = 0;
};"#;

        let pid = extract_pid_from_launchctl_output(output);
        assert_eq!(pid, None);
    }

    #[test]
    fn test_daemon_client_creation() {
        let _client = DaemonClient::new();
        // Just verify it can be created
        assert!(true);
    }

    #[test]
    fn test_is_installed() {
        let client = DaemonClient::new();
        // This will vary based on whether the daemon is actually installed
        // Just verify the check runs without panicking
        let _ = client.is_installed();
    }

    #[test]
    fn test_log_path() {
        let client = DaemonClient::new();
        if let Ok(path) = client.log_path() {
            assert!(path.contains("Library/Logs/Auxin"));
        }
    }
}
