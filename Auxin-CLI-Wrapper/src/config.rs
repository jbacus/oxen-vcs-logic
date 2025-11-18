//! Configuration file support for Auxin CLI
//!
//! Supports configuration from multiple sources with precedence:
//! 1. Environment variables (highest priority)
//! 2. Project config (`.auxin/config.toml`)
//! 3. User config (`~/.auxin/config.toml`)
//! 4. Defaults (lowest priority)
//!
//! # Example Config File
//!
//! ```toml
//! # ~/.auxin/config.toml
//! [defaults]
//! verbose = false
//! color = "auto"  # auto, always, never
//!
//! [lock]
//! timeout_hours = 4
//! auto_renew = false
//!
//! [network]
//! max_retries = 5
//! initial_backoff_ms = 1000
//! max_backoff_ms = 15000
//! connectivity_check_interval_s = 30
//!
//! [queue]
//! auto_sync = true
//! queue_dir = "~/.auxin/queue"
//!
//! [ui]
//! progress = true
//! emoji = true
//!
//! [project]
//! project_type = "auto"  # auto, logicpro, sketchup, blender
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for Auxin CLI
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub defaults: DefaultsConfig,

    #[serde(default)]
    pub lock: LockConfig,

    #[serde(default)]
    pub network: NetworkConfig,

    #[serde(default)]
    pub queue: QueueConfig,

    #[serde(default)]
    pub ui: UiConfig,

    #[serde(default)]
    pub project: ProjectConfig,
}

/// Default settings for common options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    /// Enable verbose output by default
    pub verbose: bool,

    /// Color output mode: auto, always, never
    pub color: ColorMode,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            verbose: false,
            color: ColorMode::Auto,
        }
    }
}

/// Lock operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockConfig {
    /// Default lock timeout in hours
    pub timeout_hours: u32,

    /// Automatically renew locks before expiration
    pub auto_renew: bool,

    /// Minutes before expiration to auto-renew
    pub renew_before_minutes: u32,
}

impl Default for LockConfig {
    fn default() -> Self {
        Self {
            timeout_hours: 4,
            auto_renew: false,
            renew_before_minutes: 30,
        }
    }
}

/// Network operation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Maximum number of retries for network operations
    pub max_retries: u32,

    /// Initial backoff delay in milliseconds
    pub initial_backoff_ms: u64,

    /// Maximum backoff delay in milliseconds
    pub max_backoff_ms: u64,

    /// Connectivity check interval in seconds
    pub connectivity_check_interval_s: u64,

    /// Timeout for connectivity checks in seconds
    pub connectivity_check_timeout_s: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_backoff_ms: 1000,
            max_backoff_ms: 15000,
            connectivity_check_interval_s: 30,
            connectivity_check_timeout_s: 5,
        }
    }
}

/// Offline queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueConfig {
    /// Automatically sync queue when online
    pub auto_sync: bool,

    /// Directory to store queue files
    pub queue_dir: String,

    /// Maximum number of queue entries to keep
    pub max_entries: usize,

    /// Automatically clean up completed entries after N days
    pub cleanup_after_days: u32,
}

impl Default for QueueConfig {
    fn default() -> Self {
        Self {
            auto_sync: true,
            queue_dir: "~/.auxin/queue".to_string(),
            max_entries: 1000,
            cleanup_after_days: 7,
        }
    }
}

/// UI/UX configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show progress bars/spinners
    pub progress: bool,

    /// Use emoji in output
    pub emoji: bool,

    /// Terminal width (0 = auto-detect)
    pub terminal_width: usize,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            progress: true,
            emoji: true,
            terminal_width: 0,
        }
    }
}

/// Color output mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl Default for ColorMode {
    fn default() -> Self {
        ColorMode::Auto
    }
}

/// Project type configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project type (auto-detected if not specified)
    pub project_type: ProjectType,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            project_type: ProjectType::Auto,
        }
    }
}

/// Supported project types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectType {
    /// Auto-detect based on file extensions
    Auto,
    /// Logic Pro projects (.logicx)
    LogicPro,
    /// SketchUp projects (.skp)
    SketchUp,
    /// Blender projects (.blend)
    Blender,
}

impl Default for ProjectType {
    fn default() -> Self {
        ProjectType::Auto
    }
}

impl ProjectType {
    /// Convert string to ProjectType
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "auto" => Some(ProjectType::Auto),
            "logicpro" | "logic-pro" | "logic" => Some(ProjectType::LogicPro),
            "sketchup" | "sketch-up" | "skp" => Some(ProjectType::SketchUp),
            "blender" | "blend" => Some(ProjectType::Blender),
            _ => None,
        }
    }

    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            ProjectType::Auto => "Auto-detect",
            ProjectType::LogicPro => "Logic Pro",
            ProjectType::SketchUp => "SketchUp",
            ProjectType::Blender => "Blender",
        }
    }
}

impl Config {
    /// Load configuration from all sources with proper precedence
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables
    /// 2. Project config (.auxin/config.toml)
    /// 3. User config (~/.auxin/config.toml)
    /// 4. Defaults
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        // Load user config if it exists
        if let Some(user_config_path) = Self::user_config_path() {
            if user_config_path.exists() {
                let user_config = Self::load_from_file(&user_config_path)?;
                config = Self::merge(config, user_config);
            }
        }

        // Load project config if it exists
        if let Some(project_config_path) = Self::project_config_path() {
            if project_config_path.exists() {
                let project_config = Self::load_from_file(&project_config_path)?;
                config = Self::merge(config, project_config);
            }
        }

        // Apply environment variables (highest priority)
        config = Self::apply_env_vars(config);

        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let contents = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {}", path.display()))?;

        toml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {}", path.display()))
    }

    /// Save configuration to a file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let contents = toml::to_string_pretty(self)
            .context("Failed to serialize config")?;

        fs::write(path, contents)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }

    /// Get user config path (~/.auxin/config.toml)
    pub fn user_config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| home.join(".auxin").join("config.toml"))
    }

    /// Get project config path (.auxin/config.toml in current directory)
    pub fn project_config_path() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(".auxin").join("config.toml"))
    }

    /// Merge two configs (right overwrites left for non-default values)
    fn merge(mut base: Config, overlay: Config) -> Config {
        // For simplicity, we'll just use the overlay values
        // A more sophisticated merge would check for default values
        base.defaults = overlay.defaults;
        base.lock = overlay.lock;
        base.network = overlay.network;
        base.queue = overlay.queue;
        base.ui = overlay.ui;
        base.project = overlay.project;
        base
    }

    /// Apply environment variables to config
    fn apply_env_vars(mut config: Config) -> Config {
        // AUXIN_VERBOSE
        if let Ok(val) = std::env::var("AUXIN_VERBOSE") {
            if let Ok(verbose) = val.parse::<bool>() {
                config.defaults.verbose = verbose;
            }
        }

        // AUXIN_COLOR
        if let Ok(val) = std::env::var("AUXIN_COLOR") {
            config.defaults.color = match val.to_lowercase().as_str() {
                "always" => ColorMode::Always,
                "never" => ColorMode::Never,
                _ => ColorMode::Auto,
            };
        }

        // AUXIN_LOCK_TIMEOUT
        if let Ok(val) = std::env::var("AUXIN_LOCK_TIMEOUT") {
            if let Ok(timeout) = val.parse::<u32>() {
                config.lock.timeout_hours = timeout;
            }
        }

        // AUXIN_MAX_RETRIES
        if let Ok(val) = std::env::var("AUXIN_MAX_RETRIES") {
            if let Ok(retries) = val.parse::<u32>() {
                config.network.max_retries = retries;
            }
        }

        // AUXIN_QUEUE_DIR
        if let Ok(val) = std::env::var("AUXIN_QUEUE_DIR") {
            config.queue.queue_dir = val;
        }

        // AUXIN_PROJECT_TYPE
        if let Ok(val) = std::env::var("AUXIN_PROJECT_TYPE") {
            if let Some(project_type) = ProjectType::from_str(&val) {
                config.project.project_type = project_type;
            }
        }

        config
    }

    /// Create a default config file with comments
    pub fn create_default_file(path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
        }

        let default_config = include_str!("../config.toml.example");
        fs::write(path, default_config)
            .with_context(|| format!("Failed to write config file: {}", path.display()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.defaults.verbose, false);
        assert_eq!(config.lock.timeout_hours, 4);
        assert_eq!(config.network.max_retries, 5);
        assert_eq!(config.queue.auto_sync, true);
        assert_eq!(config.ui.progress, true);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("[defaults]"));
        assert!(toml_str.contains("[lock]"));
        assert!(toml_str.contains("[network]"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
            [defaults]
            verbose = true
            color = "always"

            [lock]
            timeout_hours = 8
            auto_renew = true
            renew_before_minutes = 15

            [network]
            max_retries = 10
            initial_backoff_ms = 500
            max_backoff_ms = 10000
            connectivity_check_interval_s = 60
            connectivity_check_timeout_s = 10

            [queue]
            auto_sync = false
            queue_dir = "/tmp/queue"
            max_entries = 500
            cleanup_after_days = 14

            [ui]
            progress = false
            emoji = false
            terminal_width = 120
        "#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.defaults.verbose, true);
        assert_eq!(config.defaults.color, ColorMode::Always);
        assert_eq!(config.lock.timeout_hours, 8);
        assert_eq!(config.lock.auto_renew, true);
        assert_eq!(config.network.max_retries, 10);
        assert_eq!(config.queue.auto_sync, false);
        assert_eq!(config.ui.progress, false);
    }

    #[test]
    fn test_color_mode_serialization() {
        // Test serialization within a config struct
        let mut config = DefaultsConfig::default();

        config.color = ColorMode::Auto;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("color = \"auto\""));

        config.color = ColorMode::Always;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("color = \"always\""));

        config.color = ColorMode::Never;
        let toml_str = toml::to_string(&config).unwrap();
        assert!(toml_str.contains("color = \"never\""));
    }
}
