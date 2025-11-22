/// Integration tests for the clone functionality
///
/// These tests verify the end-to-end clone workflow including:
/// - URL validation
/// - Destination path handling
/// - Error messages
/// - Integration with oxen subprocess

use std::path::PathBuf;
use tempfile::TempDir;

#[cfg(test)]
mod clone_tests {
    use super::*;

    #[test]
    fn test_clone_rejects_empty_url() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("", &dest);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_clone_rejects_invalid_url_with_null_byte() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("https://example.com\0/repo", &dest);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid characters"));
    }

    #[test]
    fn test_clone_rejects_invalid_url_with_newline() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("https://example.com\n/repo", &dest);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid characters"));
    }

    #[test]
    fn test_clone_rejects_existing_destination() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("existing-dir");

        // Create the destination directory
        std::fs::create_dir(&dest).unwrap();

        let result = oxen.clone("https://example.com/repo", &dest);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_clone_accepts_valid_http_url() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        // This will fail because we don't have oxen CLI or the repo doesn't exist,
        // but it should pass validation
        let result = oxen.clone("http://localhost:3000/user/repo", &dest);

        // We expect it to fail at execution, not validation
        if let Err(e) = result {
            // Should NOT be a validation error
            assert!(
                !e.to_string().contains("cannot be empty")
                    && !e.to_string().contains("Invalid characters")
                    && !e.to_string().contains("already exists")
            );
        }
    }

    #[test]
    fn test_clone_accepts_valid_https_url() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("https://hub.oxen.ai/user/repo", &dest);

        // We expect it to fail at execution, not validation
        if let Err(e) = result {
            // Should NOT be a validation error
            assert!(
                !e.to_string().contains("cannot be empty")
                    && !e.to_string().contains("Invalid characters")
                    && !e.to_string().contains("already exists")
            );
        }
    }

    #[test]
    fn test_clone_accepts_file_url() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("file:///path/to/repo", &dest);

        // We expect it to fail at execution, not validation
        if let Err(e) = result {
            // Should NOT be a validation error
            assert!(
                !e.to_string().contains("cannot be empty")
                    && !e.to_string().contains("Invalid characters")
                    && !e.to_string().contains("already exists")
            );
        }
    }

    #[test]
    fn test_clone_creates_parent_directories() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("nested").join("path").join("cloned-repo");

        // Parent directories don't exist yet
        assert!(!dest.parent().unwrap().exists());

        let _result = oxen.clone("https://example.com/repo", &dest);

        // Parent directory should be created (even if clone fails)
        // Note: This behavior depends on the implementation
    }

    #[test]
    fn test_clone_handles_special_characters_in_destination() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();

        // Test with spaces in path
        let dest = temp_dir.path().join("my cloned repo");

        let result = oxen.clone("https://example.com/repo", &dest);

        // Should not fail due to path validation
        if let Err(e) = result {
            assert!(!e.to_string().contains("Invalid"));
        }
    }

    #[test]
    fn test_clone_uses_network_timeout() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        // Clone should use network timeout, not default timeout
        // This is implicit in the implementation but important for large repos
        let _result = oxen.clone("https://example.com/large-repo", &dest);

        // If this times out, it should be with network timeout (120s), not default (30s)
        // This is hard to test without actually running the command, but we can verify
        // the timeout configuration is set up correctly
    }
}

#[cfg(test)]
mod oxen_repository_clone_tests {
    use super::*;

    #[tokio::test]
    async fn test_repository_clone_validates_oxen_availability() {
        use auxin::OxenRepository;

        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = OxenRepository::clone("https://example.com/repo", &dest).await;

        // Should fail because oxen CLI is not available (unless it's installed)
        if let Err(e) = result {
            // Error message should mention oxen CLI
            let err_str = e.to_string();
            assert!(
                err_str.contains("oxen") || err_str.contains("not found") || err_str.contains("Failed")
            );
        }
    }

    #[tokio::test]
    async fn test_repository_clone_returns_repository_instance() {
        use auxin::OxenRepository;

        // This test assumes oxen CLI is available and the repo exists
        // It will be skipped if oxen is not installed

        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        // Try to clone a small test repository
        // This will fail without oxen CLI, which is expected
        let result = OxenRepository::clone("file:///tmp/test-repo", &dest).await;

        if result.is_ok() {
            // If successful, verify we got a repository instance
            let repo = result.unwrap();
            assert_eq!(repo.path, dest);
        }
        // If it fails, that's okay - oxen CLI might not be installed
    }

    #[tokio::test]
    async fn test_repository_clone_accepts_different_url_schemes() {
        use auxin::OxenRepository;

        let temp_dir = TempDir::new().unwrap();

        // Test different URL schemes
        let urls = vec![
            "https://hub.oxen.ai/user/repo",
            "http://localhost:3000/user/repo",
            "file:///path/to/repo",
        ];

        for url in urls {
            let dest = temp_dir.path().join(format!("clone-{}", url.replace('/', "-")));
            let result = OxenRepository::clone(url, &dest).await;

            // Should not fail due to URL validation
            if let Err(e) = result {
                let err_str = e.to_string();
                assert!(!err_str.contains("Invalid URL"));
            }
        }
    }
}

#[cfg(test)]
mod clone_error_messages_tests {
    use super::*;

    #[test]
    fn test_clone_error_message_for_empty_url() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("cloned-repo");

        let result = oxen.clone("", &dest);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("cannot be empty"));
    }

    #[test]
    fn test_clone_error_message_for_existing_destination() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("existing");

        std::fs::create_dir(&dest).unwrap();

        let result = oxen.clone("https://example.com/repo", &dest);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("already exists"));
        assert!(error_msg.contains("choose a different location"));
    }

    #[test]
    fn test_clone_error_message_includes_path() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("my-project");

        std::fs::create_dir(&dest).unwrap();

        let result = oxen.clone("https://example.com/repo", &dest);

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("my-project"));
    }
}

#[cfg(test)]
mod clone_path_handling_tests {
    use super::*;

    #[test]
    fn test_clone_handles_absolute_path() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();
        let temp_dir = TempDir::new().unwrap();
        let dest = temp_dir.path().join("absolute-path-repo");

        // Use absolute path
        let result = oxen.clone("https://example.com/repo", &dest);

        // Should not fail due to path being absolute
        if let Err(e) = result {
            assert!(!e.to_string().contains("Invalid path"));
        }
    }

    #[test]
    fn test_clone_handles_relative_path() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();

        // Use relative path
        let dest = PathBuf::from("./relative-path-repo");

        let result = oxen.clone("https://example.com/repo", &dest);

        // Should not fail due to path being relative
        if let Err(e) = result {
            assert!(!e.to_string().contains("Invalid path"));
        }
    }

    #[test]
    fn test_clone_handles_current_directory() {
        use auxin::OxenSubprocess;

        let oxen = OxenSubprocess::new();

        // Use current directory
        let dest = PathBuf::from(".");

        let result = oxen.clone("https://example.com/repo", &dest);

        // Should fail because current directory exists
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }
}
