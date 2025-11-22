//! Thumbnail management for commit snapshots
//!
//! Extracts and manages project thumbnails (screenshots) to provide visual
//! representation of commits in the UI. For Logic Pro projects, this extracts
//! the WindowImage.jpg file that Logic saves on each project save.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Metadata about a thumbnail image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailMetadata {
    /// Commit ID this thumbnail is associated with
    pub commit_id: String,

    /// Original source path (if known)
    pub source_path: Option<String>,

    /// Image format (jpg, png, etc.)
    pub format: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Width in pixels (if known)
    pub width: Option<u32>,

    /// Height in pixels (if known)
    pub height: Option<u32>,
}

impl ThumbnailMetadata {
    /// Create new thumbnail metadata
    pub fn new(commit_id: &str, format: &str, size_bytes: u64) -> Self {
        Self {
            commit_id: commit_id.to_string(),
            source_path: None,
            format: format.to_string(),
            size_bytes,
            width: None,
            height: None,
        }
    }

    /// Set source path
    pub fn with_source(mut self, path: &str) -> Self {
        self.source_path = Some(path.to_string());
        self
    }

    /// Set dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Manages thumbnail images for a repository
pub struct ThumbnailManager {
    /// Root directory of the repository
    #[allow(dead_code)]
    repo_root: PathBuf,

    /// Directory where thumbnails are stored
    thumbnails_dir: PathBuf,
}

impl ThumbnailManager {
    /// Create a new thumbnail manager for a repository
    pub fn new(repo_root: &Path) -> Self {
        let thumbnails_dir = repo_root.join(".auxin").join("thumbnails");
        Self {
            repo_root: repo_root.to_path_buf(),
            thumbnails_dir,
        }
    }

    /// Initialize thumbnail storage directory
    pub fn init(&self) -> Result<()> {
        if !self.thumbnails_dir.exists() {
            fs::create_dir_all(&self.thumbnails_dir)
                .context("Failed to create thumbnails directory")?;
        }
        Ok(())
    }

    /// Extract and save thumbnail from a Logic Pro project
    pub fn extract_logic_thumbnail(
        &self,
        commit_id: &str,
        project_path: &Path,
    ) -> Result<ThumbnailMetadata> {
        self.init()?;

        // Logic Pro saves WindowImage.jpg in various locations:
        // 1. <project>.logicx/000/WindowImage.jpg (older format)
        // 2. <project>.logicx/Alternatives/###/WindowImage.jpg (current format)
        let thumbnail_source = self.find_logic_window_image(project_path)?;

        // Copy to thumbnails directory
        let format = thumbnail_source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("jpg");
        let dest_filename = format!("{}.{}", commit_id, format);
        let dest_path = self.thumbnails_dir.join(&dest_filename);

        fs::copy(&thumbnail_source, &dest_path)
            .context("Failed to copy thumbnail")?;

        // Get file size
        let file_meta = fs::metadata(&dest_path)
            .context("Failed to read thumbnail metadata")?;

        let metadata = ThumbnailMetadata::new(commit_id, format, file_meta.len())
            .with_source(thumbnail_source.to_string_lossy().as_ref());

        // Save metadata
        self.save_metadata(&metadata)?;

        Ok(metadata)
    }

    /// Find the WindowImage.jpg file in a Logic Pro project
    fn find_logic_window_image(&self, project_path: &Path) -> Result<PathBuf> {
        // Check Alternatives directory first (current format)
        let alternatives_path = project_path.join("Alternatives");
        if alternatives_path.exists() && alternatives_path.is_dir() {
            if let Ok(entries) = fs::read_dir(&alternatives_path) {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let window_image = entry_path.join("WindowImage.jpg");
                        if window_image.exists() {
                            return Ok(window_image);
                        }
                        // Also check for .png variant
                        let window_image_png = entry_path.join("WindowImage.png");
                        if window_image_png.exists() {
                            return Ok(window_image_png);
                        }
                    }
                }
            }
        }

        // Check root level numbered directories (older format)
        for num in 0..10 {
            let numbered_dir = project_path.join(format!("{:03}", num));
            let window_image = numbered_dir.join("WindowImage.jpg");
            if window_image.exists() {
                return Ok(window_image);
            }
            let window_image_png = numbered_dir.join("WindowImage.png");
            if window_image_png.exists() {
                return Ok(window_image_png);
            }
        }

        Err(anyhow!(
            "No WindowImage file found in Logic Pro project: {}",
            project_path.display()
        ))
    }

    /// Add a thumbnail from an external file
    pub fn add_thumbnail(
        &self,
        commit_id: &str,
        source_file: &Path,
    ) -> Result<ThumbnailMetadata> {
        self.init()?;

        if !source_file.exists() {
            return Err(anyhow!("Source file not found: {}", source_file.display()));
        }

        let format = source_file
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("Cannot determine image format"))?;

        let dest_filename = format!("{}.{}", commit_id, format);
        let dest_path = self.thumbnails_dir.join(&dest_filename);

        fs::copy(source_file, &dest_path)
            .context("Failed to copy thumbnail")?;

        let file_meta = fs::metadata(&dest_path)
            .context("Failed to read thumbnail metadata")?;

        let metadata = ThumbnailMetadata::new(commit_id, format, file_meta.len())
            .with_source(source_file.to_string_lossy().as_ref());

        self.save_metadata(&metadata)?;

        Ok(metadata)
    }

    /// Get thumbnail metadata for a commit
    pub fn get_thumbnail(&self, commit_id: &str) -> Result<Option<ThumbnailMetadata>> {
        let metadata_path = self.thumbnails_dir.join(format!("{}.json", commit_id));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&metadata_path)
            .context("Failed to read thumbnail metadata")?;

        let metadata: ThumbnailMetadata = serde_json::from_str(&contents)
            .context("Failed to parse thumbnail metadata")?;

        Ok(Some(metadata))
    }

    /// Get path to thumbnail image file
    pub fn get_thumbnail_path(&self, commit_id: &str) -> Result<Option<PathBuf>> {
        // Try common image extensions
        for ext in &["jpg", "jpeg", "png", "gif"] {
            let path = self.thumbnails_dir.join(format!("{}.{}", commit_id, ext));
            if path.exists() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// List all thumbnails
    pub fn list_thumbnails(&self) -> Result<Vec<ThumbnailMetadata>> {
        if !self.thumbnails_dir.exists() {
            return Ok(vec![]);
        }

        let mut thumbnails = Vec::new();

        for entry in fs::read_dir(&self.thumbnails_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<ThumbnailMetadata>(&contents) {
                        thumbnails.push(metadata);
                    }
                }
            }
        }

        Ok(thumbnails)
    }

    /// Delete a thumbnail
    pub fn delete_thumbnail(&self, commit_id: &str) -> Result<()> {
        // Delete image file
        if let Some(image_path) = self.get_thumbnail_path(commit_id)? {
            fs::remove_file(&image_path)
                .context("Failed to delete thumbnail image")?;
        }

        // Delete metadata
        let metadata_path = self.thumbnails_dir.join(format!("{}.json", commit_id));
        if metadata_path.exists() {
            fs::remove_file(&metadata_path)
                .context("Failed to delete thumbnail metadata")?;
        }

        Ok(())
    }

    /// Save thumbnail metadata to JSON file
    fn save_metadata(&self, metadata: &ThumbnailMetadata) -> Result<()> {
        let path = self.thumbnails_dir.join(format!("{}.json", metadata.commit_id));
        let json = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;
        fs::write(&path, json)
            .context("Failed to write metadata file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thumbnail_manager_creation() {
        let manager = ThumbnailManager::new(Path::new("/tmp/test-repo"));
        assert_eq!(
            manager.thumbnails_dir,
            PathBuf::from("/tmp/test-repo/.auxin/thumbnails")
        );
    }

    #[test]
    fn test_thumbnail_metadata_creation() {
        let metadata = ThumbnailMetadata::new("abc123", "jpg", 50000);
        assert_eq!(metadata.commit_id, "abc123");
        assert_eq!(metadata.format, "jpg");
        assert_eq!(metadata.size_bytes, 50000);
        assert_eq!(metadata.source_path, None);
    }

    #[test]
    fn test_thumbnail_metadata_builder() {
        let metadata = ThumbnailMetadata::new("abc123", "png", 100000)
            .with_source("/path/to/image.png")
            .with_dimensions(1920, 1080);

        assert_eq!(metadata.source_path, Some("/path/to/image.png".to_string()));
        assert_eq!(metadata.width, Some(1920));
        assert_eq!(metadata.height, Some(1080));
    }
}
