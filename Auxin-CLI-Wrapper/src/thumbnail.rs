//! Thumbnail management for commit snapshots
//!
//! Extracts and manages project thumbnails (screenshots) to provide visual
//! representation of commits in the UI. For Logic Pro projects, this extracts
//! the WindowImage.jpg file that Logic saves on each project save.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Visual difference analysis between two thumbnails
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThumbnailDiff {
    /// Commit A identifier
    pub commit_a: String,

    /// Commit B identifier
    pub commit_b: String,

    /// Estimated pixel difference percentage (0.0 - 100.0)
    pub difference_percent: f64,

    /// File size difference in bytes (B - A)
    pub size_diff_bytes: i64,

    /// Dimension difference (if different sizes)
    pub dimension_diff: Option<String>,

    /// Change description
    pub description: String,
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

    /// Compare two thumbnails and generate diff
    pub fn compare_thumbnails(
        &self,
        commit_a: &str,
        commit_b: &str,
    ) -> Result<ThumbnailDiff> {
        let meta_a = self.get_thumbnail(commit_a)?
            .ok_or_else(|| anyhow!("No thumbnail for commit {}", commit_a))?;
        let meta_b = self.get_thumbnail(commit_b)?
            .ok_or_else(|| anyhow!("No thumbnail for commit {}", commit_b))?;

        let path_a = self.get_thumbnail_path(commit_a)?
            .ok_or_else(|| anyhow!("No thumbnail image for commit {}", commit_a))?;
        let path_b = self.get_thumbnail_path(commit_b)?
            .ok_or_else(|| anyhow!("No thumbnail image for commit {}", commit_b))?;

        // Calculate size difference
        let size_diff_bytes = meta_b.size_bytes as i64 - meta_a.size_bytes as i64;

        // Check dimension differences
        let dimension_diff = match (&meta_a.width, &meta_a.height, &meta_b.width, &meta_b.height) {
            (Some(w1), Some(h1), Some(w2), Some(h2)) if w1 != w2 || h1 != h2 => {
                Some(format!("{}x{} → {}x{}", w1, h1, w2, h2))
            }
            _ => None,
        };

        // Use ImageMagick compare if available for pixel-level diff
        let difference_percent = self.calculate_image_difference(&path_a, &path_b)
            .unwrap_or_else(|_| {
                // Fallback: estimate based on file size
                if size_diff_bytes == 0 {
                    0.0
                } else {
                    let max_size = meta_a.size_bytes.max(meta_b.size_bytes) as f64;
                    (size_diff_bytes.abs() as f64 / max_size) * 100.0
                }
            });

        let description = format!(
            "Visual difference: {:.1}%, Size: {} bytes",
            difference_percent,
            if size_diff_bytes > 0 { "+" } else { "" }.to_string() + &size_diff_bytes.to_string()
        );

        Ok(ThumbnailDiff {
            commit_a: commit_a.to_string(),
            commit_b: commit_b.to_string(),
            difference_percent,
            size_diff_bytes,
            dimension_diff,
            description,
        })
    }

    /// Calculate pixel-level image difference using ImageMagick compare
    fn calculate_image_difference(&self, path_a: &Path, path_b: &Path) -> Result<f64> {
        // Try using ImageMagick's compare command
        let output = Command::new("compare")
            .args(&[
                "-metric", "RMSE",
                path_a.to_str().unwrap(),
                path_b.to_str().unwrap(),
                "null:"
            ])
            .output();

        match output {
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                // Parse output like "1234.56 (0.0189)"
                if let Some(pct_str) = stderr.split('(').nth(1) {
                    if let Some(pct) = pct_str.trim_end_matches(')').parse::<f64>().ok() {
                        return Ok(pct * 100.0); // Convert to percentage
                    }
                }
                Err(anyhow!("Could not parse compare output"))
            }
            Err(_) => {
                // ImageMagick not available, use simpler method
                Err(anyhow!("ImageMagick compare not available"))
            }
        }
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
