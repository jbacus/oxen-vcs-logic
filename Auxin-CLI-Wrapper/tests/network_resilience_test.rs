/// Integration tests for network resilience functionality
///
/// Tests Phase 6 features including:
/// - Retry logic with exponential backoff
/// - Offline queue management
/// - Chunked upload with resume
/// - Circuit breaker pattern
/// - Connection health monitoring

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use auxin::network_resilience::{
        RetryPolicy, CircuitBreaker, NetworkHealthMonitor, OfflineQueue,
        ChunkedUploadManager, NetworkQuality, RetryableError
    };
    use std::time::{Duration, Instant};
    use tempfile::TempDir;

    // ===================
    // RetryPolicy Tests
    // ===================

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();

        assert_eq!(policy.max_attempts, 4);
        assert_eq!(policy.base_delay_ms, 2000);
        assert_eq!(policy.max_delay_ms, 16000);
    }

    #[test]
    fn test_retry_policy_custom() {
        let policy = RetryPolicy::new(5, 1000, 30000);

        assert_eq!(policy.max_attempts, 5);
        assert_eq!(policy.base_delay_ms, 1000);
        assert_eq!(policy.max_delay_ms, 30000);
    }

    #[test]
    fn test_retry_policy_exponential_backoff() {
        let policy = RetryPolicy::new(4, 1000, 16000);

        // First retry: 1000ms
        assert_eq!(policy.delay_for_attempt(1), Duration::from_millis(1000));

        // Second retry: 2000ms
        assert_eq!(policy.delay_for_attempt(2), Duration::from_millis(2000));

        // Third retry: 4000ms
        assert_eq!(policy.delay_for_attempt(3), Duration::from_millis(4000));

        // Fourth retry: 8000ms
        assert_eq!(policy.delay_for_attempt(4), Duration::from_millis(8000));
    }

    #[test]
    fn test_retry_policy_max_delay_cap() {
        let policy = RetryPolicy::new(10, 2000, 16000);

        // Should cap at max_delay_ms
        let delay = policy.delay_for_attempt(10);
        assert!(delay.as_millis() <= 16000);
    }

    #[test]
    fn test_retry_policy_should_retry() {
        let policy = RetryPolicy::new(3, 1000, 8000);

        assert!(policy.should_retry(1));
        assert!(policy.should_retry(2));
        assert!(policy.should_retry(3));
        assert!(!policy.should_retry(4));
    }

    #[test]
    fn test_retry_policy_classify_error_network() {
        let policy = RetryPolicy::default();

        // Network errors should be retryable
        assert!(policy.is_retryable("connection refused"));
        assert!(policy.is_retryable("network timeout"));
        assert!(policy.is_retryable("DNS resolution failed"));
    }

    #[test]
    fn test_retry_policy_classify_error_auth() {
        let policy = RetryPolicy::default();

        // Auth errors should NOT be retryable
        assert!(!policy.is_retryable("unauthorized"));
        assert!(!policy.is_retryable("forbidden"));
        assert!(!policy.is_retryable("invalid credentials"));
    }

    #[test]
    fn test_retry_policy_classify_error_rate_limit() {
        let policy = RetryPolicy::default();

        // Rate limits ARE retryable
        assert!(policy.is_retryable("rate limited"));
        assert!(policy.is_retryable("too many requests"));
    }

    // ===================
    // CircuitBreaker Tests
    // ===================

    #[test]
    fn test_circuit_breaker_initial_state() {
        let breaker = CircuitBreaker::new(5, Duration::from_secs(60));

        assert!(breaker.is_closed());
        assert!(!breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(60));

        // Record failures
        breaker.record_failure();
        assert!(breaker.is_closed());

        breaker.record_failure();
        assert!(breaker.is_closed());

        breaker.record_failure();
        assert!(breaker.is_open()); // Should be open now
    }

    #[test]
    fn test_circuit_breaker_success_resets() {
        let mut breaker = CircuitBreaker::new(3, Duration::from_secs(60));

        // Record some failures
        breaker.record_failure();
        breaker.record_failure();

        // Success should reset counter
        breaker.record_success();

        // Should need 3 more failures to open
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.is_closed());

        breaker.record_failure();
        assert!(breaker.is_open());
    }

    #[test]
    fn test_circuit_breaker_allow_request() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_secs(1));

        // Initially allows requests
        assert!(breaker.allow_request());

        // After failures, blocks requests
        breaker.record_failure();
        breaker.record_failure();

        assert!(!breaker.allow_request());
    }

    #[test]
    fn test_circuit_breaker_half_open() {
        let mut breaker = CircuitBreaker::new(2, Duration::from_millis(100));

        // Open the circuit
        breaker.record_failure();
        breaker.record_failure();
        assert!(breaker.is_open());

        // Wait for reset timeout
        std::thread::sleep(Duration::from_millis(150));

        // Should allow a test request (half-open)
        assert!(breaker.allow_request());
    }

    // ===================
    // NetworkHealthMonitor Tests
    // ===================

    #[test]
    fn test_network_health_monitor_initial() {
        let monitor = NetworkHealthMonitor::new();

        // Initial state should be unknown or checking
        let quality = monitor.get_quality();
        // Accept any initial state
    }

    #[test]
    fn test_network_quality_excellent() {
        let quality = NetworkQuality::Excellent;
        assert!(quality.is_usable());
        assert!(!quality.is_degraded());
    }

    #[test]
    fn test_network_quality_good() {
        let quality = NetworkQuality::Good;
        assert!(quality.is_usable());
        assert!(!quality.is_degraded());
    }

    #[test]
    fn test_network_quality_fair() {
        let quality = NetworkQuality::Fair;
        assert!(quality.is_usable());
        assert!(quality.is_degraded());
    }

    #[test]
    fn test_network_quality_poor() {
        let quality = NetworkQuality::Poor;
        assert!(quality.is_usable());
        assert!(quality.is_degraded());
    }

    #[test]
    fn test_network_quality_offline() {
        let quality = NetworkQuality::Offline;
        assert!(!quality.is_usable());
        assert!(quality.is_degraded());
    }

    #[test]
    fn test_network_health_from_latency() {
        // Excellent: < 50ms
        assert_eq!(NetworkQuality::from_latency(30), NetworkQuality::Excellent);

        // Good: 50-100ms
        assert_eq!(NetworkQuality::from_latency(75), NetworkQuality::Good);

        // Fair: 100-300ms
        assert_eq!(NetworkQuality::from_latency(200), NetworkQuality::Fair);

        // Poor: > 300ms
        assert_eq!(NetworkQuality::from_latency(500), NetworkQuality::Poor);
    }

    // ===================
    // OfflineQueue Tests
    // ===================

    #[test]
    fn test_offline_queue_init() {
        let temp_dir = TempDir::new().unwrap();
        let queue = OfflineQueue::new(temp_dir.path());

        assert!(queue.init().is_ok());
    }

    #[test]
    fn test_offline_queue_add_commit() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();

        let result = queue.add_commit(
            temp_dir.path(),
            "Test commit message",
            None,
        );

        assert!(result.is_ok());
        assert_eq!(queue.pending_count(), 1);
    }

    #[test]
    fn test_offline_queue_add_multiple() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();

        for i in 0..5 {
            queue.add_commit(
                temp_dir.path(),
                &format!("Commit {}", i),
                None,
            ).unwrap();
        }

        assert_eq!(queue.pending_count(), 5);
    }

    #[test]
    fn test_offline_queue_list_pending() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();

        queue.add_commit(temp_dir.path(), "First", None).unwrap();
        queue.add_commit(temp_dir.path(), "Second", None).unwrap();

        let pending = queue.list_pending().unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn test_offline_queue_clear() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();

        queue.add_commit(temp_dir.path(), "Test", None).unwrap();
        assert_eq!(queue.pending_count(), 1);

        queue.clear().unwrap();
        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_offline_queue_persistence() {
        let temp_dir = TempDir::new().unwrap();

        // Create queue and add item
        {
            let mut queue = OfflineQueue::new(temp_dir.path());
            queue.init().unwrap();
            queue.add_commit(temp_dir.path(), "Persistent", None).unwrap();
        }

        // Create new queue instance and verify persistence
        {
            let queue = OfflineQueue::new(temp_dir.path());
            assert_eq!(queue.pending_count(), 1);
        }
    }

    #[test]
    fn test_offline_queue_process_item() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();

        queue.add_commit(temp_dir.path(), "To process", None).unwrap();

        // Process the item (mark as done)
        let pending = queue.list_pending().unwrap();
        queue.mark_completed(&pending[0].id).unwrap();

        assert_eq!(queue.pending_count(), 0);
    }

    #[test]
    fn test_offline_queue_max_size() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        queue.init().unwrap();
        queue.set_max_size(3);

        // Add up to limit
        for i in 0..3 {
            assert!(queue.add_commit(temp_dir.path(), &format!("Commit {}", i), None).is_ok());
        }

        // Should fail at limit
        let result = queue.add_commit(temp_dir.path(), "Over limit", None);
        assert!(result.is_err());
    }

    // ===================
    // ChunkedUploadManager Tests
    // ===================

    #[test]
    fn test_chunked_upload_manager_new() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ChunkedUploadManager::new(temp_dir.path());

        assert!(manager.is_ok());
    }

    #[test]
    fn test_chunked_upload_calculate_chunks() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        // 10MB file with 1MB chunks = 10 chunks
        let chunks = manager.calculate_chunks(10 * 1024 * 1024, 1024 * 1024);
        assert_eq!(chunks, 10);

        // 1.5MB file with 1MB chunks = 2 chunks
        let chunks = manager.calculate_chunks(1536 * 1024, 1024 * 1024);
        assert_eq!(chunks, 2);
    }

    #[test]
    fn test_chunked_upload_session_create() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        // Create test file
        let file_path = temp_dir.path().join("large_file.bin");
        std::fs::write(&file_path, vec![0u8; 5 * 1024 * 1024]).unwrap();

        let session = manager.create_session(&file_path, "test_remote");
        assert!(session.is_ok());
    }

    #[test]
    fn test_chunked_upload_session_progress() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("test.bin");
        std::fs::write(&file_path, vec![0u8; 1024 * 1024]).unwrap();

        let session_id = manager.create_session(&file_path, "remote").unwrap();

        // Initially 0% progress
        let progress = manager.get_progress(&session_id).unwrap();
        assert_eq!(progress.completed_chunks, 0);

        // Mark chunk complete
        manager.mark_chunk_complete(&session_id, 0).unwrap();

        let progress = manager.get_progress(&session_id).unwrap();
        assert_eq!(progress.completed_chunks, 1);
    }

    #[test]
    fn test_chunked_upload_resume() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("resume_test.bin");
        std::fs::write(&file_path, vec![0u8; 3 * 1024 * 1024]).unwrap();

        let session_id = manager.create_session(&file_path, "remote").unwrap();

        // Complete first 2 chunks
        manager.mark_chunk_complete(&session_id, 0).unwrap();
        manager.mark_chunk_complete(&session_id, 1).unwrap();

        // Get next chunk to upload
        let next = manager.next_chunk(&session_id).unwrap();
        assert_eq!(next, Some(2));
    }

    #[test]
    fn test_chunked_upload_session_complete() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("complete_test.bin");
        std::fs::write(&file_path, vec![0u8; 1024 * 1024]).unwrap(); // 1 chunk

        let session_id = manager.create_session(&file_path, "remote").unwrap();
        manager.mark_chunk_complete(&session_id, 0).unwrap();

        assert!(manager.is_complete(&session_id).unwrap());
    }

    #[test]
    fn test_chunked_upload_abort_session() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("abort_test.bin");
        std::fs::write(&file_path, vec![0u8; 1024 * 1024]).unwrap();

        let session_id = manager.create_session(&file_path, "remote").unwrap();

        manager.abort_session(&session_id).unwrap();

        // Session should no longer exist
        assert!(manager.get_progress(&session_id).is_err());
    }

    #[test]
    fn test_chunked_upload_bandwidth_estimation() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = ChunkedUploadManager::new(temp_dir.path()).unwrap();

        let file_path = temp_dir.path().join("bandwidth_test.bin");
        std::fs::write(&file_path, vec![0u8; 2 * 1024 * 1024]).unwrap();

        let session_id = manager.create_session(&file_path, "remote").unwrap();

        // Record upload time for chunk
        let start = Instant::now();
        std::thread::sleep(Duration::from_millis(100));
        manager.record_chunk_time(&session_id, 0, start.elapsed()).unwrap();

        // Get ETA
        let eta = manager.estimate_remaining_time(&session_id);
        assert!(eta.is_ok());
    }

    // ===================
    // Integration Tests
    // ===================

    #[test]
    fn test_retry_with_circuit_breaker() {
        let policy = RetryPolicy::new(3, 100, 1000);
        let mut breaker = CircuitBreaker::new(2, Duration::from_secs(60));

        // Simulate retries that trip circuit breaker
        for attempt in 1..=3 {
            if !breaker.allow_request() {
                break;
            }

            // Simulate failure
            breaker.record_failure();

            if policy.should_retry(attempt) {
                std::thread::sleep(policy.delay_for_attempt(attempt));
            }
        }

        // Circuit should be open after failures
        assert!(breaker.is_open());
    }

    #[test]
    fn test_offline_queue_with_network_check() {
        let temp_dir = TempDir::new().unwrap();
        let mut queue = OfflineQueue::new(temp_dir.path());
        let monitor = NetworkHealthMonitor::new();
        queue.init().unwrap();

        // Add items to queue
        queue.add_commit(temp_dir.path(), "Offline commit", None).unwrap();

        // Check network before syncing
        let quality = monitor.get_quality();
        if quality.is_usable() {
            // Would sync here
        } else {
            // Stay queued
            assert_eq!(queue.pending_count(), 1);
        }
    }

    // ===================
    // Error Handling Tests
    // ===================

    #[test]
    fn test_retryable_error_types() {
        use auxin::network_resilience::ErrorKind;

        let network_err = RetryableError::new(ErrorKind::Network, "Connection failed");
        assert!(network_err.is_retryable());

        let auth_err = RetryableError::new(ErrorKind::Auth, "Unauthorized");
        assert!(!auth_err.is_retryable());

        let rate_err = RetryableError::new(ErrorKind::RateLimit, "Too many requests");
        assert!(rate_err.is_retryable());

        let server_err = RetryableError::new(ErrorKind::Server, "Internal error");
        assert!(server_err.is_retryable());
    }

    #[test]
    fn test_error_classification() {
        let policy = RetryPolicy::default();

        // Should classify correctly
        assert!(policy.is_retryable("ECONNREFUSED"));
        assert!(policy.is_retryable("ETIMEDOUT"));
        assert!(policy.is_retryable("ENOTFOUND"));
        assert!(!policy.is_retryable("EACCES"));
        assert!(!policy.is_retryable("EPERM"));
    }
}
