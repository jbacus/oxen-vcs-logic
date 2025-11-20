use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration as StdDuration;

/// Maximum number of retry attempts for network operations
const MAX_RETRIES: u32 = 4;

/// Initial backoff duration in milliseconds
const INITIAL_BACKOFF_MS: u64 = 2000;

/// Maximum backoff duration in milliseconds (16 seconds)
const MAX_BACKOFF_MS: u64 = 16000;

/// Connectivity state for network checks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectivityState {
    Online,
    Offline,
    Unknown,
}

/// Check current connectivity state
pub fn check_connectivity() -> ConnectivityState {
    if check_network_availability() {
        ConnectivityState::Online
    } else {
        ConnectivityState::Offline
    }
}

/// Retry policy for network operations
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    max_retries: u32,
    initial_backoff_ms: u64,
    max_backoff_ms: u64,
    verbose: bool,
}

impl RetryPolicy {
    /// Create a new retry policy
    pub fn new(max_retries: u32, initial_backoff_ms: u64, max_backoff_ms: u64) -> Self {
        Self {
            max_retries,
            initial_backoff_ms,
            max_backoff_ms,
            verbose: false,
        }
    }

    /// Enable verbose logging
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Execute an operation with retry logic
    pub fn execute<F, T>(&self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        let mut attempt = 0;
        let mut backoff_ms = self.initial_backoff_ms;

        loop {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) => {
                    attempt += 1;
                    if attempt > self.max_retries {
                        return Err(e);
                    }

                    if self.verbose {
                        crate::vlog!("Retry attempt {}/{}: {}", attempt, self.max_retries, e);
                    }

                    thread::sleep(StdDuration::from_millis(backoff_ms));
                    backoff_ms = (backoff_ms * 2).min(self.max_backoff_ms);
                }
            }
        }
    }

    /// Get the delay duration for a specific attempt number (1-indexed)
    pub fn delay_for_attempt(&self, attempt: usize) -> StdDuration {
        if attempt == 0 {
            return StdDuration::from_millis(0);
        }
        let delay = self.initial_backoff_ms * (1 << (attempt - 1).min(10));
        StdDuration::from_millis(delay.min(self.max_backoff_ms))
    }

    /// Check if we should retry for the given attempt number
    pub fn should_retry(&self, attempt: usize) -> bool {
        attempt <= self.max_retries as usize
    }

    /// Check if an error message indicates a retryable error
    pub fn is_retryable(&self, error: &str) -> bool {
        let error_lower = error.to_lowercase();

        // Network errors are retryable
        let retryable_patterns = [
            "connection refused", "connection reset", "connection timed out",
            "network", "timeout", "dns", "temporary failure", "try again",
            "rate limit", "too many requests", "503", "502", "504",
            "econnrefused", "etimedout", "enotfound", "enetunreach",
        ];

        // Auth/permission errors are NOT retryable
        let non_retryable_patterns = [
            "unauthorized", "forbidden", "invalid credentials", "authentication",
            "permission denied", "not found", "404", "401", "403",
            "eacces", "eperm",
        ];

        // Check for non-retryable first
        if non_retryable_patterns.iter().any(|p| error_lower.contains(p)) {
            return false;
        }

        // Then check for retryable
        retryable_patterns.iter().any(|p| error_lower.contains(p))
    }

    /// Get maximum number of retries
    pub fn max_attempts(&self) -> u32 {
        self.max_retries
    }

    /// Get initial backoff in milliseconds
    pub fn base_delay_ms(&self) -> u64 {
        self.initial_backoff_ms
    }

    /// Get maximum backoff in milliseconds
    pub fn max_delay_ms(&self) -> u64 {
        self.max_backoff_ms
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self::new(MAX_RETRIES, INITIAL_BACKOFF_MS, MAX_BACKOFF_MS)
    }
}

/// Represents a queued operation that failed due to network issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QueuedOperation {
    /// Unique identifier for this operation
    pub id: String,

    /// Type of operation (push, pull, lock_acquire, etc.)
    pub operation_type: OperationType,

    /// Repository path
    pub repo_path: PathBuf,

    /// Additional operation-specific data
    pub data: OperationData,

    /// When this operation was first queued
    pub queued_at: DateTime<Utc>,

    /// Number of times this operation has been attempted
    pub attempt_count: u32,

    /// Last error message
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OperationType {
    Push,
    Pull,
    LockAcquire,
    LockRelease,
    LockRenew,
    CommentSync,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OperationData {
    /// Branch name (for push/pull)
    pub branch: Option<String>,

    /// Commit message (for commits)
    pub message: Option<String>,

    /// Lock timeout (for lock operations)
    pub timeout_hours: Option<u32>,

    /// Additional key-value data
    pub extra: std::collections::HashMap<String, String>,
}

impl OperationData {
    pub fn new() -> Self {
        Self {
            branch: None,
            message: None,
            timeout_hours: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn with_branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }

    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    pub fn with_timeout(mut self, hours: u32) -> Self {
        self.timeout_hours = Some(hours);
        self
    }
}

impl Default for OperationData {
    fn default() -> Self {
        Self::new()
    }
}

/// Manages offline operation queue and network retry logic
pub struct NetworkResilienceManager {
    queue_file: PathBuf,
    operations: VecDeque<QueuedOperation>,
}

impl NetworkResilienceManager {
    /// Create a new NetworkResilienceManager with default queue location
    pub fn new() -> Self {
        let queue_file = Self::default_queue_path();
        Self {
            queue_file,
            operations: VecDeque::new(),
        }
    }

    /// Create with custom queue file path
    pub fn with_queue_path(queue_file: PathBuf) -> Self {
        Self {
            queue_file,
            operations: VecDeque::new(),
        }
    }

    /// Get default queue file path (~/.auxin/operation_queue.json)
    fn default_queue_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".auxin").join("operation_queue.json")
    }

    /// Load queued operations from disk
    pub fn load_queue(&mut self) -> Result<()> {
        if !self.queue_file.exists() {
            return Ok(());
        }

        let contents = fs::read_to_string(&self.queue_file)
            .context("Failed to read operation queue file")?;

        let ops: Vec<QueuedOperation> = serde_json::from_str(&contents)
            .context("Failed to parse operation queue")?;

        self.operations = ops.into();
        Ok(())
    }

    /// Save queued operations to disk
    pub fn save_queue(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.queue_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let ops: Vec<_> = self.operations.iter().collect();
        let json = serde_json::to_string_pretty(&ops)?;

        fs::write(&self.queue_file, json)
            .context("Failed to write operation queue file")?;

        Ok(())
    }

    /// Add an operation to the queue
    pub fn enqueue(&mut self, mut operation: QueuedOperation) -> Result<()> {
        operation.queued_at = Utc::now();
        operation.attempt_count = 0;
        self.operations.push_back(operation);
        self.save_queue()?;
        Ok(())
    }

    /// Get the next operation to retry
    pub fn dequeue(&mut self) -> Option<QueuedOperation> {
        let op = self.operations.pop_front();
        if op.is_some() {
            let _ = self.save_queue();
        }
        op
    }

    /// Peek at the next operation without removing it
    pub fn peek(&self) -> Option<&QueuedOperation> {
        self.operations.front()
    }

    /// Get number of queued operations
    pub fn queue_size(&self) -> usize {
        self.operations.len()
    }

    /// Clear all queued operations
    pub fn clear_queue(&mut self) -> Result<()> {
        self.operations.clear();
        self.save_queue()
    }

    /// Mark an operation as failed and re-queue if under retry limit
    pub fn mark_failed(&mut self, mut operation: QueuedOperation, error: String) -> Result<bool> {
        operation.attempt_count += 1;
        operation.last_error = Some(error);

        if operation.attempt_count < MAX_RETRIES {
            // Re-queue for retry
            self.operations.push_back(operation);
            self.save_queue()?;
            Ok(true) // Will retry
        } else {
            // Max retries exceeded, don't re-queue
            self.save_queue()?;
            Ok(false) // Won't retry
        }
    }

    /// Execute a network operation with retry logic
    pub fn execute_with_retry<F>(&self, mut operation: F) -> Result<()>
    where
        F: FnMut() -> Result<()>,
    {
        let mut attempt = 0;
        let mut last_error = None;

        while attempt < MAX_RETRIES {
            match operation() {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);
                    attempt += 1;

                    if attempt < MAX_RETRIES {
                        // Calculate exponential backoff
                        let backoff_ms = INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1);
                        let backoff_ms = backoff_ms.min(MAX_BACKOFF_MS);

                        crate::vlog!("Retry attempt {}/{}, waiting {}ms", attempt, MAX_RETRIES, backoff_ms);
                        thread::sleep(StdDuration::from_millis(backoff_ms));
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow!("Operation failed after {} retries", MAX_RETRIES)))
    }
}

impl Default for NetworkResilienceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Detect if an error is transient (retryable) or permanent
pub fn is_transient_error(error: &anyhow::Error) -> bool {
    let error_str = error.to_string().to_lowercase();

    // Network-related errors that are typically transient
    let transient_patterns = [
        "timeout",
        "connection refused",
        "connection reset",
        "broken pipe",
        "network unreachable",
        "temporary failure",
        "502",
        "503",
        "504",
        "try again",
    ];

    transient_patterns.iter().any(|pattern| error_str.contains(pattern))
}

/// Check if network is available by attempting to connect to Oxen Hub
pub fn check_network_availability() -> bool {
    use std::process::Command;

    // Try to ping Oxen Hub
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", "hub.oxen.ai"])
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

// ========== Circuit Breaker ==========

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    /// Normal operation - requests allowed
    Closed,
    /// Failing - requests rejected
    Open,
    /// Testing recovery - limited requests allowed
    HalfOpen,
}

/// Circuit breaker for preventing cascading failures
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    failure_threshold: u32,
    success_threshold: u32,
    last_failure_time: Option<Instant>,
    timeout: StdDuration,
}

impl CircuitBreaker {
    /// Create a new circuit breaker with default settings
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold: std::env::var("AUXIN_CIRCUIT_BREAKER_THRESHOLD")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            success_threshold: 3,
            last_failure_time: None,
            timeout: StdDuration::from_secs(
                std::env::var("AUXIN_CIRCUIT_BREAKER_TIMEOUT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60),
            ),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(failure_threshold: u32, success_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            failure_threshold,
            success_threshold,
            last_failure_time: None,
            timeout: StdDuration::from_secs(timeout_secs),
        }
    }

    /// Get current circuit state
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Check if the circuit allows requests
    pub fn allow_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(last_failure) = self.last_failure_time {
                    if last_failure.elapsed() >= self.timeout {
                        // Transition to half-open
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        crate::vlog!("Circuit breaker: Open -> HalfOpen (timeout elapsed)");
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.success_threshold {
                    // Recovery successful - close circuit
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                    crate::vlog!("Circuit breaker: HalfOpen -> Closed (recovery successful)");
                }
            }
            CircuitState::Closed => {
                // Reset failure count on success
                if self.failure_count > 0 {
                    self.failure_count = self.failure_count.saturating_sub(1);
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.failure_threshold {
                    // Too many failures - open circuit
                    self.state = CircuitState::Open;
                    self.last_failure_time = Some(Instant::now());
                    crate::vlog!(
                        "Circuit breaker: Closed -> Open (failures: {})",
                        self.failure_count
                    );
                }
            }
            CircuitState::HalfOpen => {
                // Recovery failed - back to open
                self.state = CircuitState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
                crate::vlog!("Circuit breaker: HalfOpen -> Open (recovery failed)");
            }
            CircuitState::Open => {
                // Already open, update last failure time
                self.last_failure_time = Some(Instant::now());
            }
        }
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure_time = None;
    }

    /// Get statistics about circuit breaker state
    pub fn stats(&self) -> CircuitBreakerStats {
        CircuitBreakerStats {
            state: self.state,
            failure_count: self.failure_count,
            success_count: self.success_count,
            failure_threshold: self.failure_threshold,
            success_threshold: self.success_threshold,
            time_until_retry: self.time_until_retry(),
        }
    }

    fn time_until_retry(&self) -> Option<StdDuration> {
        if self.state == CircuitState::Open {
            self.last_failure_time.map(|t| {
                let elapsed = t.elapsed();
                if elapsed < self.timeout {
                    self.timeout - elapsed
                } else {
                    StdDuration::ZERO
                }
            })
        } else {
            None
        }
    }
}

impl Default for CircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about circuit breaker state
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub failure_threshold: u32,
    pub success_threshold: u32,
    pub time_until_retry: Option<StdDuration>,
}

// ========== Adaptive Retry Policy ==========

use std::time::Instant;

/// Adaptive retry policy that adjusts behavior based on error types
#[derive(Debug, Clone)]
pub struct AdaptiveRetryPolicy {
    base_policy: RetryPolicy,
    circuit_breaker: CircuitBreaker,
    verbose: bool,
}

impl AdaptiveRetryPolicy {
    /// Create a new adaptive retry policy
    pub fn new() -> Self {
        Self {
            base_policy: RetryPolicy::default(),
            circuit_breaker: CircuitBreaker::new(),
            verbose: false,
        }
    }

    /// Create with custom base policy
    pub fn with_policy(policy: RetryPolicy) -> Self {
        Self {
            base_policy: policy,
            circuit_breaker: CircuitBreaker::new(),
            verbose: false,
        }
    }

    /// Enable verbose logging
    pub fn set_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self.base_policy = self.base_policy.set_verbose(verbose);
        self
    }

    /// Get circuit breaker stats
    pub fn circuit_stats(&self) -> CircuitBreakerStats {
        self.circuit_breaker.stats()
    }

    /// Reset circuit breaker
    pub fn reset_circuit(&mut self) {
        self.circuit_breaker.reset();
    }

    /// Execute an operation with adaptive retry logic
    pub fn execute<F, T>(&mut self, mut operation: F) -> Result<T>
    where
        F: FnMut() -> Result<T>,
    {
        // Check circuit breaker first
        if !self.circuit_breaker.allow_request() {
            let stats = self.circuit_breaker.stats();
            let wait_time = stats.time_until_retry.unwrap_or(StdDuration::ZERO);
            return Err(anyhow!(
                "Circuit breaker is open. Too many recent failures. Retry in {:.0}s",
                wait_time.as_secs_f64()
            ));
        }

        let mut attempt = 0;
        let max_retries = self.base_policy.max_retries;
        let mut backoff_ms = self.base_policy.initial_backoff_ms;

        loop {
            match operation() {
                Ok(result) => {
                    self.circuit_breaker.record_success();
                    return Ok(result);
                }
                Err(e) => {
                    attempt += 1;

                    // Check if this error is retryable
                    let should_retry = is_transient_error(&e);

                    if !should_retry || attempt > max_retries {
                        self.circuit_breaker.record_failure();
                        return Err(e);
                    }

                    // Determine backoff strategy based on error type
                    let error_str = e.to_string().to_lowercase();
                    let adjusted_backoff = if error_str.contains("rate limit") || error_str.contains("429") {
                        // Linear backoff for rate limiting - use longer delays
                        backoff_ms * 3
                    } else if error_str.contains("timeout") {
                        // Slightly longer backoff for timeouts
                        backoff_ms * 2
                    } else {
                        // Standard exponential backoff
                        backoff_ms
                    };

                    // Add jitter to prevent thundering herd (±10%)
                    let jitter = (adjusted_backoff as f64 * 0.1) as u64;
                    let jittered_backoff = adjusted_backoff + (attempt as u64 % (2 * jitter + 1)).saturating_sub(jitter);
                    let final_backoff = jittered_backoff.min(self.base_policy.max_backoff_ms);

                    if self.verbose {
                        crate::vlog!(
                            "Retry attempt {}/{}: {} (waiting {}ms)",
                            attempt,
                            max_retries,
                            e,
                            final_backoff
                        );
                    }

                    thread::sleep(StdDuration::from_millis(final_backoff));

                    // Update backoff for next iteration
                    backoff_ms = (backoff_ms * 2).min(self.base_policy.max_backoff_ms);
                }
            }
        }
    }
}

impl Default for AdaptiveRetryPolicy {
    fn default() -> Self {
        Self::new()
    }
}

// ========== Network Health Monitor ==========

/// Network health status
#[derive(Debug, Clone)]
pub struct NetworkHealth {
    /// Whether the network is currently available
    pub available: bool,
    /// Latency to hub.oxen.ai in milliseconds
    pub latency_ms: Option<u64>,
    /// Estimated bandwidth (if available)
    pub bandwidth_estimate: Option<String>,
    /// Last check timestamp
    pub last_checked: DateTime<Utc>,
    /// Connection quality rating
    pub quality: NetworkQuality,
}

/// Network quality rating
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkQuality {
    /// Excellent connection (< 50ms latency)
    Excellent,
    /// Good connection (50-150ms latency)
    Good,
    /// Fair connection (150-300ms latency)
    Fair,
    /// Poor connection (> 300ms latency)
    Poor,
    /// No connection
    Offline,
}

impl std::fmt::Display for NetworkQuality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkQuality::Excellent => write!(f, "Excellent"),
            NetworkQuality::Good => write!(f, "Good"),
            NetworkQuality::Fair => write!(f, "Fair"),
            NetworkQuality::Poor => write!(f, "Poor"),
            NetworkQuality::Offline => write!(f, "Offline"),
        }
    }
}

/// Check network health with detailed diagnostics
pub fn check_network_health() -> NetworkHealth {
    use std::process::Command;

    let _start = Instant::now();

    // Try to ping Oxen Hub with timing
    let output = Command::new("ping")
        .args(["-c", "3", "-W", "5", "hub.oxen.ai"])
        .output();

    let (available, latency_ms) = match output {
        Ok(out) => {
            if out.status.success() {
                // Parse latency from ping output
                let stdout = String::from_utf8_lossy(&out.stdout);
                let latency = parse_ping_latency(&stdout);
                (true, latency)
            } else {
                (false, None)
            }
        }
        Err(_) => (false, None),
    };

    let quality = match latency_ms {
        Some(ms) if ms < 50 => NetworkQuality::Excellent,
        Some(ms) if ms < 150 => NetworkQuality::Good,
        Some(ms) if ms < 300 => NetworkQuality::Fair,
        Some(_) => NetworkQuality::Poor,
        None => NetworkQuality::Offline,
    };

    NetworkHealth {
        available,
        latency_ms,
        bandwidth_estimate: None, // Could be enhanced with speed test
        last_checked: Utc::now(),
        quality,
    }
}

/// Parse average latency from ping output
fn parse_ping_latency(output: &str) -> Option<u64> {
    // Look for patterns like "min/avg/max/mdev = 10.123/15.456/20.789/3.456 ms"
    // or "round-trip min/avg/max/stddev = 10.123/15.456/20.789/3.456 ms"
    for line in output.lines() {
        if line.contains("avg") || line.contains("average") {
            // Try to extract the average value after the = sign
            if let Some(eq_idx) = line.find('=') {
                let after_eq = &line[eq_idx + 1..].trim();
                // Format: 10.123/15.456/20.789/3.456 ms
                // We want the second value (avg)
                if let Some(start) = after_eq.find('/') {
                    let after_first_slash = &after_eq[start + 1..];
                    if let Some(end) = after_first_slash.find('/') {
                        let avg_str = &after_first_slash[..end];
                        if let Ok(avg) = avg_str.parse::<f64>() {
                            return Some(avg as u64);
                        }
                    }
                }
            }
        }
    }

    // Fallback: look for "time=X ms" pattern
    for line in output.lines() {
        if let Some(time_idx) = line.find("time=") {
            let after_time = &line[time_idx + 5..];
            let value: String = after_time.chars().take_while(|c| c.is_numeric() || *c == '.').collect();
            if let Ok(ms) = value.parse::<f64>() {
                return Some(ms as u64);
            }
        }
    }

    None
}

/// Estimate transfer time for a file of given size
pub fn estimate_transfer_time(file_size_bytes: u64, latency_ms: Option<u64>) -> String {
    // Rough bandwidth estimates based on latency
    let bandwidth_mbps = match latency_ms {
        Some(ms) if ms < 50 => 100.0,  // Excellent: ~100 Mbps
        Some(ms) if ms < 150 => 50.0,  // Good: ~50 Mbps
        Some(ms) if ms < 300 => 20.0,  // Fair: ~20 Mbps
        Some(_) => 5.0,                 // Poor: ~5 Mbps
        None => 1.0,                    // Unknown: assume slow
    };

    let bytes_per_second = bandwidth_mbps * 1_000_000.0 / 8.0;
    let seconds = file_size_bytes as f64 / bytes_per_second;

    if seconds < 60.0 {
        format!("{:.0}s", seconds)
    } else if seconds < 3600.0 {
        format!("{:.1}m", seconds / 60.0)
    } else {
        format!("{:.1}h", seconds / 3600.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_network_resilience_manager_creation() {
        let manager = NetworkResilienceManager::new();
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_enqueue_and_dequeue() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        let op = QueuedOperation {
            id: "test-1".to_string(),
            operation_type: OperationType::Push,
            repo_path: PathBuf::from("/test/repo"),
            data: OperationData::new().with_branch("main"),
            queued_at: Utc::now(),
            attempt_count: 0,
            last_error: None,
        };

        manager.enqueue(op.clone()).unwrap();
        assert_eq!(manager.queue_size(), 1);

        let dequeued = manager.dequeue().unwrap();
        assert_eq!(dequeued.id, "test-1");
        assert_eq!(manager.queue_size(), 0);
    }

    #[test]
    fn test_queue_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");

        // Create and enqueue
        {
            let mut manager = NetworkResilienceManager::with_queue_path(queue_file.clone());
            let op = QueuedOperation {
                id: "persist-test".to_string(),
                operation_type: OperationType::LockAcquire,
                repo_path: PathBuf::from("/test"),
                data: OperationData::new().with_timeout(4),
                queued_at: Utc::now(),
                attempt_count: 0,
                last_error: None,
            };
            manager.enqueue(op).unwrap();
        }

        // Load in new instance
        {
            let mut manager = NetworkResilienceManager::with_queue_path(queue_file);
            manager.load_queue().unwrap();
            assert_eq!(manager.queue_size(), 1);

            let op = manager.peek().unwrap();
            assert_eq!(op.id, "persist-test");
        }
    }

    #[test]
    fn test_mark_failed_retry_logic() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        let op = QueuedOperation {
            id: "retry-test".to_string(),
            operation_type: OperationType::Push,
            repo_path: PathBuf::from("/test"),
            data: OperationData::new(),
            queued_at: Utc::now(),
            attempt_count: 0,
            last_error: None,
        };

        // First 3 failures should re-queue
        let will_retry = manager.mark_failed(op.clone(), "Network error".to_string()).unwrap();
        assert!(will_retry);
        assert_eq!(manager.queue_size(), 1);

        // After MAX_RETRIES, should not re-queue
        let mut op_max = op.clone();
        op_max.attempt_count = MAX_RETRIES - 1;
        let will_retry = manager.mark_failed(op_max, "Network error".to_string()).unwrap();
        assert!(!will_retry);
    }

    #[test]
    fn test_is_transient_error() {
        let timeout_err = anyhow!("Connection timeout");
        assert!(is_transient_error(&timeout_err));

        let refused_err = anyhow!("Connection refused");
        assert!(is_transient_error(&refused_err));

        let auth_err = anyhow!("Authentication failed");
        assert!(!is_transient_error(&auth_err));
    }

    #[test]
    fn test_operation_data_builder() {
        let data = OperationData::new()
            .with_branch("main")
            .with_message("Test commit")
            .with_timeout(4);

        assert_eq!(data.branch, Some("main".to_string()));
        assert_eq!(data.message, Some("Test commit".to_string()));
        assert_eq!(data.timeout_hours, Some(4));
    }

    #[test]
    fn test_clear_queue() {
        let temp_dir = TempDir::new().unwrap();
        let queue_file = temp_dir.path().join("queue.json");
        let mut manager = NetworkResilienceManager::with_queue_path(queue_file);

        // Add multiple operations
        for i in 0..5 {
            let op = QueuedOperation {
                id: format!("op-{}", i),
                operation_type: OperationType::Push,
                repo_path: PathBuf::from("/test"),
                data: OperationData::new(),
                queued_at: Utc::now(),
                attempt_count: 0,
                last_error: None,
            };
            manager.enqueue(op).unwrap();
        }

        assert_eq!(manager.queue_size(), 5);

        manager.clear_queue().unwrap();
        assert_eq!(manager.queue_size(), 0);
    }

    // ========== Circuit Breaker Tests ==========

    #[test]
    fn test_circuit_breaker_initial_state() {
        let cb = CircuitBreaker::new();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut cb = CircuitBreaker::with_thresholds(3, 2, 60);

        // Should be closed initially
        assert!(cb.allow_request());

        // Record failures
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Should not allow requests when open
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_circuit_breaker_success_reduces_failures() {
        let mut cb = CircuitBreaker::with_thresholds(5, 2, 60);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count, 2);

        cb.record_success();
        assert_eq!(cb.failure_count, 1);
    }

    #[test]
    fn test_circuit_breaker_recovery() {
        let mut cb = CircuitBreaker::with_thresholds(2, 2, 0); // 0 timeout for instant recovery

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Should transition to half-open after timeout
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Successful operations should close it
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_circuit_breaker_half_open_failure() {
        let mut cb = CircuitBreaker::with_thresholds(2, 2, 0);

        // Open and transition to half-open
        cb.record_failure();
        cb.record_failure();
        cb.allow_request();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        // Failure in half-open should go back to open
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_circuit_breaker_reset() {
        let mut cb = CircuitBreaker::with_thresholds(2, 2, 60);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count, 0);
    }

    #[test]
    fn test_circuit_breaker_stats() {
        let mut cb = CircuitBreaker::with_thresholds(5, 3, 60);

        cb.record_failure();
        cb.record_failure();

        let stats = cb.stats();
        assert_eq!(stats.state, CircuitState::Closed);
        assert_eq!(stats.failure_count, 2);
        assert_eq!(stats.failure_threshold, 5);
        assert_eq!(stats.success_threshold, 3);
    }

    // ========== Adaptive Retry Policy Tests ==========

    #[test]
    fn test_adaptive_retry_policy_success() {
        let mut policy = AdaptiveRetryPolicy::new();

        let result = policy.execute(|| Ok(42));
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_adaptive_retry_policy_eventual_success() {
        let mut policy = AdaptiveRetryPolicy::with_policy(
            RetryPolicy::new(3, 10, 100)
        );

        let mut attempt = 0;
        let result = policy.execute(|| {
            attempt += 1;
            if attempt < 3 {
                Err(anyhow!("Connection timeout"))
            } else {
                Ok(42)
            }
        });

        assert_eq!(result.unwrap(), 42);
        assert_eq!(attempt, 3);
    }

    #[test]
    fn test_adaptive_retry_policy_max_retries() {
        let mut policy = AdaptiveRetryPolicy::with_policy(
            RetryPolicy::new(2, 10, 100)
        );

        let mut attempt = 0;
        let result: Result<i32> = policy.execute(|| {
            attempt += 1;
            Err(anyhow!("Connection refused"))
        });

        assert!(result.is_err());
        assert_eq!(attempt, 3); // Initial + 2 retries
    }

    #[test]
    fn test_adaptive_retry_policy_non_retryable_error() {
        let mut policy = AdaptiveRetryPolicy::new();

        let mut attempt = 0;
        let result: Result<i32> = policy.execute(|| {
            attempt += 1;
            Err(anyhow!("Permission denied"))
        });

        assert!(result.is_err());
        assert_eq!(attempt, 1); // No retries for non-transient errors
    }

    #[test]
    fn test_adaptive_retry_policy_circuit_breaker_integration() {
        let mut policy = AdaptiveRetryPolicy::with_policy(
            RetryPolicy::new(1, 10, 100)
        );

        // Trigger multiple failures to open circuit
        for _ in 0..5 {
            let _result: Result<i32> = policy.execute(|| {
                Err(anyhow!("Network error"))
            });
        }

        // Circuit should be open now
        let stats = policy.circuit_stats();
        assert_eq!(stats.state, CircuitState::Open);

        // Should fail immediately due to open circuit
        let result: Result<i32> = policy.execute(|| Ok(42));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Circuit breaker"));
    }

    #[test]
    fn test_adaptive_retry_policy_reset_circuit() {
        let mut policy = AdaptiveRetryPolicy::new();

        // Open circuit
        for _ in 0..5 {
            let _: Result<i32> = policy.execute(|| Err(anyhow!("timeout")));
        }

        assert_eq!(policy.circuit_stats().state, CircuitState::Open);

        policy.reset_circuit();
        assert_eq!(policy.circuit_stats().state, CircuitState::Closed);
    }

    // ========== Network Health Tests ==========

    #[test]
    fn test_parse_ping_latency_linux_format() {
        let output = "rtt min/avg/max/mdev = 10.123/15.456/20.789/3.456 ms";
        let latency = parse_ping_latency(output);
        assert_eq!(latency, Some(15));
    }

    #[test]
    fn test_parse_ping_latency_macos_format() {
        let output = "round-trip min/avg/max/stddev = 10.123/15.456/20.789/3.456 ms";
        let latency = parse_ping_latency(output);
        assert_eq!(latency, Some(15));
    }

    #[test]
    fn test_parse_ping_latency_time_format() {
        let output = "64 bytes from 1.2.3.4: icmp_seq=1 ttl=54 time=25.6 ms";
        let latency = parse_ping_latency(output);
        assert_eq!(latency, Some(25));
    }

    #[test]
    fn test_parse_ping_latency_no_match() {
        let output = "no latency info here";
        let latency = parse_ping_latency(output);
        assert_eq!(latency, None);
    }

    #[test]
    fn test_estimate_transfer_time_small_file() {
        let estimate = estimate_transfer_time(1_000_000, Some(50)); // 1MB, good connection
        assert!(estimate.contains("s")); // Should be seconds
    }

    #[test]
    fn test_estimate_transfer_time_large_file() {
        let estimate = estimate_transfer_time(1_000_000_000, Some(200)); // 1GB, fair connection
        assert!(estimate.contains("m") || estimate.contains("h")); // Should be minutes or hours
    }

    #[test]
    fn test_network_quality_display() {
        assert_eq!(format!("{}", NetworkQuality::Excellent), "Excellent");
        assert_eq!(format!("{}", NetworkQuality::Good), "Good");
        assert_eq!(format!("{}", NetworkQuality::Fair), "Fair");
        assert_eq!(format!("{}", NetworkQuality::Poor), "Poor");
        assert_eq!(format!("{}", NetworkQuality::Offline), "Offline");
    }

    #[test]
    fn test_connectivity_state() {
        // Just test that the function doesn't panic
        let _ = check_connectivity();
    }

    #[test]
    fn test_retry_policy_builder() {
        let policy = RetryPolicy::new(5, 1000, 10000).set_verbose(true);
        assert_eq!(policy.max_retries, 5);
        assert_eq!(policy.initial_backoff_ms, 1000);
        assert_eq!(policy.max_backoff_ms, 10000);
        assert!(policy.verbose);
    }
}
