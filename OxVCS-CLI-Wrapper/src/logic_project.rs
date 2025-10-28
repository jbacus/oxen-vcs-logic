use anyhow::{anyhow, Context, Result};
use colored::Colorize;
use std::path::{Path, PathBuf};
use crate::{vlog, info};

/// Represents a Logic Pro folder project structure
#[derive(Debug, Clone)]
pub struct LogicProject {
    pub path: PathBuf,
    pub project_data_path: PathBuf,
}

impl LogicProject {
    /// Detects if the given path is a valid Logic Pro folder project
    ///
    /// A valid Logic Pro folder project must:
    /// - Be a directory ending with .logicx
    /// - Contain a projectData file
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();

        vlog!("=== Logic Pro Project Detection ===");
        vlog!("Input path: {}", path.display());

        // Check if path exists
        vlog!("Checking if path exists...");
        if !path.exists() {
            vlog!("❌ Path does not exist");
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }
        vlog!("✓ Path exists");

        // Check if it's a directory
        vlog!("Checking if path is a directory...");
        if !path.is_dir() {
            vlog!("❌ Path is not a directory");
            return Err(anyhow!("Path is not a directory: {}", path.display()));
        }
        vlog!("✓ Path is a directory");

        // Canonicalize the path to resolve relative paths like "." to absolute paths
        // This ensures we can properly check the extension even when running from inside the .logicx directory
        vlog!("Canonicalizing path to resolve relative paths...");
        let canonical_path = std::fs::canonicalize(path)
            .context("Failed to canonicalize path")?;
        vlog!("Canonical path: {}", canonical_path.display());

        // Check if it has .logicx extension
        vlog!("Checking for .logicx extension...");
        let extension = canonical_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        vlog!("Found extension: '{}'", extension);

        if extension != "logicx" {
            vlog!("❌ Extension is not 'logicx'");
            return Err(anyhow!(
                "Path is not a Logic Pro folder project (.logicx): {}",
                canonical_path.display()
            ));
        }
        vlog!("✓ Valid .logicx extension");

        // Check for ProjectData file in Logic Pro's actual structure
        // Logic Pro stores project data in Alternatives/###/ProjectData
        vlog!("Searching for ProjectData file...");
        let project_data_path = Self::find_project_data(&canonical_path)?;

        info!("Successfully detected Logic Pro project: {}", canonical_path.display());
        vlog!("ProjectData location: {}", project_data_path.display());

        Ok(LogicProject {
            path: canonical_path,
            project_data_path,
        })
    }

    /// Finds the ProjectData file in a Logic Pro project
    ///
    /// Logic Pro stores project data in various locations:
    /// 1. Alternatives/###/ProjectData (most common, numbered alternatives)
    /// 2. ProjectData (root level, older format)
    /// 3. projectData (root level, case variation)
    fn find_project_data(project_path: &Path) -> Result<PathBuf> {
        vlog!("--- Searching for ProjectData file ---");

        // First, check for Alternatives directory (standard Logic Pro structure)
        let alternatives_path = project_path.join("Alternatives");
        vlog!("Checking for Alternatives directory: {}", alternatives_path.display());

        if alternatives_path.exists() && alternatives_path.is_dir() {
            vlog!("✓ Alternatives directory exists");

            // Look for numbered subdirectories in Alternatives/
            vlog!("Scanning subdirectories in Alternatives/...");
            if let Ok(entries) = std::fs::read_dir(&alternatives_path) {
                let mut found_dirs = Vec::new();

                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        let dir_name = entry_path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("?");
                        found_dirs.push(dir_name.to_string());

                        vlog!("  Checking subdirectory: {}", dir_name);

                        // Check for ProjectData in this alternative
                        let project_data = entry_path.join("ProjectData");
                        vlog!("    Looking for: {}", project_data.display());

                        if project_data.exists() {
                            vlog!("    ✓ Found ProjectData!");
                            return Ok(project_data);
                        } else {
                            vlog!("    ❌ ProjectData not found in this directory");
                        }
                    }
                }

                if !found_dirs.is_empty() {
                    vlog!("Found {} subdirectories: {}", found_dirs.len(), found_dirs.join(", "));
                } else {
                    vlog!("No subdirectories found in Alternatives/");
                }
            } else {
                vlog!("❌ Failed to read Alternatives directory");
            }
        } else {
            vlog!("❌ Alternatives directory does not exist");
        }

        // Fallback: Check for ProjectData at root level (various case variations)
        vlog!("Checking for ProjectData at root level...");
        let possible_names = vec!["ProjectData", "projectData", "Project Data"];

        for name in possible_names {
            let path = project_path.join(name);
            vlog!("  Checking: {}", path.display());
            if path.exists() {
                vlog!("  ✓ Found: {}", name);
                return Ok(path);
            } else {
                vlog!("  ❌ Not found: {}", name);
            }
        }

        vlog!("❌ No ProjectData file found in any expected location");

        Err(anyhow!(
            "No ProjectData file found in {}. \n\
             Expected locations:\n\
             - Alternatives/###/ProjectData\n\
             - ProjectData (root)\n\
             - projectData (root)\n\n\
             This may not be a valid Logic Pro project, or it hasn't been saved yet.",
            project_path.display()
        ))
    }

    /// Returns the name of the Logic Pro project
    pub fn name(&self) -> String {
        self.path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// Lists all key paths in the Logic Pro project that should be tracked
    pub fn tracked_paths(&self) -> Vec<PathBuf> {
        vec![
            self.path.join("Alternatives"),
            self.path.join("Resources"),
        ]
    }

    /// Lists paths that should be ignored
    pub fn ignored_patterns() -> Vec<&'static str> {
        vec![
            "Bounces/",
            "Freeze Files/",
            "*.nosync",
            "Autosave/",
            ".DS_Store",
            "*.smbdelete*",
            "Media.localized/",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    // Helper function to create a temporary .logicx project
    fn create_test_project(name: &str) -> PathBuf {
        let temp_dir = std::env::temp_dir().join(name);
        let alternatives_dir = temp_dir.join("Alternatives").join("001");
        fs::create_dir_all(&alternatives_dir).unwrap();

        let project_data = alternatives_dir.join("ProjectData");
        fs::write(&project_data, b"test project data").unwrap();

        temp_dir
    }

    #[test]
    fn test_detect_invalid_extension() {
        let temp_dir = std::env::temp_dir().join("test_project");
        let _ = fs::create_dir_all(&temp_dir);

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a Logic Pro folder project"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_nonexistent_path() {
        let nonexistent = std::env::temp_dir().join("nonexistent_path_12345.logicx");
        let result = LogicProject::detect(&nonexistent);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("does not exist"));
    }

    #[test]
    fn test_detect_file_not_directory() {
        let temp_file = std::env::temp_dir().join("test_file.logicx");
        fs::write(&temp_file, b"not a directory").unwrap();

        let result = LogicProject::detect(&temp_file);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not a directory"));

        let _ = fs::remove_file(&temp_file);
    }

    #[test]
    fn test_detect_missing_project_data() {
        let temp_dir = std::env::temp_dir().join("empty_project.logicx");
        fs::create_dir_all(&temp_dir).unwrap();

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No ProjectData file found"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_with_current_directory() {
        // Create a test .logicx directory with Logic Pro structure
        let temp_dir = std::env::temp_dir().join("test_project.logicx");
        let alternatives_dir = temp_dir.join("Alternatives").join("001");
        let _ = fs::create_dir_all(&alternatives_dir);

        // Create ProjectData file in Alternatives structure
        let project_data = alternatives_dir.join("ProjectData");
        let _ = fs::write(&project_data, b"test");

        // Save current directory
        let original_dir = env::current_dir().unwrap();

        // Change to the .logicx directory and test with "."
        env::set_current_dir(&temp_dir).unwrap();
        let result = LogicProject::detect(".");

        // Restore original directory
        env::set_current_dir(original_dir).unwrap();

        // Verify it worked
        assert!(result.is_ok());
        if let Ok(project) = result {
            assert!(project.project_data_path.exists());
            assert!(project.project_data_path.to_string_lossy().contains("ProjectData"));
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_alternatives_structure() {
        // Create a test .logicx directory with Alternatives/###/ProjectData structure
        let temp_dir = std::env::temp_dir().join("test_alternatives.logicx");
        let alternatives_dir = temp_dir.join("Alternatives").join("004");
        let _ = fs::create_dir_all(&alternatives_dir);

        // Create ProjectData file
        let project_data = alternatives_dir.join("ProjectData");
        let _ = fs::write(&project_data, b"test data");

        // Test detection
        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_ok());

        if let Ok(project) = result {
            assert_eq!(project.path, temp_dir.canonicalize().unwrap());
            assert!(project.project_data_path.ends_with("ProjectData"));
            assert!(project.project_data_path.exists());
        }

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_root_level_project_data() {
        // Test older format with ProjectData at root
        let temp_dir = std::env::temp_dir().join("old_project.logicx");
        fs::create_dir_all(&temp_dir).unwrap();

        let project_data = temp_dir.join("ProjectData");
        fs::write(&project_data, b"old format data").unwrap();

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_ok());

        if let Ok(project) = result {
            assert!(project.project_data_path.ends_with("ProjectData"));
            assert_eq!(project.project_data_path.parent().unwrap(), temp_dir.canonicalize().unwrap());
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_lowercase_project_data() {
        // Test case variation: projectData instead of ProjectData
        let temp_dir = std::env::temp_dir().join("lowercase_project.logicx");
        fs::create_dir_all(&temp_dir).unwrap();

        let project_data = temp_dir.join("projectData");
        fs::write(&project_data, b"lowercase variant").unwrap();

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_ok());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_multiple_alternatives() {
        // Test project with multiple alternative directories
        let temp_dir = std::env::temp_dir().join("multi_alt.logicx");
        let alt_001 = temp_dir.join("Alternatives").join("001");
        let alt_002 = temp_dir.join("Alternatives").join("002");

        fs::create_dir_all(&alt_001).unwrap();
        fs::create_dir_all(&alt_002).unwrap();

        // Only create ProjectData in 001
        fs::write(alt_001.join("ProjectData"), b"data 001").unwrap();

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_ok());

        if let Ok(project) = result {
            assert!(project.project_data_path.to_string_lossy().contains("001"));
        }

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_name() {
        let temp_dir = create_test_project("my_song.logicx");
        let project = LogicProject::detect(&temp_dir).unwrap();

        assert_eq!(project.name(), "my_song");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_name_with_spaces() {
        let temp_dir = create_test_project("My Amazing Song.logicx");
        let project = LogicProject::detect(&temp_dir).unwrap();

        assert_eq!(project.name(), "My Amazing Song");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_tracked_paths() {
        let temp_dir = create_test_project("track_test.logicx");
        let project = LogicProject::detect(&temp_dir).unwrap();

        let tracked = project.tracked_paths();
        assert_eq!(tracked.len(), 2);

        // Check that paths end with expected directories
        assert!(tracked.iter().any(|p| p.ends_with("Alternatives")));
        assert!(tracked.iter().any(|p| p.ends_with("Resources")));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_ignored_patterns() {
        let patterns = LogicProject::ignored_patterns();

        // Essential patterns that must be present
        assert!(patterns.contains(&"Bounces/"));
        assert!(patterns.contains(&"Freeze Files/"));
        assert!(patterns.contains(&".DS_Store"));
        assert!(patterns.contains(&"*.nosync"));
        assert!(patterns.contains(&"Autosave/"));
        assert!(patterns.contains(&"Media.localized/"));
        assert!(patterns.contains(&"*.smbdelete*"));

        // Should have at least 7 patterns
        assert!(patterns.len() >= 7);
    }

    #[test]
    fn test_ignored_patterns_all_types() {
        let patterns = LogicProject::ignored_patterns();

        // Should contain directory patterns (ending with /)
        assert!(patterns.iter().any(|p| p.ends_with('/')));

        // Should contain wildcard patterns
        assert!(patterns.iter().any(|p| p.contains('*')));

        // Should contain exact filename patterns
        assert!(patterns.iter().any(|p| !p.contains('*') && !p.ends_with('/')));
    }

    #[test]
    fn test_project_path_is_canonical() {
        let temp_dir = create_test_project("canonical_test.logicx");
        let project = LogicProject::detect(&temp_dir).unwrap();

        // Path should be absolute (canonical)
        assert!(project.path.is_absolute());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_project_data_path_exists() {
        let temp_dir = create_test_project("exists_test.logicx");
        let project = LogicProject::detect(&temp_dir).unwrap();

        // ProjectData path should exist
        assert!(project.project_data_path.exists());
        assert!(project.project_data_path.is_file());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_detect_with_symlink() {
        // This test may not work on all systems, so we'll make it conditional
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let temp_dir = create_test_project("real_project.logicx");
            let link_path = std::env::temp_dir().join("link_project.logicx");

            if symlink(&temp_dir, &link_path).is_ok() {
                let result = LogicProject::detect(&link_path);
                assert!(result.is_ok());

                let _ = fs::remove_file(&link_path);
            }

            let _ = fs::remove_dir_all(&temp_dir);
        }
    }
}
