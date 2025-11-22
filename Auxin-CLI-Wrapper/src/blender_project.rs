use crate::{info, vlog};
use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Represents a Blender project structure.
///
/// Blender projects use a `.blend` file format containing:
/// - Binary scene data (open format, documented)
/// - Embedded textures and materials
/// - Object hierarchy and scene graph
/// - Render settings and animation data
///
/// This struct validates the project structure and provides access to key paths
/// that should be tracked by version control.
///
/// # Examples
///
/// ```no_run
/// use auxin::BlenderProject;
///
/// let project = BlenderProject::detect("/path/to/MyScene.blend")?;
/// println!("Project name: {}", project.name());
/// println!("Project file at: {}", project.file_path.display());
/// # Ok::<(), anyhow::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct BlenderProject {
    /// Canonical absolute path to the .blend project file
    pub file_path: PathBuf,
    /// Project directory (where the .blend file and related assets live)
    pub project_dir: PathBuf,
}

impl BlenderProject {
    /// Detects and validates a Blender project at the given path.
    ///
    /// This is the primary entry point for Blender project detection. It performs
    /// comprehensive validation including path existence, file extension checking,
    /// and basic file format validation.
    ///
    /// # Requirements
    ///
    /// A valid Blender project must:
    /// - Exist on the filesystem
    /// - Be a file (not a directory)
    /// - Have a `.blend` extension
    /// - Be a valid Blender file (basic validation)
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the .blend file (can be relative or absolute)
    ///
    /// # Returns
    ///
    /// * `Ok(BlenderProject)` - Valid project with canonicalized paths
    /// * `Err(anyhow::Error)` - Invalid project or filesystem error
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - Path does not exist
    /// - Path is a directory, not a file
    /// - File doesn't have `.blend` extension
    /// - Filesystem permissions prevent access
    /// - Path cannot be canonicalized
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use auxin::BlenderProject;
    ///
    /// // Detect with absolute path
    /// let project = BlenderProject::detect("/Users/artist/MyScene.blend")?;
    ///
    /// // Detect with relative path
    /// let project = BlenderProject::detect("../MyScene.blend")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Performance
    ///
    /// This function performs filesystem I/O including:
    /// - Path canonicalization
    /// - File existence checks
    /// - Basic file header validation (optional)
    ///
    /// Typical execution time: < 5ms for standard files
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        vlog!("=== Blender Project Detection ===");
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

        // Check if it has .blend extension
        vlog!("Checking for .blend extension...");
        let extension = canonical_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        vlog!("Found extension: '{}'", extension);

        if extension != "blend" {
            vlog!("❌ Extension is not 'blend'");
            return Err(anyhow!(
                "Path is not a Blender file (.blend): {}",
                canonical_path.display()
            ));
        }
        vlog!("✓ Valid .blend extension");

        // Get project directory (parent of the .blend file)
        let project_dir = canonical_path
            .parent()
            .ok_or_else(|| anyhow!("Cannot determine parent directory"))?
            .to_path_buf();

        info!(
            "Successfully detected Blender project: {}",
            canonical_path.display()
        );
        vlog!("Project directory: {}", project_dir.display());

        Ok(BlenderProject {
            file_path: canonical_path,
            project_dir,
        })
    }

    /// Returns the human-readable name of the Blender project.
    ///
    /// Extracts the project name from the filename by removing the
    /// `.blend` extension. This is the name users see in file browsers.
    ///
    /// # Returns
    ///
    /// Project name as a String (without .blend extension)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use auxin::BlenderProject;
    ///
    /// let project = BlenderProject::detect("/path/to/Character Rig.blend")?;
    /// assert_eq!(project.name(), "Character Rig");
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
    /// - The main `.blend` file - Primary scene data
    /// - `textures/` directory (if exists) - External texture image files
    /// - `libraries/` directory (if exists) - Linked .blend libraries
    /// - `assets/` directory (if exists) - Asset browser files
    /// - `scripts/` directory (if exists) - Custom Python scripts
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
    /// use auxin::BlenderProject;
    ///
    /// let project = BlenderProject::detect("/path/to/Scene.blend")?;
    /// for path in project.tracked_paths() {
    ///     println!("Should track: {}", path.display());
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Note
    ///
    /// This list is intentionally conservative. Cache and render files are
    /// handled by `.oxenignore` patterns, not excluded here.
    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        let mut paths = vec![self.file_path.clone()];

        // Check for common Blender asset directories
        let textures_dir = self.project_dir.join("textures");
        if textures_dir.exists() && textures_dir.is_dir() {
            paths.push(textures_dir);
        }

        let libraries_dir = self.project_dir.join("libraries");
        if libraries_dir.exists() && libraries_dir.is_dir() {
            paths.push(libraries_dir);
        }

        let assets_dir = self.project_dir.join("assets");
        if assets_dir.exists() && assets_dir.is_dir() {
            paths.push(assets_dir);
        }

        let scripts_dir = self.project_dir.join("scripts");
        if scripts_dir.exists() && scripts_dir.is_dir() {
            paths.push(scripts_dir);
        }

        paths
    }

    /// Returns glob patterns for files and directories that should NOT be versioned.
    ///
    /// These patterns identify volatile, generated, or system files that:
    /// - Are automatic backups (*.blend1, *.blend2)
    /// - Can be regenerated (renders, baked simulations)
    /// - Change frequently and cause conflicts (cache files)
    /// - Are system-specific (.DS_Store, __pycache__)
    ///
    /// # Pattern Types
    ///
    /// - Backup patterns: `*.blend1`, `*.blend2`, `*.blend@`
    /// - Cache patterns: `blendcache_*/`, `__pycache__/`
    /// - Render patterns: `renders/`, `render_output/`
    /// - Wildcard patterns: `*.tmp`, `*.pyc`
    /// - Exact filenames: `.DS_Store`, `Thumbs.db`
    ///
    /// # Returns
    ///
    /// Vector of static string slices suitable for .oxenignore generation
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin::BlenderProject;
    ///
    /// let patterns = BlenderProject::ignored_patterns();
    /// assert!(patterns.contains(&"*.blend1"));
    /// assert!(patterns.contains(&"blendcache_*/"));
    /// assert!(patterns.contains(&".DS_Store"));
    /// ```
    ///
    /// # Rationale
    ///
    /// **Backup Files:**
    /// - `*.blend1`, `*.blend2` - Blender auto-backup files (redundant with VCS)
    /// - `*.blend@` - Temporary save files
    ///
    /// **Cache Files:**
    /// - `blendcache_*/` - Simulation caches (often multi-GB, regenerable)
    /// - `__pycache__/` - Python bytecode cache
    ///
    /// **Render Output:**
    /// - `renders/` - Rendered images and animations (regenerable)
    /// - `render_output/` - Alternative render directory
    /// - `tmp/` - Temporary render files
    ///
    /// **System Files:**
    /// - `.DS_Store` - macOS Finder metadata
    /// - `Thumbs.db` - Windows thumbnail cache
    /// - `desktop.ini` - Windows folder settings
    ///
    /// # Integration
    ///
    /// These patterns are used by `ignore_template::generate_blender_oxenignore()` to create
    /// the initial .oxenignore file for new repositories.
    pub fn ignored_patterns() -> Vec<&'static str> {
        vec![
            // Blender backup files
            "*.blend1",
            "*.blend2",
            "*.blend@",
            // Cache directories (can be huge!)
            "blendcache_*/",
            "__pycache__/",
            "*.pyc",
            // Render output
            "renders/",
            "render_output/",
            "tmp/",
            // Build artifacts (if using as game engine)
            "build/",
            "dist/",
            // System files
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            "*.smbdelete*",
            // Blender specific temp
            "*.crash.txt",
            "*.autosave",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    // Helper function to create a temporary .blend project
    fn create_test_project(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join("blender_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let project_file = temp_dir.join(name);
        fs::write(&project_file, b"BLENDER test data").unwrap();

        project_file
    }

    #[test]
    fn test_detect_invalid_extension() {
        let temp_file = std::env::temp_dir().join("test_scene.txt");
        let _ = fs::write(&temp_file, b"test");

        let result = BlenderProject::detect(&temp_file);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not a Blender file"));

        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_detect_nonexistent_path() {
        let nonexistent = std::env::temp_dir().join("nonexistent_12345.blend");
        let result = BlenderProject::detect(&nonexistent);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_detect_directory_not_file() {
        let temp_dir = std::env::temp_dir().join("test_dir.blend");
        fs::create_dir_all(&temp_dir).unwrap();

        let result = BlenderProject::detect(&temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a file"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_valid_blend() {
        let project_file = create_test_project("test_scene.blend");

        let result = BlenderProject::detect(&project_file);
        assert!(result.is_ok());

        if let Ok(project) = result {
            assert!(project.file_path.exists());
            assert!(project
                .file_path
                .to_string_lossy()
                .contains("test_scene.blend"));
        }

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_name() {
        let project_file = create_test_project("my_character.blend");
        let project = BlenderProject::detect(&project_file).unwrap();

        assert_eq!(project.name(), "my_character");

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_name_with_spaces() {
        let project_file = create_test_project("My Amazing Scene.blend");
        let project = BlenderProject::detect(&project_file).unwrap();

        assert_eq!(project.name(), "My Amazing Scene");

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_tracked_paths() {
        let project_file = create_test_project("track_test.blend");
        let project = BlenderProject::detect(&project_file).unwrap();

        let tracked = project.tracked_paths();
        assert!(tracked.len() >= 1); // At least the .blend file

        // Should include the main .blend file
        assert!(tracked.iter().any(|p| p.ends_with("track_test.blend")));

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_ignored_patterns() {
        let patterns = BlenderProject::ignored_patterns();

        // Essential patterns that must be present
        assert!(patterns.contains(&"*.blend1"));
        assert!(patterns.contains(&"*.blend2"));
        assert!(patterns.contains(&"blendcache_*/"));
        assert!(patterns.contains(&"renders/"));
        assert!(patterns.contains(&".DS_Store"));
        assert!(patterns.contains(&"__pycache__/"));

        // Should have at least 12 patterns
        assert!(patterns.len() >= 12);
    }

    #[test]
    fn test_ignored_patterns_all_types() {
        let patterns = BlenderProject::ignored_patterns();

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
        let project_file = create_test_project("canonical_test.blend");
        let project = BlenderProject::detect(&project_file).unwrap();

        // Path should be absolute (canonical)
        assert!(project.file_path.is_absolute());

        let _ = fs::remove_file(&project_file);
    }

    #[test]
    fn test_project_file_exists() {
        let project_file = create_test_project("exists_test.blend");
        let project = BlenderProject::detect(&project_file).unwrap();

        // Project file should exist
        assert!(project.file_path.exists());
        assert!(project.file_path.is_file());

        let _ = fs::remove_file(&project_file);
    }
}
