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

            // TODO: Poll daemon for updates (placeholder)
            // self.poll_daemon_updates().await?;

            if self.should_quit {
                break;
            }
        }
        Ok(())
    }

    /// Handle keyboard input
    fn handle_key_event(&mut self, code: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match (code, modifiers) {
            // Quit on 'q' or Ctrl+C
            (KeyCode::Char('q'), _) | (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }
            // Refresh status on 'r'
            (KeyCode::Char('r'), _) => {
                self.log(LogLevel::Info, "Refreshing status...");
                // TODO: Trigger status refresh
            }
            // Clear log on 'c'
            (KeyCode::Char('c'), _) => {
                self.activity_log.clear();
                self.log(LogLevel::Info, "Log cleared");
            }
            // Show help on '?'
            (KeyCode::Char('?'), _) | (KeyCode::Char('h'), _) => {
                self.show_help();
            }
            _ => {}
        }
        Ok(())
    }

    /// Add help message to log
    fn show_help(&mut self) {
        self.log(LogLevel::Info, "Keyboard shortcuts:");
        self.log(LogLevel::Info, "  q       - Quit console");
        self.log(LogLevel::Info, "  r       - Refresh status");
        self.log(LogLevel::Info, "  c       - Clear activity log");
        self.log(LogLevel::Info, "  ? or h  - Show this help");
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
        let shortcuts = vec![
            Span::styled("q", Style::default().fg(Color::Cyan)),
            Span::raw(":Quit  "),
            Span::styled("r", Style::default().fg(Color::Cyan)),
            Span::raw(":Refresh  "),
            Span::styled("c", Style::default().fg(Color::Cyan)),
            Span::raw(":Clear  "),
            Span::styled("?", Style::default().fg(Color::Cyan)),
            Span::raw(":Help"),
        ];

        let footer = Paragraph::new(Line::from(shortcuts))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::White)),
            )
            .alignment(Alignment::Center);

        f.render_widget(footer, area);
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
