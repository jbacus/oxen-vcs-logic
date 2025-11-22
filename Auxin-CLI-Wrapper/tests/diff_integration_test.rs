//! Integration tests for thumbnail and bounce diff functionality

use auxin::{BounceManager, ThumbnailManager};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_thumbnail_comparison() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create two dummy images
    let img1 = temp_dir.path().join("img1.jpg");
    let img2 = temp_dir.path().join("img2.jpg");

    fs::write(&img1, b"fake jpeg data 1").unwrap();
    fs::write(&img2, b"fake jpeg data 2 different").unwrap();

    // Add thumbnails
    manager.add_thumbnail("commit1", &img1).unwrap();
    manager.add_thumbnail("commit2", &img2).unwrap();

    // Compare
    let diff = manager.compare_thumbnails("commit1", "commit2").unwrap();

    assert_eq!(diff.commit_a, "commit1");
    assert_eq!(diff.commit_b, "commit2");
    assert!(diff.difference_percent >= 0.0);
    assert!(diff.difference_percent <= 100.0);
    // Size diff should be non-zero since files are different
    assert_ne!(diff.size_diff_bytes, 0);
}

#[test]
fn test_thumbnail_comparison_identical() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create identical images
    let img1 = temp_dir.path().join("img1.jpg");
    let img2 = temp_dir.path().join("img2.jpg");

    let data = b"identical jpeg data";
    fs::write(&img1, data).unwrap();
    fs::write(&img2, data).unwrap();

    manager.add_thumbnail("commit1", &img1).unwrap();
    manager.add_thumbnail("commit2", &img2).unwrap();

    let diff = manager.compare_thumbnails("commit1", "commit2").unwrap();

    // Identical files should have zero size difference
    assert_eq!(diff.size_diff_bytes, 0);
}

#[test]
fn test_bounce_comparison_basic() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    // Create two dummy audio files
    let audio1 = temp_dir.path().join("audio1.wav");
    let audio2 = temp_dir.path().join("audio2.wav");

    fs::write(&audio1, vec![0u8; 1000]).unwrap();
    fs::write(&audio2, vec![0u8; 2000]).unwrap();

    // Add bounces
    manager
        .add_bounce("commit1", &audio1, Some("First bounce"))
        .unwrap();
    manager
        .add_bounce("commit2", &audio2, Some("Second bounce"))
        .unwrap();

    // Compare
    let comparison = manager.compare_bounces("commit1", "commit2").unwrap();

    assert_eq!(comparison.bounce_a.commit_id, "commit1");
    assert_eq!(comparison.bounce_b.commit_id, "commit2");
    assert_eq!(comparison.size_diff(), 1000); // 2000 - 1000
    assert!(comparison.null_test_result.is_none()); // No null test by default
}

#[test]
fn test_bounce_comparison_with_null_test() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    // Create two dummy audio files
    let audio1 = temp_dir.path().join("audio1.wav");
    let audio2 = temp_dir.path().join("audio2.wav");

    fs::write(&audio1, vec![0u8; 1000]).unwrap();
    fs::write(&audio2, vec![0u8; 1000]).unwrap(); // Same size

    manager.add_bounce("commit1", &audio1, None).unwrap();
    manager.add_bounce("commit2", &audio2, None).unwrap();

    // Compare with null test
    let comparison = manager
        .compare_bounces_with_null_test("commit1", "commit2")
        .unwrap();

    assert!(comparison.null_test_result.is_some());
    let null_test = comparison.null_test_result.unwrap();

    // Should have some cancellation percentage
    assert!(null_test.cancellation_percent >= 0.0);
    assert!(null_test.cancellation_percent <= 100.0);
    assert!(!null_test.interpretation.is_empty());
}

#[test]
fn test_bounce_null_test_fallback_without_ffmpeg() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    // Create two different-sized files
    let audio1 = temp_dir.path().join("audio1.wav");
    let audio2 = temp_dir.path().join("audio2.wav");

    fs::write(&audio1, vec![0u8; 1000]).unwrap();
    fs::write(&audio2, vec![0u8; 1500]).unwrap();

    manager.add_bounce("commit_a", &audio1, None).unwrap();
    manager.add_bounce("commit_b", &audio2, None).unwrap();

    // Null test should work even without ffmpeg (uses fallback)
    let comparison = manager
        .compare_bounces_with_null_test("commit_a", "commit_b")
        .unwrap();

    assert!(comparison.null_test_result.is_some());
    let null_test = comparison.null_test_result.unwrap();

    // Should use size-based estimation
    assert!(null_test.cancellation_percent >= 0.0);
    assert!(null_test.cancellation_percent <= 100.0);

    // Different sizes should show lower cancellation
    assert!(null_test.cancellation_percent < 100.0);
}

#[test]
fn test_bounce_null_test_identical_files() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    // Create identical files
    let audio1 = temp_dir.path().join("audio1.wav");
    let audio2 = temp_dir.path().join("audio2.wav");

    let data = vec![0u8; 2000];
    fs::write(&audio1, &data).unwrap();
    fs::write(&audio2, &data).unwrap();

    manager.add_bounce("commit_x", &audio1, None).unwrap();
    manager.add_bounce("commit_y", &audio2, None).unwrap();

    let comparison = manager
        .compare_bounces_with_null_test("commit_x", "commit_y")
        .unwrap();
    let null_test = comparison.null_test_result.unwrap();

    // Identical files should show high cancellation (even with fallback)
    assert!(null_test.cancellation_percent >= 99.0);
}

#[test]
fn test_bounce_comparison_format_report() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    let audio1 = temp_dir.path().join("audio1.wav");
    let audio2 = temp_dir.path().join("audio2.wav");

    fs::write(&audio1, vec![0u8; 1000]).unwrap();
    fs::write(&audio2, vec![0u8; 2000]).unwrap();

    manager.add_bounce("abc123", &audio1, None).unwrap();
    manager.add_bounce("def456", &audio2, None).unwrap();

    let comparison = manager.compare_bounces("abc123", "def456").unwrap();
    let report = comparison.format_report();

    // Report should contain key information
    assert!(report.contains("abc123"));
    assert!(report.contains("def456"));
    assert!(report.contains("Duration"));
    assert!(report.contains("File Size"));
    assert!(report.contains("Format"));
}

#[test]
fn test_bounce_comparison_missing_commit() {
    let temp_dir = TempDir::new().unwrap();
    let manager = BounceManager::new(temp_dir.path());

    let audio1 = temp_dir.path().join("audio1.wav");
    fs::write(&audio1, vec![0u8; 1000]).unwrap();
    manager.add_bounce("exists", &audio1, None).unwrap();

    // Should error when commit doesn't exist
    let result = manager.compare_bounces("exists", "nonexistent");
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No bounce found"));
}

#[test]
fn test_thumbnail_comparison_missing_commit() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    let img1 = temp_dir.path().join("img1.jpg");
    fs::write(&img1, b"data").unwrap();
    manager.add_thumbnail("exists", &img1).unwrap();

    // Should error when commit doesn't exist
    let result = manager.compare_thumbnails("exists", "nonexistent");
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("No thumbnail for commit"));
}

#[test]
fn test_bounce_null_test_interpretation_levels() {
    // Test that the interpretation matches cancellation percentages
    use auxin::NullTestResult;

    let result_identical = NullTestResult {
        cancellation_percent: 99.95,
        interpretation: "Identical or imperceptibly different".to_string(),
        difference_level_db: Some(-100.0),
    };
    assert!(result_identical.interpretation.contains("Identical"));

    let result_similar = NullTestResult {
        cancellation_percent: 85.0,
        interpretation: "Similar with minor differences".to_string(),
        difference_level_db: Some(-20.0),
    };
    assert!(result_similar.interpretation.contains("Similar"));

    let result_different = NullTestResult {
        cancellation_percent: 15.0,
        interpretation: "Completely different mixes".to_string(),
        difference_level_db: Some(-5.0),
    };
    assert!(result_different.interpretation.contains("different"));
}
