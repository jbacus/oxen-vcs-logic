/// Generates a complete `.oxenignore` file template for Logic Pro projects.
///
/// Creates a well-organized ignore file with comprehensive patterns for files
/// that should NOT be version controlled. The template includes four main sections:
/// volatile/generated files, system files, cache files, and a custom section for
/// user additions.
///
/// # Purpose
///
/// The `.oxenignore` file prevents version control bloat and conflicts by excluding:
/// - **Generated audio** (bounces, freeze files) - Large, regenerable files
/// - **Volatile data** (autosaves, temp files) - Creates noisy, conflicting commits
/// - **System metadata** (.DS_Store, etc.) - User/machine-specific, no value in VCS
///
/// # Template Structure
///
/// ```text
/// # Oxen VCS - Logic Pro Ignore Rules
///
/// # Volatile/Generated Files
/// Bounces/
/// Freeze Files/
/// Autosave/
/// *.nosync
///
/// # System Files
/// .DS_Store
/// *.smbdelete*
/// .Trashes
///
/// # Cache and Temporary Files
/// *.cache
/// *.tmp
/// *~
///
/// # Custom Ignore Patterns
/// (empty for user to fill)
/// ```
///
/// # Returns
///
/// Complete `.oxenignore` file content as a String, ready to write to disk.
///
/// # Pattern Sources
///
/// All patterns are consistent with `LogicProject::ignored_patterns()` and include:
/// - Directory patterns (trailing slash): `Bounces/`, `Freeze Files/`
/// - Wildcard patterns: `*.nosync`, `*.cache`, `*~`
/// - Exact filenames: `.DS_Store`, `.Trashes`
///
/// # Examples
///
/// ```no_run
/// use auxin_cli::generate_oxenignore;
/// use std::fs;
///
/// // Generate and write to disk
/// let content = generate_oxenignore();
/// fs::write("/path/to/project.logicx/.oxenignore", content).unwrap();
/// ```
///
/// ```
/// use auxin_cli::generate_oxenignore;
///
/// // Verify essential patterns are present
/// let content = generate_oxenignore();
/// assert!(content.contains("Bounces/"));
/// assert!(content.contains("Freeze Files/"));
/// assert!(content.contains(".DS_Store"));
/// ```
///
/// # Integration
///
/// This function is called automatically during repository initialization:
/// 1. User runs `auxin init /path/to/project.logicx`
/// 2. `.oxenignore` file is created in project root
/// 3. Oxen uses patterns to exclude files from tracking
///
/// Users can customize by editing the "Custom Ignore Patterns" section.
///
/// # Idempotence
///
/// This function is deterministic - multiple calls return identical output.
/// Safe to regenerate if `.oxenignore` is accidentally deleted.
///
/// # Design Rationale
///
/// **Why exclude Bounces/ and Freeze Files/?**
/// - Large files (100MB+ common) that bloat repository
/// - Easily regenerable from project state
/// - Frequent changes cause merge conflicts
/// - Users intentionally export when needed
///
/// **Why exclude Autosave/?**
/// - Creates commits every few minutes (too granular)
/// - Not intentional save points
/// - Can conflict with manual saves
/// - Users rarely want to revert to autosaves
///
/// **Why exclude .DS_Store?**
/// - macOS Finder metadata (no value in Logic Pro context)
/// - Differs per machine/user
/// - Changes on every directory access
/// - Creates noisy, meaningless commits
///
/// # See Also
///
/// - `LogicProject::ignored_patterns()` - Source of truth for patterns
/// - `.oxenignore` documentation: https://docs.oxen.ai/concepts/oxenignore
pub fn generate_oxenignore() -> String {
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

/// Generates a complete `.oxenignore` file template for SketchUp projects.
///
/// Creates a well-organized ignore file with comprehensive patterns for files
/// that should NOT be version controlled. The template includes four main sections:
/// backup/temp files, generated output, cache files, and a custom section for
/// user additions.
///
/// # Purpose
///
/// The `.oxenignore` file prevents version control bloat and conflicts by excluding:
/// - **Backup files** (*.skb, autosaves) - Automatically generated backups
/// - **Generated output** (renders, exports) - Large, regenerable files
/// - **Cache data** (thumbnails, temp) - Volatile, machine-specific data
/// - **System metadata** (.DS_Store, etc.) - User/machine-specific, no value in VCS
///
/// # Template Structure
///
/// ```text
/// # Oxen VCS - SketchUp Ignore Rules
///
/// # Backup and Temporary Files
/// *.skb
/// *~.skp
/// *.tmp
///
/// # Generated Output
/// exports/
/// renders/
/// output/
///
/// # Cache and Thumbnails
/// .thumbnails/
/// cache/
///
/// # System Files
/// .DS_Store
/// Thumbs.db
/// desktop.ini
///
/// # Custom Ignore Patterns
/// (empty for user to fill)
/// ```
///
/// # Returns
///
/// Complete `.oxenignore` file content as a String, ready to write to disk.
///
/// # Pattern Sources
///
/// All patterns are consistent with `SketchUpProject::ignored_patterns()` and include:
/// - Backup patterns: `*.skb`, `*~.skp`
/// - Directory patterns (trailing slash): `exports/`, `renders/`
/// - Wildcard patterns: `*.tmp`, `*.cache`
/// - Exact filenames: `.DS_Store`, `Thumbs.db`
///
/// # Examples
///
/// ```no_run
/// use auxin_cli::generate_sketchup_oxenignore;
/// use std::fs;
///
/// // Generate and write to disk
/// let content = generate_sketchup_oxenignore();
/// fs::write("/path/to/project/.oxenignore", content).unwrap();
/// ```
///
/// # Integration
///
/// This function is called automatically during repository initialization:
/// 1. User runs `auxin init /path/to/model.skp --type sketchup`
/// 2. `.oxenignore` file is created in project directory
/// 3. Oxen uses patterns to exclude files from tracking
///
/// Users can customize by editing the "Custom Ignore Patterns" section.
///
/// # Design Rationale
///
/// **Why exclude *.skb files?**
/// - SketchUp automatically creates backup files with .skb extension
/// - These are copies of previous saves, redundant with version control
/// - Can bloat repository size quickly
///
/// **Why exclude exports/ and renders/?**
/// - Large files (images, videos, 3D exports) that bloat repository
/// - Easily regenerable from the source .skp file
/// - Users intentionally export when needed
///
/// **Why exclude .thumbnails/?**
/// - Machine-generated preview images
/// - Differs per machine/view settings
/// - Creates noisy, meaningless commits
///
/// # See Also
///
/// - `SketchUpProject::ignored_patterns()` - Source of truth for patterns
/// - `.oxenignore` documentation: https://docs.oxen.ai/concepts/oxenignore
pub fn generate_sketchup_oxenignore() -> String {
    let mut content = String::new();
    content.push_str("# Oxen VCS - SketchUp Ignore Rules\n");
    content.push_str("# Auto-generated ignore file for SketchUp projects\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Backup and Temporary Files\n");
    content.push_str("# ===================================\n");
    content.push_str("# SketchUp automatically creates backup files that should not be versioned\n\n");
    content.push_str("*.skb\n");
    content.push_str("*~.skp\n");
    content.push_str("*.tmp\n");
    content.push_str("*.swp\n");
    content.push_str(".sketchup_session\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Generated Output\n");
    content.push_str("# ===================================\n");
    content.push_str("# Exported and rendered files that can be regenerated\n\n");
    content.push_str("exports/\n");
    content.push_str("renders/\n");
    content.push_str("output/\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Cache and Thumbnails\n");
    content.push_str("# ===================================\n");
    content.push_str("# Generated preview and cache files\n\n");
    content.push_str(".thumbnails/\n");
    content.push_str("cache/\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# System Files\n");
    content.push_str("# ===================================\n");
    content.push_str("# OS-specific metadata\n\n");
    content.push_str(".DS_Store\n");
    content.push_str("Thumbs.db\n");
    content.push_str("desktop.ini\n");
    content.push_str("*.smbdelete*\n\n");

    content.push_str("# ===================================\n");
    content.push_str("# Custom Ignore Patterns\n");
    content.push_str("# ===================================\n");
    content.push_str("# Add your custom patterns below\n\n");

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logic_project::LogicProject;

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
        assert!(content
            .lines()
            .any(|l| !l.starts_with('#') && l.contains('*')));
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

    // ==================== SketchUp Tests ====================

    #[test]
    fn test_generate_sketchup_oxenignore_contains_essential_patterns() {
        let content = generate_sketchup_oxenignore();

        // All essential SketchUp patterns
        assert!(content.contains("*.skb"));
        assert!(content.contains("exports/"));
        assert!(content.contains("renders/"));
        assert!(content.contains(".DS_Store"));
        assert!(content.contains("Thumbs.db"));
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_sections() {
        let content = generate_sketchup_oxenignore();

        assert!(content.contains("Backup and Temporary Files"));
        assert!(content.contains("Generated Output"));
        assert!(content.contains("System Files"));
        assert!(content.contains("Custom Ignore Patterns"));
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_header() {
        let content = generate_sketchup_oxenignore();

        assert!(content.contains("Oxen VCS - SketchUp Ignore Rules"));
        assert!(content.contains("Auto-generated"));
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_all_system_files() {
        let content = generate_sketchup_oxenignore();

        assert!(content.contains(".DS_Store"));
        assert!(content.contains("Thumbs.db"));
        assert!(content.contains("desktop.ini"));
        assert!(content.contains("*.smbdelete*"));
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_backup_patterns() {
        let content = generate_sketchup_oxenignore();

        assert!(content.contains("*.skb"));
        assert!(content.contains("*~.skp"));
        assert!(content.contains("*.tmp"));
    }

    #[test]
    fn test_generate_sketchup_oxenignore_is_not_empty() {
        let content = generate_sketchup_oxenignore();
        assert!(!content.is_empty());
        assert!(content.len() > 100); // Should be substantial
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_comments() {
        let content = generate_sketchup_oxenignore();

        // Should have explanatory comments
        let comment_count = content.matches('#').count();
        assert!(comment_count > 10, "Should have multiple comment lines");
    }

    #[test]
    fn test_generate_sketchup_oxenignore_has_directory_patterns() {
        let content = generate_sketchup_oxenignore();

        // Should have patterns ending with /
        let lines: Vec<&str> = content.lines().collect();
        let dir_patterns: Vec<&str> = lines
            .iter()
            .filter(|l| !l.starts_with('#') && l.ends_with('/'))
            .copied()
            .collect();

        assert!(!dir_patterns.is_empty(), "Should have directory patterns");
        assert!(dir_patterns.len() >= 3); // At least exports/, renders/, cache/
    }

    #[test]
    fn test_generate_sketchup_oxenignore_idempotent() {
        // Calling multiple times should produce identical results
        let first = generate_sketchup_oxenignore();
        let second = generate_sketchup_oxenignore();

        assert_eq!(first, second, "generate_sketchup_oxenignore should be deterministic");
    }

    #[test]
    fn test_generate_sketchup_oxenignore_no_duplicate_patterns() {
        let content = generate_sketchup_oxenignore();

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
