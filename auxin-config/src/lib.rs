use figment::{Figment, providers::{Format, Toml, Env}};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    #[serde(default)]
    pub defaults: Defaults,
    #[serde(default)]
    pub lock: Lock,
    #[serde(default)]
    pub network: Network,
    #[serde(default)]
    pub queue: Queue,
    #[serde(default)]
    pub ui: Ui,
    #[serde(default)]
    pub project: Project,
    #[serde(default)]
    pub cli: Cli,
    #[serde(default)]
    pub server: Server,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Defaults {
    #[serde(default = "default_false")]
    pub verbose: bool,
    #[serde(default = "default_color")]
    pub color: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Lock {
    #[serde(default = "default_lock_timeout")]
    pub timeout_hours: i64,
    #[serde(default = "default_false")]
    pub auto_renew: bool,
    #[serde(default = "default_renew_before")]
    pub renew_before_minutes: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Network {
    #[serde(default = "default_max_retries")]
    pub max_retries: i64,
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff_ms: i64,
    #[serde(default = "default_max_backoff")]
    pub max_backoff_ms: i64,
    #[serde(default = "default_connectivity_interval")]
    pub connectivity_check_interval_s: i64,
    #[serde(default = "default_connectivity_timeout")]
    pub connectivity_check_timeout_s: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Queue {
    #[serde(default = "default_true")]
    pub auto_sync: bool,
    #[serde(default = "default_queue_dir")]
    pub queue_dir: String,
    #[serde(default = "default_max_entries")]
    pub max_entries: i64,
    #[serde(default = "default_cleanup_days")]
    pub cleanup_after_days: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Ui {
    #[serde(default = "default_true")]
    pub progress: bool,
    #[serde(default = "default_true")]
    pub emoji: bool,
    #[serde(default = "default_terminal_width")]
    pub terminal_width: i64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Project {
    #[serde(default = "default_project_type")]
    pub project_type: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Cli {
    #[serde(default = "default_server_url")]
    pub url: String,
    #[serde(default)]
    pub token: String,
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: i64,
    #[serde(default = "default_true")]
    pub use_server_locks: bool,
    #[serde(default = "default_true")]
    pub use_server_metadata: bool,
    #[serde(default = "default_namespace")]
    pub default_namespace: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Server {
    #[serde(default = "default_sync_dir")]
    pub sync_dir: String,
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: i64,
    #[serde(default = "default_auth_secret")]
    pub auth_token_secret: String,
    #[serde(default = "default_token_expiry")]
    pub auth_token_expiry_hours: i64,
    #[serde(default = "default_false")]
    pub enable_redis_locks: bool,
    #[serde(default = "default_false")]
    pub enable_web_ui: bool,
    #[serde(default)]
    pub redis_url: String,
    #[serde(default)]
    pub database_url: String,
}

// Default value functions for serde
fn default_false() -> bool { false }
fn default_true() -> bool { true }
fn default_color() -> String { "auto".to_string() }
fn default_lock_timeout() -> i64 { 4 }
fn default_renew_before() -> i64 { 30 }
fn default_max_retries() -> i64 { 5 }
fn default_initial_backoff() -> i64 { 1000 }
fn default_max_backoff() -> i64 { 15000 }
fn default_connectivity_interval() -> i64 { 30 }
fn default_connectivity_timeout() -> i64 { 5 }
fn default_queue_dir() -> String { "~/.auxin/queue".to_string() }
fn default_max_entries() -> i64 { 1000 }
fn default_cleanup_days() -> i64 { 7 }
fn default_terminal_width() -> i64 { 0 }
fn default_project_type() -> String { "auto".to_string() }
fn default_server_url() -> String { "http://localhost:3000".to_string() }
fn default_timeout_secs() -> i64 { 30 }
fn default_namespace() -> String { "community".to_string() }
fn default_sync_dir() -> String { "/var/oxen/data".to_string() }
fn default_host() -> String { "0.0.0.0".to_string() }
fn default_port() -> i64 { 3000 }
fn default_auth_secret() -> String { "dev_secret_change_in_production".to_string() }
fn default_token_expiry() -> i64 { 24 }

// Default trait implementations
impl Default for Defaults {
    fn default() -> Self {
        Self {
            verbose: default_false(),
            color: default_color(),
        }
    }
}

impl Default for Lock {
    fn default() -> Self {
        Self {
            timeout_hours: default_lock_timeout(),
            auto_renew: default_false(),
            renew_before_minutes: default_renew_before(),
        }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_backoff_ms: default_initial_backoff(),
            max_backoff_ms: default_max_backoff(),
            connectivity_check_interval_s: default_connectivity_interval(),
            connectivity_check_timeout_s: default_connectivity_timeout(),
        }
    }
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            auto_sync: default_true(),
            queue_dir: default_queue_dir(),
            max_entries: default_max_entries(),
            cleanup_after_days: default_cleanup_days(),
        }
    }
}

impl Default for Ui {
    fn default() -> Self {
        Self {
            progress: default_true(),
            emoji: default_true(),
            terminal_width: default_terminal_width(),
        }
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            project_type: default_project_type(),
        }
    }
}

impl Default for Cli {
    fn default() -> Self {
        Self {
            url: default_server_url(),
            token: String::new(),
            timeout_secs: default_timeout_secs(),
            use_server_locks: default_true(),
            use_server_metadata: default_true(),
            default_namespace: default_namespace(),
        }
    }
}

impl Default for Server {
    fn default() -> Self {
        Self {
            sync_dir: default_sync_dir(),
            host: default_host(),
            port: default_port(),
            auth_token_secret: default_auth_secret(),
            auth_token_expiry_hours: default_token_expiry(),
            enable_redis_locks: default_false(),
            enable_web_ui: default_false(),
            redis_url: String::new(),
            database_url: String::new(),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            defaults: Defaults::default(),
            lock: Lock::default(),
            network: Network::default(),
            queue: Queue::default(),
            ui: Ui::default(),
            project: Project::default(),
            cli: Cli::default(),
            server: Server::default(),
        }
    }
}

// Main configuration loading
impl Config {
    pub fn load() -> anyhow::Result<Config> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        let user_config_path = home_dir.join(".auxin/config.toml");
        let project_config_path = PathBuf::from(".auxin/config.toml");

        let config: Config = Figment::new()
            .merge(Toml::file(user_config_path))
            .merge(Toml::file(project_config_path))
            .merge(Env::prefixed("AUXIN_"))
            .extract()?;

        Ok(config)
    }

    pub fn project_config_path() -> Option<PathBuf> {
        let path = PathBuf::from(".auxin/config.toml");
        if path.exists() || path.parent().map(|p| p.exists()).unwrap_or(false) {
            Some(path)
        } else {
            None
        }
    }

    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        use std::io::Write;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Serialize to TOML
        let toml_str = toml::to_string_pretty(self)?;

        // Write to file
        let mut file = std::fs::File::create(path)?;
        file.write_all(toml_str.as_bytes())?;

        Ok(())
    }
}

pub fn load_config() -> anyhow::Result<Config> {
    Config::load()
}