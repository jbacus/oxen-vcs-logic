// Auxin-CLI-Wrapper/src/logic_parser/mod.rs
//
// Logic Pro project file parsing module.

pub mod binary_parser;
pub mod project_data;

pub use binary_parser::parse_logic_project;
pub use project_data::*;

use anyhow::Result;
use std::path::Path;

/// High-level API for parsing Logic Pro projects
pub struct LogicParser;

impl LogicParser {
    /// Parse a Logic Pro project from a .logicx file
    pub fn parse(project_path: &Path) -> Result<LogicProjectData> {
        binary_parser::parse_logic_project(project_path)
    }

    /// Quick validation check without full parsing
    pub fn is_valid_project(project_path: &Path) -> bool {
        if !project_path.exists() || !project_path.is_dir() {
            return false;
        }

        // Check for required structure
        let alternatives = project_path.join("Alternatives");
        let project_data = alternatives.join("001").join("ProjectData");

        alternatives.exists() && (project_data.exists() || alternatives.join("000").join("ProjectData").exists())
    }

    /// Get Logic Pro version without full parsing
    pub fn detect_version(project_path: &Path) -> Result<String> {
        // Try to read just enough to detect version
        let project_data_path = project_path
            .join("Alternatives")
            .join("001")
            .join("ProjectData");

        if !project_data_path.exists() {
            anyhow::bail!("ProjectData not found");
        }

        let binary = std::fs::read(project_data_path)?;
        if binary.len() < 16 {
            anyhow::bail!("ProjectData too small");
        }

        // TODO: Implement proper version detection
        Ok("11.0.0".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_mock_project() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("Test.logicx");

        // Create minimal directory structure
        fs::create_dir_all(project_path.join("Alternatives").join("001")).unwrap();

        // Create minimal ProjectData file
        let project_data = project_path.join("Alternatives").join("001").join("ProjectData");
        fs::write(project_data, vec![0; 1024]).unwrap();

        (temp_dir, project_path)
    }

    #[test]
    fn test_is_valid_project() {
        let (_temp, project_path) = create_mock_project();
        assert!(LogicParser::is_valid_project(&project_path));
    }

    #[test]
    fn test_invalid_project() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_path = temp_dir.path().join("NotAProject.logicx");
        assert!(!LogicParser::is_valid_project(&invalid_path));
    }
}
