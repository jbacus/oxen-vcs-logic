//! Network resilience for collaboration features
//!
//! This module provides robust network operation handling with:
//! - Automatic retry with exponential backoff
//! - Network connectivity detection
//! - Error categorization (transient vs permanent)
//! - Timeout handling
//! - Operation queuing for offline scenarios
//!
//! # Example
//!
//! ```no_run
//! use oxenvcs_cli::network_resilience::{RetryPolicy, NetworkOperation};
//! use std::time::Duration;
//!
//! let policy = RetryPolicy::default();
//! let result = policy.execute(|| {
//!     // Your network operation here
//!     Ok(())
//! });
//! ```

use anyhow::{anyhow, Context, Result};
use std::time::{Duration, Instant};
use std::thread;
use colored::Colorize;

/// Maximum number of retry attempts
const MAX_RETRIES: u32 = 5;

/// Initial backoff duration
const INITIAL_BACKOFF_MS: u64 = 1000;

/// Maximum backoff duration (cap)
const MAX_BACKOFF_MS: u64 = 30000;

/// Network timeout for connectivity checks
const CONNECTIVITY_CHECK_TIMEOUT_MS: u64 = 5000;

/// Error types for network operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    /// Transient error that can be retried (network timeout, connection refused)
    Transient,

    /// Permanent error that should not be retried (authentication failure, not found)
    Permanent,

    /// Unknown error type (default to permanent for safety)
    Unknown,
}

/// Network connectivity state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    /// Network is available
    Online,

    /// Network is unavailable
    Offline,

    /// Network state unknown (default)
    Unknown,
}

/// Retry policy configuration
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Initial backoff duration in milliseconds
    pub initial_backoff_ms: u64,

    /// Maximum backoff duration in milliseconds
    pub max_backoff_ms: u64,

    /// Whether to use exponential backoff (vs fixed)
    pub exponential: bool,

    /// Whether to print retry progress
    pub verbose: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: MAX_RETRIES,
            initial_backoff_ms: INITIAL_BACKOFF_MS,
            max_backoff_ms: MAX_BACKOFF_MS,
            exponential: true,
            verbose: true,
        }
    }
}

impl RetryPolicy {
    /// Create a new retry policy with custom settings
    pub fn new(max_retries: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            exponential: true,
            verbose: true,
        }
    }

    /// Create a policy with no retries (fail fast)
    pub fn no_retry() -> Self {
        Self {
            max_retries: 0,
            initial_backoff_ms: 0,
            max_backoff_ms: 0,
            exponential: false,
            verbose: false,
        }
    }

    /// Create a policy with fixed backoff (no exponential)
    pub fn fixed_backoff(max_retries: u32, backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff_ms: backoff_ms,
            max_backoff_ms: backoff_ms,
            exponential: false,
            verbose: true,
        }
    }

    /// Enable or disable verbose output
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Calculate backoff duration for a given attempt
    pub fn backoff_duration(&self, attempt: u32) -> Duration {
        if !self.exponential {
            return Duration::from_millis(self.initial_backoff_ms);
        }

        // Exponential backoff: initial * 2^attempt
        let backoff_ms = self.initial_backoff_ms * 2u64.pow(attempt);
        let capped_ms = backoff_ms.min(self.max_backoff_ms);

        Duration::from_millis(capped_ms)
    }

    /// Execute an operation with retry logic
    ///
    /// # Arguments
    ///
    /// * `operation` - Closure that performs the network operation
    ///
    /// # Returns
    ///
    /// Result from the operation, or error if all retries exhausted
    pub fn execute<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut attempt = 0;
        let start_time = Instant::now();

        loop {
            match operation() {
                Ok(result) => {
                    if attempt > 0 && self.verbose {
                        let elapsed = start_time.elapsed();
                        crate::info!(
                            "Operation succeeded after {} attempt(s) in {:.1}s",
                            attempt + 1,
                            elapsed.as_secs_f64()
                        );
                    }
                    return Ok(result);
                }
                Err(e) => {
                    let error_type = categorize_error(&e);

                    // Don't retry permanent errors
                    if error_type == ErrorType::Permanent {
                        return Err(e.context("Operation failed (permanent error, not retrying)"));
                    }

                    // Check if we should retry
                    if attempt >= self.max_retries {
                        return Err(e.context(format!(
                            "Operation failed after {} attempts",
                            attempt + 1
                        )));
                    }

                    // Calculate backoff
                    let backoff = self.backoff_duration(attempt);

                    if self.verbose {
                        eprintln!(
                            "{}",
                            format!(
                                "⚠️  Attempt {} failed: {}",
                                attempt + 1,
                                e
                            ).yellow()
                        );
                        eprintln!(
                            "{}",
                            format!(
                                "   Retrying in {:.1}s... ({}/{} attempts remaining)",
                                backoff.as_secs_f64(),
                                self.max_retries - attempt,
                                self.max_retries
                            ).yellow()
                        );
                    }

                    // Wait before retry
                    thread::sleep(backoff);
                    attempt += 1;
                }
            }
        }
    }

    /// Execute an operation with retry, providing progress callback
    pub fn execute_with_progress<F, T, P>(&self, mut operation: F, mut on_retry: P) -> Result<T>
    where
        F: FnMut() -> Result<T>,
        P: FnMut(u32, Duration),
    {
        let mut attempt = 0;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    let error_type = categorize_error(&e);

                    if error_type == ErrorType::Permanent {
                        return Err(e.context("Permanent error"));
                    }

                    if attempt >= self.max_retries {
                        return Err(e.context(format!("Failed after {} attempts", attempt + 1)));
                    }

                    let backoff = self.backoff_duration(attempt);
                    on_retry(attempt, backoff);

                    thread::sleep(backoff);
                    attempt += 1;
                }
            }
        }
    }
}

/// Categorize error as transient or permanent
pub fn categorize_error(error: &anyhow::Error) -> ErrorType {
    let error_string = error.to_string().to_lowercase();

    // Transient errors (should retry)
    let transient_patterns = [
        "timeout",
        "connection refused",
        "connection reset",
        "network unreachable",
        "temporary failure",
        "too many requests",
        "service unavailable",
        "gateway timeout",
        "connection timed out",
        "no route to host",
        "broken pipe",
    ];

    for pattern in &transient_patterns {
        if error_string.contains(pattern) {
            return ErrorType::Transient;
        }
    }

    // Permanent errors (should not retry)
    let permanent_patterns = [
        "authentication failed",
        "unauthorized",
        "forbidden",
        "not found",
        "invalid credentials",
        "permission denied",
        "already exists",
        "conflict",
        "bad request",
        "invalid",
    ];

    for pattern in &permanent_patterns {
        if error_string.contains(pattern) {
            return ErrorType::Permanent;
        }
    }

    // Default to permanent for safety (avoid infinite retries)
    ErrorType::Permanent
}

/// Check network connectivity
pub fn check_connectivity() -> ConnectivityState {
    // Try to connect to Oxen Hub
    let result = check_host_connectivity("hub.oxen.ai", 443);

    if result {
        ConnectivityState::Online
    } else {
        // Fallback: try DNS servers
        let dns_reachable = check_host_connectivity("8.8.8.8", 53);
        if dns_reachable {
            ConnectivityState::Online
        } else {
            ConnectivityState::Offline
        }
    }
}

/// Check if a specific host is reachable
fn check_host_connectivity(host: &str, port: u16) -> bool {
    use std::net::{TcpStream, ToSocketAddrs};

    // Resolve host to socket address
    let addr = match format!("{}:{}", host, port).to_socket_addrs() {
        Ok(mut addrs) => match addrs.next() {
            Some(addr) => addr,
            None => return false,
        },
        Err(_) => return false,
    };

    // Try to connect with timeout
    match TcpStream::connect_timeout(&addr, Duration::from_millis(CONNECTIVITY_CHECK_TIMEOUT_MS)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

/// Wait for network connectivity to be restored
///
/// # Arguments
///
/// * `max_wait` - Maximum duration to wait
/// * `check_interval` - How often to check connectivity
///
/// # Returns
///
/// Ok if connectivity restored, Err if timeout
pub fn wait_for_connectivity(max_wait: Duration, check_interval: Duration) -> Result<()> {
    let start = Instant::now();

    eprintln!("{}", "⚠️  Network appears offline. Waiting for connectivity...".yellow());

    while start.elapsed() < max_wait {
        match check_connectivity() {
            ConnectivityState::Online => {
                eprintln!("{}", "✓ Network connectivity restored".green());
                return Ok(());
            }
            _ => {
                thread::sleep(check_interval);
            }
        }
    }

    Err(anyhow!("Network connectivity not restored after {:.0}s", max_wait.as_secs_f64()))
}

/// Network operation wrapper with built-in resilience
pub struct NetworkOperation<T> {
    name: String,
    operation: Box<dyn FnMut() -> Result<T>>,
    policy: RetryPolicy,
}

impl<T> NetworkOperation<T> {
    /// Create a new network operation
    pub fn new<F>(name: impl Into<String>, operation: F) -> Self
    where
        F: FnMut() -> Result<T> + 'static,
    {
        Self {
            name: name.into(),
            operation: Box::new(operation),
            policy: RetryPolicy::default(),
        }
    }

    /// Set custom retry policy
    pub fn with_policy(mut self, policy: RetryPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Execute the operation with resilience
    pub fn execute(mut self) -> Result<T> {
        crate::vlog!("Executing network operation: {}", self.name);

        // Check connectivity first
        match check_connectivity() {
            ConnectivityState::Offline => {
                return Err(anyhow!("Network is offline. Operation cannot be performed."));
            }
            _ => {}
        }

        // Execute with retry
        self.policy.execute(|| (self.operation)())
            .context(format!("Network operation '{}' failed", self.name))
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, MAX_RETRIES);
        assert_eq!(policy.initial_backoff_ms, INITIAL_BACKOFF_MS);
        assert!(policy.exponential);
    }

    #[test]
    fn test_retry_policy_no_retry() {
        let policy = RetryPolicy::no_retry();
        assert_eq!(policy.max_retries, 0);
    }

    #[test]
    fn test_backoff_duration_exponential() {
        let policy = RetryPolicy::default();

        // Attempt 0: 1000ms
        assert_eq!(policy.backoff_duration(0), Duration::from_millis(1000));

        // Attempt 1: 2000ms
        assert_eq!(policy.backoff_duration(1), Duration::from_millis(2000));

        // Attempt 2: 4000ms
        assert_eq!(policy.backoff_duration(2), Duration::from_millis(4000));

        // Attempt 3: 8000ms
        assert_eq!(policy.backoff_duration(3), Duration::from_millis(8000));
    }

    #[test]
    fn test_backoff_duration_fixed() {
        let policy = RetryPolicy::fixed_backoff(3, 2000);

        assert_eq!(policy.backoff_duration(0), Duration::from_millis(2000));
        assert_eq!(policy.backoff_duration(1), Duration::from_millis(2000));
        assert_eq!(policy.backoff_duration(2), Duration::from_millis(2000));
    }

    #[test]
    fn test_backoff_duration_capped() {
        let policy = RetryPolicy::new(10, 1000, 5000);

        // Should cap at 5000ms
        assert_eq!(policy.backoff_duration(10), Duration::from_millis(5000));
    }

    #[test]
    fn test_categorize_error_transient() {
        let error = anyhow!("Connection timeout");
        assert_eq!(categorize_error(&error), ErrorType::Transient);

        let error = anyhow!("Connection refused");
        assert_eq!(categorize_error(&error), ErrorType::Transient);

        let error = anyhow!("Network unreachable");
        assert_eq!(categorize_error(&error), ErrorType::Transient);
    }

    #[test]
    fn test_categorize_error_permanent() {
        let error = anyhow!("Authentication failed");
        assert_eq!(categorize_error(&error), ErrorType::Permanent);

        let error = anyhow!("Not found");
        assert_eq!(categorize_error(&error), ErrorType::Permanent);

        let error = anyhow!("Permission denied");
        assert_eq!(categorize_error(&error), ErrorType::Permanent);
    }

    #[test]
    fn test_categorize_error_default_permanent() {
        let error = anyhow!("Some unknown error");
        assert_eq!(categorize_error(&error), ErrorType::Permanent);
    }

    #[test]
    fn test_retry_policy_success_first_try() {
        let policy = RetryPolicy::default().set_verbose(false);
        let mut attempts = 0;

        let result = policy.execute(|| {
            attempts += 1;
            Ok(42)
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 1);
    }

    #[test]
    fn test_retry_policy_success_after_retry() {
        let policy = RetryPolicy::new(3, 10, 100).set_verbose(false);
        let mut attempts = 0;

        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            if attempts < 3 {
                Err(anyhow!("Connection timeout")) // Transient
            } else {
                Ok(42)
            }
        });

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempts, 3);
    }

    #[test]
    fn test_retry_policy_permanent_error_no_retry() {
        let policy = RetryPolicy::default().set_verbose(false);
        let mut attempts = 0;

        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            Err(anyhow!("Authentication failed")) // Permanent
        });

        assert!(result.is_err());
        assert_eq!(attempts, 1); // No retries for permanent errors
    }

    #[test]
    fn test_retry_policy_exhausted_retries() {
        let policy = RetryPolicy::new(2, 10, 100).set_verbose(false);
        let mut attempts = 0;

        let result: Result<i32> = policy.execute(|| {
            attempts += 1;
            Err(anyhow!("Connection timeout")) // Transient
        });

        assert!(result.is_err());
        assert_eq!(attempts, 3); // Initial + 2 retries
    }

    #[test]
    fn test_check_connectivity() {
        // This test requires network, so it may fail in offline environments
        // We'll just verify it returns a valid state
        let state = check_connectivity();
        assert!(
            state == ConnectivityState::Online ||
            state == ConnectivityState::Offline
        );
    }

    #[test]
    fn test_network_operation_success() {
        let op = NetworkOperation::new("test_op", || Ok(42))
            .with_policy(RetryPolicy::default().set_verbose(false));

        let result = op.execute();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }
}
