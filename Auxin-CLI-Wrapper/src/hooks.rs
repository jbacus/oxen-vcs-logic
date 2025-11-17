/// Workflow automation hooks for Auxin
///
/// Provides pre-commit and post-commit hooks for automated workflows:
/// - Pre-commit: Validation, metadata extraction, auto-formatting
/// - Post-commit: Notifications, backups, triggers
///
/// # Hook Types
///
/// **Pre-commit hooks** run before creating a commit. They can:
/// - Validate metadata completeness
/// - Extract BPM/key from Logic Pro project files
/// - Check file sizes and warn about large files
/// - Run custom validation scripts
/// - Abort commit if validation fails
///
/// **Post-commit hooks** run after successful commit. They can:
/// - Send notifications (email, Slack, Discord)
/// - Trigger CI/CD pipelines
/// - Create automatic backups
/// - Update external tracking systems
/// - Run custom scripts
///
/// # Usage
///
/// ```no_run
/// use auxin_cli::hooks::{HookManager, HookType};
/// use auxin_cli::CommitMetadata;
/// # use anyhow::Result;
///
/// # fn main() -> Result<()> {
/// let manager = HookManager::new("/path/to/repo");
/// let metadata = CommitMetadata::new("Test commit");
///
/// // Run pre-commit hooks
/// manager.run_hooks(HookType::PreCommit, &metadata)?;
///
/// // Run post-commit hooks
/// manager.run_hooks(HookType::PostCommit, &metadata)?;
/// # Ok(())
/// # }
/// ```

use crate::CommitMetadata;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Type of hook
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HookType {
    PreCommit,
    PostCommit,
}

impl HookType {
    /// Get the directory name for this hook type
    pub fn dir_name(&self) -> &'static str {
        match self {
            HookType::PreCommit => "pre-commit",
            HookType::PostCommit => "post-commit",
        }
    }
}

/// Built-in hook that can be enabled/disabled
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuiltInHook {
    /// Hook name
    pub name: String,
    /// Hook description
    pub description: String,
    /// Whether the hook is enabled
    pub enabled: bool,
    /// Hook type (pre or post commit)
    pub hook_type: HookType,
}

/// Hook manager for executing workflow automation
pub struct HookManager {
    /// Path to the repository
    repo_path: PathBuf,
}

impl HookManager {
    /// Create a new hook manager
    pub fn new(repo_path: impl AsRef<Path>) -> Self {
        Self {
            repo_path: repo_path.as_ref().to_path_buf(),
        }
    }

    /// Get the hooks directory path
    fn hooks_dir(&self) -> PathBuf {
        self.repo_path.join(".oxen").join("hooks")
    }

    /// Get the directory for a specific hook type
    fn hook_type_dir(&self, hook_type: HookType) -> PathBuf {
        self.hooks_dir().join(hook_type.dir_name())
    }

    /// Initialize the hooks directory structure
    pub fn init(&self) -> Result<()> {
        let hooks_dir = self.hooks_dir();

        // Create hooks directory
        fs::create_dir_all(&hooks_dir)
            .context("Failed to create hooks directory")?;

        // Create subdirectories for each hook type
        fs::create_dir_all(self.hook_type_dir(HookType::PreCommit))?;
        fs::create_dir_all(self.hook_type_dir(HookType::PostCommit))?;

        // Create README
        let readme_path = hooks_dir.join("README.md");
        if !readme_path.exists() {
            let readme_content = HOOKS_README;
            fs::write(&readme_path, readme_content)?;
        }

        Ok(())
    }

    /// Run all enabled hooks of a specific type
    pub fn run_hooks(&self, hook_type: HookType, metadata: &CommitMetadata) -> Result<bool> {
        let hooks_dir = self.hook_type_dir(hook_type);

        if !hooks_dir.exists() {
            // No hooks directory, that's fine
            return Ok(true);
        }

        // Get all hook scripts in the directory
        let mut entries: Vec<_> = fs::read_dir(&hooks_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_type().map(|ft| ft.is_file()).unwrap_or(false)
            })
            .collect();

        // Sort by name for predictable execution order
        entries.sort_by_key(|e| e.file_name());

        // Run each hook
        for entry in entries {
            let hook_path = entry.path();
            let hook_name = hook_path.file_name().unwrap().to_string_lossy();

            // Skip non-executable files and README
            if hook_name.starts_with('.') || hook_name == "README.md" {
                continue;
            }

            println!("Running {} hook: {}", hook_type.dir_name(), hook_name);

            let success = self.run_hook(&hook_path, metadata)?;

            if !success {
                eprintln!("Hook failed: {}", hook_name);
                if matches!(hook_type, HookType::PreCommit) {
                    // Pre-commit hooks can abort the commit
                    return Ok(false);
                }
                // Post-commit hooks don't abort, just warn
            }
        }

        Ok(true)
    }

    /// Run a single hook script
    fn run_hook(&self, hook_path: &Path, metadata: &CommitMetadata) -> Result<bool> {
        // Prepare environment variables for the hook
        let output = Command::new(hook_path)
            .env("AUXIN_MESSAGE", &metadata.message)
            .env("AUXIN_BPM", metadata.bpm.map(|b| b.to_string()).unwrap_or_default())
            .env("AUXIN_SAMPLE_RATE", metadata.sample_rate.map(|s| s.to_string()).unwrap_or_default())
            .env("AUXIN_KEY", metadata.key_signature.as_deref().unwrap_or(""))
            .env("AUXIN_TAGS", metadata.tags.join(","))
            .env("AUXIN_REPO_PATH", &self.repo_path)
            .output()
            .with_context(|| format!("Failed to execute hook: {:?}", hook_path))?;

        // Print hook output
        if !output.stdout.is_empty() {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        if !output.stderr.is_empty() {
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }

        Ok(output.status.success())
    }

    /// List all hooks
    pub fn list_hooks(&self) -> Result<Vec<(HookType, String)>> {
        let mut hooks = Vec::new();

        for hook_type in [HookType::PreCommit, HookType::PostCommit] {
            let hooks_dir = self.hook_type_dir(hook_type);

            if !hooks_dir.exists() {
                continue;
            }

            for entry in fs::read_dir(&hooks_dir)? {
                let entry = entry?;
                let name = entry.file_name().to_string_lossy().to_string();

                if !name.starts_with('.') && name != "README.md" {
                    hooks.push((hook_type, name));
                }
            }
        }

        Ok(hooks)
    }

    /// Install a built-in hook from a template
    pub fn install_builtin(&self, name: &str, hook_type: HookType) -> Result<()> {
        let hook_content = match (hook_type, name) {
            (HookType::PreCommit, "validate-metadata") => HOOK_VALIDATE_METADATA,
            (HookType::PreCommit, "check-file-sizes") => HOOK_CHECK_FILE_SIZES,
            (HookType::PostCommit, "notify") => HOOK_NOTIFY,
            (HookType::PostCommit, "backup") => HOOK_BACKUP,
            _ => return Err(anyhow::anyhow!("Unknown built-in hook: {}", name)),
        };

        let hook_path = self.hook_type_dir(hook_type).join(name);

        // Write the hook script
        fs::write(&hook_path, hook_content)?;

        // Make it executable (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&hook_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&hook_path, perms)?;
        }

        println!("Installed {} hook: {}", hook_type.dir_name(), name);

        Ok(())
    }

    /// Remove a hook
    pub fn remove_hook(&self, name: &str, hook_type: HookType) -> Result<()> {
        let hook_path = self.hook_type_dir(hook_type).join(name);

        if hook_path.exists() {
            fs::remove_file(&hook_path)?;
            println!("Removed {} hook: {}", hook_type.dir_name(), name);
        } else {
            return Err(anyhow::anyhow!("Hook not found: {}", name));
        }

        Ok(())
    }

    /// Get available built-in hooks
    pub fn list_builtins() -> Vec<BuiltInHook> {
        vec![
            BuiltInHook {
                name: "validate-metadata".to_string(),
                description: "Ensure BPM and sample rate are set before commit".to_string(),
                enabled: false,
                hook_type: HookType::PreCommit,
            },
            BuiltInHook {
                name: "check-file-sizes".to_string(),
                description: "Warn about files larger than 100MB".to_string(),
                enabled: false,
                hook_type: HookType::PreCommit,
            },
            BuiltInHook {
                name: "notify".to_string(),
                description: "Send notification after commit (customize script)".to_string(),
                enabled: false,
                hook_type: HookType::PostCommit,
            },
            BuiltInHook {
                name: "backup".to_string(),
                description: "Create timestamped backup after commit".to_string(),
                enabled: false,
                hook_type: HookType::PostCommit,
            },
        ]
    }
}

// Built-in hook templates

const HOOKS_README: &str = r#"# Auxin Hooks

This directory contains workflow automation hooks for your Auxin repository.

## Hook Types

### Pre-Commit Hooks (`pre-commit/`)
Run before creating a commit. Can abort the commit if validation fails.

### Post-Commit Hooks (`post-commit/`)
Run after a successful commit. Cannot abort the commit.

## Creating Custom Hooks

1. Create a script in the appropriate directory
2. Make it executable (`chmod +x hook-name`)
3. Use any scripting language (bash, python, ruby, etc.)

### Available Environment Variables

- `AUXIN_MESSAGE` - Commit message
- `AUXIN_BPM` - BPM value (if set)
- `AUXIN_SAMPLE_RATE` - Sample rate (if set)
- `AUXIN_KEY` - Key signature (if set)
- `AUXIN_TAGS` - Comma-separated tags
- `AUXIN_REPO_PATH` - Path to the repository

See the documentation for examples and more information.
"#;

const HOOK_VALIDATE_METADATA: &str = r#"#!/bin/bash
# Pre-commit hook: Validate metadata

if [ -z "$AUXIN_BPM" ]; then
    echo "ERROR: BPM not set. Please provide BPM metadata."
    exit 1
fi

if [ -z "$AUXIN_SAMPLE_RATE" ]; then
    echo "ERROR: Sample rate not set. Please provide sample rate metadata."
    exit 1
fi

echo "✓ Metadata validation passed"
exit 0
"#;

const HOOK_CHECK_FILE_SIZES: &str = r#"#!/bin/bash
# Pre-commit hook: Check file sizes

MAX_SIZE=$((100 * 1024 * 1024))  # 100MB in bytes

cd "$AUXIN_REPO_PATH" || exit 1

# Find large files
large_files=$(find . -type f -size +${MAX_SIZE}c -not -path "./.oxen/*" 2>/dev/null)

if [ -n "$large_files" ]; then
    echo "WARNING: Large files detected (>100MB):"
    echo "$large_files"
    echo ""
    echo "Consider using .oxenignore for generated/temporary files"
fi

exit 0  # Don't abort commit, just warn
"#;

const HOOK_NOTIFY: &str = r#"#!/bin/bash
# Post-commit hook: Send notification

# Customize this script to send notifications via your preferred method
# Examples:
# - Send email
# - Post to Slack
# - Send to Discord webhook
# - Update project management system

echo "New commit: $AUXIN_MESSAGE"
echo "BPM: $AUXIN_BPM, Sample Rate: $AUXIN_SAMPLE_RATE, Key: $AUXIN_KEY"

# Example: macOS notification
if command -v osascript &> /dev/null; then
    osascript -e "display notification \"$AUXIN_MESSAGE\" with title \"Auxin Commit\""
fi

# Example: Linux notification
if command -v notify-send &> /dev/null; then
    notify-send "Auxin Commit" "$AUXIN_MESSAGE"
fi

exit 0
"#;

const HOOK_BACKUP: &str = r#"#!/bin/bash
# Post-commit hook: Create backup

BACKUP_DIR="$HOME/.auxin-backups"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
PROJECT_NAME=$(basename "$AUXIN_REPO_PATH")

mkdir -p "$BACKUP_DIR"

# Create timestamped backup
BACKUP_PATH="$BACKUP_DIR/${PROJECT_NAME}_${TIMESTAMP}.tar.gz"

echo "Creating backup: $BACKUP_PATH"
tar -czf "$BACKUP_PATH" -C "$(dirname "$AUXIN_REPO_PATH")" "$(basename "$AUXIN_REPO_PATH")" 2>/dev/null

if [ $? -eq 0 ]; then
    echo "✓ Backup created successfully"
else
    echo "WARNING: Backup failed"
fi

# Keep only last 10 backups
cd "$BACKUP_DIR" || exit 0
ls -t ${PROJECT_NAME}_*.tar.gz | tail -n +11 | xargs -r rm

exit 0
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_hook_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = HookManager::new(dir.path());

        assert_eq!(manager.repo_path, dir.path());
    }

    #[test]
    fn test_init_creates_directories() {
        let dir = tempdir().unwrap();
        let manager = HookManager::new(dir.path());

        manager.init().unwrap();

        assert!(manager.hooks_dir().exists());
        assert!(manager.hook_type_dir(HookType::PreCommit).exists());
        assert!(manager.hook_type_dir(HookType::PostCommit).exists());
    }

    #[test]
    fn test_list_builtins() {
        let builtins = HookManager::list_builtins();

        assert!(!builtins.is_empty());
        assert!(builtins.iter().any(|h| h.name == "validate-metadata"));
        assert!(builtins.iter().any(|h| h.name == "check-file-sizes"));
    }

    #[test]
    fn test_hook_type_dir_name() {
        assert_eq!(HookType::PreCommit.dir_name(), "pre-commit");
        assert_eq!(HookType::PostCommit.dir_name(), "post-commit");
    }

    #[test]
    fn test_install_builtin() {
        let dir = tempdir().unwrap();
        let manager = HookManager::new(dir.path());

        manager.init().unwrap();
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();

        let hook_path = manager.hook_type_dir(HookType::PreCommit).join("validate-metadata");
        assert!(hook_path.exists());
    }

    #[test]
    fn test_list_hooks() {
        let dir = tempdir().unwrap();
        let manager = HookManager::new(dir.path());

        manager.init().unwrap();
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();

        let hooks = manager.list_hooks().unwrap();
        assert!(!hooks.is_empty());
        assert!(hooks.iter().any(|(_, name)| name == "validate-metadata"));
    }

    #[test]
    fn test_remove_hook() {
        let dir = tempdir().unwrap();
        let manager = HookManager::new(dir.path());

        manager.init().unwrap();
        manager.install_builtin("validate-metadata", HookType::PreCommit).unwrap();
        manager.remove_hook("validate-metadata", HookType::PreCommit).unwrap();

        let hook_path = manager.hook_type_dir(HookType::PreCommit).join("validate-metadata");
        assert!(!hook_path.exists());
    }
}
