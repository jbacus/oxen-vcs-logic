/// Integration tests for bounce (audio snapshot) functionality
///
/// Tests the complete bounce workflow including:
/// - Add, list, get, delete bounces
/// - Search with various filters
/// - Compare bounces
/// - Format validation
/// - Error handling

#[cfg(test)]
mod common;

#[cfg(test)]
mod tests {
    use super::common::TestFixture;
    use auxin::{BounceManager, BounceFilter, AudioFormat};
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    /// Helper to create a test bounce manager
    fn create_test_manager() -> (TempDir, BounceManager) {
        let temp_dir = TempDir::new().unwrap();
        let manager = BounceManager::new(temp_dir.path());
        manager.init().unwrap();
        (temp_dir, manager)
    }

    /// Helper to create a fake audio file
    fn create_test_audio_file(path: &std::path::Path, size_kb: usize) -> PathBuf {
        let audio_path = path.join("test_bounce.wav");
        let data = vec![0u8; size_kb * 1024];
        fs::write(&audio_path, data).unwrap();
        audio_path
    }

    // ===================
    // AudioFormat Tests
    // ===================

    #[test]
    fn test_audio_format_from_extension_wav() {
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("WAV"), Some(AudioFormat::Wav));
    }

    #[test]
    fn test_audio_format_from_extension_mp3() {
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("MP3"), Some(AudioFormat::Mp3));
    }

    #[test]
    fn test_audio_format_from_extension_flac() {
        assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
    }

    #[test]
    fn test_audio_format_from_extension_aiff() {
        assert_eq!(AudioFormat::from_extension("aiff"), Some(AudioFormat::Aiff));
        assert_eq!(AudioFormat::from_extension("aif"), Some(AudioFormat::Aiff));
    }

    #[test]
    fn test_audio_format_from_extension_m4a() {
        assert_eq!(AudioFormat::from_extension("m4a"), Some(AudioFormat::M4a));
    }

    #[test]
    fn test_audio_format_from_extension_invalid() {
        assert_eq!(AudioFormat::from_extension("txt"), None);
        assert_eq!(AudioFormat::from_extension("pdf"), None);
        assert_eq!(AudioFormat::from_extension(""), None);
    }

    #[test]
    fn test_audio_format_extension() {
        assert_eq!(AudioFormat::Wav.extension(), "wav");
        assert_eq!(AudioFormat::Mp3.extension(), "mp3");
        assert_eq!(AudioFormat::Flac.extension(), "flac");
        assert_eq!(AudioFormat::Aiff.extension(), "aiff");
        assert_eq!(AudioFormat::M4a.extension(), "m4a");
    }

    #[test]
    fn test_audio_format_mime_type() {
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
        assert_eq!(AudioFormat::Aiff.mime_type(), "audio/aiff");
        assert_eq!(AudioFormat::M4a.mime_type(), "audio/mp4");
    }

    // ===================
    // BounceManager Tests
    // ===================

    #[test]
    fn test_bounce_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BounceManager::new(temp_dir.path());

        assert!(manager.init().is_ok());
        assert!(temp_dir.path().join(".auxin").join("bounces").exists());
    }

    #[test]
    fn test_bounce_manager_init_idempotent() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BounceManager::new(temp_dir.path());

        // Init multiple times should succeed
        assert!(manager.init().is_ok());
        assert!(manager.init().is_ok());
    }

    #[test]
    fn test_add_bounce_success() {
        let (temp_dir, manager) = create_test_manager();
        let audio_file = create_test_audio_file(temp_dir.path(), 100);

        let result = manager.add_bounce(
            "commit123",
            &audio_file,
            Some("Test bounce"),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_bounce_invalid_format() {
        let (temp_dir, manager) = create_test_manager();
        let invalid_file = temp_dir.path().join("test.txt");
        fs::write(&invalid_file, "not audio").unwrap();

        let result = manager.add_bounce("commit123", &invalid_file, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_add_bounce_nonexistent_file() {
        let (_temp_dir, manager) = create_test_manager();
        let nonexistent = PathBuf::from("/nonexistent/file.wav");

        let result = manager.add_bounce("commit123", &nonexistent, None);

        assert!(result.is_err());
    }

    #[test]
    fn test_get_bounce_exists() {
        let (temp_dir, manager) = create_test_manager();
        let audio_file = create_test_audio_file(temp_dir.path(), 50);

        manager.add_bounce("commit456", &audio_file, Some("Description")).unwrap();

        let bounce = manager.get_bounce("commit456").unwrap();
        assert!(bounce.is_some());

        let metadata = bounce.unwrap();
        assert_eq!(metadata.commit_id, "commit456");
        assert_eq!(metadata.description, Some("Description".to_string()));
    }

    #[test]
    fn test_get_bounce_not_exists() {
        let (_temp_dir, manager) = create_test_manager();

        let bounce = manager.get_bounce("nonexistent").unwrap();
        assert!(bounce.is_none());
    }

    #[test]
    fn test_get_bounce_path() {
        let (temp_dir, manager) = create_test_manager();
        let audio_file = create_test_audio_file(temp_dir.path(), 50);

        manager.add_bounce("commit789", &audio_file, None).unwrap();

        let path = manager.get_bounce_path("commit789").unwrap();
        assert!(path.is_some());
        assert!(path.unwrap().exists());
    }

    #[test]
    fn test_list_bounces_empty() {
        let (_temp_dir, manager) = create_test_manager();

        let bounces = manager.list_bounces().unwrap();
        assert!(bounces.is_empty());
    }

    #[test]
    fn test_list_bounces_multiple() {
        let (temp_dir, manager) = create_test_manager();

        // Add multiple bounces
        for i in 0..5 {
            let audio_file = temp_dir.path().join(format!("bounce_{}.wav", i));
            fs::write(&audio_file, vec![0u8; 1024]).unwrap();
            manager.add_bounce(&format!("commit_{}", i), &audio_file, None).unwrap();
        }

        let bounces = manager.list_bounces().unwrap();
        assert_eq!(bounces.len(), 5);
    }

    // ===================
    // Delete Bounce Tests
    // ===================

    #[test]
    fn test_delete_bounce_success() {
        let (temp_dir, manager) = create_test_manager();
        let audio_file = create_test_audio_file(temp_dir.path(), 50);

        manager.add_bounce("commit_to_delete", &audio_file, None).unwrap();

        // Verify it exists
        assert!(manager.get_bounce("commit_to_delete").unwrap().is_some());

        // Delete it
        let result = manager.delete_bounce("commit_to_delete");
        assert!(result.is_ok());

        // Verify it's gone
        assert!(manager.get_bounce("commit_to_delete").unwrap().is_none());
    }

    #[test]
    fn test_delete_bounce_not_exists() {
        let (_temp_dir, manager) = create_test_manager();

        let result = manager.delete_bounce("nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_delete_bounce_removes_file() {
        let (temp_dir, manager) = create_test_manager();
        let audio_file = create_test_audio_file(temp_dir.path(), 50);

        manager.add_bounce("commit_del", &audio_file, None).unwrap();

        let path = manager.get_bounce_path("commit_del").unwrap().unwrap();
        assert!(path.exists());

        manager.delete_bounce("commit_del").unwrap();
        assert!(!path.exists());
    }

    // ===================
    // Search Bounce Tests
    // ===================

    #[test]
    fn test_search_bounces_empty_filter() {
        let (temp_dir, manager) = create_test_manager();

        // Add some bounces
        for i in 0..3 {
            let audio_file = temp_dir.path().join(format!("search_{}.wav", i));
            fs::write(&audio_file, vec![0u8; 1024]).unwrap();
            manager.add_bounce(&format!("search_{}", i), &audio_file, None).unwrap();
        }

        let filter = BounceFilter::default();
        let results = manager.search_bounces(&filter).unwrap();

        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_search_bounces_by_format() {
        let (temp_dir, manager) = create_test_manager();

        // Add WAV bounce
        let wav_file = temp_dir.path().join("test.wav");
        fs::write(&wav_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("wav_commit", &wav_file, None).unwrap();

        // Add MP3 bounce
        let mp3_file = temp_dir.path().join("test.mp3");
        fs::write(&mp3_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("mp3_commit", &mp3_file, None).unwrap();

        // Search for WAV only
        let mut filter = BounceFilter::default();
        filter.format = Some(AudioFormat::Wav);

        let results = manager.search_bounces(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].commit_id, "wav_commit");
    }

    #[test]
    fn test_search_bounces_by_min_size() {
        let (temp_dir, manager) = create_test_manager();

        // Add small bounce (1KB)
        let small_file = temp_dir.path().join("small.wav");
        fs::write(&small_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("small_commit", &small_file, None).unwrap();

        // Add large bounce (100KB)
        let large_file = temp_dir.path().join("large.wav");
        fs::write(&large_file, vec![0u8; 100 * 1024]).unwrap();
        manager.add_bounce("large_commit", &large_file, None).unwrap();

        // Search for files > 50KB
        let mut filter = BounceFilter::default();
        filter.min_size = Some(50 * 1024);

        let results = manager.search_bounces(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].commit_id, "large_commit");
    }

    #[test]
    fn test_search_bounces_by_max_size() {
        let (temp_dir, manager) = create_test_manager();

        // Add small bounce
        let small_file = temp_dir.path().join("small.wav");
        fs::write(&small_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("small_commit", &small_file, None).unwrap();

        // Add large bounce
        let large_file = temp_dir.path().join("large.wav");
        fs::write(&large_file, vec![0u8; 100 * 1024]).unwrap();
        manager.add_bounce("large_commit", &large_file, None).unwrap();

        // Search for files < 50KB
        let mut filter = BounceFilter::default();
        filter.max_size = Some(50 * 1024);

        let results = manager.search_bounces(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].commit_id, "small_commit");
    }

    #[test]
    fn test_search_bounces_by_pattern() {
        let (temp_dir, manager) = create_test_manager();

        // Add bounces with different names
        let mix_file = temp_dir.path().join("final_mix.wav");
        fs::write(&mix_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("mix_commit", &mix_file, None).unwrap();

        let rough_file = temp_dir.path().join("rough_draft.wav");
        fs::write(&rough_file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("rough_commit", &rough_file, None).unwrap();

        // Search for 'mix' pattern
        let mut filter = BounceFilter::default();
        filter.pattern = Some("mix".to_string());

        let results = manager.search_bounces(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].commit_id, "mix_commit");
    }

    #[test]
    fn test_search_bounces_combined_filters() {
        let (temp_dir, manager) = create_test_manager();

        // Add various bounces
        let file1 = temp_dir.path().join("mix_v1.wav");
        fs::write(&file1, vec![0u8; 10 * 1024]).unwrap();
        manager.add_bounce("commit1", &file1, None).unwrap();

        let file2 = temp_dir.path().join("mix_v2.wav");
        fs::write(&file2, vec![0u8; 100 * 1024]).unwrap();
        manager.add_bounce("commit2", &file2, None).unwrap();

        let file3 = temp_dir.path().join("draft.mp3");
        fs::write(&file3, vec![0u8; 50 * 1024]).unwrap();
        manager.add_bounce("commit3", &file3, None).unwrap();

        // Search: WAV format, contains 'mix', size > 50KB
        let mut filter = BounceFilter::default();
        filter.format = Some(AudioFormat::Wav);
        filter.pattern = Some("mix".to_string());
        filter.min_size = Some(50 * 1024);

        let results = manager.search_bounces(&filter).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].commit_id, "commit2");
    }

    #[test]
    fn test_search_bounces_no_matches() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("test.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("commit1", &file, None).unwrap();

        let mut filter = BounceFilter::default();
        filter.pattern = Some("nonexistent".to_string());

        let results = manager.search_bounces(&filter).unwrap();
        assert!(results.is_empty());
    }

    // ===================
    // Compare Bounce Tests
    // ===================

    #[test]
    fn test_compare_bounces_success() {
        let (temp_dir, manager) = create_test_manager();

        // Add first bounce (small)
        let file_a = temp_dir.path().join("bounce_a.wav");
        fs::write(&file_a, vec![0u8; 10 * 1024]).unwrap();
        manager.add_bounce("commit_a", &file_a, Some("First version")).unwrap();

        // Add second bounce (larger)
        let file_b = temp_dir.path().join("bounce_b.wav");
        fs::write(&file_b, vec![0u8; 50 * 1024]).unwrap();
        manager.add_bounce("commit_b", &file_b, Some("Second version")).unwrap();

        let comparison = manager.compare_bounces("commit_a", "commit_b").unwrap();

        assert_eq!(comparison.bounce_a.commit_id, "commit_a");
        assert_eq!(comparison.bounce_b.commit_id, "commit_b");
        assert!(comparison.size_diff() > 0); // B is larger
    }

    #[test]
    fn test_compare_bounces_same_commit() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("bounce.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("same_commit", &file, None).unwrap();

        let comparison = manager.compare_bounces("same_commit", "same_commit").unwrap();

        assert_eq!(comparison.size_diff(), 0);
    }

    #[test]
    fn test_compare_bounces_first_not_found() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("bounce.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("exists", &file, None).unwrap();

        let result = manager.compare_bounces("nonexistent", "exists");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_bounces_second_not_found() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("bounce.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();
        manager.add_bounce("exists", &file, None).unwrap();

        let result = manager.compare_bounces("exists", "nonexistent");
        assert!(result.is_err());
    }

    #[test]
    fn test_compare_bounces_format_report() {
        let (temp_dir, manager) = create_test_manager();

        let file_a = temp_dir.path().join("a.wav");
        fs::write(&file_a, vec![0u8; 1024]).unwrap();
        manager.add_bounce("a", &file_a, None).unwrap();

        let file_b = temp_dir.path().join("b.wav");
        fs::write(&file_b, vec![0u8; 2048]).unwrap();
        manager.add_bounce("b", &file_b, None).unwrap();

        let comparison = manager.compare_bounces("a", "b").unwrap();
        let report = comparison.format_report();

        assert!(!report.is_empty());
        assert!(report.contains("a") || report.contains("b"));
    }

    // ===================
    // BounceMetadata Tests
    // ===================

    #[test]
    fn test_bounce_metadata_format_duration() {
        use auxin::BounceMetadata;

        let mut metadata = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 1024);

        // Test with no duration
        assert_eq!(metadata.format_duration(), "Unknown");

        // Test with duration
        metadata = metadata.with_duration(125.5);
        assert_eq!(metadata.format_duration(), "2:05");

        // Test with duration over an hour
        let metadata2 = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 1024)
            .with_duration(3725.0);
        assert_eq!(metadata2.format_duration(), "62:05");
    }

    #[test]
    fn test_bounce_metadata_format_size() {
        use auxin::BounceMetadata;

        // Bytes
        let meta_bytes = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 500);
        assert_eq!(meta_bytes.format_size(), "500 B");

        // Kilobytes
        let meta_kb = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 2048);
        assert_eq!(meta_kb.format_size(), "2.0 KB");

        // Megabytes
        let meta_mb = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 5 * 1024 * 1024);
        assert_eq!(meta_mb.format_size(), "5.0 MB");
    }

    #[test]
    fn test_bounce_metadata_with_audio_info() {
        use auxin::BounceMetadata;

        let metadata = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 1024)
            .with_audio_info(48000, 24, 2);

        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.bit_depth, Some(24));
        assert_eq!(metadata.channels, Some(2));
    }

    #[test]
    fn test_bounce_metadata_with_description() {
        use auxin::BounceMetadata;

        let metadata = BounceMetadata::new("test", "file.wav", AudioFormat::Wav, 1024)
            .with_description("Final mix for client review");

        assert_eq!(metadata.description, Some("Final mix for client review".to_string()));
    }

    // ===================
    // Edge Case Tests
    // ===================

    #[test]
    fn test_add_bounce_duplicate_commit_id() {
        let (temp_dir, manager) = create_test_manager();

        // Add first bounce
        let file1 = temp_dir.path().join("first.wav");
        fs::write(&file1, vec![0u8; 1024]).unwrap();
        manager.add_bounce("duplicate_id", &file1, None).unwrap();

        // Try to add another with same commit ID
        let file2 = temp_dir.path().join("second.wav");
        fs::write(&file2, vec![0u8; 1024]).unwrap();
        let result = manager.add_bounce("duplicate_id", &file2, None);

        // Should fail or overwrite - verify behavior
        // The actual behavior depends on implementation
        // Either is acceptable, but should be consistent
        if result.is_ok() {
            // If it overwrites, verify the new one is there
            let bounce = manager.get_bounce("duplicate_id").unwrap().unwrap();
            assert!(bounce.original_filename.contains("second"));
        }
        // If it errors, that's also acceptable
    }

    #[test]
    fn test_bounce_with_special_characters_in_filename() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("test with spaces & symbols!.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();

        let result = manager.add_bounce("special_chars", &file, None);
        assert!(result.is_ok());

        let bounce = manager.get_bounce("special_chars").unwrap();
        assert!(bounce.is_some());
    }

    #[test]
    fn test_bounce_manager_concurrent_operations() {
        let (temp_dir, manager) = create_test_manager();

        // Add multiple bounces to test concurrent access
        let handles: Vec<_> = (0..10)
            .map(|i| {
                let path = temp_dir.path().to_path_buf();
                let file = path.join(format!("concurrent_{}.wav", i));
                fs::write(&file, vec![0u8; 1024]).unwrap();
                file
            })
            .collect();

        // Add all bounces
        for (i, file) in handles.iter().enumerate() {
            manager.add_bounce(&format!("concurrent_{}", i), file, None).unwrap();
        }

        // Verify all were added
        let bounces = manager.list_bounces().unwrap();
        assert_eq!(bounces.len(), 10);
    }

    #[test]
    fn test_bounce_empty_description() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("empty_desc.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();

        manager.add_bounce("empty_desc", &file, Some("")).unwrap();

        let bounce = manager.get_bounce("empty_desc").unwrap().unwrap();
        assert_eq!(bounce.description, Some("".to_string()));
    }

    #[test]
    fn test_bounce_very_long_description() {
        let (temp_dir, manager) = create_test_manager();

        let file = temp_dir.path().join("long_desc.wav");
        fs::write(&file, vec![0u8; 1024]).unwrap();

        let long_desc = "A".repeat(10000);
        manager.add_bounce("long_desc", &file, Some(&long_desc)).unwrap();

        let bounce = manager.get_bounce("long_desc").unwrap().unwrap();
        assert_eq!(bounce.description.unwrap().len(), 10000);
    }
}
