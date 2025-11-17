/// Common test utilities for OxVCS CLI Wrapper tests
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub mod mock_oxen_hub;

/// Test fixture for creating Logic Pro project structures
pub struct TestFixture {
    #[allow(dead_code)] // Kept alive to prevent directory deletion
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
}

impl TestFixture {
    /// Create a new test fixture with a minimal Logic Pro project structure
    pub fn new() -> Self {
        Self::with_name("TestProject")
    }

    /// Create a new test fixture with a custom project name
    pub fn with_name(name: &str) -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let project_path = temp_dir.path().join(format!("{}.logicx", name));

        // Create Logic Pro project structure
        fs::create_dir_all(&project_path).expect("Failed to create project directory");
        fs::create_dir_all(project_path.join("Alternatives"))
            .expect("Failed to create Alternatives directory");
        fs::create_dir_all(project_path.join("Media")).expect("Failed to create Media directory");

        Self {
            temp_dir,
            project_path,
        }
    }

    /// Create an audio file in the Media directory with specified size
    ///
    /// # Arguments
    /// * `name` - Filename (e.g., "audio.wav")
    /// * `size_mb` - Size in megabytes
    ///
    /// # Returns
    /// Path to the created audio file
    pub fn create_audio_file(&self, name: &str, size_mb: usize) -> PathBuf {
        let media = self.project_path.join("Media");
        let file_path = media.join(name);
        let data = vec![0u8; size_mb * 1024 * 1024];
        fs::write(&file_path, data).expect("Failed to write audio file");
        file_path
    }

    /// Create a projectData file with metadata
    ///
    /// # Arguments
    /// * `bpm` - Tempo in beats per minute
    /// * `sample_rate` - Sample rate in Hz
    /// * `key` - Optional key signature
    pub fn create_project_data(&self, bpm: u32, sample_rate: u32, key: Option<&str>) {
        let key_element = if let Some(k) = key {
            format!("    <key>{}</key>\n", k)
        } else {
            String::new()
        };

        let xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<project>
    <tempo>{}</tempo>
    <sampleRate>{}</sampleRate>
{}
</project>
"#,
            bpm, sample_rate, key_element
        );

        fs::write(self.project_path.join("projectData"), xml).expect("Failed to write projectData");
    }

    /// Get the path to the project
    pub fn path(&self) -> &Path {
        &self.project_path
    }

    /// Add a file with text content to the project
    #[allow(dead_code)] // Utility function for future tests
    pub fn add_text_file(&self, relative_path: &str, content: &str) -> PathBuf {
        let file_path = self.project_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create parent directory");
        }
        fs::write(&file_path, content).expect("Failed to write text file");
        file_path
    }
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a test project with specified total size
///
/// # Arguments
/// * `size_mb` - Total size of audio files to create
///
/// # Returns
/// Test fixture with the specified size
#[allow(dead_code)] // Utility function for future tests
pub fn create_test_project_with_size(size_mb: usize) -> TestFixture {
    let fixture = TestFixture::new();
    fixture.create_audio_file("large.wav", size_mb);
    fixture
}

/// Create a fully-configured test project with metadata
///
/// # Returns
/// Test fixture with projectData configured
#[allow(dead_code)] // Utility function for future tests
pub fn create_test_project_with_metadata() -> TestFixture {
    let fixture = TestFixture::new();
    fixture.create_project_data(120, 48000, Some("C"));
    fixture
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creates_valid_structure() {
        let fixture = TestFixture::new();
        assert!(fixture.project_path.exists());
        assert!(fixture.project_path.join("Alternatives").exists());
        assert!(fixture.project_path.join("Media").exists());
    }

    #[test]
    fn test_create_audio_file() {
        let fixture = TestFixture::new();
        let audio_path = fixture.create_audio_file("test.wav", 1);
        assert!(audio_path.exists());

        let metadata = fs::metadata(&audio_path).unwrap();
        assert_eq!(metadata.len(), 1024 * 1024); // 1MB
    }

    #[test]
    fn test_create_project_data() {
        let fixture = TestFixture::new();
        fixture.create_project_data(140, 96000, Some("Dm"));

        let project_data_path = fixture.project_path.join("projectData");
        assert!(project_data_path.exists());

        let content = fs::read_to_string(project_data_path).unwrap();
        assert!(content.contains("<tempo>140</tempo>"));
        assert!(content.contains("<sampleRate>96000</sampleRate>"));
        assert!(content.contains("<key>Dm</key>"));
    }
}
