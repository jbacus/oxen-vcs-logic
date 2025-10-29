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

        // All essential Logic Pro patterns
        assert!(content.contains("Bounces/"));
        assert!(content.contains("Freeze Files/"));
        assert!(content.contains(".DS_Store"));
        assert!(content.contains("*.nosync"));
        assert!(content.contains("Autosave/"));
        assert!(content.contains("Media.localized/"));
    }

    #[test]
    fn test_generate_oxenignore_has_sections() {
        let content = generate_oxenignore();

        assert!(content.contains("Volatile/Generated"));
        assert!(content.contains("System Files"));
        assert!(content.contains("Custom Ignore Patterns"));
        assert!(content.contains("Cache and Temporary Files"));
    }

    #[test]
    fn test_generate_oxenignore_has_header() {
        let content = generate_oxenignore();

        assert!(content.contains("Oxen VCS - Logic Pro Ignore Rules"));
        assert!(content.contains("Auto-generated"));
    }

    #[test]
    fn test_generate_oxenignore_has_all_system_files() {
        let content = generate_oxenignore();

        assert!(content.contains(".DS_Store"));
        assert!(content.contains("*.smbdelete*"));
        assert!(content.contains(".TemporaryItems"));
        assert!(content.contains(".Trashes"));
        assert!(content.contains(".fseventsd"));
    }

    #[test]
    fn test_generate_oxenignore_has_cache_patterns() {
        let content = generate_oxenignore();

        assert!(content.contains("*.cache"));
        assert!(content.contains("*.tmp"));
        assert!(content.contains("*~"));
    }

    #[test]
    fn test_generate_oxenignore_consistency_with_logic_project() {
        let content = generate_oxenignore();
        let project_patterns = LogicProject::ignored_patterns();

        // All patterns from LogicProject should appear in the template
        for pattern in project_patterns {
            assert!(
                content.contains(pattern),
                "Pattern '{}' from LogicProject::ignored_patterns() not found in template",
                pattern
            );
        }
    }

    #[test]
    fn test_generate_oxenignore_is_not_empty() {
        let content = generate_oxenignore();
        assert!(!content.is_empty());
        assert!(content.len() > 100); // Should be substantial
    }

    #[test]
    fn test_generate_oxenignore_has_comments() {
        let content = generate_oxenignore();

        // Should have explanatory comments
        let comment_count = content.matches('#').count();
        assert!(comment_count > 10, "Should have multiple comment lines");
    }

    #[test]
    fn test_generate_oxenignore_sections_are_separated() {
        let content = generate_oxenignore();

        // Sections should be separated by blank lines and dividers
        assert!(content.contains("==="));
        assert!(content.contains("\n\n"));
    }

    #[test]
    fn test_generate_oxenignore_has_directory_patterns() {
        let content = generate_oxenignore();

        // Should have patterns ending with /
        let lines: Vec<&str> = content.lines().collect();
        let dir_patterns: Vec<&str> = lines
            .iter()
            .filter(|l| !l.starts_with('#') && l.ends_with('/'))
            .copied()
            .collect();

        assert!(!dir_patterns.is_empty(), "Should have directory patterns");
        assert!(dir_patterns.len() >= 3); // At least Bounces/, Freeze Files/, Autosave/
    }

    #[test]
    fn test_generate_oxenignore_has_wildcard_patterns() {
        let content = generate_oxenignore();

        // Should have patterns with wildcards
        assert!(content.lines().any(|l| !l.starts_with('#') && l.contains('*')));
    }

    #[test]
    fn test_generate_oxenignore_format_valid() {
        let content = generate_oxenignore();

        // Each non-comment, non-empty line should be a valid pattern
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') && !trimmed.contains("===") {
                // Should not have leading/trailing whitespace
                assert_eq!(trimmed, line);
            }
        }
    }

    #[test]
    fn test_generate_oxenignore_custom_section_is_empty() {
        let content = generate_oxenignore();

        // Find the custom section
        let custom_section_start = content.find("Custom Ignore Patterns").unwrap();
        let after_custom = &content[custom_section_start..];

        // After the custom section header, there should only be comments and whitespace
        let lines_after_custom: Vec<&str> = after_custom
            .lines()
            .skip(3) // Skip the section header lines
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .collect();

        assert!(
            lines_after_custom.is_empty(),
            "Custom section should be empty for users to fill"
        );
    }

    #[test]
    fn test_generate_oxenignore_all_sections_present() {
        let content = generate_oxenignore();

        let required_sections = vec![
            "Volatile/Generated",
            "System Files",
            "Cache and Temporary Files",
            "Custom Ignore Patterns",
        ];

        for section in required_sections {
            assert!(
                content.contains(section),
                "Missing required section: {}",
                section
            );
        }
    }

    #[test]
    fn test_generate_oxenignore_idempotent() {
        // Calling multiple times should produce identical results
        let first = generate_oxenignore();
        let second = generate_oxenignore();

        assert_eq!(first, second, "generate_oxenignore should be deterministic");
    }

    #[test]
    fn test_generate_oxenignore_newline_terminated() {
        let content = generate_oxenignore();

        // Should end with newline(s)
        assert!(content.ends_with('\n'));
    }

    #[test]
    fn test_generate_oxenignore_no_duplicate_patterns() {
        let content = generate_oxenignore();

        // Extract all pattern lines (non-comment, non-empty)
        let patterns: Vec<&str> = content
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.contains("==="))
            .collect();

        // Check for duplicates
        let unique_patterns: std::collections::HashSet<_> = patterns.iter().collect();

        assert_eq!(
            patterns.len(),
            unique_patterns.len(),
            "Should not have duplicate patterns"
        );
    }
}
