//! Integration tests for thumbnail management

use auxin::{ThumbnailManager, ThumbnailMetadata};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_thumbnail_manager_init() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());

    // Should initialize thumbnails directory
    manager.init().unwrap();

    let thumbs_dir = temp_dir.path().join(".auxin").join("thumbnails");
    assert!(thumbs_dir.exists());
    assert!(thumbs_dir.is_dir());
}

#[test]
fn test_add_thumbnail() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create a dummy image file
    let source_file = temp_dir.path().join("test_image.jpg");
    fs::write(&source_file, b"fake jpeg data").unwrap();

    // Add the thumbnail
    let metadata = manager
        .add_thumbnail("commit123", &source_file)
        .unwrap();

    assert_eq!(metadata.commit_id, "commit123");
    assert_eq!(metadata.format, "jpg");
    assert_eq!(metadata.size_bytes, 14); // length of "fake jpeg data"

    // Verify thumbnail file was created
    let thumbnail_path = manager.get_thumbnail_path("commit123").unwrap();
    assert!(thumbnail_path.is_some());
    assert!(thumbnail_path.unwrap().exists());
}

#[test]
fn test_get_thumbnail() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create and add thumbnail
    let source_file = temp_dir.path().join("test_image.png");
    fs::write(&source_file, b"fake png data").unwrap();
    manager.add_thumbnail("commit456", &source_file).unwrap();

    // Retrieve thumbnail metadata
    let metadata = manager.get_thumbnail("commit456").unwrap();
    assert!(metadata.is_some());

    let meta = metadata.unwrap();
    assert_eq!(meta.commit_id, "commit456");
    assert_eq!(meta.format, "png");
}

#[test]
fn test_list_thumbnails() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Add multiple thumbnails
    let source1 = temp_dir.path().join("img1.jpg");
    let source2 = temp_dir.path().join("img2.png");
    fs::write(&source1, b"data1").unwrap();
    fs::write(&source2, b"data2").unwrap();

    manager.add_thumbnail("commit1", &source1).unwrap();
    manager.add_thumbnail("commit2", &source2).unwrap();

    // List all thumbnails
    let thumbnails = manager.list_thumbnails().unwrap();
    assert_eq!(thumbnails.len(), 2);

    let commit_ids: Vec<String> = thumbnails.iter().map(|t| t.commit_id.clone()).collect();
    assert!(commit_ids.contains(&"commit1".to_string()));
    assert!(commit_ids.contains(&"commit2".to_string()));
}

#[test]
fn test_delete_thumbnail() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create and add thumbnail
    let source_file = temp_dir.path().join("test_image.jpg");
    fs::write(&source_file, b"fake data").unwrap();
    manager.add_thumbnail("commit789", &source_file).unwrap();

    // Verify it exists
    assert!(manager.get_thumbnail("commit789").unwrap().is_some());

    // Delete it
    manager.delete_thumbnail("commit789").unwrap();

    // Verify it's gone
    assert!(manager.get_thumbnail("commit789").unwrap().is_none());
    assert!(manager.get_thumbnail_path("commit789").unwrap().is_none());
}

#[test]
fn test_thumbnail_metadata_with_dimensions() {
    let metadata = ThumbnailMetadata::new("commit123", "png", 5000)
        .with_source("/path/to/image.png")
        .with_dimensions(1920, 1080);

    assert_eq!(metadata.commit_id, "commit123");
    assert_eq!(metadata.format, "png");
    assert_eq!(metadata.size_bytes, 5000);
    assert_eq!(metadata.source_path, Some("/path/to/image.png".to_string()));
    assert_eq!(metadata.width, Some(1920));
    assert_eq!(metadata.height, Some(1080));
}

#[test]
fn test_add_thumbnail_nonexistent_file() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    let nonexistent = temp_dir.path().join("does_not_exist.jpg");
    let result = manager.add_thumbnail("commit999", &nonexistent);

    assert!(result.is_err());
}

#[test]
fn test_get_thumbnail_path_different_formats() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Test different image formats
    for ext in &["jpg", "png", "gif"] {
        let source = temp_dir.path().join(format!("test.{}", ext));
        fs::write(&source, b"data").unwrap();

        let commit_id = format!("commit_{}", ext);
        manager.add_thumbnail(&commit_id, &source).unwrap();

        let path_opt = manager.get_thumbnail_path(&commit_id).unwrap();
        assert!(path_opt.is_some());
        let path = path_opt.unwrap();
        let extension = path.extension().and_then(|e| e.to_str());
        assert_eq!(extension, Some(*ext));
    }
}

#[test]
fn test_extract_logic_thumbnail_from_logicx_structure() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create a mock Logic Pro project structure
    let project_path = temp_dir.path().join("TestProject.logicx");
    let alternatives_path = project_path.join("Alternatives").join("001");
    fs::create_dir_all(&alternatives_path).unwrap();

    // Create a fake WindowImage.jpg
    let window_image = alternatives_path.join("WindowImage.jpg");
    let fake_image_data = b"fake jpeg screenshot data from Logic Pro";
    fs::write(&window_image, fake_image_data).unwrap();

    // Extract thumbnail
    let result = manager.extract_logic_thumbnail("commit_logic", &project_path);

    assert!(result.is_ok());
    let metadata = result.unwrap();
    assert_eq!(metadata.commit_id, "commit_logic");
    assert_eq!(metadata.format, "jpg");
    assert_eq!(metadata.size_bytes, fake_image_data.len() as u64);

    // Verify file was copied
    let thumbnail_path = manager.get_thumbnail_path("commit_logic").unwrap();
    assert!(thumbnail_path.is_some());
}

#[test]
fn test_extract_logic_thumbnail_old_format() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create old-style Logic Pro structure (numbered directory)
    let project_path = temp_dir.path().join("OldProject.logicx");
    let numbered_path = project_path.join("000");
    fs::create_dir_all(&numbered_path).unwrap();

    // Create WindowImage in old location
    let window_image = numbered_path.join("WindowImage.jpg");
    fs::write(&window_image, b"old format screenshot").unwrap();

    // Extract should find it
    let result = manager.extract_logic_thumbnail("commit_old", &project_path);
    assert!(result.is_ok());
}

#[test]
fn test_extract_logic_thumbnail_missing() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create project without WindowImage
    let project_path = temp_dir.path().join("EmptyProject.logicx");
    fs::create_dir_all(&project_path).unwrap();

    // Should fail gracefully
    let result = manager.extract_logic_thumbnail("commit_empty", &project_path);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("No WindowImage"));
}

#[test]
fn test_compare_thumbnails_fallback_without_imagemagick() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    // Create two different-sized images
    let img1 = temp_dir.path().join("img1.jpg");
    let img2 = temp_dir.path().join("img2.jpg");

    fs::write(&img1, vec![0u8; 1000]).unwrap();
    fs::write(&img2, vec![0u8; 1500]).unwrap();

    manager.add_thumbnail("commit_a", &img1).unwrap();
    manager.add_thumbnail("commit_b", &img2).unwrap();

    // Compare - should work even without ImageMagick (uses size fallback)
    let diff = manager.compare_thumbnails("commit_a", "commit_b").unwrap();

    assert_eq!(diff.commit_a, "commit_a");
    assert_eq!(diff.commit_b, "commit_b");
    assert_eq!(diff.size_diff_bytes, 500); // 1500 - 1000

    // Difference percent should be calculated from size
    // (500 / 1500) * 100 = 33.33%
    assert!(diff.difference_percent > 0.0);
    assert!(diff.difference_percent <= 100.0);
}

#[test]
fn test_compare_thumbnails_dimension_change() {
    let temp_dir = TempDir::new().unwrap();
    let manager = ThumbnailManager::new(temp_dir.path());
    manager.init().unwrap();

    let img1 = temp_dir.path().join("img1.jpg");
    let img2 = temp_dir.path().join("img2.jpg");

    fs::write(&img1, b"data1").unwrap();
    fs::write(&img2, b"data2").unwrap();

    // Add with different dimensions
    let mut meta1 = manager.add_thumbnail("commit_1", &img1).unwrap();
    let mut meta2 = manager.add_thumbnail("commit_2", &img2).unwrap();

    // Manually set dimensions for testing
    meta1.width = Some(1920);
    meta1.height = Some(1080);
    meta2.width = Some(3840);
    meta2.height = Some(2160);

    // Re-save metadata
    let json1 = serde_json::to_string_pretty(&meta1).unwrap();
    let json2 = serde_json::to_string_pretty(&meta2).unwrap();
    fs::write(temp_dir.path().join(".auxin/thumbnails/commit_1.json"), json1).unwrap();
    fs::write(temp_dir.path().join(".auxin/thumbnails/commit_2.json"), json2).unwrap();

    let diff = manager.compare_thumbnails("commit_1", "commit_2").unwrap();

    assert!(diff.dimension_diff.is_some());
    let dim_diff = diff.dimension_diff.unwrap();
    assert!(dim_diff.contains("1920x1080"));
    assert!(dim_diff.contains("3840x2160"));
}
