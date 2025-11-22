//! Integration tests for thumbnail and bounce diff functionality

use auxin::{ThumbnailManager, BounceManager};
use std::fs;
use std::path::Path;
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
    manager.add_bounce("commit1", &audio1, Some("First bounce")).unwrap();
    manager.add_bounce("commit2", &audio2, Some("Second bounce")).unwrap();

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
    let comparison = manager.compare_bounces_with_null_test("commit1", "commit2").unwrap();

    assert!(comparison.null_test_result.is_some());
    let null_test = comparison.null_test_result.unwrap();

    // Should have some cancellation percentage
    assert!(null_test.cancellation_percent >= 0.0);
    assert!(null_test.cancellation_percent <= 100.0);
    assert!(!null_test.interpretation.is_empty());
}
