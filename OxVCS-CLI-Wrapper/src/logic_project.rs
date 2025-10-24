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

        // Check if it has .logicx extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        if extension != "logicx" {
            return Err(anyhow!(
                "Path is not a Logic Pro folder project (.logicx): {}",
                path.display()
            ));
        }

        // Check for projectData file
        let project_data_path = path.join("projectData");
        if !project_data_path.exists() {
            return Err(anyhow!(
                "No projectData file found in {}",
                path.display()
            ));
        }

        Ok(LogicProject {
            path: path.to_path_buf(),
            project_data_path,
        })
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
            self.path.join("projectData"),
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

    #[test]
    fn test_detect_invalid_extension() {
        let temp_dir = std::env::temp_dir().join("test_project");
        let _ = fs::create_dir_all(&temp_dir);

        let result = LogicProject::detect(&temp_dir);
        assert!(result.is_err());

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
