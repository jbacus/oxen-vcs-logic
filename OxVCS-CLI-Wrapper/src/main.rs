use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use oxenvcs_cli::{lock_integration, logger, progress, success, vlog, warn, CommitMetadata, OxenRepository};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "oxenvcs-cli")]
#[command(version)]
#[command(about = "Oxen.ai CLI wrapper for Logic Pro version control")]
#[command(long_about = "Oxen.ai CLI wrapper for Logic Pro version control

This tool provides Git-like version control specifically designed for Logic Pro
projects (.logicx). It integrates with Oxen.ai to efficiently track and manage
your music production projects with support for audio-specific metadata.

FEATURES:
  • Automatic detection and setup for Logic Pro projects
  • Audio metadata tracking (BPM, sample rate, key signature)
  • Draft branch workflow for auto-commits
  • Efficient handling of large audio files
  • Ignore patterns for cache and temporary files

BASIC WORKFLOW:
  1. Initialize: oxenvcs-cli init --logic .
  2. Make changes in Logic Pro
  3. Stage: oxenvcs-cli add --all
  4. Commit: oxenvcs-cli commit -m \"Added drum track\" --bpm 120
  5. View history: oxenvcs-cli log

For more information, visit: https://github.com/your-repo")]
struct Cli {
    /// Enable verbose debug output
    #[arg(short, long, global = true)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum LockCommands {
    /// Acquire exclusive lock for editing
    #[command(long_about = "Acquire exclusive lock for editing

USAGE:
    oxenvcs-cli lock acquire [--timeout <HOURS>]

DESCRIPTION:
    Acquires an exclusive lock on the project, preventing other users from
    making changes. This is essential for team workflows to avoid merge conflicts
    with binary Logic Pro files.

    The lock includes:
      • Lock holder identification (username@hostname)
      • Timeout (default: 4 hours, prevents abandoned locks)
      • Automatic release when lock holder commits

OPTIONS:
    --timeout <HOURS>    Lock expiration time in hours (default: 4)

EXAMPLES:
    # Acquire lock with default 4-hour timeout
    oxenvcs-cli lock acquire

    # Acquire lock with 8-hour timeout
    oxenvcs-cli lock acquire --timeout 8")]
    Acquire {
        #[arg(long, default_value = "4", help = "Lock timeout in hours")]
        timeout: u64,
    },

    /// Release the lock you currently hold
    #[command(long_about = "Release the lock you currently hold

USAGE:
    oxenvcs-cli lock release

DESCRIPTION:
    Releases the exclusive lock you currently hold on the project, allowing
    other team members to acquire it and make changes.

    You should release the lock when:
      • You're done editing for the session
      • You've committed your changes
      • You need to switch to a different task

EXAMPLES:
    # Release your lock
    oxenvcs-cli lock release")]
    Release,

    /// Show current lock status
    #[command(long_about = "Show current lock status

USAGE:
    oxenvcs-cli lock status

DESCRIPTION:
    Displays information about the current project lock:
      • Whether the project is locked
      • Who holds the lock
      • When the lock was acquired
      • When the lock expires
      • Time remaining

EXAMPLES:
    # Check lock status
    oxenvcs-cli lock status")]
    Status,

    /// Force break an existing lock (admin only)
    #[command(long_about = "Force break an existing lock (admin only)

USAGE:
    oxenvcs-cli lock break --force

DESCRIPTION:
    Forcibly breaks an existing lock held by another user. This should only
    be used in emergencies when:
      • The lock holder is unavailable
      • The lock has expired but wasn't auto-released
      • You need immediate access to resolve an issue

    WARNING: Breaking someone else's lock may cause them to lose unsaved work!

    Requires --force flag to prevent accidental use.

EXAMPLES:
    # Force break the lock
    oxenvcs-cli lock break --force")]
    Break {
        #[arg(long, help = "Confirm you want to force break the lock")]
        force: bool,
    },
}

#[derive(Subcommand)]
enum DaemonCommands {
    /// Check daemon status
    #[command(long_about = "Check daemon status

USAGE:
    oxenvcs-cli daemon status

DESCRIPTION:
    Displays the current status of the OxVCS background daemon, including:
      • Whether the daemon is running
      • Process ID (if running)
      • Number of monitored projects
      • Uptime information

EXAMPLES:
    # Check daemon status
    oxenvcs-cli daemon status")]
    Status,

    /// Start the daemon service
    #[command(long_about = "Start the daemon service

USAGE:
    oxenvcs-cli daemon start

DESCRIPTION:
    Starts the OxVCS background daemon service using launchctl.
    The daemon provides:
      • Automatic file monitoring for Logic Pro projects
      • Auto-commit on file changes (with debounce)
      • Power management (save before sleep/shutdown)
      • Lock management for team collaboration

    The daemon runs in the background and starts automatically on login.

EXAMPLES:
    # Start the daemon
    oxenvcs-cli daemon start")]
    Start,

    /// Stop the daemon service
    #[command(long_about = "Stop the daemon service

USAGE:
    oxenvcs-cli daemon stop

DESCRIPTION:
    Stops the OxVCS background daemon service.
    This will:
      • Stop file monitoring for all projects
      • Disable auto-commits
      • Stop power management hooks

    Note: Projects remain tracked; monitoring resumes when daemon restarts.

EXAMPLES:
    # Stop the daemon
    oxenvcs-cli daemon stop")]
    Stop,

    /// Restart the daemon service
    #[command(long_about = "Restart the daemon service

USAGE:
    oxenvcs-cli daemon restart

DESCRIPTION:
    Stops and then starts the daemon service.
    Useful after:
      • Updating the daemon binary
      • Changing configuration settings
      • Recovering from errors

EXAMPLES:
    # Restart the daemon
    oxenvcs-cli daemon restart")]
    Restart,

    /// Show daemon logs
    #[command(long_about = "Show daemon logs

USAGE:
    oxenvcs-cli daemon logs [--lines <N>]

DESCRIPTION:
    Displays recent entries from the daemon log file.
    Useful for debugging and monitoring daemon activity.

OPTIONS:
    --lines <N>    Number of recent log lines to show (default: 50)

EXAMPLES:
    # Show last 50 log lines
    oxenvcs-cli daemon logs

    # Show last 100 log lines
    oxenvcs-cli daemon logs --lines 100")]
    Logs {
        #[arg(long, default_value = "50", help = "Number of log lines to show")]
        lines: usize,
    },
}

#[derive(Subcommand)]
enum AuthCommands {
    /// Login to Oxen Hub with API credentials
    #[command(long_about = "Login to Oxen Hub with API credentials

USAGE:
    oxenvcs-cli auth login

DESCRIPTION:
    Authenticate with Oxen Hub by providing your username and API key.
    Credentials are securely stored in the system keychain via oxen config.

    To get your API key:
      1. Visit https://hub.oxen.ai
      2. Sign in or create an account
      3. Go to Settings → API Keys
      4. Copy your API key

    After login, you can push/pull projects to/from Oxen Hub.

EXAMPLES:
    # Interactive login (prompts for credentials)
    oxenvcs-cli auth login")]
    Login,

    /// Logout from Oxen Hub
    #[command(long_about = "Logout from Oxen Hub

USAGE:
    oxenvcs-cli auth logout

DESCRIPTION:
    Remove stored Oxen Hub credentials from the system.
    After logout, push/pull operations will fail until you login again.

EXAMPLES:
    # Logout
    oxenvcs-cli auth logout")]
    Logout,

    /// Show current authentication status
    #[command(long_about = "Show current authentication status

USAGE:
    oxenvcs-cli auth status

DESCRIPTION:
    Display information about the currently authenticated user:
      • Username
      • Oxen Hub URL
      • Authentication status

EXAMPLES:
    # Check auth status
    oxenvcs-cli auth status")]
    Status,

    /// Test authentication with Oxen Hub
    #[command(long_about = "Test authentication with Oxen Hub

USAGE:
    oxenvcs-cli auth test

DESCRIPTION:
    Verify that your stored credentials are valid by testing
    connection to Oxen Hub. This is useful for troubleshooting
    authentication issues.

EXAMPLES:
    # Test authentication
    oxenvcs-cli auth test")]
    Test,
}

#[derive(Subcommand)]
enum HooksCommands {
    /// Initialize hooks directory
    Init,

    /// List all installed hooks
    List,

    /// List available built-in hooks
    Builtins,

    /// Install a built-in hook
    Install {
        #[arg(value_name = "HOOK_NAME", help = "Name of built-in hook to install")]
        name: String,

        #[arg(long, value_name = "TYPE", default_value = "pre-commit", help = "Hook type (pre-commit or post-commit)")]
        hook_type: String,
    },

    /// Remove an installed hook
    Remove {
        #[arg(value_name = "HOOK_NAME", help = "Name of hook to remove")]
        name: String,

        #[arg(long, value_name = "TYPE", default_value = "pre-commit", help = "Hook type (pre-commit or post-commit)")]
        hook_type: String,
    },
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new Oxen repository for a Logic Pro project
    #[command(long_about = "Initialize a new Oxen repository for a Logic Pro project

USAGE:
    oxenvcs-cli init --logic <PATH>
    oxenvcs-cli init <PATH>

DESCRIPTION:
    Creates a new Oxen repository at the specified path. When used with --logic,
    it will:
      • Detect and validate the Logic Pro project structure
      • Create an .oxenignore file with Logic-specific patterns
      • Set up a draft branch workflow for auto-commits
      • Initialize tracking for projectData, Alternatives, and Resources

    The PATH can be:
      • Current directory: .
      • Relative path: Demo_Project.logicx
      • Absolute path: /Users/you/Music/Logic/MyProject.logicx

EXAMPLES:
    # Initialize from inside a Logic project directory
    cd Demo_Project.logicx
    oxenvcs-cli init --logic .

    # Initialize from parent directory
    oxenvcs-cli init --logic Demo_Project.logicx

    # Initialize a generic Oxen repository (not Logic-specific)
    oxenvcs-cli init /path/to/folder")]
    Init {
        #[arg(value_name = "PATH", help = "Path to the project directory")]
        path: PathBuf,

        #[arg(
            long,
            help = "Initialize for Logic Pro project (auto-detect and configure)"
        )]
        logic: bool,
    },

    /// Stage changes to be committed
    #[command(long_about = "Stage changes to be committed

USAGE:
    oxenvcs-cli add --all
    oxenvcs-cli add <PATHS>...

DESCRIPTION:
    Stages changes in the working directory for the next commit. Similar to 'git add'.
    Use --all to stage all changes, or specify individual files/directories.

EXAMPLES:
    # Stage all changes in the repository
    oxenvcs-cli add --all

    # Stage specific files
    oxenvcs-cli add projectData Alternatives/Take001

    # Stage a directory
    oxenvcs-cli add Resources/")]
    Add {
        #[arg(value_name = "PATHS", help = "Files or directories to stage")]
        paths: Vec<PathBuf>,

        #[arg(long, short, help = "Stage all changes in the repository")]
        all: bool,
    },

    /// Create a commit with optional audio metadata
    #[command(long_about = "Create a commit with optional audio metadata

USAGE:
    oxenvcs-cli commit -m <MESSAGE> [OPTIONS]

DESCRIPTION:
    Creates a new commit with the currently staged changes. You can attach
    audio production metadata to help track the evolution of your project.

    Metadata includes:
      • BPM (tempo)
      • Sample rate (Hz)
      • Key signature
      • Tags for categorization

EXAMPLES:
    # Simple commit
    oxenvcs-cli commit -m \"Initial project setup\"

    # Commit with audio metadata
    oxenvcs-cli commit -m \"Added bass line\" --bpm 120 --key \"A Minor\"

    # Commit with multiple tags
    oxenvcs-cli commit -m \"Final mix\" --sample-rate 44100 --tags \"mixing,mastered\"

    # Full metadata commit
    oxenvcs-cli commit -m \"Verse 2 complete\" \\
        --bpm 128 \\
        --sample-rate 48000 \\
        --key \"C Major\" \\
        --tags \"verse,arrangement\"")]
    Commit {
        #[arg(short, long, help = "Commit message describing the changes")]
        message: String,

        #[arg(long, help = "Beats per minute (tempo) of the project")]
        bpm: Option<f32>,

        #[arg(long, help = "Sample rate in Hz (e.g., 44100, 48000, 96000)")]
        sample_rate: Option<u32>,

        #[arg(long, help = "Key signature (e.g., 'C Major', 'A Minor', 'F# Minor')")]
        key: Option<String>,

        #[arg(
            long,
            help = "Tags for categorization (comma-separated, e.g., 'mixing,draft')"
        )]
        tags: Option<String>,
    },

    /// Show commit history
    #[command(long_about = "Show commit history

USAGE:
    oxenvcs-cli log [OPTIONS]

DESCRIPTION:
    Displays the commit history for the repository, showing commit IDs, authors,
    timestamps, and messages. Audio metadata (BPM, key, etc.) is displayed if
    present in the commit message.

    Filter commits by metadata, tags, or date range to find specific versions.

EXAMPLES:
    # Show all commits
    oxenvcs-cli log

    # Show only the last 5 commits
    oxenvcs-cli log --limit 5

    # Show commits with specific BPM
    oxenvcs-cli log --bpm 128

    # Show commits with specific tag
    oxenvcs-cli log --tag mixing

    # Show commits since a date
    oxenvcs-cli log --since \"2025-01-01\"

    # Combine filters
    oxenvcs-cli log --bpm 120 --tag vocals --limit 10")]
    Log {
        #[arg(short, long, help = "Maximum number of commits to display")]
        limit: Option<usize>,

        #[arg(long, help = "Filter by BPM (e.g., 120, 128)")]
        bpm: Option<f32>,

        #[arg(long, help = "Filter by tag (e.g., 'mixing', 'vocals')")]
        tag: Option<String>,

        #[arg(long, help = "Filter by key signature (e.g., 'C Major')")]
        key: Option<String>,

        #[arg(long, help = "Show commits since date (YYYY-MM-DD)")]
        since: Option<String>,
    },

    /// Restore project to a previous commit
    #[command(long_about = "Restore project to a previous commit

USAGE:
    oxenvcs-cli restore <COMMIT_ID>

DESCRIPTION:
    Restores the project to the state at the specified commit. This checks out
    the files from that commit, allowing you to return to a previous version.

    WARNING: Make sure to commit any current changes before restoring, or they
    will be lost.

    You can find commit IDs using the 'log' command.

EXAMPLES:
    # Find commit IDs
    oxenvcs-cli log --limit 5

    # Restore to a specific commit
    oxenvcs-cli restore abc123def

    # Restore to a commit (full hash)
    oxenvcs-cli restore abc123def456789012345678901234567890")]
    Restore {
        #[arg(
            value_name = "COMMIT_ID",
            help = "Commit ID to restore to (from 'log' command)"
        )]
        commit_id: String,
    },

    /// Show repository status
    #[command(long_about = "Show repository status

USAGE:
    oxenvcs-cli status

DESCRIPTION:
    Displays the current state of the working directory and staging area:
      • Staged files (ready to commit)
      • Modified files (changed but not staged)
      • Untracked files (new files not yet added)

    This is similar to 'git status' and helps you see what changes are pending.

EXAMPLES:
    # Check repository status
    oxenvcs-cli status")]
    Status,

    /// Show detailed information about a commit
    #[command(long_about = "Show detailed information about a commit

USAGE:
    oxenvcs-cli show <COMMIT_ID>

DESCRIPTION:
    Displays comprehensive information about a specific commit, including:
      • Full commit message
      • Audio metadata (BPM, sample rate, key signature, tags)
      • Author and timestamp
      • Files changed
      • Commit statistics

EXAMPLES:
    # Show details of a recent commit
    oxenvcs-cli show abc123f

    # Show details with full hash
    oxenvcs-cli show abc123def456789012345678901234567890")]
    Show {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to show details for")]
        commit_id: String,
    },

    /// Show changes between commits or working directory
    #[command(long_about = "Show changes between commits or working directory

USAGE:
    oxenvcs-cli diff [COMMIT_ID]

DESCRIPTION:
    Shows file-level changes in the repository:
      • Without arguments: shows changes in working directory vs last commit
      • With commit ID: shows changes between that commit and working directory
      • With two IDs: shows changes between two commits

    Displays:
      • Modified files with size changes
      • Added files
      • Deleted files
      • Total size impact

EXAMPLES:
    # Show uncommitted changes
    oxenvcs-cli diff

    # Show changes since specific commit
    oxenvcs-cli diff abc123f

    # Compare two commits (future enhancement)
    # oxenvcs-cli diff abc123f def456a")]
    Diff {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to compare against (optional)")]
        commit_id: Option<String>,
    },

    /// Compare metadata between two commits
    #[command(long_about = "Compare metadata between two commits

USAGE:
    oxenvcs-cli compare <COMMIT_A> <COMMIT_B>
    oxenvcs-cli compare <COMMIT_A> <COMMIT_B> --format json

DESCRIPTION:
    Performs semantic diff between two commits, showing changes in:
      • Commit message
      • BPM (tempo)
      • Sample rate
      • Key signature
      • Tags

    This helps understand what changed in the project's audio characteristics
    between versions, beyond just file changes.

OPTIONS:
    --format <FORMAT>    Output format: text (default), colored, json, compact
    --plain              Disable colored output

EXAMPLES:
    # Compare two commits with colored output
    oxenvcs-cli compare abc123f def456a

    # Compare with plain text (no colors)
    oxenvcs-cli compare abc123f def456a --plain

    # Compare with JSON output
    oxenvcs-cli compare abc123f def456a --format json

    # Compare with compact one-line summary
    oxenvcs-cli compare abc123f def456a --format compact")]
    Compare {
        #[arg(value_name = "COMMIT_A", help = "First commit ID (older)")]
        commit_a: String,

        #[arg(value_name = "COMMIT_B", help = "Second commit ID (newer)")]
        commit_b: String,

        #[arg(long, value_name = "FORMAT", default_value = "colored", help = "Output format (text, colored, json, compact)")]
        format: String,

        #[arg(long, help = "Disable colored output")]
        plain: bool,
    },

    /// Search commit history with advanced filtering
    #[command(long_about = "Search commit history with advanced filtering

USAGE:
    oxenvcs-cli search <QUERY>
    oxenvcs-cli search bpm:120-140 key:minor tag:mixing

DESCRIPTION:
    Smart search across commit history with metadata-based filtering.
    Supports natural language-style queries with multiple criteria:

    Query Syntax:
      • bpm:120-140      - BPM range
      • bpm:>120         - BPM greater than
      • bpm:<140         - BPM less than
      • sr:48000         - Exact sample rate
      • key:minor        - Key signature (fuzzy match)
      • tag:mixing       - Has tag (single)
      • tag:mix,vocal    - Has any of these tags (OR logic)
      • msg:final        - Message contains text
      • limit:10         - Limit results

    Multiple criteria can be combined (AND logic):
      bpm:120-140 key:minor tag:mixing

OPTIONS:
    --format <FORMAT>    Output format: list (default), compact, json
    --ranked             Sort by relevance score

EXAMPLES:
    # Find all commits between 120-140 BPM
    oxenvcs-cli search \"bpm:120-140\"

    # Find commits in minor keys with mixing tag
    oxenvcs-cli search \"key:minor tag:mixing\"

    # Find high BPM commits (>140)
    oxenvcs-cli search \"bpm:>140\"

    # Find commits with 'final' in message
    oxenvcs-cli search \"msg:final\"

    # Combined search with limit
    oxenvcs-cli search \"bpm:120-140 key:minor tag:vocals limit:5\"

    # Get compact one-line summaries
    oxenvcs-cli search \"bpm:>128\" --format compact

    # Ranked by relevance
    oxenvcs-cli search \"bpm:120-140 tag:mixing\" --ranked")]
    Search {
        #[arg(value_name = "QUERY", help = "Search query string")]
        query: String,

        #[arg(long, value_name = "FORMAT", default_value = "list", help = "Output format (list, compact, json)")]
        format: String,

        #[arg(long, help = "Sort results by relevance score")]
        ranked: bool,
    },

    /// Manage project locks for team collaboration
    #[command(subcommand)]
    Lock(LockCommands),

    /// Authenticate with Oxen Hub for remote collaboration
    #[command(subcommand)]
    Auth(AuthCommands),

    /// Compare metadata between two Logic Pro project versions
    #[command(name = "metadata-diff")]
    #[command(long_about = "Compare metadata between two Logic Pro project versions

USAGE:
    oxenvcs-cli metadata-diff <PROJECT_A> <PROJECT_B>
    oxenvcs-cli metadata-diff <PROJECT_A> <PROJECT_B> --output json

DESCRIPTION:
    Analyzes and compares the metadata of two Logic Pro project versions,
    generating a detailed report of what changed. Detects:
      • Global changes (tempo, sample rate, key signature)
      • Track additions, removals, and modifications
      • EQ changes (frequency, gain, Q factor)
      • Compressor changes (threshold, ratio, attack, release)
      • Volume and pan adjustments
      • Automation curve changes
      • Plugin parameter changes

    The output is a human-readable report showing exactly what changed
    between versions, making it easy to understand project evolution.

EXAMPLES:
    # Compare two project versions
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx

    # Output as JSON for programmatic use
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --output json

    # Compare with colored output
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --color

    # Verbose mode with technical details
    oxenvcs-cli metadata-diff Project_v1.logicx Project_v2.logicx --verbose")]
    MetadataDiff {
        #[arg(value_name = "PROJECT_A", help = "First Logic Pro project (.logicx)")]
        project_a: PathBuf,

        #[arg(value_name = "PROJECT_B", help = "Second Logic Pro project (.logicx)")]
        project_b: PathBuf,

        #[arg(
            long,
            value_name = "FORMAT",
            default_value = "text",
            help = "Output format (text or json)"
        )]
        output: String,

        #[arg(long, help = "Use colored output (default: auto-detect)")]
        color: bool,

        #[arg(long, short, help = "Include technical details in output")]
        verbose: bool,
    },

    /// Control the background daemon service
    #[command(subcommand)]
    Daemon(DaemonCommands),

    /// Manage workflow automation hooks
    #[command(long_about = "Manage workflow automation hooks

USAGE:
    oxenvcs-cli hooks init
    oxenvcs-cli hooks list
    oxenvcs-cli hooks install <HOOK_NAME>

DESCRIPTION:
    Workflow automation hooks allow you to run custom scripts before and after
    commits. Use hooks for:

    Pre-commit hooks (run before creating commit):
      • Validate metadata completeness
      • Check file sizes
      • Run linting or formatting
      • Verify project structure

    Post-commit hooks (run after successful commit):
      • Send notifications
      • Create backups
      • Trigger CI/CD pipelines
      • Update tracking systems

EXAMPLES:
    # Initialize hooks directory
    oxenvcs-cli hooks init

    # List all installed hooks
    oxenvcs-cli hooks list

    # List available built-in hooks
    oxenvcs-cli hooks builtins

    # Install a built-in hook
    oxenvcs-cli hooks install validate-metadata

    # Install a post-commit hook
    oxenvcs-cli hooks install backup --hook-type post-commit

    # Remove a hook
    oxenvcs-cli hooks remove validate-metadata")]
    #[command(subcommand)]
    Hooks(HooksCommands),

    /// Launch interactive console for real-time monitoring
    #[command(long_about = "Launch interactive console for real-time monitoring

USAGE:
    oxenvcs-cli console [PATH]

DESCRIPTION:
    Launches a full-screen interactive TUI (Terminal User Interface) that provides
    real-time monitoring and control of your Logic Pro project version control.

    Features:
      • Live daemon status display
      • Real-time activity log with auto-updates
      • Repository status (staged, modified, untracked files)
      • Keyboard shortcuts for common operations
      • Color-coded output for clarity

    The console runs until you press 'q' to quit.

KEYBOARD SHORTCUTS:
    q       - Quit console
    r       - Refresh status
    c       - Clear activity log
    ? or h  - Show help

EXAMPLES:
    # Launch console in current directory
    oxenvcs-cli console

    # Launch console for specific project
    oxenvcs-cli console ~/Music/MyProject.logicx")]
    Console {
        #[arg(
            value_name = "PATH",
            help = "Path to Logic Pro project (default: current directory)"
        )]
        path: Option<PathBuf>,
    },

    /// Show recent project activity timeline
    #[command(long_about = "Show recent project activity timeline

USAGE:
    oxenvcs-cli activity [--limit <N>]

DESCRIPTION:
    Displays a timeline of recent project activity including commits,
    lock operations, and comments. Helps teams stay synchronized on
    project progress.

    Activity types shown:
      • Commits with metadata (BPM, sample rate, key)
      • Lock acquisitions/releases
      • Comments on commits
      • Branch creations

OPTIONS:
    --limit <N>    Number of recent activities to show (default: 10)

EXAMPLES:
    # Show last 10 activities
    oxenvcs-cli activity

    # Show last 20 activities
    oxenvcs-cli activity --limit 20")]
    Activity {
        #[arg(long, default_value = "10", help = "Number of activities to show")]
        limit: usize,
    },

    /// Show team members and their contributions
    #[command(long_about = "Show team members and their contributions

USAGE:
    oxenvcs-cli team

DESCRIPTION:
    Discovers team members from commit history and displays their
    contribution statistics. Helps identify who's working on the project.

    Information shown:
      • Member name (username@hostname)
      • Number of commits
      • Last activity timestamp
      • Contribution percentage

EXAMPLES:
    # Show team members
    oxenvcs-cli team")]
    Team,

    /// Manage offline operation queue
    #[command(subcommand)]
    Queue(QueueCommands),

    /// Manage comments on commits
    #[command(subcommand)]
    Comment(CommentCommands),

    /// Generate shell completion scripts
    #[command(long_about = "Generate shell completion scripts

USAGE:
    oxenvcs-cli completions <SHELL>

DESCRIPTION:
    Generates shell completion scripts for the specified shell.
    This enables tab completion for commands, subcommands, and flags.

SUPPORTED SHELLS:
    • bash
    • zsh
    • fish
    • powershell

INSTALLATION:
    # Bash (Linux)
    oxenvcs-cli completions bash > /etc/bash_completion.d/oxenvcs-cli

    # Bash (macOS with Homebrew)
    oxenvcs-cli completions bash > /usr/local/etc/bash_completion.d/oxenvcs-cli

    # Zsh
    oxenvcs-cli completions zsh > /usr/local/share/zsh/site-functions/_oxenvcs-cli

    # Fish
    oxenvcs-cli completions fish > ~/.config/fish/completions/oxenvcs-cli.fish

    # PowerShell
    oxenvcs-cli completions powershell > oxenvcs-cli.ps1

EXAMPLES:
    # Generate bash completions
    oxenvcs-cli completions bash

    # Install for current user (bash)
    oxenvcs-cli completions bash > ~/.local/share/bash-completion/completions/oxenvcs-cli")]
    Completions {
        #[arg(value_name = "SHELL", help = "Shell to generate completions for (bash, zsh, fish, powershell)")]
        shell: String,
    },
}

#[derive(Subcommand)]
enum CommentCommands {
    /// Add a comment to a commit
    #[command(long_about = "Add a comment to a commit

USAGE:
    oxenvcs-cli comment add <COMMIT_ID> <TEXT>

DESCRIPTION:
    Adds a comment to a specific commit. Comments are stored in the
    repository and synced across team members.

    Use cases:
      • Code review feedback
      • Mix notes for specific versions
      • Questions about changes
      • Track decisions

EXAMPLES:
    # Add comment to latest commit
    oxenvcs-cli comment add HEAD \"Vocals need more reverb\"

    # Add comment to specific commit
    oxenvcs-cli comment add abc123 \"Great mix on this version!\"")]
    Add {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID or HEAD")]
        commit_id: String,

        #[arg(value_name = "TEXT", help = "Comment text")]
        text: String,
    },

    /// List comments on a commit
    #[command(long_about = "List comments on a commit

USAGE:
    oxenvcs-cli comment list [COMMIT_ID]

DESCRIPTION:
    Shows all comments on a specific commit. If no commit ID is provided,
    shows comments on the latest commit (HEAD).

EXAMPLES:
    # Show comments on latest commit
    oxenvcs-cli comment list

    # Show comments on specific commit
    oxenvcs-cli comment list abc123")]
    List {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID or HEAD (optional)")]
        commit_id: Option<String>,
    },
}

#[derive(Subcommand)]
enum QueueCommands {
    /// Show pending operations in the queue
    #[command(long_about = "Show pending operations in the queue

USAGE:
    oxenvcs-cli queue status

DESCRIPTION:
    Displays all pending operations that are queued due to network unavailability.
    Shows operation type, priority, queued time, and number of retry attempts.

EXAMPLES:
    # Show all pending operations
    oxenvcs-cli queue status")]
    Status,

    /// Manually sync all pending operations
    #[command(long_about = "Manually sync all pending operations

USAGE:
    oxenvcs-cli queue sync

DESCRIPTION:
    Attempts to execute all pending operations in the queue. Operations are
    executed in priority order (highest first), then by age (oldest first).

    Only executes if network connectivity is available. Failed operations
    remain in the queue for retry.

EXAMPLES:
    # Sync all pending operations
    oxenvcs-cli queue sync")]
    Sync,

    /// Clear completed operations from the queue
    #[command(long_about = "Clear completed operations from the queue

USAGE:
    oxenvcs-cli queue clear [--all]

DESCRIPTION:
    Removes completed operations from the queue to free up disk space.
    Use --all to remove ALL operations (including pending).

EXAMPLES:
    # Clear only completed operations
    oxenvcs-cli queue clear

    # Clear all operations (pending and completed)
    oxenvcs-cli queue clear --all")]
    Clear {
        #[arg(long, help = "Clear all operations including pending")]
        all: bool,
    },

    /// Remove a specific operation from the queue
    #[command(long_about = "Remove a specific operation from the queue

USAGE:
    oxenvcs-cli queue remove <ENTRY_ID>

DESCRIPTION:
    Removes a specific operation from the queue by its entry ID.
    Use 'queue status' to see entry IDs.

EXAMPLES:
    # Remove a specific operation
    oxenvcs-cli queue remove 01234567-89ab-cdef-0123-456789abcdef")]
    Remove {
        #[arg(value_name = "ENTRY_ID", help = "Queue entry ID to remove")]
        entry_id: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Enable verbose logging if requested
    logger::set_verbose(cli.verbose);

    match cli.command {
        Commands::Init { path, logic } => {
            vlog!("Starting initialization for path: {}", path.display());
            vlog!("Logic Pro mode: {}", logic);

            if logic {
                // Multi-step initialization with progress feedback
                let pb = progress::spinner("Validating Logic Pro project structure...");

                vlog!("Initializing Logic Pro project repository...");
                let _repo = OxenRepository::init_for_logic_project(&path).await?;

                progress::finish_success(&pb, "Logic Pro project repository initialized");
                println!();
                progress::success(&format!("Repository created at: {}", path.display()));
                progress::success("Initial commit created on main branch");
                progress::success("Draft branch created and checked out");
                println!();
                progress::info("You're all set! Start working in Logic Pro:");
                println!("  • Changes will be automatically tracked on the draft branch");
                println!("  • Create milestone commits: oxenvcs-cli commit -m \"Your message\" --bpm 120");
                println!("  • View history: oxenvcs-cli log");
                println!("  • Restore to any commit: oxenvcs-cli restore <commit-id>");
            } else {
                let pb = progress::spinner(&format!("Initializing Oxen repository at {}...", path.display()));

                vlog!("Initializing generic Oxen repository...");
                let _repo = OxenRepository::init(&path).await?;

                progress::finish_success(
                    &pb,
                    &format!("Oxen repository initialized at: {}", path.display())
                );
            }
            Ok(())
        }

        Commands::Add { paths, all } => {
            let repo = OxenRepository::new(".");

            if all {
                let pb = progress::spinner("Staging all changes...");
                repo.stage_all().await?;
                progress::finish_success(&pb, "All changes staged");
                println!();
                progress::info("Next step: oxenvcs-cli commit -m \"Your message\"");
            } else {
                if paths.is_empty() {
                    progress::error("Please provide paths to stage or use --all");
                    std::process::exit(1);
                }
                let pb = progress::spinner(&format!("Staging {} file(s)...", paths.len()));
                repo.stage_changes(paths).await?;
                progress::finish_success(&pb, "Files staged");
            }

            Ok(())
        }

        Commands::Commit {
            message,
            bpm,
            sample_rate,
            key,
            tags,
        } => {
            let pb = progress::spinner("Preparing commit...");
            let repo = OxenRepository::new(".");

            let mut metadata = CommitMetadata::new(message.clone());

            if let Some(bpm) = bpm {
                metadata = metadata.with_bpm(bpm);
            }

            if let Some(sr) = sample_rate {
                metadata = metadata.with_sample_rate(sr);
            }

            if let Some(ref key_val) = key {
                metadata = metadata.with_key_signature(key_val.clone());
            }

            if let Some(ref tags_str) = tags {
                for tag in tags_str.split(',') {
                    metadata = metadata.with_tag(tag.trim());
                }
            }

            pb.set_message("Creating commit...");
            let commit_id = repo.create_commit(metadata).await?;

            progress::finish_success(&pb, &format!("Commit created: {}", commit_id));

            // Show commit details
            println!();
            progress::info("Commit Details:");
            println!("  Message: {}", message);
            if let Some(ref bpm_val) = bpm {
                println!("  BPM: {}", bpm_val);
            }
            if let Some(ref sr_val) = sample_rate {
                println!("  Sample Rate: {} Hz", sr_val);
            }
            if let Some(ref key_val) = key {
                println!("  Key: {}", key_val);
            }
            if let Some(ref tags_val) = tags {
                println!("  Tags: {}", tags_val);
            }

            Ok(())
        }

        Commands::Log { limit, bpm, tag, key, since } => {
            let repo = OxenRepository::new(".");

            let mut commits = repo.get_history(None).await?;

            if commits.is_empty() {
                println!();
                progress::info("No commits yet");
                println!();
                progress::info("Create your first commit:");
                println!("  oxenvcs-cli add --all");
                println!("  oxenvcs-cli commit -m \"Initial commit\"");
                return Ok(());
            }

            // Apply filters
            let total_before_filter = commits.len();
            let mut filters_applied = vec![];

            if let Some(bpm_filter) = bpm {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("BPM:") && line.contains(&bpm_filter.to_string())
                    })
                });
                filters_applied.push(format!("BPM = {}", bpm_filter));
            }

            if let Some(tag_filter) = &tag {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("Tags:") && line.to_lowercase().contains(&tag_filter.to_lowercase())
                    })
                });
                filters_applied.push(format!("tag = {}", tag_filter));
            }

            if let Some(key_filter) = &key {
                commits.retain(|c| {
                    c.message.lines().any(|line| {
                        line.contains("Key:") && line.to_lowercase().contains(&key_filter.to_lowercase())
                    })
                });
                filters_applied.push(format!("key = {}", key_filter));
            }

            if let Some(_since_filter) = &since {
                // TODO: Implement date filtering when commit timestamps are available
                progress::warning("Date filtering not yet implemented (commit timestamps needed)");
            }

            // Apply limit after filtering
            if let Some(lim) = limit {
                commits.truncate(lim);
            }

            // Show results
            println!();
            println!("┌─ Commit History ────────────────────────────────────────┐");
            if !filters_applied.is_empty() {
                println!("│ Filters: {}                                              │", filters_applied.join(", "));
                println!("│ Found {} of {} commit(s)                                 │", commits.len(), total_before_filter);
            } else if let Some(_lim) = limit {
                println!("│ Showing last {} of {} commit(s)                          │", commits.len(), total_before_filter);
            } else {
                println!("│ Showing all {} commit(s)                                 │", commits.len());
            }
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if commits.is_empty() {
                progress::info("No commits match the specified filters");
                return Ok(());
            }

            for (idx, commit) in commits.iter().enumerate() {
                let short_id = &commit.id[..7.min(commit.id.len())];

                // Visual timeline with bullets
                println!("{} {} - {}", "●".cyan(), short_id.bright_yellow(), "now".bright_black());

                // Commit message (indented)
                let lines: Vec<&str> = commit.message.lines().collect();
                if let Some(first_line) = lines.first() {
                    println!("  │ {}", first_line.bright_white());
                }

                // Additional metadata if present in message
                for line in lines.iter().skip(1) {
                    if !line.trim().is_empty() {
                        if line.contains("BPM:") || line.contains("Sample Rate:") || line.contains("Key:") || line.contains("Tags:") {
                            println!("  │ {}", line.trim().bright_black());
                        } else {
                            println!("  │ {}", line.trim());
                        }
                    }
                }

                // Add spacing between commits (except last one)
                if idx < commits.len() - 1 {
                    println!("  │");
                }
            }

            println!();
            progress::info(&format!("Showing {} commit(s)", commits.len()));

            Ok(())
        }

        Commands::Restore { commit_id } => {
            let pb = progress::spinner(&format!("Restoring to commit {}...", &commit_id[..7.min(commit_id.len())]));
            let repo = OxenRepository::new(".");

            pb.set_message("Checking out files...");
            repo.restore(&commit_id).await?;

            progress::finish_success(&pb, &format!("Restored to commit {}", &commit_id[..7.min(commit_id.len())]));
            println!();
            progress::warning("Your working directory has been updated to match this commit");
            progress::info("To create a new commit from here, use:");
            println!("  oxenvcs-cli add --all");
            println!("  oxenvcs-cli commit -m \"Your message\"");

            Ok(())
        }

        Commands::Status => {
            let repo = OxenRepository::new(".");

            let status = repo.status().await?;

            // Header
            println!();
            println!("┌─ Repository Status ─────────────────────────────────────┐");
            println!("│                                                          │");

            let total_changes = status.staged.len() + status.modified.len() + status.untracked.len();

            if total_changes == 0 {
                println!("│  ✓ Working directory clean                              │");
            } else {
                println!("│  Changes: {} staged, {} modified, {} untracked",
                    status.staged.len().to_string().green(),
                    status.modified.len().to_string().yellow(),
                    status.untracked.len().to_string().cyan(),
                );
            }

            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if !status.staged.is_empty() {
                println!("{} Staged files ({}):", "●".green(), status.staged.len());
                for path in &status.staged {
                    println!("  {} {}", "+".green(), path.display());
                }
                println!();
            }

            if !status.modified.is_empty() {
                println!("{} Modified files ({}):", "◆".yellow(), status.modified.len());
                for path in &status.modified {
                    println!("  {} {}", "M".yellow(), path.display());
                }
                println!();
            }

            if !status.untracked.is_empty() {
                println!("{} Untracked files ({}):", "?".cyan(), status.untracked.len());
                for path in &status.untracked {
                    println!("  {} {}", "?".cyan(), path.display());
                }
                println!();
            }

            // Next steps suggestion
            if total_changes > 0 {
                if status.staged.is_empty() && (!status.modified.is_empty() || !status.untracked.is_empty()) {
                    progress::info("Next step: oxenvcs-cli add --all");
                } else if !status.staged.is_empty() {
                    progress::info("Next step: oxenvcs-cli commit -m \"Your message\"");
                }
            }

            Ok(())
        }

        Commands::Show { commit_id } => {
            let repo = OxenRepository::new(".");

            // Get all commits to find the one we want
            let commits = repo.get_history(None).await?;

            let commit = commits.iter().find(|c| {
                c.id.starts_with(&commit_id) || c.id == commit_id
            });

            if let Some(commit) = commit {
                println!();
                println!("┌─ Commit Details ────────────────────────────────────────┐");
                println!("│                                                          │");
                println!("│  Commit: {}                                      │", commit.id.bright_yellow());
                println!("│                                                          │");
                println!("└──────────────────────────────────────────────────────────┘");
                println!();

                // Parse commit message and metadata
                let lines: Vec<&str> = commit.message.lines().collect();

                // First line is the commit message
                if let Some(first_line) = lines.first() {
                    println!("{}", "Message:".bright_white().bold());
                    println!("  {}", first_line);
                    println!();
                }

                // Extract metadata
                let mut metadata_found = false;
                for line in lines.iter().skip(1) {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        if trimmed.starts_with("BPM:") || trimmed.starts_with("Sample Rate:") ||
                           trimmed.starts_with("Key:") || trimmed.starts_with("Tags:") {
                            if !metadata_found {
                                println!("{}", "Metadata:".bright_white().bold());
                                metadata_found = true;
                            }
                            println!("  {}", trimmed.bright_black());
                        } else {
                            // Additional commit message content
                            println!("  {}", trimmed);
                        }
                    }
                }

                println!();
                progress::info(&format!("Use 'oxenvcs-cli restore {}' to restore to this commit", &commit.id[..7.min(commit.id.len())]));
            } else {
                progress::error(&format!("Commit not found: {}", commit_id));
                std::process::exit(1);
            }

            Ok(())
        }

        Commands::Diff { commit_id } => {
            let repo = OxenRepository::new(".");

            println!();
            if let Some(cid) = &commit_id {
                println!("┌─ Changes Since Commit {} ─────────────────────┐", &cid[..7.min(cid.len())].bright_yellow());
            } else {
                println!("┌─ Uncommitted Changes ───────────────────────────────────┐");
            }
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            // Get current status
            let status = repo.status().await?;

            let total_files = status.staged.len() + status.modified.len() + status.untracked.len();

            if total_files == 0 {
                progress::info("No changes in working directory");
                return Ok(());
            }

            // Modified files
            if !status.modified.is_empty() {
                println!("{} Modified files ({}):", "◆".yellow(), status.modified.len());
                for path in &status.modified {
                    // Try to get file size info
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let size = metadata.len();
                        println!("  {} {} {}", "~".yellow(), path.display(), format!("({} bytes)", size).bright_black());
                    } else {
                        println!("  {} {}", "~".yellow(), path.display());
                    }
                }
                println!();
            }

            // Untracked (new) files
            if !status.untracked.is_empty() {
                println!("{} Added files ({}):", "◆".green(), status.untracked.len());
                for path in &status.untracked {
                    if let Ok(metadata) = std::fs::metadata(path) {
                        let size = metadata.len();
                        let size_mb = size as f64 / 1_048_576.0;
                        if size_mb >= 1.0 {
                            println!("  {} {} {}", "+".green(), path.display(), format!("({:.1} MB)", size_mb).bright_black());
                        } else {
                            println!("  {} {} {}", "+".green(), path.display(), format!("({} bytes)", size).bright_black());
                        }
                    } else {
                        println!("  {} {}", "+".green(), path.display());
                    }
                }
                println!();
            }

            // Summary
            let total_modified = status.modified.len();
            let total_added = status.untracked.len();

            progress::info(&format!(
                "Total changes: {} modified, {} added",
                total_modified.to_string().yellow(),
                total_added.to_string().green()
            ));

            if !status.staged.is_empty() {
                println!();
                progress::info("Some changes are already staged. Use 'oxenvcs-cli status' for details");
            }

            Ok(())
        }

        Commands::Compare {
            commit_a,
            commit_b,
            format,
            plain,
        } => {
            use oxenvcs_cli::CommitMetadata;

            let repo = OxenRepository::new(".");

            vlog!("Fetching commit A: {}", commit_a);
            vlog!("Fetching commit B: {}", commit_b);

            // Get commit history to find the two commits
            let commits = repo.get_history(None).await?;

            // Find commit A
            let commit_a_info = commits
                .iter()
                .find(|c| c.id.starts_with(&commit_a))
                .ok_or_else(|| anyhow::anyhow!("Commit not found: {}", commit_a))?;

            // Find commit B
            let commit_b_info = commits
                .iter()
                .find(|c| c.id.starts_with(&commit_b))
                .ok_or_else(|| anyhow::anyhow!("Commit not found: {}", commit_b))?;

            vlog!("Found commit A: {}", commit_a_info.id);
            vlog!("Found commit B: {}", commit_b_info.id);

            // Parse commit metadata
            let metadata_a = CommitMetadata::parse_commit_message(&commit_a_info.message);
            let metadata_b = CommitMetadata::parse_commit_message(&commit_b_info.message);

            println!();
            println!(
                "┌─ Comparing {} → {} ─────────────┐",
                &commit_a_info.id[..7].bright_cyan(),
                &commit_b_info.id[..7].bright_cyan()
            );
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            // Output based on format
            match format.as_str() {
                "json" => {
                    // Create a simple JSON structure
                    let json_output = serde_json::json!({
                        "commit_a": {
                            "id": &commit_a_info.id,
                            "metadata": &metadata_a
                        },
                        "commit_b": {
                            "id": &commit_b_info.id,
                            "metadata": &metadata_b
                        }
                    });
                    println!("{}", serde_json::to_string_pretty(&json_output)?);
                }
                "compact" => {
                    let summary = metadata_a.compare_compact(&metadata_b);
                    println!("{}", summary);
                }
                "text" => {
                    println!("{}", metadata_a.compare_with_plain(&metadata_b));
                }
                "colored" | _ => {
                    if plain {
                        println!("{}", metadata_a.compare_with_plain(&metadata_b));
                    } else {
                        println!("{}", metadata_a.compare_with(&metadata_b));
                    }
                }
            }

            Ok(())
        }

        Commands::Search {
            query,
            format,
            ranked,
        } => {
            use oxenvcs_cli::search::SearchEngine;

            let repo = OxenRepository::new(".");

            vlog!("Parsing search query: {}", query);

            // Parse the natural language query
            let search_query = SearchEngine::parse_query(&query);

            vlog!("Fetching commit history...");
            let commits = repo.get_history(None).await?;

            vlog!("Executing search...");
            let engine = SearchEngine::new();
            let mut results = engine.search(&commits, &search_query);

            // Sort by relevance if requested
            if ranked {
                results.sort_by(|a, b| {
                    let score_b = engine.relevance_score(b, &search_query);
                    let score_a = engine.relevance_score(a, &search_query);
                    score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
                });
            }

            println!();
            println!(
                "┌─ Search Results ({} matches) ─────────────────────┐",
                results.len().to_string().bright_cyan()
            );
            println!("│ Query: {:<49} │", query.bright_yellow());
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if results.is_empty() {
                progress::warning("No commits match your search criteria");
                println!();
                println!("Try:");
                println!("  • Broadening your BPM range");
                println!("  • Using partial key matches (e.g., 'minor' instead of 'A Minor')");
                println!("  • Checking tag spellings");
                return Ok(());
            }

            // Output based on format
            match format.as_str() {
                "json" => {
                    let json = serde_json::to_string_pretty(&results)?;
                    println!("{}", json);
                }
                "compact" => {
                    for (i, commit) in results.iter().enumerate() {
                        let metadata = CommitMetadata::parse_commit_message(&commit.message);
                        let short_id = if commit.id.len() >= 7 {
                            &commit.id[..7]
                        } else {
                            &commit.id
                        };

                        let mut parts = vec![format!("{}", short_id.bright_cyan())];

                        if let Some(bpm) = metadata.bpm {
                            parts.push(format!("{}BPM", bpm));
                        }
                        if let Some(sr) = metadata.sample_rate {
                            parts.push(format!("{}Hz", sr));
                        }
                        if let Some(ref key) = metadata.key_signature {
                            parts.push(key.clone());
                        }

                        let first_line = metadata.message.lines().next().unwrap_or(&metadata.message);
                        parts.push(first_line.to_string());

                        if ranked {
                            let score = engine.relevance_score(commit, &search_query);
                            parts.push(format!("(score: {:.1})", score).dimmed().to_string());
                        }

                        println!("{}. {}", i + 1, parts.join(" │ "));
                    }
                }
                "list" | _ => {
                    for (i, commit) in results.iter().enumerate() {
                        let metadata = CommitMetadata::parse_commit_message(&commit.message);
                        let short_id = if commit.id.len() >= 7 {
                            &commit.id[..7]
                        } else {
                            &commit.id
                        };

                        println!("{}. {} {}",
                            format!("{}", i + 1).dimmed(),
                            "Commit".bright_black(),
                            short_id.bright_cyan()
                        );

                        let first_line = metadata.message.lines().next().unwrap_or(&metadata.message);
                        println!("   Message: {}", first_line);

                        if let Some(bpm) = metadata.bpm {
                            println!("   BPM: {}", format!("{}", bpm).yellow());
                        }
                        if let Some(sr) = metadata.sample_rate {
                            println!("   Sample Rate: {} Hz", sr);
                        }
                        if let Some(ref key) = metadata.key_signature {
                            println!("   Key: {}", key.green());
                        }
                        if !metadata.tags.is_empty() {
                            println!("   Tags: {}", metadata.tags.join(", ").bright_black());
                        }

                        if ranked {
                            let score = engine.relevance_score(commit, &search_query);
                            println!("   Relevance: {:.1}", score);
                        }

                        println!();
                    }
                }
            }

            progress::success(&format!("Found {} matching commit{}",
                results.len(),
                if results.len() == 1 { "" } else { "s" }
            ));

            Ok(())
        }

        Commands::Lock(lock_cmd) => {
            use oxenvcs_cli::lock_integration;
            use oxenvcs_cli::network_resilience::{check_connectivity, ConnectivityState};
            use oxenvcs_cli::{OfflineQueue, QueuedOperation};
            use std::env;

            let current_dir = env::current_dir()?;

            // Auto-sync pending queue if online (for all lock commands)
            match check_connectivity() {
                ConnectivityState::Online => {
                    let mut queue = OfflineQueue::new()?;
                    let pending_count = queue.pending().len();

                    if pending_count > 0 {
                        vlog!("Auto-syncing {} pending operation(s) before lock operation...", pending_count);
                        let report = queue.sync_all()?;

                        if !report.failed.is_empty() {
                            warn!("{} queued operation(s) failed to sync", report.failed.len());
                        }
                    }
                }
                _ => {}
            }

            match lock_cmd {
                LockCommands::Acquire { timeout } => {
                    // Check connectivity
                    match check_connectivity() {
                        ConnectivityState::Offline => {
                            // Queue the operation
                            let mut queue = OfflineQueue::new()?;
                            let user_id = lock_integration::get_user_identifier();

                            let entry_id = queue.enqueue_with_priority(QueuedOperation::AcquireLock {
                                project_path: current_dir.to_string_lossy().to_string(),
                                user_id: user_id.clone(),
                                timeout_hours: timeout as u32,
                            }, 100)?; // High priority

                            warn!("Network is offline - operation queued");
                            println!();
                            println!("  {} {}", "Queued:".bold(), "Acquire lock".yellow());
                            println!("  {} {}", "User:".dimmed(), user_id.dimmed());
                            println!("  {} {} hours", "Timeout:".dimmed(), timeout);
                            println!("  {} {}", "Entry ID:".dimmed(), &entry_id[..8].dimmed());
                            println!();
                            progress::info("Lock will be acquired when network is available");
                            progress::info("Use 'oxenvcs-cli queue sync' to retry manually");
                        }
                        _ => {
                            // Execute normally (online or unknown)
                            lock_integration::handle_lock_acquire(&current_dir, timeout)?;
                        }
                    }
                }

                LockCommands::Release => {
                    // Check connectivity
                    match check_connectivity() {
                        ConnectivityState::Offline => {
                            // Queue the operation
                            let mut queue = OfflineQueue::new()?;

                            // We don't know the lock_id when offline, so use a placeholder
                            // The execute_entry will look up the actual lock
                            let entry_id = queue.enqueue_with_priority(QueuedOperation::ReleaseLock {
                                project_path: current_dir.to_string_lossy().to_string(),
                                lock_id: "pending".to_string(), // Will be looked up during execution
                            }, 100)?; // High priority

                            warn!("Network is offline - operation queued");
                            println!();
                            println!("  {} {}", "Queued:".bold(), "Release lock".yellow());
                            println!("  {} {}", "Entry ID:".dimmed(), &entry_id[..8].dimmed());
                            println!();
                            progress::info("Lock will be released when network is available");
                            progress::info("Use 'oxenvcs-cli queue sync' to retry manually");
                        }
                        _ => {
                            // Execute normally (online or unknown)
                            lock_integration::handle_lock_release(&current_dir)?;
                        }
                    }
                }

                LockCommands::Status => {
                    // Status check always executes (even when offline, shows local state)
                    lock_integration::handle_lock_status(&current_dir)?;
                }

                LockCommands::Break { force } => {
                    // Break force always executes (administrative override)
                    lock_integration::handle_lock_break(&current_dir, force)?;
                }
            }

            Ok(())
        }

        Commands::Auth(auth_cmd) => {
            use oxenvcs_cli::AuthManager;

            let auth = AuthManager::new();

            match auth_cmd {
                AuthCommands::Login => {
                    use std::io::{self, Write};

                    println!();
                    println!("┌─ Oxen Hub Authentication ──────────────────────────────┐");
                    println!("│                                                          │");
                    println!("│  Login to Oxen Hub to enable remote collaboration       │");
                    println!("│                                                          │");
                    println!("│  Get your API key from: https://hub.oxen.ai             │");
                    println!("│  Settings → API Keys → Create New Key                   │");
                    println!("│                                                          │");
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();

                    // Prompt for username
                    print!("Username: ");
                    io::stdout().flush()?;
                    let mut username = String::new();
                    io::stdin().read_line(&mut username)?;
                    let username = username.trim();

                    if username.is_empty() {
                        progress::error("Username cannot be empty");
                        std::process::exit(1);
                    }

                    // Prompt for API key (hidden input would be better, but keep it simple for now)
                    print!("API Key: ");
                    io::stdout().flush()?;
                    let mut api_key = String::new();
                    io::stdin().read_line(&mut api_key)?;
                    let api_key = api_key.trim();

                    if api_key.is_empty() {
                        progress::error("API key cannot be empty");
                        std::process::exit(1);
                    }

                    // Store credentials
                    let pb = progress::spinner("Storing credentials...");

                    match auth.store_credentials(username, api_key) {
                        Ok(_) => {
                            progress::finish_success(&pb, "Credentials stored");
                            println!();
                            progress::success(&format!("Authenticated as: {}", username));
                            progress::info("You can now push/pull projects to Oxen Hub");
                            println!();
                            progress::info("Test authentication with: oxenvcs-cli auth test");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to store credentials");
                            progress::error(&format!("Error: {}", e));
                            std::process::exit(1);
                        }
                    }
                }

                AuthCommands::Logout => {
                    let pb = progress::spinner("Clearing credentials...");

                    match auth.clear_credentials() {
                        Ok(_) => {
                            progress::finish_success(&pb, "Logged out");
                            println!();
                            progress::success("Credentials removed");
                            progress::info("You are now logged out from Oxen Hub");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to clear credentials");
                            progress::error(&format!("Error: {}", e));
                            std::process::exit(1);
                        }
                    }
                }

                AuthCommands::Status => {
                    println!();
                    println!("┌─ Authentication Status ─────────────────────────────────┐");
                    println!("│                                                          │");

                    match auth.get_credentials() {
                        Ok(Some(creds)) => {
                            println!("│  Status: {} Authenticated                            │", "●".green());
                            println!("│                                                          │");
                            println!("│  Username: {}                                    │", creds.username);
                            println!("│  Hub URL:  {}                      │", creds.hub_url);
                            println!("│                                                          │");
                            println!("└──────────────────────────────────────────────────────────┘");
                            println!();
                            progress::info("Run 'oxenvcs-cli auth test' to verify connection");
                        }
                        Ok(None) => {
                            println!("│  Status: {} Not authenticated                        │", "○".yellow());
                            println!("│                                                          │");
                            println!("│  You need to login to use remote features               │");
                            println!("│                                                          │");
                            println!("└──────────────────────────────────────────────────────────┘");
                            println!();
                            progress::info("Login with: oxenvcs-cli auth login");
                        }
                        Err(e) => {
                            println!("│  Status: {} Error                                    │", "✗".red());
                            println!("│                                                          │");
                            println!("│  Error reading credentials                               │");
                            println!("│                                                          │");
                            println!("└──────────────────────────────────────────────────────────┘");
                            println!();
                            progress::error(&format!("Error: {}", e));
                        }
                    }
                }

                AuthCommands::Test => {
                    let pb = progress::spinner("Testing authentication...");

                    match auth.test_authentication() {
                        Ok(username) => {
                            progress::finish_success(&pb, "Authentication verified");
                            println!();
                            progress::success(&format!("Successfully authenticated as: {}", username));
                            progress::info("Your credentials are valid");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Authentication failed");
                            println!();
                            progress::error("Unable to verify authentication");
                            progress::info(&format!("Error: {}", e));
                            println!();
                            progress::info("Try logging in again: oxenvcs-cli auth login");
                            std::process::exit(1);
                        }
                    }
                }
            }

            Ok(())
        }

        Commands::MetadataDiff {
            project_a,
            project_b,
            output,
            color,
            verbose: verbose_flag,
        } => {
            use oxenvcs_cli::{LogicParser, MetadataDiffer};

            vlog!("Parsing project A: {}", project_a.display());

            // Validate paths
            if !LogicParser::is_valid_project(&project_a) {
                anyhow::bail!("Invalid Logic Pro project: {}", project_a.display());
            }

            if !LogicParser::is_valid_project(&project_b) {
                anyhow::bail!("Invalid Logic Pro project: {}", project_b.display());
            }

            // Parse both projects
            let data_a = LogicParser::parse(&project_a)
                .map_err(|e| anyhow::anyhow!("Failed to parse project A: {}", e))?;

            vlog!("Parsing project B: {}", project_b.display());
            let data_b = LogicParser::parse(&project_b)
                .map_err(|e| anyhow::anyhow!("Failed to parse project B: {}", e))?;

            // Generate diff
            vlog!("Computing metadata diff");
            let diff = MetadataDiffer::compare(&data_a, &data_b);

            // Output result
            match output.as_str() {
                "json" => {
                    let json = MetadataDiffer::to_json(&diff)?;
                    println!("{}", json);
                }
                "text" | _ => {
                    // Determine color usage
                    let use_color = if color {
                        true
                    } else {
                        // Auto-detect TTY
                        atty::is(atty::Stream::Stdout)
                    };

                    let report = MetadataDiffer::generate_report_with_options(
                        &diff,
                        use_color,
                        verbose_flag || cli.verbose,
                    );
                    println!("{}", report);
                }
            }

            success!("Metadata diff completed");
            Ok(())
        }

        Commands::Daemon(daemon_cmd) => {
            use oxenvcs_cli::daemon_client::DaemonClient;

            let client = DaemonClient::new();

            match daemon_cmd {
                DaemonCommands::Status => {
                    let pb = progress::spinner("Checking daemon status...");
                    let status = client.status()?;
                    pb.finish_and_clear();

                    println!();
                    println!("┌─ Daemon Status ─────────────────────────────────────────┐");

                    if status.is_running {
                        println!("│  Status: {} Running", "●".green());
                        if let Some(pid) = status.pid {
                            println!("│  PID: {}", pid.to_string().bright_yellow());
                        }
                    } else {
                        println!("│  Status: {} Stopped", "●".red());
                    }

                    if let Some(count) = status.project_count {
                        println!("│  Monitored Projects: {}", count);
                    }
                    if let Some(version) = status.version {
                        println!("│  Version: {}", version);
                    }
                    if let Some(uptime) = status.uptime {
                        println!("│  Uptime: {:.1} hours", uptime / 3600.0);
                    }

                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();

                    if !status.is_running {
                        progress::info("Start the daemon with: oxenvcs-cli daemon start");
                    }

                    Ok(())
                }

                DaemonCommands::Start => {
                    // Check if already running
                    let status = client.status()?;
                    if status.is_running {
                        progress::warning("Daemon is already running");
                        if let Some(pid) = status.pid {
                            println!("  PID: {}", pid);
                        }
                        return Ok(());
                    }

                    // Check if installed
                    if !client.is_installed() {
                        progress::error("Daemon is not installed");
                        progress::info("Please run the installer: ./install.sh");
                        anyhow::bail!("Daemon not installed");
                    }

                    let pb = progress::spinner("Starting daemon...");
                    client.start()?;
                    progress::finish_success(&pb, "Daemon started");

                    // Show updated status
                    let status = client.status()?;
                    if let Some(pid) = status.pid {
                        println!();
                        progress::info(&format!("Daemon running with PID: {}", pid));
                    }

                    Ok(())
                }

                DaemonCommands::Stop => {
                    // Check if running
                    let status = client.status()?;
                    if !status.is_running {
                        progress::warning("Daemon is not running");
                        return Ok(());
                    }

                    let pb = progress::spinner("Stopping daemon...");
                    client.stop()?;
                    progress::finish_success(&pb, "Daemon stopped");

                    Ok(())
                }

                DaemonCommands::Restart => {
                    let pb = progress::spinner("Restarting daemon...");
                    client.restart()?;
                    progress::finish_success(&pb, "Daemon restarted");

                    // Show updated status
                    let status = client.status()?;
                    if let Some(pid) = status.pid {
                        println!();
                        progress::info(&format!("Daemon running with PID: {}", pid));
                    }

                    Ok(())
                }

                DaemonCommands::Logs { lines } => {
                    let log_path = client.log_path()?;

                    println!();
                    println!("┌─ Daemon Logs ───────────────────────────────────────────┐");
                    println!("│  Path: {}", log_path);
                    println!("│  Lines: {}", lines);
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();

                    let log_lines = client.tail_logs(lines)?;

                    if log_lines.is_empty() {
                        progress::info("No log entries found");
                    } else {
                        for line in log_lines {
                            println!("{}", line);
                        }
                    }

                    Ok(())
                }
            }
        }

        Commands::Hooks(hooks_cmd) => {
            use oxenvcs_cli::hooks::{HookManager, HookType};

            let repo_path = std::env::current_dir()?;
            let manager = HookManager::new(&repo_path);

            match hooks_cmd {
                HooksCommands::Init => {
                    let pb = progress::spinner("Initializing hooks directory...");
                    manager.init()?;
                    progress::finish_success(&pb, "Hooks directory initialized");

                    println!();
                    println!("┌─ Hooks Initialized ─────────────────────────────────────┐");
                    println!("│                                                          │");
                    println!("│  Created:                                                │");
                    println!("│    .oxen/hooks/pre-commit/                               │");
                    println!("│    .oxen/hooks/post-commit/                              │");
                    println!("│    .oxen/hooks/README.md                                 │");
                    println!("│                                                          │");
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();
                    progress::info("See .oxen/hooks/README.md for documentation");
                    progress::info("Install built-in hooks with: oxenvcs-cli hooks install <name>");

                    Ok(())
                }

                HooksCommands::List => {
                    let hooks = manager.list_hooks()?;

                    println!();
                    println!("┌─ Installed Hooks ───────────────────────────────────────┐");
                    println!("│                                                          │");

                    if hooks.is_empty() {
                        println!("│  No hooks installed                                      │");
                    } else {
                        let pre_commit: Vec<_> = hooks
                            .iter()
                            .filter(|(t, _)| matches!(t, HookType::PreCommit))
                            .collect();
                        let post_commit: Vec<_> = hooks
                            .iter()
                            .filter(|(t, _)| matches!(t, HookType::PostCommit))
                            .collect();

                        if !pre_commit.is_empty() {
                            println!("│  Pre-commit hooks:                                       │");
                            for (_, name) in pre_commit {
                                println!("│    • {:<51} │", name);
                            }
                            println!("│                                                          │");
                        }

                        if !post_commit.is_empty() {
                            println!("│  Post-commit hooks:                                      │");
                            for (_, name) in post_commit {
                                println!("│    • {:<51} │", name);
                            }
                        }
                    }

                    println!("│                                                          │");
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();

                    Ok(())
                }

                HooksCommands::Builtins => {
                    let builtins = HookManager::list_builtins();

                    println!();
                    println!("┌─ Available Built-in Hooks ──────────────────────────────┐");
                    println!("│                                                          │");

                    for hook in builtins {
                        let type_str = match hook.hook_type {
                            HookType::PreCommit => "pre-commit",
                            HookType::PostCommit => "post-commit",
                        };
                        println!("│  {} ({})                                 ", hook.name.bright_yellow(), type_str.dimmed());
                        println!("│    {:<55} │", hook.description);
                        println!("│                                                          │");
                    }

                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();
                    progress::info("Install with: oxenvcs-cli hooks install <name>");

                    Ok(())
                }

                HooksCommands::Install { name, hook_type } => {
                    // Parse hook type
                    let hook_type = match hook_type.as_str() {
                        "pre-commit" => HookType::PreCommit,
                        "post-commit" => HookType::PostCommit,
                        _ => {
                            anyhow::bail!("Invalid hook type: {}. Use 'pre-commit' or 'post-commit'", hook_type);
                        }
                    };

                    // Ensure hooks directory exists
                    manager.init()?;

                    let pb = progress::spinner(&format!("Installing {} hook...", name));
                    manager.install_builtin(&name, hook_type)?;
                    progress::finish_success(&pb, &format!("Installed {} hook", name));

                    println!();
                    progress::success(&format!("Hook '{}' installed successfully", name));
                    progress::info(&format!("Edit at: .oxen/hooks/{}/{}", hook_type.dir_name(), name));

                    Ok(())
                }

                HooksCommands::Remove { name, hook_type } => {
                    // Parse hook type
                    let hook_type = match hook_type.as_str() {
                        "pre-commit" => HookType::PreCommit,
                        "post-commit" => HookType::PostCommit,
                        _ => {
                            anyhow::bail!("Invalid hook type: {}. Use 'pre-commit' or 'post-commit'", hook_type);
                        }
                    };

                    let pb = progress::spinner(&format!("Removing {} hook...", name));
                    manager.remove_hook(&name, hook_type)?;
                    progress::finish_success(&pb, &format!("Removed {} hook", name));

                    Ok(())
                }
            }
        }

        Commands::Console { path } => {
            use oxenvcs_cli::console::{Console, DaemonStatus as ConsoleDaemonStatus};
            use oxenvcs_cli::daemon_client::DaemonClient;

            // Determine project path
            let project_path = match path {
                Some(p) => p,
                None => std::env::current_dir()
                    .context("Failed to get current directory")?,
            };

            vlog!("Launching console for project: {}", project_path.display());

            // Validate it's a Logic Pro project or Oxen repository
            // TODO: Add validation once we have repository detection

            // Create and run console
            let mut console = Console::new(project_path);

            // Check daemon status
            let daemon_client = DaemonClient::new();
            let status = daemon_client.status().unwrap_or_else(|_| {
                oxenvcs_cli::daemon_client::DaemonStatus {
                    is_running: false,
                    pid: None,
                    project_count: None,
                    version: None,
                    uptime: None,
                }
            });

            // Set initial daemon status
            let console_status = if status.is_running {
                ConsoleDaemonStatus::Running
            } else {
                ConsoleDaemonStatus::Stopped
            };
            console.set_daemon_status(console_status);

            // Run the console
            console.run().await?;

            success!("Console exited");
            Ok(())
        }

        Commands::Activity { limit } => {
            use oxenvcs_cli::ActivityFeed;
            use std::env;

            let current_dir = env::current_dir()?;
            let feed = ActivityFeed::new();

            let pb = progress::spinner("Fetching project activity...");
            let activities = feed.get_recent_activity(&current_dir, limit)?;
            pb.finish_and_clear();

            if activities.is_empty() {
                println!();
                progress::info("No activity found");
                println!("This project has no commit history yet.");
                println!();
                progress::info("Create your first commit:");
                println!("  oxenvcs-cli add --all");
                println!("  oxenvcs-cli commit -m \"Initial commit\" --bpm 120");
                return Ok(());
            }

            println!();
            println!("┌─ Project Activity ──────────────────────────────────────┐");
            println!("│                                                          │");
            println!("│  Recent activity ({} commits)                            │", activities.len());
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            for (i, activity) in activities.iter().enumerate() {
                let icon = activity.activity_type.icon();
                let author = if activity.author.len() > 20 {
                    format!("{}...", &activity.author[..17])
                } else {
                    activity.author.clone()
                };

                println!("{} {} {} - {}",
                    (i + 1).to_string().bright_black(),
                    icon,
                    author.cyan(),
                    activity.message.white()
                );

                // Show metadata if present
                if !activity.metadata.is_empty() {
                    for (key, value) in &activity.metadata {
                        println!("    {} {}: {}", "│".bright_black(), key.bright_black(), value.bright_black());
                    }
                }
            }

            println!();
            progress::info(&format!("Showing {} most recent activities", activities.len()));

            Ok(())
        }

        Commands::Team => {
            use oxenvcs_cli::TeamManager;
            use std::env;

            let current_dir = env::current_dir()?;
            let team_mgr = TeamManager::new();

            let pb = progress::spinner("Discovering team members...");
            let members = team_mgr.discover_team_members(&current_dir)?;
            pb.finish_and_clear();

            if members.is_empty() {
                println!();
                progress::warning("No team members found");
                println!("This project has no commit history with author information.");
                return Ok(());
            }

            let total_commits: usize = members.iter().map(|m| m.commit_count).sum();

            println!();
            println!("┌─ Team Members ──────────────────────────────────────────┐");
            println!("│                                                          │");
            println!("│  {} members · {} total commits                           │",
                members.len(), total_commits);
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            for (i, member) in members.iter().enumerate() {
                let percentage = (member.commit_count as f64 / total_commits as f64 * 100.0) as usize;
                let bar_length = (percentage / 5).min(20);
                let bar = "█".repeat(bar_length);

                println!("{} {} {} commits ({}%)",
                    (i + 1).to_string().bright_black(),
                    member.name.cyan(),
                    member.commit_count.to_string().green(),
                    percentage
                );
                println!("   {} {}", bar.green(), " ".repeat(20 - bar_length));
            }

            println!();
            progress::success(&format!("Found {} team members", members.len()));

            Ok(())
        }

        Commands::Queue(queue_cmd) => {
            use oxenvcs_cli::OfflineQueue;
            use colored::Colorize;

            let mut queue = OfflineQueue::new()?;

            match queue_cmd {
                QueueCommands::Status => {
                    let pending = queue.pending();
                    let stats = queue.stats();

                    println!("\n{}", "Offline Operation Queue".bold());
                    println!("{}", "=".repeat(50));
                    println!();

                    if pending.is_empty() {
                        progress::success("No pending operations");
                        println!("\n  All operations have been synced!");
                    } else {
                        println!("  {} {}", "Pending:".bold(), pending.len().to_string().yellow());
                        println!("  {} {}", "Completed:".bold(), stats.completed.to_string().green());
                        println!("  {} {}", "Failed:".bold(), stats.failed.to_string().red());
                        println!();

                        for (i, entry) in pending.iter().enumerate() {
                            let age = chrono::Utc::now().signed_duration_since(entry.queued_at);
                            let age_str = if age.num_hours() > 0 {
                                format!("{}h ago", age.num_hours())
                            } else if age.num_minutes() > 0 {
                                format!("{}m ago", age.num_minutes())
                            } else {
                                format!("{}s ago", age.num_seconds())
                            };

                            println!("  {}. {} {}",
                                (i + 1).to_string().cyan(),
                                entry.operation.description(),
                                format!("({})", age_str).dimmed()
                            );
                            println!("     {} {} | {} {} | {} {}",
                                "ID:".dimmed(),
                                &entry.id[..8].dimmed(),
                                "Priority:".dimmed(),
                                entry.priority.to_string().dimmed(),
                                "Attempts:".dimmed(),
                                entry.attempts.to_string().dimmed()
                            );
                            println!();
                        }

                        println!("  {}", "Use 'oxenvcs-cli queue sync' to sync pending operations".dimmed());
                    }

                    Ok(())
                }

                QueueCommands::Sync => {
                    use oxenvcs_cli::network_resilience::{check_connectivity, ConnectivityState};

                    println!("\n{}", "Syncing Offline Queue".bold());
                    println!("{}", "=".repeat(50));
                    println!();

                    // Check connectivity
                    match check_connectivity() {
                        ConnectivityState::Offline => {
                            progress::error("Network is offline - cannot sync");
                            println!("\n  Operations will remain queued until network is available");
                            return Ok(());
                        }
                        ConnectivityState::Unknown => {
                            warn!("Network state unknown, attempting sync anyway");
                        }
                        ConnectivityState::Online => {
                            progress::success("Network is online");
                        }
                    }

                    let pending_count = queue.pending().len();
                    if pending_count == 0 {
                        progress::success("No pending operations to sync");
                        return Ok(());
                    }

                    println!("  Syncing {} pending operation(s)...\n", pending_count);

                    let report = queue.sync_all()?;

                    println!();
                    println!("{}", "Sync Results".bold());
                    println!("{}", "=".repeat(50));
                    println!("  {} {}", "Succeeded:".bold(), report.succeeded.len().to_string().green());
                    println!("  {} {}", "Failed:".bold(), report.failed.len().to_string().red());

                    if !report.failed.is_empty() {
                        println!();
                        println!("{}", "Failed Operations:".bold());
                        for (id, error) in &report.failed {
                            println!("  {} {} - {}", "✗".red(), &id[..8], error.dimmed());
                        }
                    }

                    println!();

                    if report.failed.is_empty() {
                        progress::success("All operations synced successfully!");
                    } else {
                        warn!("Some operations failed - they remain queued for retry");
                    }

                    Ok(())
                }

                QueueCommands::Clear { all } => {
                    if all {
                        // Clear everything
                        let total = queue.pending().len() + queue.stats().completed;

                        // Remove all entries
                        let entry_ids: Vec<String> = queue.pending().iter()
                            .map(|e| e.id.clone())
                            .collect();

                        for id in entry_ids {
                            queue.remove(&id)?;
                        }

                        queue.clear_completed()?;

                        progress::success(&format!("Cleared {} total operation(s)", total));
                    } else {
                        // Clear only completed
                        let completed_count = queue.stats().completed;
                        queue.clear_completed()?;
                        progress::success(&format!("Cleared {} completed operation(s)", completed_count));
                    }

                    Ok(())
                }

                QueueCommands::Remove { entry_id } => {
                    queue.remove(&entry_id)?;
                    progress::success(&format!("Removed operation {}", &entry_id[..8]));
                    Ok(())
                }
            }
        }

        Commands::Completions { shell } => {
            use clap::CommandFactory;
            use clap_complete::{generate, Shell};
            use std::io;

            let shell_type = match shell.to_lowercase().as_str() {
                "bash" => Shell::Bash,
                "zsh" => Shell::Zsh,
                "fish" => Shell::Fish,
                "powershell" => Shell::PowerShell,
                _ => {
                    progress::error(&format!("Unsupported shell: {}", shell));
                    println!("\nSupported shells: bash, zsh, fish, powershell");
                    std::process::exit(1);
                }
            };

            let mut cmd = Cli::command();
            let bin_name = "oxenvcs-cli";

            generate(shell_type, &mut cmd, bin_name, &mut io::stdout());

            Ok(())
        }

        Commands::Comment(comment_cmd) => {
            use oxenvcs_cli::CommentManager;
            use std::env;

            let current_dir = env::current_dir()?;
            let comment_mgr = CommentManager::new();

            match comment_cmd {
                CommentCommands::Add { commit_id, text } => {
                    let user = lock_integration::get_user_identifier();

                    let pb = progress::spinner("Adding comment...");

                    match comment_mgr.add_comment(&current_dir, &commit_id, &user, &text) {
                        Ok(comment) => {
                            progress::finish_success(&pb, "Comment added");
                            println!();
                            progress::success("Comment added successfully");
                            println!();
                            println!("  💬 {} said:", user.cyan());
                            println!("     \"{}\"", text);
                            println!();
                            progress::info("Comment stored in .oxen/comments/");
                            println!("  Commit and push to share with team:");
                            println!("    oxen add .oxen/comments/");
                            println!("    oxen commit -m \"Add comment\"");
                            println!("    oxen push origin main");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to add comment");
                            println!();
                            progress::error(&format!("{}", e));
                            std::process::exit(1);
                        }
                    }

                    Ok(())
                }

                CommentCommands::List { commit_id } => {
                    let commit = commit_id.as_deref().unwrap_or("HEAD");

                    let pb = progress::spinner("Fetching comments...");
                    let comments = comment_mgr.get_comments(&current_dir, commit)?;
                    pb.finish_and_clear();

                    if comments.is_empty() {
                        println!();
                        progress::info("No comments on this commit");
                        println!();
                        progress::info("Add a comment:");
                        println!("  oxenvcs-cli comment add {} \"Your comment here\"", commit);
                        return Ok(());
                    }

                    println!();
                    println!("┌─ Comments on {} ─────────────────────────────────┐",
                        if commit.len() > 8 { &commit[..8] } else { commit });
                    println!("│                                                          │");
                    println!("│  {} comments                                              │", comments.len());
                    println!("│                                                          │");
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();

                    for (i, comment) in comments.iter().enumerate() {
                        println!("{} 💬 {} said:",
                            (i + 1).to_string().bright_black(),
                            comment.author.cyan()
                        );
                        println!("   \"{}\"", comment.text);
                        println!("   {}",
                            comment.created_at.format("%Y-%m-%d %H:%M UTC").to_string().bright_black()
                        );
                        if i < comments.len() - 1 {
                            println!();
                        }
                    }

                    println!();
                    progress::success(&format!("Showing {} comments", comments.len()));

                    Ok(())
                }
            }
        }
    }
}
