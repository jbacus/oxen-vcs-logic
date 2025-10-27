use anyhow::{anyhow, Context, Result};
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

    #[test]
    fn test_detect_invalid_extension() {
        let temp_dir = std::env::temp_dir().join("test_project");
        let _ = fs::create_dir_all(&temp_dir);

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_err());

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
    fn test_ignored_patterns() {
        let patterns = LogicProject::ignored_patterns();
        assert!(patterns.contains(&"Bounces/"));
        assert!(patterns.contains(&"Freeze Files/"));
        assert!(patterns.contains(&".DS_Store"));
    }
}
