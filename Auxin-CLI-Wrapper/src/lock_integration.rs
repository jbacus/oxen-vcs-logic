/// Integration module for remote lock management with CLI
///
/// This module provides helper functions to integrate RemoteLockManager
/// with the CLI commands, handling user feedback and error presentation.

use crate::remote_lock::RemoteLockManager;
use crate::{progress, RemoteLock};
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

/// Get user identifier for lock operations
pub fn get_user_identifier() -> String {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string());

    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    format!("{}@{}", username, hostname)
}

/// Handle lock acquisition with UI feedback
pub fn handle_lock_acquire(repo_path: &Path, timeout_hours: u64) -> Result<()> {
    let manager = RemoteLockManager::new();
    let pb = progress::spinner("Acquiring project lock...");

    let user_id = get_user_identifier();

    match manager.acquire_lock(repo_path, &user_id, timeout_hours as u32) {
        Ok(lock) => {
            progress::finish_success(&pb, "Lock acquired");

            println!();
            println!("┌─ Lock Acquired ─────────────────────────────────────────┐");
            println!("│                                                          │");
            println!("│  ✓ You now have exclusive editing rights                │");
            println!("│                                                          │");
            println!("│  Lock ID: {:<44} │", truncate(&lock.lock_id, 44));
            println!(
                "│  Expires in: {} hours{:<36} │",
                timeout_hours,
                ""
            );
            println!(
                "│  Expires at: {}{:<27} │",
                lock.expires_at.format("%Y-%m-%d %H:%M UTC"),
                ""
            );
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();
            progress::info("You can now safely edit the project in Logic Pro");
            progress::info("The lock is stored in the remote repository");
            println!();
            progress::warning("Remember to release the lock when done:");
            println!("  auxin lock release");

            Ok(())
        }
        Err(e) => {
            progress::finish_error(&pb, "Failed to acquire lock");
            println!();
            progress::error(&format!("{}", e));
            println!();
            progress::info("Possible reasons:");
            println!("  • Project is locked by another user");
            println!("  • No remote repository configured");
            println!("  • Network/connectivity issues");
            println!();
            progress::info("Check lock status: auxin lock status");
            std::process::exit(1);
        }
    }
}

/// Handle lock release with UI feedback
pub fn handle_lock_release(repo_path: &Path) -> Result<()> {
    let manager = RemoteLockManager::new();
    let pb = progress::spinner("Releasing project lock...");

    // Get current lock to find lock ID
    match manager.get_lock(repo_path)? {
        Some(lock) => match manager.release_lock(repo_path, &lock.lock_id) {
            Ok(_) => {
                progress::finish_success(&pb, "Lock released");
                println!();
                progress::success("Lock released successfully");
                progress::info("Other team members can now acquire the lock");
                Ok(())
            }
            Err(e) => {
                progress::finish_error(&pb, "Failed to release lock");
                println!();
                progress::error(&format!("{}", e));
                std::process::exit(1);
            }
        },
        None => {
            progress::finish_error(&pb, "No lock found");
            println!();
            progress::warning("No lock exists for this project");
            progress::info("The project is already unlocked");
            Ok(())
        }
    }
}

/// Handle lock status check with UI feedback
pub fn handle_lock_status(repo_path: &Path) -> Result<()> {
    let manager = RemoteLockManager::new();
    let pb = progress::spinner("Checking lock status...");

    match manager.get_lock(repo_path) {
        Ok(Some(lock)) => {
            pb.finish_and_clear();

            println!();
            println!("┌─ Lock Status ───────────────────────────────────────────┐");
            println!("│                                                          │");

            if lock.is_expired() {
                println!("│  Status: {} Expired{:<42} │", "○".yellow(), "");
                println!("│                                                          │");
                println!("│  This lock has expired and can be overwritten           │");
            } else if lock.is_stale() {
                println!("│  Status: {} Stale{:<44} │", "◐".yellow(), "");
                println!("│                                                          │");
                println!("│  No heartbeat for >1 hour (may be abandoned)            │");
            } else {
                println!("│  Status: {} Locked{:<42} │", "●".red(), "");
            }

            println!("│                                                          │");
            println!(
                "│  Holder:    {:<45} │",
                truncate(&lock.locked_by, 45)
            );
            println!(
                "│  Acquired:  {}{:<27} │",
                lock.acquired_at.format("%Y-%m-%d %H:%M UTC"),
                ""
            );
            println!(
                "│  Expires:   {}{:<27} │",
                lock.expires_at.format("%Y-%m-%d %H:%M UTC"),
                ""
            );

            let remaining = lock.remaining_time();
            if remaining.num_seconds() > 0 {
                let hours = remaining.num_hours();
                let mins = remaining.num_minutes() % 60;
                println!(
                    "│  Remaining: {}h {}m{:<38} │",
                    hours,
                    mins,
                    ""
                );
            }

            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if lock.is_owned_by_current_user() {
                progress::success("You own this lock");
                progress::info("Release with: auxin lock release");
            } else {
                progress::warning("Locked by another user");
                progress::info("Wait for expiration or contact lock holder");
            }

            Ok(())
        }
        Ok(None) => {
            pb.finish_and_clear();

            println!();
            println!("┌─ Lock Status ───────────────────────────────────────────┐");
            println!("│                                                          │");
            println!("│  Status: {} Unlocked{:<40} │", "●".green(), "");
            println!("│                                                          │");
            println!("│  The project is available for editing                    │");
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();
            progress::info("Acquire lock with: auxin lock acquire");

            Ok(())
        }
        Err(e) => {
            progress::finish_error(&pb, "Failed to check lock status");
            println!();
            progress::error(&format!("{}", e));
            std::process::exit(1);
        }
    }
}

/// Handle force lock break with UI feedback
pub fn handle_lock_break(repo_path: &Path, force: bool) -> Result<()> {
    if !force {
        progress::error("The --force flag is required to break a lock");
        progress::info("This prevents accidental lock breaks");
        progress::info("Use: auxin lock break --force");
        std::process::exit(1);
    }

    let manager = RemoteLockManager::new();

    println!();
    progress::warning("⚠ BREAKING LOCK");
    println!();
    println!("This will forcibly remove the current lock.");
    println!("The lock holder may lose unsaved work!");
    println!();

    let pb = progress::spinner("Breaking lock...");

    match manager.force_break_lock(repo_path) {
        Ok(_) => {
            progress::finish_success(&pb, "Lock forcibly broken");
            println!();
            progress::success("Lock has been forcibly removed");
            progress::warning("Notify the previous lock holder!");
            Ok(())
        }
        Err(e) => {
            progress::finish_error(&pb, "Failed to break lock");
            println!();
            progress::error(&format!("{}", e));
            std::process::exit(1);
        }
    }
}

/// Truncate string to max length, adding ellipsis if needed
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_user_identifier() {
        let id = get_user_identifier();
        assert!(id.contains('@'));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
        assert_eq!(truncate("exact", 5), "exact");
    }

    // Note: The following tests require a real Oxen repository for full integration testing.
    // For unit testing without Oxen, we test the logic flow and error handling.

    #[test]
    fn test_handle_lock_status_no_repo() {
        use std::env;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let non_repo = temp_dir.path();

        // Should fail gracefully when not in an Oxen repo
        let result = handle_lock_status(non_repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_lock_release_no_repo() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let non_repo = temp_dir.path();

        // Should fail gracefully when not in an Oxen repo
        let result = handle_lock_release(non_repo);
        assert!(result.is_err());
    }

    #[test]
    fn test_handle_lock_break_requires_force() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let non_repo = temp_dir.path();

        // Without force flag, should fail
        let result = handle_lock_break(non_repo, false);
        assert!(result.is_err());
    }

    #[test]
    fn test_user_identifier_format() {
        let id = get_user_identifier();

        // Should contain @ separator
        assert!(id.contains('@'), "User identifier should contain @");

        // Should have both username and hostname parts
        let parts: Vec<&str> = id.split('@').collect();
        assert_eq!(parts.len(), 2, "Should have exactly one @ separator");
        assert!(!parts[0].is_empty(), "Username part should not be empty");
        assert!(!parts[1].is_empty(), "Hostname part should not be empty");
    }
}
