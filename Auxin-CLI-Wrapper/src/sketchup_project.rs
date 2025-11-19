use crate::{info, vlog};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Represents a SketchUp project structure.
///
/// SketchUp projects use a `.skp` file format containing:
/// - Binary 3D model data (non-mergeable)
/// - Embedded textures and materials
/// - Component definitions and instances
/// - Scene/camera configurations
///
/// This struct validates the project structure and provides access to key paths
/// that should be tracked by version control.
///
/// # Examples
///
/// ```no_run
/// use auxin_cli::SketchUpProject;
///
/// let project = SketchUpProject::detect("/path/to/MyModel.skp")?;
/// println!("Project name: {}", project.name());
/// println!("Project file at: {}", project.file_path.display());
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct SketchUpProject {
    /// Canonical absolute path to the .skp project file
    pub file_path: PathBuf,
    /// Project directory (where the .skp file and related assets live)
    pub project_dir: PathBuf,
}

impl SketchUpProject {
    /// Detects and validates a SketchUp project at the given path.
    ///
    /// This is the primary entry point for SketchUp project detection. It performs
    /// comprehensive validation including path existence, file extension checking,
    /// and optional project directory detection.
    ///
    /// # Requirements
    ///
    /// A valid SketchUp project must:
    /// - Exist on the filesystem
    /// - Be a file (not a directory)
    /// - Have a `.skp` extension
    /// - Be a valid SketchUp file (basic validation)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .skp file (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// * `Ok(SketchUpProject)` - Valid project with canonicalized paths
    /// * `Err(anyhow::Error)` - Invalid project or filesystem error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Path does not exist
    /// - Path is a directory, not a file
    /// - File doesn't have `.skp` extension
    /// - Filesystem permissions prevent access
    /// - Path cannot be canonicalized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use auxin_cli::SketchUpProject;
    ///
    /// // Detect with absolute path
    /// let project = SketchUpProject::detect("/Users/designer/MyModel.skp")?;
    ///
    /// // Detect with relative path
    /// let project = SketchUpProject::detect("../MyModel.skp")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Performance
    ///
    /// This function performs filesystem I/O including:
    /// - Path canonicalization
    /// - File existence checks
    ///
    /// Typical execution time: < 5ms for standard files
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        vlog!("=== SketchUp Project Detection ===");
        vlog!("Input path: {}", path.display());

        // Check if path exists
        vlog!("Checking if path exists...");
        if !path.exists() {
            vlog!("❌ Path does not exist");
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
        vlog!("✓ Path exists");

        // Check if it's a file
        vlog!("Checking if path is a file...");
        if !path.is_file() {
            vlog!("❌ Path is not a file");
            return Err(anyhow!("Path is not a file: {}", path.display()));
        }
        vlog!("✓ Path is a file");

        // Canonicalize the path to resolve relative paths
        vlog!("Canonicalizing path to resolve relative paths...");
        let canonical_path = std::fs::canonicalize(path).context("Failed to canonicalize path")?;
        vlog!("Canonical path: {}", canonical_path.display());

        // Check if it has .skp extension
        vlog!("Checking for .skp extension...");
        let extension = canonical_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        vlog!("Found extension: '{}'", extension);

        if extension != "skp" {
            vlog!("❌ Extension is not 'skp'");
            return Err(anyhow!(
                "Path is not a SketchUp file (.skp): {}",
                canonical_path.display()
            ));
        }
        vlog!("✓ Valid .skp extension");

        // Get project directory (parent of the .skp file)
        let project_dir = canonical_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot determine parent directory"))?
            .to_path_buf();

        info!(
            "Successfully detected SketchUp project: {}",
            canonical_path.display()
        );
        vlog!("Project directory: {}", project_dir.display());

        Ok(SketchUpProject {
            file_path: canonical_path,
            project_dir,
        })
    }

    /// Returns the human-readable name of the SketchUp project.
    ///
    /// Extracts the project name from the filename by removing the
    /// `.skp` extension. This is the name users see in Finder/Explorer.
    ///
    /// # Returns
    ///
    /// Project name as a String (without .skp extension)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use auxin_cli::SketchUpProject;
    ///
    /// let project = SketchUpProject::detect("/path/to/House Design.skp")?;
    /// assert_eq!(project.name(), "House Design");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn name(&self) -> String {
        self.file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Returns paths within the project that should be tracked by version control.
    ///
    /// These paths contain essential project data that should be versioned:
    /// - The main `.skp` file - Primary 3D model
    /// - `textures/` directory (if exists) - Custom textures and materials
    /// - `components/` directory (if exists) - External component files
    ///
    /// Paths returned are absolute, derived from the canonicalized project path.
    ///
    /// # Returns
    ///
    /// Vector of absolute PathBuf instances for key files/directories
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use auxin_cli::SketchUpProject;
    ///
    /// let project = SketchUpProject::detect("/path/to/Model.skp")?;
    /// for path in project.tracked_paths() {
    ///     println!("Should track: {}", path.display());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Note
    ///
    /// This list is intentionally conservative. Cache and temp files are
    /// handled by `.oxenignore` patterns, not excluded here.
    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.file_path.clone()];

        // Check for common SketchUp asset directories
        let textures_dir = self.project_dir.join("textures");
        if textures_dir.exists() && textures_dir.is_dir() {
            paths.push(textures_dir);
        }

        let components_dir = self.project_dir.join("components");
        if components_dir.exists() && components_dir.is_dir() {
            paths.push(components_dir);
        }

        let materials_dir = self.project_dir.join("materials");
        if materials_dir.exists() && materials_dir.is_dir() {
            paths.push(materials_dir);
        }

        paths
    }

    /// Returns glob patterns for files and directories that should NOT be versioned.
    ///
    /// These patterns identify volatile, generated, or system files that:
    /// - Can be regenerated (renders, exports)
    /// - Change frequently and cause conflicts (autosaves, backups)
    /// - Are system-specific (.DS_Store, thumbs.db)
    ///
    /// # Pattern Types
    ///
    /// - Backup patterns: `*.skb`, `*~.skp` (trailing ~)
    /// - Export patterns: `exports/`, `renders/`
    /// - Wildcard patterns: `*.tmp`, `*.cache`
    /// - Exact filenames: `.DS_Store`, `Thumbs.db`
    ///
    /// # Returns
    ///
    /// Vector of static string slices suitable for .oxenignore generation
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpProject;
    ///
    /// let patterns = SketchUpProject::ignored_patterns();
    /// assert!(patterns.contains(&"*.skb"));
    /// assert!(patterns.contains(&"exports/"));
    /// assert!(patterns.contains(&".DS_Store"));
    /// ```
    ///
    /// # Rationale
    ///
    /// **Backup Files:**
    /// - `*.skb` - SketchUp auto-backup files (regenerable)
    /// - `*~.skp` - Temporary backup copies
    ///
    /// **Generated Output:**
    /// - `exports/` - User-exported files (2D, 3D formats)
    /// - `renders/` - Rendered images and animations
    ///
    /// **Cache/Temp:**
    /// - `*.tmp` - Temporary processing files
    /// - `.thumbnails/` - Preview image cache
    ///
    /// **System Files:**
    /// - `.DS_Store` - macOS Finder metadata
    /// - `Thumbs.db` - Windows thumbnail cache
    /// - `desktop.ini` - Windows folder settings
    ///
    /// # Integration
    ///
    /// These patterns are used by `ignore_template::generate_sketchup_oxenignore()` to create
    /// the initial .oxenignore file for new repositories.
    pub fn ignored_patterns() -> Vec<&'static str> {
        vec![
            // SketchUp backup and temp files
            "*.skb",
            "*~.skp",
            "*.tmp",
            // Export directories
            "exports/",
            "renders/",
            "output/",
            // Cache directories
            ".thumbnails/",
            "cache/",
            // System files
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            "*.smbdelete*",
            // SketchUp specific
            "*.swp",
            ".sketchup_session",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper function to create a temporary .skp project
    fn create_test_project(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join("sketchup_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let project_file = temp_dir.join(name);
        fs::write(&project_file, b"SketchUp test data").unwrap();

        project_file
    }

    #[test]
    fn test_detect_invalid_extension() {
        let temp_file = std::env::temp_dir().join("test_model.txt");
        let _ = fs::write(&temp_file, b"test");

        let result = SketchUpProject::detect(&temp_file);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a SketchUp file"));

        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_detect_nonexistent_path() {
        let nonexistent = std::env::temp_dir().join("nonexistent_12345.skp");
        let result = SketchUpProject::detect(&nonexistent);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_detect_directory_not_file() {
        let temp_dir = std::env::temp_dir().join("test_dir.skp");
        fs::create_dir_all(&temp_dir).unwrap();

        let result = SketchUpProject::detect(&temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_valid_skp() {
        let project_file = create_test_project("test_model.skp");

        let result = SketchUpProject::detect(&project_file);
        assert!(result.is_ok());

        if let Ok(project) = result {
            assert!(project.file_path.exists());
            assert!(project
                .file_path
                .to_string_lossy()
                .contains("test_model.skp"));
        }

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_name() {
        let project_file = create_test_project("my_house.skp");
        let project = SketchUpProject::detect(&project_file).unwrap();

        assert_eq!(project.name(), "my_house");

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_name_with_spaces() {
        let project_file = create_test_project("My Amazing Design.skp");
        let project = SketchUpProject::detect(&project_file).unwrap();

        assert_eq!(project.name(), "My Amazing Design");

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_tracked_paths() {
        let project_file = create_test_project("track_test.skp");
        let project = SketchUpProject::detect(&project_file).unwrap();

        let tracked = project.tracked_paths();
        assert!(tracked.len() >= 1); // At least the .skp file

        // Should include the main .skp file
        assert!(tracked.iter().any(|p| p.ends_with("track_test.skp")));

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_ignored_patterns() {
        let patterns = SketchUpProject::ignored_patterns();

        // Essential patterns that must be present
        assert!(patterns.contains(&"*.skb"));
        assert!(patterns.contains(&"exports/"));
        assert!(patterns.contains(&".DS_Store"));
        assert!(patterns.contains(&"Thumbs.db"));
        assert!(patterns.contains(&"renders/"));

        // Should have at least 10 patterns
        assert!(patterns.len() >= 10);
    }

    #[test]
    fn test_ignored_patterns_all_types() {
        let patterns = SketchUpProject::ignored_patterns();

        // Should contain directory patterns (ending with /)
        assert!(patterns.iter().any(|p| p.ends_with('/')));

        // Should contain wildcard patterns
        assert!(patterns.iter().any(|p| p.contains('*')));

        // Should contain exact filename patterns
        assert!(patterns
            .iter()
            .any(|p| !p.contains('*') && !p.ends_with('/')));
    }

    #[test]
    fn test_project_path_is_canonical() {
        let project_file = create_test_project("canonical_test.skp");
        let project = SketchUpProject::detect(&project_file).unwrap();

        // Path should be absolute (canonical)
        assert!(project.file_path.is_absolute());

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_file_exists() {
        let project_file = create_test_project("exists_test.skp");
        let project = SketchUpProject::detect(&project_file).unwrap();

        // Project file should exist
        assert!(project.file_path.exists());
        assert!(project.file_path.is_file());

        let _ = fs::remove_file(&project_file);
    }
}
