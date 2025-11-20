// Integration tests to verify feature flag behavior
// Tests that the correct implementation is compiled based on features

#[test]
fn test_mock_feature_is_active() {
    // This test verifies we're using the mock implementation
    // by checking that it compiles and can initialize repositories

    use auxin_server::repo::RepositoryOps;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let repo = RepositoryOps::init(temp_dir.path()).unwrap();

    // In mock mode, init should create basic directory structure
    // even without the Oxen CLI being available
    assert!(temp_dir.path().join(".oxen").exists());
    assert!(temp_dir.path().join(".oxen/metadata").exists());
    assert!(temp_dir.path().join(".oxen/locks").exists());

    // Verify we can get the path
    assert_eq!(repo.path(), temp_dir.path());
}

#[test]
fn test_auxin_features_available() {
    // Verify Auxin-specific features are available regardless of feature flag
    use auxin_server::repo::RepositoryOps;
    use auxin_server::extensions::LogicProMetadata;
    use tempfile::TempDir;

    let temp_dir = TempDir::new().unwrap();
    let repo = RepositoryOps::init(temp_dir.path()).unwrap();

    // Metadata operations should always work
    let metadata = LogicProMetadata {
        bpm: Some(128.0),
        sample_rate: Some(48000),
        key_signature: Some("A Minor".to_string()),
        tags: vec![],
    };

    assert!(repo.store_metadata("test-id", &metadata).is_ok());
    assert!(repo.get_metadata("test-id").is_ok());

    // Lock operations should always work
    assert!(repo.acquire_lock("user", "machine", 24).is_ok());
}

#[cfg(feature = "mock-oxen")]
#[test]
fn test_mock_oxen_feature_enabled() {
    // This test only compiles if mock-oxen feature is enabled
    println!("mock-oxen feature is enabled");
}

#[cfg(feature = "full-oxen")]
#[test]
fn test_full_oxen_feature_enabled() {
    // This test only compiles if full-oxen feature is enabled
    println!("full-oxen feature is enabled");
}

#[cfg(not(any(feature = "mock-oxen", feature = "full-oxen")))]
#[test]
fn test_no_oxen_feature_enabled() {
    // This should never happen with current config
    panic!("No Oxen feature enabled!");
}
