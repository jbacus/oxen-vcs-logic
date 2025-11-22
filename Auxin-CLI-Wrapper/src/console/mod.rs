/// Interactive TUI console for real-time monitoring and control
///
/// Provides a full-screen terminal interface with:
/// - Live daemon status
/// - Activity log with real-time updates
/// - Repository status display
/// - Keyboard shortcuts for common operations
use anyhow::{Context, Result};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use crate::{CommitMetadata, OxenRepository};

/// Maximum number of activity log entries to retain
const MAX_LOG_ENTRIES: usize = 100;

/// Polling interval for daemon status updates (milliseconds)
const POLL_INTERVAL_MS: u64 = 2000;

/// Console application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMode {
    /// Normal monitoring view
    Normal,
    /// Interactive commit dialog
    CommitDialog,
    /// Interactive restore browser
    RestoreBrowser,
    /// Compare commits (semantic diff)
    Compare,
    /// Search commits
    Search,
    /// Hooks management
    Hooks,
    /// Help screen
    Help,
}

/// Console application state
pub struct Console {
    /// Path to the Logic Pro project being monitored
    pub project_path: PathBuf,
    /// Activity log entries (most recent first)
    pub activity_log: Vec<LogEntry>,
    /// Current daemon status
    pub daemon_status: DaemonStatus,
    /// Repository status (staged, modified, etc.)
    pub repo_status: Option<RepositoryStatus>,
    /// Whether the console should exit
    pub should_quit: bool,
    /// Current UI mode
    pub mode: ConsoleMode,
    /// Commit dialog state
    commit_dialog: CommitDialogState,
    /// Restore browser state
    restore_browser: RestoreBrowserState,
    /// Compare mode state
    compare_state: CompareState,
    /// Search mode state
    search_state: SearchState,
    /// Hooks mode state
    hooks_state: HooksState,
    /// Last daemon poll time
    last_poll: SystemTime,
}

/// State for commit dialog
#[derive(Debug, Clone, Default)]
struct CommitDialogState {
    message: String,
    bpm: String,
    sample_rate: String,
    key: String,
    tags: String,
    active_field: usize, // 0=message, 1=bpm, 2=sample_rate, 3=key, 4=tags
}

/// State for restore browser
#[derive(Debug, Clone, Default)]
struct RestoreBrowserState {
    commits: Vec<CommitEntry>,
    selected_index: usize,
    #[allow(dead_code)]
    loading: bool,
}

/// Commit entry for restore browser
#[derive(Debug, Clone)]
struct CommitEntry {
    id: String,
    short_id: String,
    message: String,
    timestamp: String,
}

/// State for compare mode (semantic diff)
#[derive(Debug, Clone, Default)]
struct CompareState {
    commits: Vec<CommitEntry>,
    selected_a: usize,
    selected_b: usize,
    active_selector: u8, // 0=commit A, 1=commit B
    diff_result: Option<String>,
}

/// State for search mode
#[derive(Debug, Clone, Default)]
struct SearchState {
    query: String,
    results: Vec<CommitEntry>,
    selected_index: usize,
}

/// State for hooks mode
#[derive(Debug, Clone, Default)]
struct HooksState {
    hooks: Vec<(String, String)>, // (type, name)
    selected_index: usize,
}

/// Single entry in the activity log
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// When the log entry was created
    pub timestamp: SystemTime,
    /// Severity level of the log entry
    pub level: LogLevel,
    /// The log message
    pub message: String,
}

/// Log entry severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Daemon connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DaemonStatus {
    Running,
    Stopped,
    Unknown,
}

/// Repository status snapshot
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// Number of staged files
    pub staged: usize,
    /// Number of modified files
    pub modified: usize,
    /// Number of untracked files
    pub untracked: usize,
}

impl Console {
    /// Create a new console instance for the given project
    pub fn new(project_path: PathBuf) -> Self {
        Self {
            project_path,
            activity_log: Vec::new(),
            daemon_status: DaemonStatus::Unknown,
            repo_status: None,
            should_quit: false,
            mode: ConsoleMode::Normal,
            commit_dialog: CommitDialogState::default(),
            restore_browser: RestoreBrowserState::default(),
            compare_state: CompareState::default(),
            search_state: SearchState::default(),
            hooks_state: HooksState::default(),
            last_poll: SystemTime::now(),
        }
    }

    /// Add an entry to the activity log
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        self.activity_log.insert(
            0,
            LogEntry {
                timestamp: SystemTime::now(),
                level,
                message: message.into(),
            },
        );

        // Prune old entries
        if self.activity_log.len() > MAX_LOG_ENTRIES {
            self.activity_log.truncate(MAX_LOG_ENTRIES);
        }
    }

    /// Update daemon status
    pub fn set_daemon_status(&mut self, status: DaemonStatus) {
        if self.daemon_status != status {
            self.daemon_status = status;
            let msg = match status {
                DaemonStatus::Running => "Daemon connected",
                DaemonStatus::Stopped => "Daemon stopped",
                DaemonStatus::Unknown => "Daemon status unknown",
            };
            self.log(LogLevel::Info, msg);
        }
    }

    /// Update repository status
    pub fn set_repo_status(&mut self, staged: usize, modified: usize, untracked: usize) {
        self.repo_status = Some(RepositoryStatus {
            staged,
            modified,
            untracked,
        });
    }

    /// Run the console application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode().context("Failed to enable raw mode")?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .context("Failed to setup terminal")?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).context("Failed to create terminal")?;

        // Add welcome message
        self.log(
            LogLevel::Info,
            format!("Monitoring project: {}", self.project_path.display()),
        );

        // Main event loop
        let res = self.event_loop(&mut terminal).await;

        // Restore terminal
        disable_raw_mode().context("Failed to disable raw mode")?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )
        .context("Failed to restore terminal")?;
        terminal.show_cursor().context("Failed to show cursor")?;

        res
    }

    /// Main event loop
    async fn event_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            // Poll for events with timeout
            if event::poll(Duration::from_millis(250))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key.code, key.modifiers)?;
                }
            }

            // Poll daemon for updates periodically
            self.poll_daemon_updates()?;

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Poll daemon for status updates
    fn poll_daemon_updates(&mut self) -> Result<()> {
        let now = SystemTime::now();
        let elapsed = now
            .duration_since(self.last_poll)
            .unwrap_or(Duration::from_secs(0));

        // Only poll every POLL_INTERVAL_MS milliseconds
        if elapsed.as_millis() >= POLL_INTERVAL_MS as u128 {
            self.last_poll = now;

            // Check daemon status
            let daemon_client = crate::daemon_client::DaemonClient::new();
            match daemon_client.status() {
                Ok(status) => {
                    let new_status = if status.is_running {
                        DaemonStatus::Running
                    } else {
                        DaemonStatus::Stopped
                    };

                    // Log status changes
                    if new_status != self.daemon_status {
                        match new_status {
                            DaemonStatus::Running => {
                                self.log(LogLevel::Success, "Daemon started");
                            }
                            DaemonStatus::Stopped => {
                                self.log(LogLevel::Warning, "Daemon stopped");
                            }
                            DaemonStatus::Unknown => {
                                self.log(LogLevel::Info, "Daemon status unknown");
                            }
                        }
                        self.daemon_status = new_status;
                    }
                }
                Err(_) => {
                    if self.daemon_status != DaemonStatus::Unknown {
                        self.log(LogLevel::Warning, "Unable to reach daemon");
                        self.daemon_status = DaemonStatus::Unknown;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.mode {
            ConsoleMode::Normal => self.handle_normal_mode_key(code, modifiers),
            ConsoleMode::CommitDialog => self.handle_commit_dialog_key(code, modifiers),
            ConsoleMode::RestoreBrowser => self.handle_restore_browser_key(code, modifiers),
            ConsoleMode::Compare => self.handle_compare_mode_key(code, modifiers),
            ConsoleMode::Search => self.handle_search_mode_key(code, modifiers),
            ConsoleMode::Hooks => self.handle_hooks_mode_key(code, modifiers),
            ConsoleMode::Help => self.handle_help_mode_key(code, modifiers),
        }
    }

    /// Handle keyboard in normal mode
    fn handle_normal_mode_key(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match (code, modifiers) {
            // Quit on 'q' or Ctrl+C
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            // Refresh status on 'r'
            (KeyCode::Char('r'), _) => {
                self.log(LogLevel::Info, "Refreshing status...");
                self.refresh_repo_status();
            }
            // Clear log on 'c'
            (KeyCode::Char('c'), _) => {
                self.activity_log.clear();
                self.log(LogLevel::Info, "Log cleared");
            }
            // Open commit dialog on 'i'
            (KeyCode::Char('i'), _) => {
                self.mode = ConsoleMode::CommitDialog;
                self.commit_dialog = CommitDialogState::default();
                self.log(LogLevel::Info, "Opened commit dialog");
            }
            // Open restore browser on 'l'
            (KeyCode::Char('l'), _) => {
                self.mode = ConsoleMode::RestoreBrowser;
                self.restore_browser = RestoreBrowserState::default();
                self.log(LogLevel::Info, "Opened restore browser");
                self.load_commits();
            }
            // Open compare mode on 'd'
            (KeyCode::Char('d'), _) => {
                self.mode = ConsoleMode::Compare;
                self.compare_state = CompareState::default();
                self.log(LogLevel::Info, "Opened compare mode");
                self.load_commits_for_compare();
            }
            // Open search mode on 's'
            (KeyCode::Char('s'), _) => {
                self.mode = ConsoleMode::Search;
                self.search_state = SearchState::default();
                self.log(LogLevel::Info, "Opened search mode");
            }
            // Open hooks mode on 'k'
            (KeyCode::Char('k'), _) => {
                self.mode = ConsoleMode::Hooks;
                self.hooks_state = HooksState::default();
                self.log(LogLevel::Info, "Opened hooks manager");
                self.load_hooks();
            }
            // Show help on '?' or 'h'
            (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) => {
                self.mode = ConsoleMode::Help;
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard in commit dialog mode
    fn handle_commit_dialog_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match code {
            // Cancel on Esc
            KeyCode::Esc => {
                self.mode = ConsoleMode::Normal;
                self.log(LogLevel::Info, "Cancelled commit");
            }
            // Submit on Enter
            KeyCode::Enter => {
                if self.commit_dialog.message.is_empty() {
                    self.log(LogLevel::Warning, "Commit message is required");
                } else {
                    self.execute_commit();
                    self.mode = ConsoleMode::Normal;
                }
            }
            // Tab to next field
            KeyCode::Tab => {
                self.commit_dialog.active_field = (self.commit_dialog.active_field + 1) % 5;
            }
            // BackTab to previous field
            KeyCode::BackTab => {
                self.commit_dialog.active_field = if self.commit_dialog.active_field == 0 {
                    4
                } else {
                    self.commit_dialog.active_field - 1
                };
            }
            // Backspace to delete
            KeyCode::Backspace => {
                let field = self.get_active_field_mut();
                field.pop();
            }
            // Type character
            KeyCode::Char(c) => {
                let field = self.get_active_field_mut();
                field.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard in restore browser mode
    fn handle_restore_browser_key(
        &mut self,
        code: KeyCode,
        _modifiers: KeyModifiers,
    ) -> Result<()> {
        match code {
            // Cancel on Esc
            KeyCode::Esc => {
                self.mode = ConsoleMode::Normal;
                self.log(LogLevel::Info, "Closed restore browser");
            }
            // Navigate up
            KeyCode::Up => {
                if self.restore_browser.selected_index > 0 {
                    self.restore_browser.selected_index -= 1;
                }
            }
            // Navigate down
            KeyCode::Down => {
                if self.restore_browser.selected_index
                    < self.restore_browser.commits.len().saturating_sub(1)
                {
                    self.restore_browser.selected_index += 1;
                }
            }
            // Restore on Enter
            KeyCode::Enter => {
                if !self.restore_browser.commits.is_empty() {
                    let commit_id = self.restore_browser.commits
                        [self.restore_browser.selected_index]
                        .id
                        .clone();
                    self.execute_restore(&commit_id);
                    self.mode = ConsoleMode::Normal;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard in help mode
    fn handle_help_mode_key(&mut self, _code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        // Any key returns to normal mode
        self.mode = ConsoleMode::Normal;
        Ok(())
    }

    /// Handle keyboard in compare mode
    fn handle_compare_mode_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match code {
            // Cancel on Esc
            KeyCode::Esc => {
                self.mode = ConsoleMode::Normal;
                self.log(LogLevel::Info, "Closed compare mode");
            }
            // Tab to switch active selector
            KeyCode::Tab => {
                self.compare_state.active_selector = 1 - self.compare_state.active_selector;
            }
            // Navigate up
            KeyCode::Up => {
                if self.compare_state.active_selector == 0 && self.compare_state.selected_a > 0 {
                    self.compare_state.selected_a -= 1;
                } else if self.compare_state.active_selector == 1
                    && self.compare_state.selected_b > 0
                {
                    self.compare_state.selected_b -= 1;
                }
            }
            // Navigate down
            KeyCode::Down => {
                let max_idx = self.compare_state.commits.len().saturating_sub(1);
                if self.compare_state.active_selector == 0
                    && self.compare_state.selected_a < max_idx
                {
                    self.compare_state.selected_a += 1;
                } else if self.compare_state.active_selector == 1
                    && self.compare_state.selected_b < max_idx
                {
                    self.compare_state.selected_b += 1;
                }
            }
            // Execute comparison on Enter
            KeyCode::Enter => {
                self.execute_compare();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard in search mode
    fn handle_search_mode_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match code {
            // Cancel on Esc
            KeyCode::Esc => {
                self.mode = ConsoleMode::Normal;
                self.log(LogLevel::Info, "Closed search mode");
            }
            // Execute search on Enter
            KeyCode::Enter => {
                self.execute_search();
            }
            // Navigate results with up/down
            KeyCode::Up => {
                if self.search_state.selected_index > 0 {
                    self.search_state.selected_index -= 1;
                }
            }
            KeyCode::Down => {
                if self.search_state.selected_index
                    < self.search_state.results.len().saturating_sub(1)
                {
                    self.search_state.selected_index += 1;
                }
            }
            // Backspace to delete
            KeyCode::Backspace => {
                self.search_state.query.pop();
                // Clear results when query changes
                self.search_state.results.clear();
            }
            // Type character
            KeyCode::Char(c) => {
                self.search_state.query.push(c);
                // Clear results when query changes
                self.search_state.results.clear();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle keyboard in hooks mode
    fn handle_hooks_mode_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        match code {
            // Cancel on Esc
            KeyCode::Esc => {
                self.mode = ConsoleMode::Normal;
                self.log(LogLevel::Info, "Closed hooks manager");
            }
            // Navigate up
            KeyCode::Up => {
                if self.hooks_state.selected_index > 0 {
                    self.hooks_state.selected_index -= 1;
                }
            }
            // Navigate down
            KeyCode::Down => {
                if self.hooks_state.selected_index < self.hooks_state.hooks.len().saturating_sub(1)
                {
                    self.hooks_state.selected_index += 1;
                }
            }
            // Delete hook on 'd'
            KeyCode::Char('d') => {
                if !self.hooks_state.hooks.is_empty() {
                    self.delete_selected_hook();
                }
            }
            // Refresh hooks list on 'r'
            KeyCode::Char('r') => {
                self.load_hooks();
            }
            _ => {}
        }
        Ok(())
    }

    /// Get mutable reference to active field in commit dialog
    fn get_active_field_mut(&mut self) -> &mut String {
        match self.commit_dialog.active_field {
            0 => &mut self.commit_dialog.message,
            1 => &mut self.commit_dialog.bpm,
            2 => &mut self.commit_dialog.sample_rate,
            3 => &mut self.commit_dialog.key,
            4 => &mut self.commit_dialog.tags,
            _ => &mut self.commit_dialog.message,
        }
    }

    /// Refresh repository status
    fn refresh_repo_status(&mut self) {
        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Warning, "Not an Oxen repository");
            return;
        }

        // Get repository status using async runtime
        let project_path = self.project_path.clone();
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.status().await
            })
        }) {
            Ok(status) => {
                self.set_repo_status(
                    status.staged.len(),
                    status.modified.len(),
                    status.untracked.len(),
                );
                self.log(
                    LogLevel::Success,
                    format!(
                        "Status: {} staged, {} modified, {} untracked",
                        status.staged.len(),
                        status.modified.len(),
                        status.untracked.len()
                    ),
                );
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to get status: {}", e));
            }
        }
    }

    /// Load commit history
    fn load_commits(&mut self) {
        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Warning, "Not an Oxen repository");
            return;
        }

        // Get commit history using async runtime
        let project_path = self.project_path.clone();
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.get_history(Some(20)).await // Limit to 20 commits
            })
        }) {
            Ok(commits) => {
                self.restore_browser.commits = commits
                    .iter()
                    .map(|commit| {
                        let short_id = if commit.id.len() >= 7 {
                            commit.id[..7].to_string()
                        } else {
                            commit.id.clone()
                        };

                        // Extract first line of message
                        let first_line = commit
                            .message
                            .lines()
                            .next()
                            .unwrap_or(&commit.message)
                            .to_string();

                        CommitEntry {
                            id: commit.id.clone(),
                            short_id,
                            message: first_line,
                            timestamp: "now".to_string(), // TODO: Calculate relative time
                        }
                    })
                    .collect();

                self.log(
                    LogLevel::Success,
                    format!("Loaded {} commits", self.restore_browser.commits.len()),
                );
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to load commits: {}", e));
                // Show empty list on error
                self.restore_browser.commits = vec![];
            }
        }
    }

    /// Execute commit with current dialog values
    fn execute_commit(&mut self) {
        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Error, "Not an Oxen repository");
            return;
        }

        // Build commit metadata
        let mut metadata = CommitMetadata::new(self.commit_dialog.message.clone());

        // Add optional metadata
        if !self.commit_dialog.bpm.is_empty() {
            if let Ok(bpm) = self.commit_dialog.bpm.parse::<f32>() {
                metadata = metadata.with_bpm(bpm);
            }
        }

        if !self.commit_dialog.sample_rate.is_empty() {
            if let Ok(sr) = self.commit_dialog.sample_rate.parse::<u32>() {
                metadata = metadata.with_sample_rate(sr);
            }
        }

        if !self.commit_dialog.key.is_empty() {
            metadata = metadata.with_key_signature(self.commit_dialog.key.clone());
        }

        if !self.commit_dialog.tags.is_empty() {
            for tag in self.commit_dialog.tags.split(',') {
                metadata = metadata.with_tag(tag.trim().to_string());
            }
        }

        // Create commit using async runtime
        let project_path = self.project_path.clone();

        self.log(LogLevel::Info, "Creating commit...");

        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.create_commit(metadata).await
            })
        }) {
            Ok(commit_id) => {
                let short_id = if commit_id.len() >= 7 {
                    &commit_id[..7]
                } else {
                    &commit_id
                };
                self.log(LogLevel::Success, format!("Commit created: {}", short_id));
                // Refresh status after commit
                self.refresh_repo_status();
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to create commit: {}", e));
            }
        }
    }

    /// Execute restore to specified commit
    fn execute_restore(&mut self, commit_id: &str) {
        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Error, "Not an Oxen repository");
            return;
        }

        self.log(
            LogLevel::Info,
            format!(
                "Restoring to commit {}...",
                &commit_id[..7.min(commit_id.len())]
            ),
        );

        // Restore using async runtime
        let project_path = self.project_path.clone();
        let commit_id_owned = commit_id.to_string();

        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.restore(&commit_id_owned).await
            })
        }) {
            Ok(_) => {
                self.log(
                    LogLevel::Success,
                    format!(
                        "Restored to commit: {}",
                        &commit_id[..7.min(commit_id.len())]
                    ),
                );
                // Refresh status after restore
                self.refresh_repo_status();
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to restore: {}", e));
            }
        }
    }

    /// Load commits for compare mode
    fn load_commits_for_compare(&mut self) {
        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Warning, "Not an Oxen repository");
            return;
        }

        // Get commit history using async runtime
        let project_path = self.project_path.clone();
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.get_history(Some(20)).await // Limit to 20 commits
            })
        }) {
            Ok(commits) => {
                self.compare_state.commits = commits
                    .iter()
                    .map(|commit| {
                        let short_id = if commit.id.len() >= 7 {
                            commit.id[..7].to_string()
                        } else {
                            commit.id.clone()
                        };

                        let first_line = commit
                            .message
                            .lines()
                            .next()
                            .unwrap_or(&commit.message)
                            .to_string();

                        CommitEntry {
                            id: commit.id.clone(),
                            short_id,
                            message: first_line,
                            timestamp: "now".to_string(),
                        }
                    })
                    .collect();

                self.log(
                    LogLevel::Success,
                    format!(
                        "Loaded {} commits for comparison",
                        self.compare_state.commits.len()
                    ),
                );
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to load commits: {}", e));
                self.compare_state.commits = vec![];
            }
        }
    }

    /// Execute semantic comparison of two commits
    fn execute_compare(&mut self) {
        if self.compare_state.commits.is_empty() {
            self.log(LogLevel::Warning, "No commits available to compare");
            return;
        }

        let commit_a = &self.compare_state.commits[self.compare_state.selected_a];
        let commit_b = &self.compare_state.commits[self.compare_state.selected_b];

        // Parse metadata from both commits
        let metadata_a = CommitMetadata::parse_commit_message(&commit_a.message);
        let metadata_b = CommitMetadata::parse_commit_message(&commit_b.message);

        // Generate colored diff
        let diff = metadata_a.compare_with(&metadata_b);
        self.compare_state.diff_result = Some(diff);

        self.log(
            LogLevel::Success,
            format!("Compared {} vs {}", &commit_a.short_id, &commit_b.short_id),
        );
    }

    /// Execute search query
    fn execute_search(&mut self) {
        use crate::search::SearchEngine;

        if self.search_state.query.is_empty() {
            self.log(LogLevel::Warning, "Search query is empty");
            return;
        }

        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Error, "Not an Oxen repository");
            return;
        }

        self.log(
            LogLevel::Info,
            format!("Searching: {}", self.search_state.query),
        );

        // Get all commits
        let project_path = self.project_path.clone();
        match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let repo = OxenRepository::new(&project_path);
                repo.get_history(None).await
            })
        }) {
            Ok(commits) => {
                // Parse and execute search
                let query = SearchEngine::parse_query(&self.search_state.query);
                let engine = SearchEngine::new();
                let results = engine.search(&commits, &query);

                self.search_state.results = results
                    .iter()
                    .map(|commit| {
                        let short_id = if commit.id.len() >= 7 {
                            commit.id[..7].to_string()
                        } else {
                            commit.id.clone()
                        };

                        let first_line = commit
                            .message
                            .lines()
                            .next()
                            .unwrap_or(&commit.message)
                            .to_string();

                        CommitEntry {
                            id: commit.id.clone(),
                            short_id,
                            message: first_line,
                            timestamp: "now".to_string(),
                        }
                    })
                    .collect();

                self.log(
                    LogLevel::Success,
                    format!("Found {} matching commits", self.search_state.results.len()),
                );
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Search failed: {}", e));
                self.search_state.results = vec![];
            }
        }
    }

    /// Load hooks from repository
    fn load_hooks(&mut self) {
        use crate::hooks::HookManager;

        // Check if repository exists
        let oxen_dir = self.project_path.join(".oxen");
        if !oxen_dir.exists() {
            self.log(LogLevel::Warning, "Not an Oxen repository");
            return;
        }

        let manager = HookManager::new(&self.project_path);

        match manager.list_hooks() {
            Ok(hooks) => {
                self.hooks_state.hooks = hooks
                    .iter()
                    .map(|(hook_type, name)| {
                        let type_str = match hook_type {
                            crate::hooks::HookType::PreCommit => "pre-commit",
                            crate::hooks::HookType::PostCommit => "post-commit",
                        };
                        (type_str.to_string(), name.clone())
                    })
                    .collect();

                self.log(
                    LogLevel::Success,
                    format!("Loaded {} hooks", self.hooks_state.hooks.len()),
                );
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to load hooks: {}", e));
                self.hooks_state.hooks = vec![];
            }
        }
    }

    /// Delete the currently selected hook
    fn delete_selected_hook(&mut self) {
        use crate::hooks::{HookManager, HookType};

        if self.hooks_state.hooks.is_empty() {
            return;
        }

        let (hook_type_str, hook_name) = &self.hooks_state.hooks[self.hooks_state.selected_index];

        let hook_type = match hook_type_str.as_str() {
            "pre-commit" => HookType::PreCommit,
            "post-commit" => HookType::PostCommit,
            _ => {
                self.log(LogLevel::Error, "Invalid hook type");
                return;
            }
        };

        let manager = HookManager::new(&self.project_path);

        match manager.remove_hook(hook_name, hook_type) {
            Ok(_) => {
                self.log(LogLevel::Success, format!("Deleted hook: {}", hook_name));
                // Reload hooks list
                self.load_hooks();
                // Adjust selection if needed
                if self.hooks_state.selected_index >= self.hooks_state.hooks.len()
                    && self.hooks_state.selected_index > 0
                {
                    self.hooks_state.selected_index -= 1;
                }
            }
            Err(e) => {
                self.log(LogLevel::Error, format!("Failed to delete hook: {}", e));
            }
        }
    }

    /// Render the UI
    fn ui(&self, f: &mut Frame) {
        let size = f.size();

        // Create main layout: header, body, footer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Body
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Render header
        self.render_header(f, chunks[0]);

        // Render body based on mode
        match self.mode {
            ConsoleMode::Normal => {
                // Split body into status panel and activity log
                let body_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(30), // Status panel
                        Constraint::Percentage(70), // Activity log
                    ])
                    .split(chunks[1]);

                // Render status panel
                self.render_status_panel(f, body_chunks[0]);

                // Render activity log
                self.render_activity_log(f, body_chunks[1]);
            }
            ConsoleMode::CommitDialog => {
                // Render activity log as background
                self.render_activity_log(f, chunks[1]);
                // Render commit dialog as overlay
                self.render_commit_dialog(f, chunks[1]);
            }
            ConsoleMode::RestoreBrowser => {
                // Render restore browser
                self.render_restore_browser(f, chunks[1]);
            }
            ConsoleMode::Compare => {
                // Render compare mode
                self.render_compare_mode(f, chunks[1]);
            }
            ConsoleMode::Search => {
                // Render search mode
                self.render_search_mode(f, chunks[1]);
            }
            ConsoleMode::Hooks => {
                // Render hooks mode
                self.render_hooks_mode(f, chunks[1]);
            }
            ConsoleMode::Help => {
                // Render help screen
                self.render_help_screen(f, chunks[1]);
            }
        }

        // Render footer
        self.render_footer(f, chunks[2]);
    }

    /// Render header
    fn render_header(&self, f: &mut Frame, area: Rect) {
        let title = vec![
            Span::raw("Auxin Console"),
            Span::raw(" - "),
            Span::styled(
                self.project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown"),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        let header = Paragraph::new(Line::from(title))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .alignment(Alignment::Center);

        f.render_widget(header, area);
    }

    /// Render status panel
    fn render_status_panel(&self, f: &mut Frame, area: Rect) {
        let mut status_lines = vec![];

        // Daemon status
        let daemon_indicator = match self.daemon_status {
            DaemonStatus::Running => Span::styled("● Running", Style::default().fg(Color::Green)),
            DaemonStatus::Stopped => Span::styled("● Stopped", Style::default().fg(Color::Red)),
            DaemonStatus::Unknown => Span::styled("● Unknown", Style::default().fg(Color::Yellow)),
        };
        status_lines.push(Line::from(vec![Span::raw("Daemon: "), daemon_indicator]));

        status_lines.push(Line::from(""));

        // Repository status
        if let Some(ref repo) = self.repo_status {
            status_lines.push(Line::from(Span::styled(
                "Repository:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            status_lines.push(Line::from(vec![
                Span::raw("  Staged: "),
                Span::styled(repo.staged.to_string(), Style::default().fg(Color::Green)),
            ]));
            status_lines.push(Line::from(vec![
                Span::raw("  Modified: "),
                Span::styled(
                    repo.modified.to_string(),
                    Style::default().fg(Color::Yellow),
                ),
            ]));
            status_lines.push(Line::from(vec![
                Span::raw("  Untracked: "),
                Span::styled(repo.untracked.to_string(), Style::default().fg(Color::Cyan)),
            ]));
        } else {
            status_lines.push(Line::from(Span::styled(
                "Repository: Not loaded",
                Style::default().fg(Color::DarkGray),
            )));
        }

        let status_panel = Paragraph::new(status_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .title("Status"),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(status_panel, area);
    }

    /// Render activity log
    fn render_activity_log(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .activity_log
            .iter()
            .map(|entry| {
                let (prefix, style) = match entry.level {
                    LogLevel::Info => ("ℹ", Style::default().fg(Color::Cyan)),
                    LogLevel::Success => ("✓", Style::default().fg(Color::Green)),
                    LogLevel::Warning => ("⚠", Style::default().fg(Color::Yellow)),
                    LogLevel::Error => ("✗", Style::default().fg(Color::Red)),
                };

                // Format timestamp (simple HH:MM:SS)
                let time_str = format_timestamp(entry.timestamp);

                let content = Line::from(vec![
                    Span::styled(time_str, Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(prefix, style),
                    Span::raw(" "),
                    Span::raw(&entry.message),
                ]);

                ListItem::new(content)
            })
            .collect();

        let activity_log = List::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::White))
                .title("Activity Log"),
        );

        f.render_widget(activity_log, area);
    }

    /// Render footer with keyboard shortcuts
    fn render_footer(&self, f: &mut Frame, area: Rect) {
        let shortcuts = match self.mode {
            ConsoleMode::Normal => vec![
                Span::styled("q", Style::default().fg(Color::Cyan)),
                Span::raw(":Quit  "),
                Span::styled("i", Style::default().fg(Color::Cyan)),
                Span::raw(":Commit  "),
                Span::styled("l", Style::default().fg(Color::Cyan)),
                Span::raw(":Log  "),
                Span::styled("d", Style::default().fg(Color::Cyan)),
                Span::raw(":Diff  "),
                Span::styled("s", Style::default().fg(Color::Cyan)),
                Span::raw(":Search  "),
                Span::styled("k", Style::default().fg(Color::Cyan)),
                Span::raw(":Hooks  "),
                Span::styled("?", Style::default().fg(Color::Cyan)),
                Span::raw(":Help"),
            ],
            ConsoleMode::CommitDialog => vec![
                Span::styled("Tab", Style::default().fg(Color::Cyan)),
                Span::raw(":Next Field  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(":Submit  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(":Cancel"),
            ],
            ConsoleMode::RestoreBrowser => vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(":Navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(":Restore  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(":Cancel"),
            ],
            ConsoleMode::Compare => vec![
                Span::styled("Tab", Style::default().fg(Color::Cyan)),
                Span::raw(":Switch  "),
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(":Navigate  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(":Compare  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(":Cancel"),
            ],
            ConsoleMode::Search => vec![
                Span::styled("Type", Style::default().fg(Color::Cyan)),
                Span::raw(":Query  "),
                Span::styled("Enter", Style::default().fg(Color::Green)),
                Span::raw(":Search  "),
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(":Navigate  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(":Cancel"),
            ],
            ConsoleMode::Hooks => vec![
                Span::styled("↑↓", Style::default().fg(Color::Cyan)),
                Span::raw(":Navigate  "),
                Span::styled("d", Style::default().fg(Color::Red)),
                Span::raw(":Delete  "),
                Span::styled("r", Style::default().fg(Color::Cyan)),
                Span::raw(":Refresh  "),
                Span::styled("Esc", Style::default().fg(Color::Red)),
                Span::raw(":Cancel"),
            ],
            ConsoleMode::Help => vec![
                Span::styled("Any Key", Style::default().fg(Color::Cyan)),
                Span::raw(":Return to Console"),
            ],
        };

        let footer = Paragraph::new(Line::from(shortcuts))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
    }

    /// Render commit dialog overlay
    fn render_commit_dialog(&self, f: &mut Frame, area: Rect) {
        // Center the dialog
        let dialog_width = area.width.min(60);
        let dialog_height = area.height.min(20);
        let x = (area.width.saturating_sub(dialog_width)) / 2;
        let y = (area.height.saturating_sub(dialog_height)) / 2;

        let dialog_area = Rect {
            x: area.x + x,
            y: area.y + y,
            width: dialog_width,
            height: dialog_height,
        };

        // Create form fields
        let fields = vec![
            Line::from(vec![
                Span::raw("Message: "),
                Span::styled(
                    &self.commit_dialog.message,
                    if self.commit_dialog.active_field == 0 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("BPM: "),
                Span::styled(
                    &self.commit_dialog.bpm,
                    if self.commit_dialog.active_field == 1 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Sample Rate: "),
                Span::styled(
                    &self.commit_dialog.sample_rate,
                    if self.commit_dialog.active_field == 2 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Key: "),
                Span::styled(
                    &self.commit_dialog.key,
                    if self.commit_dialog.active_field == 3 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::raw("Tags: "),
                Span::styled(
                    &self.commit_dialog.tags,
                    if self.commit_dialog.active_field == 4 {
                        Style::default().fg(Color::Yellow)
                    } else {
                        Style::default()
                    },
                ),
            ]),
        ];

        let dialog = Paragraph::new(fields)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Green))
                    .title("Create Commit"),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(dialog, dialog_area);
    }

    /// Render restore browser
    fn render_restore_browser(&self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self
            .restore_browser
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let style = if i == self.restore_browser.selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let content = Line::from(vec![
                    Span::styled(&commit.short_id, Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(&commit.message, style),
                    Span::raw(" ("),
                    Span::styled(&commit.timestamp, Style::default().fg(Color::DarkGray)),
                    Span::raw(")"),
                ]);

                ListItem::new(content)
            })
            .collect();

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .title("Commit History (↑↓ to navigate, Enter to restore, Esc to cancel)"),
            )
            .highlight_symbol("► ");

        f.render_widget(list, area);
    }

    /// Render help screen
    fn render_help_screen(&self, f: &mut Frame, area: Rect) {
        let help_text = vec![
            Line::from(""),
            Line::from(Span::styled(
                "Auxin Console - Keyboard Shortcuts",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Normal Mode:"),
            Line::from("  q         - Quit console"),
            Line::from("  i         - Open commit dialog"),
            Line::from("  l         - Open restore browser (log)"),
            Line::from("  d         - Open compare mode (semantic diff)"),
            Line::from("  s         - Open search mode"),
            Line::from("  k         - Open hooks manager"),
            Line::from("  r         - Refresh repository status"),
            Line::from("  c         - Clear activity log"),
            Line::from("  ?  or h   - Show this help"),
            Line::from(""),
            Line::from("Commit Dialog:"),
            Line::from("  Tab       - Move to next field"),
            Line::from("  Shift+Tab - Move to previous field"),
            Line::from("  Enter     - Submit commit"),
            Line::from("  Esc       - Cancel"),
            Line::from(""),
            Line::from("Restore Browser:"),
            Line::from("  ↑ / ↓     - Navigate commit list"),
            Line::from("  Enter     - Restore selected commit"),
            Line::from("  Esc       - Close browser"),
            Line::from(""),
            Line::from("Compare Mode:"),
            Line::from("  Tab       - Switch between commit A and B"),
            Line::from("  ↑ / ↓     - Navigate commit list"),
            Line::from("  Enter     - Execute comparison"),
            Line::from("  Esc       - Close compare mode"),
            Line::from(""),
            Line::from("Search Mode:"),
            Line::from("  Type      - Enter search query (e.g., bpm:120-140)"),
            Line::from("  Enter     - Execute search"),
            Line::from("  ↑ / ↓     - Navigate results"),
            Line::from("  Esc       - Close search mode"),
            Line::from(""),
            Line::from("Hooks Mode:"),
            Line::from("  ↑ / ↓     - Navigate hook list"),
            Line::from("  d         - Delete selected hook"),
            Line::from("  r         - Refresh hook list"),
            Line::from("  Esc       - Close hooks mode"),
            Line::from(""),
            Line::from(Span::styled(
                "Press any key to return to console",
                Style::default().fg(Color::Green),
            )),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Cyan))
                    .title("Help"),
            )
            .alignment(Alignment::Left);

        f.render_widget(help, area);
    }

    /// Render compare mode
    fn render_compare_mode(&self, f: &mut Frame, area: Rect) {
        // Split into two columns: commit selectors and diff result
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(50), // Commit selectors
                Constraint::Percentage(50), // Diff result
            ])
            .split(area);

        // Left side: Two commit selectors
        let selector_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Commit A
                Constraint::Percentage(50), // Commit B
            ])
            .split(chunks[0]);

        // Render commit A selector
        self.render_commit_selector(
            f,
            selector_chunks[0],
            "Commit A",
            self.compare_state.selected_a,
            self.compare_state.active_selector == 0,
        );

        // Render commit B selector
        self.render_commit_selector(
            f,
            selector_chunks[1],
            "Commit B",
            self.compare_state.selected_b,
            self.compare_state.active_selector == 1,
        );

        // Right side: Diff result
        let diff_text = if let Some(ref diff) = self.compare_state.diff_result {
            diff.clone()
        } else {
            "Press Enter to compare selected commits".to_string()
        };

        let diff_paragraph = Paragraph::new(diff_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White))
                    .title("Semantic Diff"),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(diff_paragraph, chunks[1]);
    }

    /// Helper to render a commit selector
    fn render_commit_selector(
        &self,
        f: &mut Frame,
        area: Rect,
        title: &str,
        selected_index: usize,
        is_active: bool,
    ) {
        let items: Vec<ListItem> = self
            .compare_state
            .commits
            .iter()
            .enumerate()
            .map(|(i, commit)| {
                let style = if i == selected_index {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                let content = Line::from(vec![
                    Span::styled(&commit.short_id, Style::default().fg(Color::Cyan)),
                    Span::raw(" - "),
                    Span::styled(&commit.message, style),
                ]);

                ListItem::new(content)
            })
            .collect();

        let border_color = if is_active {
            Color::Green
        } else {
            Color::White
        };

        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color))
                    .title(title),
            )
            .highlight_symbol("► ");

        f.render_widget(list, area);
    }

    /// Render search mode
    fn render_search_mode(&self, f: &mut Frame, area: Rect) {
        // Split into query input and results
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Query input
                Constraint::Min(0),    // Results
            ])
            .split(area);

        // Query input
        let query_text = format!("Query: {}", self.search_state.query);
        let query_paragraph = Paragraph::new(query_text).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green))
                .title("Search (e.g., bpm:120-140 key:minor tag:mixing)"),
        );

        f.render_widget(query_paragraph, chunks[0]);

        // Results list
        if self.search_state.results.is_empty() {
            let empty_text = if self.search_state.query.is_empty() {
                "Enter a search query and press Enter to search"
            } else {
                "No results found. Press Enter to search."
            };

            let empty_paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title("Search Results"),
                )
                .alignment(Alignment::Center);

            f.render_widget(empty_paragraph, chunks[1]);
        } else {
            let items: Vec<ListItem> = self
                .search_state
                .results
                .iter()
                .enumerate()
                .map(|(i, commit)| {
                    let style = if i == self.search_state.selected_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let content = Line::from(vec![
                        Span::styled(&commit.short_id, Style::default().fg(Color::Cyan)),
                        Span::raw(" - "),
                        Span::styled(&commit.message, style),
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title(format!(
                            "Search Results ({} found)",
                            self.search_state.results.len()
                        )),
                )
                .highlight_symbol("► ");

            f.render_widget(list, chunks[1]);
        }
    }

    /// Render hooks mode
    fn render_hooks_mode(&self, f: &mut Frame, area: Rect) {
        if self.hooks_state.hooks.is_empty() {
            let empty_text = vec![
                Line::from(""),
                Line::from(Span::styled(
                    "No hooks installed",
                    Style::default().fg(Color::Yellow),
                )),
                Line::from(""),
                Line::from("Install built-in hooks with:"),
                Line::from("  auxin hooks install <name>"),
                Line::from(""),
                Line::from("Available built-in hooks:"),
                Line::from("  validate-metadata (pre-commit)"),
                Line::from("  check-file-sizes (pre-commit)"),
                Line::from("  notify (post-commit)"),
                Line::from("  backup (post-commit)"),
            ];

            let empty_paragraph = Paragraph::new(empty_text)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title("Hooks Manager"),
                )
                .alignment(Alignment::Center);

            f.render_widget(empty_paragraph, area);
        } else {
            let items: Vec<ListItem> = self
                .hooks_state
                .hooks
                .iter()
                .enumerate()
                .map(|(i, (hook_type, name))| {
                    let style = if i == self.hooks_state.selected_index {
                        Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default()
                    };

                    let type_color = match hook_type.as_str() {
                        "pre-commit" => Color::Green,
                        "post-commit" => Color::Cyan,
                        _ => Color::White,
                    };

                    let content = Line::from(vec![
                        Span::styled(hook_type, Style::default().fg(type_color)),
                        Span::raw(" / "),
                        Span::styled(name, style),
                    ]);

                    ListItem::new(content)
                })
                .collect();

            let list = List::new(items)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_style(Style::default().fg(Color::White))
                        .title(format!(
                            "Hooks Manager ({} hooks, press 'd' to delete)",
                            self.hooks_state.hooks.len()
                        )),
                )
                .highlight_symbol("► ");

            f.render_widget(list, area);
        }
    }
}

/// Format a SystemTime as HH:MM:SS
fn format_timestamp(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    let duration = time.duration_since(UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    let hours = (secs / 3600) % 24;
    let minutes = (secs / 60) % 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_console_creation() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.activity_log.len(), 0);
        assert_eq!(console.daemon_status, DaemonStatus::Unknown);
        assert!(console.repo_status.is_none());
        assert!(!console.should_quit);
    }

    #[test]
    fn test_log_entry() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.log(LogLevel::Info, "Test message");
        assert_eq!(console.activity_log.len(), 1);
        assert_eq!(console.activity_log[0].message, "Test message");
        assert_eq!(console.activity_log[0].level, LogLevel::Info);
    }

    #[test]
    fn test_log_pruning() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        // Add more than MAX_LOG_ENTRIES
        for i in 0..150 {
            console.log(LogLevel::Info, format!("Message {}", i));
        }
        assert_eq!(console.activity_log.len(), MAX_LOG_ENTRIES);
        // Most recent should be "Message 149"
        assert_eq!(console.activity_log[0].message, "Message 149");
    }

    #[test]
    fn test_daemon_status_update() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.set_daemon_status(DaemonStatus::Running);
        assert_eq!(console.daemon_status, DaemonStatus::Running);
        assert_eq!(console.activity_log.len(), 1);
        assert!(console.activity_log[0].message.contains("connected"));
    }

    #[test]
    fn test_repo_status_update() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.set_repo_status(2, 3, 1);
        assert!(console.repo_status.is_some());
        let status = console.repo_status.unwrap();
        assert_eq!(status.staged, 2);
        assert_eq!(status.modified, 3);
        assert_eq!(status.untracked, 1);
    }

    // CompareState Tests

    #[test]
    fn test_compare_state_default() {
        let state = CompareState::default();
        assert_eq!(state.commits.len(), 0);
        assert_eq!(state.selected_a, 0);
        assert_eq!(state.selected_b, 0);
        assert_eq!(state.active_selector, 0);
        assert!(state.diff_result.is_none());
    }

    #[test]
    fn test_compare_mode_initialization() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.compare_state.commits.len(), 0);
        assert_eq!(console.compare_state.active_selector, 0);
    }

    #[test]
    fn test_compare_state_selector_switching() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Compare;

        // Initially on selector 0
        assert_eq!(console.compare_state.active_selector, 0);

        // Simulate Tab key to switch
        console
            .handle_compare_mode_key(KeyCode::Tab, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.active_selector, 1);

        // Tab again to switch back
        console
            .handle_compare_mode_key(KeyCode::Tab, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.active_selector, 0);
    }

    #[test]
    fn test_compare_mode_navigation() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Compare;

        // Add some test commits
        console.compare_state.commits = vec![
            CommitEntry {
                id: "abc123".to_string(),
                short_id: "abc123".to_string(),
                message: "Commit 1".to_string(),
                timestamp: "now".to_string(),
            },
            CommitEntry {
                id: "def456".to_string(),
                short_id: "def456".to_string(),
                message: "Commit 2".to_string(),
                timestamp: "now".to_string(),
            },
            CommitEntry {
                id: "ghi789".to_string(),
                short_id: "ghi789".to_string(),
                message: "Commit 3".to_string(),
                timestamp: "now".to_string(),
            },
        ];

        // Navigate selector A down
        console.compare_state.active_selector = 0;
        console.compare_state.selected_a = 0;
        console
            .handle_compare_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.selected_a, 1);

        // Navigate selector A up
        console
            .handle_compare_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.selected_a, 0);

        // Switch to selector B
        console.compare_state.active_selector = 1;
        console.compare_state.selected_b = 0;
        console
            .handle_compare_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.selected_b, 1);
    }

    #[test]
    fn test_compare_mode_navigation_boundaries() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Compare;

        // Add commits
        console.compare_state.commits = vec![
            CommitEntry {
                id: "abc123".to_string(),
                short_id: "abc123".to_string(),
                message: "Commit 1".to_string(),
                timestamp: "now".to_string(),
            },
            CommitEntry {
                id: "def456".to_string(),
                short_id: "def456".to_string(),
                message: "Commit 2".to_string(),
                timestamp: "now".to_string(),
            },
        ];

        // Try to navigate up from 0 (should stay at 0)
        console.compare_state.active_selector = 0;
        console.compare_state.selected_a = 0;
        console
            .handle_compare_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.selected_a, 0);

        // Navigate to last item
        console.compare_state.selected_a = 1;
        // Try to navigate down from last (should stay at last)
        console
            .handle_compare_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.compare_state.selected_a, 1);
    }

    #[test]
    fn test_compare_mode_exit() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Compare;

        console
            .handle_compare_mode_key(KeyCode::Esc, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Normal);
    }

    // SearchState Tests

    #[test]
    fn test_search_state_default() {
        let state = SearchState::default();
        assert_eq!(state.query, "");
        assert_eq!(state.results.len(), 0);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_search_mode_initialization() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.search_state.query, "");
        assert_eq!(console.search_state.results.len(), 0);
    }

    #[test]
    fn test_search_mode_query_input() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        // Type some characters
        console
            .handle_search_mode_key(KeyCode::Char('b'), KeyModifiers::empty())
            .unwrap();
        console
            .handle_search_mode_key(KeyCode::Char('p'), KeyModifiers::empty())
            .unwrap();
        console
            .handle_search_mode_key(KeyCode::Char('m'), KeyModifiers::empty())
            .unwrap();

        assert_eq!(console.search_state.query, "bpm");
    }

    #[test]
    fn test_search_mode_backspace() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        // Type and delete
        console
            .handle_search_mode_key(KeyCode::Char('t'), KeyModifiers::empty())
            .unwrap();
        console
            .handle_search_mode_key(KeyCode::Char('e'), KeyModifiers::empty())
            .unwrap();
        console
            .handle_search_mode_key(KeyCode::Char('s'), KeyModifiers::empty())
            .unwrap();
        console
            .handle_search_mode_key(KeyCode::Char('t'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.query, "test");

        console
            .handle_search_mode_key(KeyCode::Backspace, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.query, "tes");

        console
            .handle_search_mode_key(KeyCode::Backspace, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.query, "te");
    }

    #[test]
    fn test_search_mode_results_navigation() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        // Add test results
        console.search_state.results = vec![
            CommitEntry {
                id: "abc123".to_string(),
                short_id: "abc123".to_string(),
                message: "Result 1".to_string(),
                timestamp: "now".to_string(),
            },
            CommitEntry {
                id: "def456".to_string(),
                short_id: "def456".to_string(),
                message: "Result 2".to_string(),
                timestamp: "now".to_string(),
            },
            CommitEntry {
                id: "ghi789".to_string(),
                short_id: "ghi789".to_string(),
                message: "Result 3".to_string(),
                timestamp: "now".to_string(),
            },
        ];

        console.search_state.selected_index = 0;

        // Navigate down
        console
            .handle_search_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.selected_index, 1);

        console
            .handle_search_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.selected_index, 2);

        // Navigate up
        console
            .handle_search_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.selected_index, 1);
    }

    #[test]
    fn test_search_mode_navigation_boundaries() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        console.search_state.results = vec![CommitEntry {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            message: "Result 1".to_string(),
            timestamp: "now".to_string(),
        }];

        console.search_state.selected_index = 0;

        // Try to go up from 0
        console
            .handle_search_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.selected_index, 0);

        // Try to go down from last
        console
            .handle_search_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.selected_index, 0);
    }

    #[test]
    fn test_search_mode_exit() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        console
            .handle_search_mode_key(KeyCode::Esc, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Normal);
    }

    #[test]
    fn test_search_mode_clears_results_on_query_change() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Search;

        // Add some results
        console.search_state.results = vec![CommitEntry {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            message: "Result 1".to_string(),
            timestamp: "now".to_string(),
        }];

        // Type a character - should clear results
        console
            .handle_search_mode_key(KeyCode::Char('a'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.results.len(), 0);

        // Add results again
        console.search_state.results = vec![CommitEntry {
            id: "abc123".to_string(),
            short_id: "abc123".to_string(),
            message: "Result 1".to_string(),
            timestamp: "now".to_string(),
        }];

        // Backspace - should also clear results
        console
            .handle_search_mode_key(KeyCode::Backspace, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.search_state.results.len(), 0);
    }

    // HooksState Tests

    #[test]
    fn test_hooks_state_default() {
        let state = HooksState::default();
        assert_eq!(state.hooks.len(), 0);
        assert_eq!(state.selected_index, 0);
    }

    #[test]
    fn test_hooks_mode_initialization() {
        let console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.hooks_state.hooks.len(), 0);
        assert_eq!(console.hooks_state.selected_index, 0);
    }

    #[test]
    fn test_hooks_mode_navigation() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Hooks;

        // Add test hooks
        console.hooks_state.hooks = vec![
            ("pre-commit".to_string(), "validate-metadata".to_string()),
            ("post-commit".to_string(), "notify".to_string()),
            ("pre-commit".to_string(), "check-file-sizes".to_string()),
        ];

        console.hooks_state.selected_index = 0;

        // Navigate down
        console
            .handle_hooks_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.hooks_state.selected_index, 1);

        console
            .handle_hooks_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.hooks_state.selected_index, 2);

        // Navigate up
        console
            .handle_hooks_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.hooks_state.selected_index, 1);
    }

    #[test]
    fn test_hooks_mode_navigation_boundaries() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Hooks;

        console.hooks_state.hooks = vec![
            ("pre-commit".to_string(), "hook1".to_string()),
            ("post-commit".to_string(), "hook2".to_string()),
        ];

        console.hooks_state.selected_index = 0;

        // Try to go up from 0
        console
            .handle_hooks_mode_key(KeyCode::Up, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.hooks_state.selected_index, 0);

        // Go to last
        console.hooks_state.selected_index = 1;
        // Try to go down from last
        console
            .handle_hooks_mode_key(KeyCode::Down, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.hooks_state.selected_index, 1);
    }

    #[test]
    fn test_hooks_mode_exit() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        console.mode = ConsoleMode::Hooks;

        console
            .handle_hooks_mode_key(KeyCode::Esc, KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Normal);
    }

    // Mode Transition Tests

    #[test]
    fn test_mode_transition_to_compare() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.mode, ConsoleMode::Normal);

        // Press 'd' to enter compare mode
        console
            .handle_normal_mode_key(KeyCode::Char('d'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Compare);
    }

    #[test]
    fn test_mode_transition_to_search() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.mode, ConsoleMode::Normal);

        // Press 's' to enter search mode
        console
            .handle_normal_mode_key(KeyCode::Char('s'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Search);
    }

    #[test]
    fn test_mode_transition_to_hooks() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        assert_eq!(console.mode, ConsoleMode::Normal);

        // Press 'k' to enter hooks mode
        console
            .handle_normal_mode_key(KeyCode::Char('k'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Hooks);
    }

    #[test]
    fn test_mode_transition_resets_state() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        // Enter search mode and type a query
        console
            .handle_normal_mode_key(KeyCode::Char('s'), KeyModifiers::empty())
            .unwrap();
        console.search_state.query = "old query".to_string();

        // Exit and re-enter search mode
        console
            .handle_search_mode_key(KeyCode::Esc, KeyModifiers::empty())
            .unwrap();
        console
            .handle_normal_mode_key(KeyCode::Char('s'), KeyModifiers::empty())
            .unwrap();

        // State should be reset
        assert_eq!(console.search_state.query, "");
    }

    #[test]
    fn test_quit_keyboard_shortcut() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        assert!(!console.should_quit);

        console
            .handle_normal_mode_key(KeyCode::Char('q'), KeyModifiers::empty())
            .unwrap();
        assert!(console.should_quit);
    }

    #[test]
    fn test_ctrl_c_keyboard_shortcut() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));
        assert!(!console.should_quit);

        console
            .handle_normal_mode_key(KeyCode::Char('c'), KeyModifiers::CONTROL)
            .unwrap();
        assert!(console.should_quit);
    }

    #[test]
    fn test_help_mode_transitions() {
        let mut console = Console::new(PathBuf::from("/test/project.logicx"));

        // Enter help mode with '?'
        console
            .handle_normal_mode_key(KeyCode::Char('?'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Help);

        // Any key exits help mode
        console
            .handle_help_mode_key(KeyCode::Char('x'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Normal);

        // Enter help mode with 'h'
        console
            .handle_normal_mode_key(KeyCode::Char('h'), KeyModifiers::empty())
            .unwrap();
        assert_eq!(console.mode, ConsoleMode::Help);
    }

    // CommitEntry Tests

    #[test]
    fn test_commit_entry_creation() {
        let entry = CommitEntry {
            id: "abc123def456".to_string(),
            short_id: "abc123d".to_string(),
            message: "Test commit".to_string(),
            timestamp: "2 hours ago".to_string(),
        };

        assert_eq!(entry.id, "abc123def456");
        assert_eq!(entry.short_id, "abc123d");
        assert_eq!(entry.message, "Test commit");
        assert_eq!(entry.timestamp, "2 hours ago");
    }

    // format_timestamp Tests

    #[test]
    fn test_format_timestamp() {
        use std::time::UNIX_EPOCH;

        // Test known timestamp
        let time = UNIX_EPOCH + std::time::Duration::from_secs(3661); // 1 hour, 1 minute, 1 second
        let formatted = format_timestamp(time);
        assert_eq!(formatted, "01:01:01");
    }

    #[test]
    fn test_format_timestamp_zero() {
        use std::time::UNIX_EPOCH;

        let formatted = format_timestamp(UNIX_EPOCH);
        assert_eq!(formatted, "00:00:00");
    }
}
