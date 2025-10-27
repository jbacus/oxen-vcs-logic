use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};

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

        // Check if path exists
        if !path.exists() {
            return Err(anyhow!("Path does not exist: {}", path.display()));
        }

        // Check if it's a directory
        if !path.is_dir() {
            return Err(anyhow!("Path is not a directory: {}", path.display()));
        }

        // Canonicalize the path to resolve relative paths like "." to absolute paths
        // This ensures we can properly check the extension even when running from inside the .logicx directory
        let canonical_path = std::fs::canonicalize(path)
            .context("Failed to canonicalize path")?;

        // Check if it has .logicx extension
        let extension = canonical_path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if extension != "logicx" {
            return Err(anyhow!(
                "Path is not a Logic Pro folder project (.logicx): {}",
                canonical_path.display()
            ));
        }

        // Check for ProjectData file in Logic Pro's actual structure
        // Logic Pro stores project data in Alternatives/###/ProjectData
        let project_data_path = Self::find_project_data(&canonical_path)?;

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
        // First, check for Alternatives directory (standard Logic Pro structure)
        let alternatives_path = project_path.join("Alternatives");

        if alternatives_path.exists() && alternatives_path.is_dir() {
            // Look for numbered subdirectories in Alternatives/
            if let Ok(entries) = std::fs::read_dir(&alternatives_path) {
                for entry in entries.flatten() {
                    if entry.path().is_dir() {
                        // Check for ProjectData in this alternative
                        let project_data = entry.path().join("ProjectData");
                        if project_data.exists() {
                            return Ok(project_data);
                        }
                    }
                }
            }
        }

        // Fallback: Check for ProjectData at root level (various case variations)
        let possible_names = vec!["ProjectData", "projectData", "Project Data"];

        for name in possible_names {
            let path = project_path.join(name);
            if path.exists() {
                return Ok(path);
            }
        }

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
