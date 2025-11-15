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

/// Maximum number of activity log entries to retain
const MAX_LOG_ENTRIES: usize = 100;

/// Polling interval for daemon status updates (milliseconds)
const POLL_INTERVAL_MS: u64 = 2000;

/// Console application mode
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConsoleMode {
    /// Normal monitoring view
    Normal,
    /// Interactive commit dialog
    CommitDialog,
    /// Interactive restore browser
    RestoreBrowser,
    /// Help screen
    Help,
}

/// Console application state
pub struct Console {
    /// Path to the Logic Pro project being monitored
    project_path: PathBuf,
    /// Activity log entries (most recent first)
    activity_log: Vec<LogEntry>,
    /// Current daemon status
    daemon_status: DaemonStatus,
    /// Repository status (staged, modified, etc.)
    repo_status: Option<RepositoryStatus>,
    /// Whether the console should exit
    should_quit: bool,
    /// Current UI mode
    mode: ConsoleMode,
    /// Commit dialog state
    commit_dialog: CommitDialogState,
    /// Restore browser state
    restore_browser: RestoreBrowserState,
    /// Last daemon poll time
    last_poll: SystemTime,
}

/// State for commit dialog
#[derive(Debug, Clone)]
struct CommitDialogState {
    message: String,
    bpm: String,
    sample_rate: String,
    key: String,
    tags: String,
    active_field: usize, // 0=message, 1=bpm, 2=sample_rate, 3=key, 4=tags
}

impl Default for CommitDialogState {
    fn default() -> Self {
        Self {
            message: String::new(),
            bpm: String::new(),
            sample_rate: String::new(),
            key: String::new(),
            tags: String::new(),
            active_field: 0,
        }
    }
}

/// State for restore browser
#[derive(Debug, Clone)]
struct RestoreBrowserState {
    commits: Vec<CommitEntry>,
    selected_index: usize,
    loading: bool,
}

impl Default for RestoreBrowserState {
    fn default() -> Self {
        Self {
            commits: Vec::new(),
            selected_index: 0,
            loading: false,
        }
    }
}

/// Commit entry for restore browser
#[derive(Debug, Clone)]
struct CommitEntry {
    id: String,
    short_id: String,
    message: String,
    timestamp: String,
}

/// Single entry in the activity log
#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: SystemTime,
    level: LogLevel,
    message: String,
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
    staged: usize,
    modified: usize,
    untracked: usize,
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
    async fn event_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<()> {
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
        let elapsed = now.duration_since(self.last_poll).unwrap_or(Duration::from_secs(0));

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
    fn handle_restore_browser_key(&mut self, code: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
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
                if self.restore_browser.selected_index < self.restore_browser.commits.len().saturating_sub(1) {
                    self.restore_browser.selected_index += 1;
                }
            }
            // Restore on Enter
            KeyCode::Enter => {
                if !self.restore_browser.commits.is_empty() {
                    let commit_id = self.restore_browser.commits[self.restore_browser.selected_index].id.clone();
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
        // TODO: Call oxenvcs-cli status to get current repo state
        // For now, just update the log
        self.log(LogLevel::Info, "Repository status refreshed");
    }

    /// Load commit history
    fn load_commits(&mut self) {
        // TODO: Call oxenvcs-cli log to get commits
        // For now, add placeholder data
        self.restore_browser.commits = vec![
            CommitEntry {
                id: "abc123def456".to_string(),
                short_id: "abc123d".to_string(),
                message: "Added vocals - BPM 120".to_string(),
                timestamp: "2 hours ago".to_string(),
            },
            CommitEntry {
                id: "def456ghi789".to_string(),
                short_id: "def456g".to_string(),
                message: "Drum tracking complete".to_string(),
                timestamp: "5 hours ago".to_string(),
            },
            CommitEntry {
                id: "ghi789jkl012".to_string(),
                short_id: "ghi789j".to_string(),
                message: "Initial project setup".to_string(),
                timestamp: "1 day ago".to_string(),
            },
        ];
        self.log(LogLevel::Info, format!("Loaded {} commits", self.restore_browser.commits.len()));
    }

    /// Execute commit with current dialog values
    fn execute_commit(&mut self) {
        // TODO: Call oxenvcs-cli commit with parameters
        let msg = format!(
            "Creating commit: \"{}\" (BPM: {}, SR: {}, Key: {}, Tags: {})",
            self.commit_dialog.message,
            if self.commit_dialog.bpm.is_empty() { "none" } else { &self.commit_dialog.bpm },
            if self.commit_dialog.sample_rate.is_empty() { "none" } else { &self.commit_dialog.sample_rate },
            if self.commit_dialog.key.is_empty() { "none" } else { &self.commit_dialog.key },
            if self.commit_dialog.tags.is_empty() { "none" } else { &self.commit_dialog.tags }
        );
        self.log(LogLevel::Success, msg);
    }

    /// Execute restore to specified commit
    fn execute_restore(&mut self, commit_id: &str) {
        // TODO: Call oxenvcs-cli restore
        self.log(LogLevel::Success, format!("Restoring to commit: {}", commit_id));
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
            Span::raw("OxVCS Console"),
            Span::raw(" - "),
            Span::styled(
                self.project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown"),
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
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
        status_lines.push(Line::from(vec![
            Span::raw("Daemon: "),
            daemon_indicator,
        ]));

        status_lines.push(Line::from(""));

        // Repository status
        if let Some(ref repo) = self.repo_status {
            status_lines.push(Line::from(Span::styled(
                "Repository:",
                Style::default().add_modifier(Modifier::BOLD),
            )));
            status_lines.push(Line::from(vec![
                Span::raw("  Staged: "),
                Span::styled(
                    repo.staged.to_string(),
                    Style::default().fg(Color::Green),
                ),
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
                Span::styled(
                    repo.untracked.to_string(),
                    Style::default().fg(Color::Cyan),
                ),
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
                Span::styled("r", Style::default().fg(Color::Cyan)),
                Span::raw(":Refresh  "),
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
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
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
                "OxVCS Console - Keyboard Shortcuts",
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from("Normal Mode:"),
            Line::from("  q         - Quit console"),
            Line::from("  i         - Open commit dialog"),
            Line::from("  l         - Open restore browser (log)"),
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
}
