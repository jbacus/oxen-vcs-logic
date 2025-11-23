//! Screenshot capture for project snapshots
//!
//! Captures visual snapshots of application windows when commits are created.
//! These screenshots provide a visual record of the project state at commit time,
//! complementing the metadata and audio bounces.

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Metadata about a screenshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotMetadata {
    /// Commit ID this screenshot is associated with
    pub commit_id: String,

    /// Application that was captured
    pub application: String,

    /// Window title at time of capture
    pub window_title: Option<String>,

    /// Image format (png, jpg)
    pub format: String,

    /// File size in bytes
    pub size_bytes: u64,

    /// Width in pixels
    pub width: Option<u32>,

    /// Height in pixels
    pub height: Option<u32>,

    /// When the screenshot was captured
    pub captured_at: chrono::DateTime<chrono::Utc>,
}

impl ScreenshotMetadata {
    /// Create new screenshot metadata
    pub fn new(commit_id: &str, application: &str, format: &str, size_bytes: u64) -> Self {
        Self {
            commit_id: commit_id.to_string(),
            application: application.to_string(),
            window_title: None,
            format: format.to_string(),
            size_bytes,
            width: None,
            height: None,
            captured_at: chrono::Utc::now(),
        }
    }

    /// Set window title
    pub fn with_window_title(mut self, title: &str) -> Self {
        self.window_title = Some(title.to_string());
        self
    }

    /// Set dimensions
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }
}

/// Manages screenshot capture and storage
pub struct ScreenshotManager {
    /// Root directory of the repository
    repo_root: PathBuf,

    /// Directory where screenshots are stored
    screenshots_dir: PathBuf,
}

impl ScreenshotManager {
    /// Create a new screenshot manager for a repository
    pub fn new(repo_root: &Path) -> Self {
        let screenshots_dir = repo_root.join(".auxin").join("screenshots");
        Self {
            repo_root: repo_root.to_path_buf(),
            screenshots_dir,
        }
    }

    /// Initialize screenshot storage directory
    pub fn init(&self) -> Result<()> {
        if !self.screenshots_dir.exists() {
            fs::create_dir_all(&self.screenshots_dir)
                .context("Failed to create screenshots directory")?;
        }
        Ok(())
    }

    /// Capture a screenshot of the frontmost window
    ///
    /// Uses macOS screencapture to capture the active window
    pub fn capture_frontmost_window(
        &self,
        commit_id: &str,
        application: &str,
    ) -> Result<ScreenshotMetadata> {
        self.init()?;

        // Generate filename: commit_id.png
        let screenshot_filename = format!("{}.png", commit_id);
        let screenshot_path = self.screenshots_dir.join(&screenshot_filename);

        // Use screencapture to capture the frontmost window
        // -o: Don't show window shadow
        // -x: Don't play sound
        // -w: Window mode - capture the frontmost window
        let status = Command::new("screencapture")
            .args(&["-o", "-x", "-w", screenshot_path.to_str().unwrap()])
            .status()
            .context("Failed to run screencapture command")?;

        if !status.success() {
            return Err(anyhow!("screencapture command failed"));
        }

        // Verify the screenshot was created
        if !screenshot_path.exists() {
            return Err(anyhow!("Screenshot file was not created"));
        }

        // Get file size
        let file_meta =
            fs::metadata(&screenshot_path).context("Failed to read screenshot metadata")?;

        let metadata = ScreenshotMetadata::new(commit_id, application, "png", file_meta.len());

        // Try to get dimensions using sips (macOS image tool)
        if let Ok(dims) = self.get_image_dimensions(&screenshot_path) {
            let metadata = metadata.with_dimensions(dims.0, dims.1);
            self.save_metadata(&metadata)?;
            Ok(metadata)
        } else {
            self.save_metadata(&metadata)?;
            Ok(metadata)
        }
    }

    /// Capture a screenshot by window ID
    ///
    /// Uses macOS screencapture to capture a specific window
    pub fn capture_window_by_id(
        &self,
        commit_id: &str,
        window_id: u32,
        application: &str,
    ) -> Result<ScreenshotMetadata> {
        self.init()?;

        let screenshot_filename = format!("{}.png", commit_id);
        let screenshot_path = self.screenshots_dir.join(&screenshot_filename);

        // Use screencapture with -l (window ID)
        let status = Command::new("screencapture")
            .args(&[
                "-o",
                "-x",
                "-l",
                &window_id.to_string(),
                screenshot_path.to_str().unwrap(),
            ])
            .status()
            .context("Failed to run screencapture command")?;

        if !status.success() {
            return Err(anyhow!("screencapture command failed"));
        }

        if !screenshot_path.exists() {
            return Err(anyhow!("Screenshot file was not created"));
        }

        let file_meta =
            fs::metadata(&screenshot_path).context("Failed to read screenshot metadata")?;

        let metadata = ScreenshotMetadata::new(commit_id, application, "png", file_meta.len());

        if let Ok(dims) = self.get_image_dimensions(&screenshot_path) {
            let metadata = metadata.with_dimensions(dims.0, dims.1);
            self.save_metadata(&metadata)?;
            Ok(metadata)
        } else {
            self.save_metadata(&metadata)?;
            Ok(metadata)
        }
    }

    /// Find and capture the window for a specific application
    ///
    /// This searches for windows belonging to the specified application
    /// and captures the first one found
    pub fn capture_application_window(
        &self,
        commit_id: &str,
        application: &str,
    ) -> Result<ScreenshotMetadata> {
        self.init()?;

        // Try to find the application's window using AppleScript
        let script = format!(
            r#"
            tell application "System Events"
                tell process "{}"
                    if exists window 1 then
                        get id of window 1
                    end if
                end tell
            end tell
            "#,
            application
        );

        let output = Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .context("Failed to run AppleScript")?;

        if output.status.success() {
            let window_id_str = String::from_utf8_lossy(&output.stdout);
            if let Ok(window_id) = window_id_str.trim().parse::<u32>() {
                return self.capture_window_by_id(commit_id, window_id, application);
            }
        }

        // Fallback: capture frontmost window
        self.capture_frontmost_window(commit_id, application)
    }

    /// Get screenshot metadata for a commit
    pub fn get_screenshot(&self, commit_id: &str) -> Result<Option<ScreenshotMetadata>> {
        let metadata_path = self.screenshots_dir.join(format!("{}.json", commit_id));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let contents =
            fs::read_to_string(&metadata_path).context("Failed to read screenshot metadata")?;

        let metadata: ScreenshotMetadata =
            serde_json::from_str(&contents).context("Failed to parse screenshot metadata")?;

        Ok(Some(metadata))
    }

    /// Get path to screenshot file
    pub fn get_screenshot_path(&self, commit_id: &str) -> Result<Option<PathBuf>> {
        for ext in &["png", "jpg", "jpeg"] {
            let path = self.screenshots_dir.join(format!("{}.{}", commit_id, ext));
            if path.exists() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// List all screenshots
    pub fn list_screenshots(&self) -> Result<Vec<ScreenshotMetadata>> {
        if !self.screenshots_dir.exists() {
            return Ok(vec![]);
        }

        let mut screenshots = Vec::new();

        for entry in fs::read_dir(&self.screenshots_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<ScreenshotMetadata>(&contents) {
                        screenshots.push(metadata);
                    }
                }
            }
        }

        // Sort by capture date (newest first)
        screenshots.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));

        Ok(screenshots)
    }

    /// Delete a screenshot
    pub fn delete_screenshot(&self, commit_id: &str) -> Result<()> {
        // Delete image file
        if let Some(image_path) = self.get_screenshot_path(commit_id)? {
            fs::remove_file(&image_path).context("Failed to delete screenshot image")?;
        }

        // Delete metadata
        let metadata_path = self.screenshots_dir.join(format!("{}.json", commit_id));
        if metadata_path.exists() {
            fs::remove_file(&metadata_path).context("Failed to delete screenshot metadata")?;
        }

        Ok(())
    }

    /// Get image dimensions using sips (macOS image tool)
    fn get_image_dimensions(&self, path: &Path) -> Result<(u32, u32)> {
        let output = Command::new("sips")
            .args(&["-g", "pixelWidth", "-g", "pixelHeight", path.to_str().unwrap()])
            .output()
            .context("Failed to run sips command")?;

        if !output.status.success() {
            return Err(anyhow!("sips command failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut width: Option<u32> = None;
        let mut height: Option<u32> = None;

        for line in stdout.lines() {
            if line.contains("pixelWidth:") {
                if let Some(value_str) = line.split(':').nth(1) {
                    if let Ok(val) = value_str.trim().parse::<u32>() {
                        width = Some(val);
                    }
                }
            } else if line.contains("pixelHeight:") {
                if let Some(value_str) = line.split(':').nth(1) {
                    if let Ok(val) = value_str.trim().parse::<u32>() {
                        height = Some(val);
                    }
                }
            }
        }

        match (width, height) {
            (Some(w), Some(h)) => Ok((w, h)),
            _ => Err(anyhow!("Could not parse dimensions")),
        }
    }

    /// Save screenshot metadata to JSON file
    fn save_metadata(&self, metadata: &ScreenshotMetadata) -> Result<()> {
        let path = self
            .screenshots_dir
            .join(format!("{}.json", metadata.commit_id));
        let json =
            serde_json::to_string_pretty(metadata).context("Failed to serialize metadata")?;
        fs::write(&path, json).context("Failed to write metadata file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenshot_manager_creation() {
        let manager = ScreenshotManager::new(Path::new("/tmp/test-repo"));
        assert_eq!(
            manager.screenshots_dir,
            PathBuf::from("/tmp/test-repo/.auxin/screenshots")
        );
    }

    #[test]
    fn test_screenshot_metadata_creation() {
        let metadata = ScreenshotMetadata::new("abc123", "Logic Pro", "png", 500000);
        assert_eq!(metadata.commit_id, "abc123");
        assert_eq!(metadata.application, "Logic Pro");
        assert_eq!(metadata.format, "png");
        assert_eq!(metadata.size_bytes, 500000);
    }

    #[test]
    fn test_screenshot_metadata_builder() {
        let metadata = ScreenshotMetadata::new("abc123", "SketchUp", "png", 100000)
            .with_window_title("My Project")
            .with_dimensions(1920, 1080);

        assert_eq!(metadata.window_title, Some("My Project".to_string()));
        assert_eq!(metadata.width, Some(1920));
        assert_eq!(metadata.height, Some(1080));
    }
}
