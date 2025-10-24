use crate::logic_project::LogicProject;

/// Generates a .oxenignore file content with asset classification rules
///
/// The template organizes ignored files into categories:
/// - Volatile/Generated: Temporary files created during project work
/// - System: OS-specific metadata files
pub fn generate_oxenignore() -> String {
    let patterns = LogicProject::ignored_patterns();

    let mut content = String::new();
    content.push_str("# Oxen VCS - Logic Pro Ignore Rules\n");
    content.push_str("# Auto-generated ignore file for Logic Pro folder projects\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Volatile/Generated Files\n");
    content.push_str("# ===================================\n");
    content.push_str("# These files are regenerated and should not be versioned\n\n");
    content.push_str("Bounces/\n");
    content.push_str("Freeze Files/\n");
    content.push_str("*.nosync\n");
    content.push_str("Autosave/\n");
    content.push_str("Media.localized/\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# System Files\n");
    content.push_str("# ===================================\n");
    content.push_str("# OS-specific metadata\n\n");
    content.push_str(".DS_Store\n");
    content.push_str("*.smbdelete*\n");
    content.push_str(".TemporaryItems\n");
    content.push_str(".Trashes\n");
    content.push_str(".fseventsd\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Cache and Temporary Files\n");
    content.push_str("# ===================================\n\n");
    content.push_str("*.cache\n");
    content.push_str("*.tmp\n");
    content.push_str("*~\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Custom Ignore Patterns\n");
    content.push_str("# ===================================\n");
    content.push_str("# Add your custom patterns below\n\n");

    content
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_oxenignore_contains_essential_patterns() {
        let content = generate_oxenignore();

        assert!(content.contains("Bounces/"));
        assert!(content.contains("Freeze Files/"));
        assert!(content.contains(".DS_Store"));
        assert!(content.contains("*.nosync"));
    }

    #[test]
    fn test_generate_oxenignore_has_sections() {
        let content = generate_oxenignore();

        assert!(content.contains("Volatile/Generated"));
        assert!(content.contains("System Files"));
        assert!(content.contains("Custom Ignore Patterns"));
    }
}
