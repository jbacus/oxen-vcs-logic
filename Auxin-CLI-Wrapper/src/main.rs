use anyhow::Context;
use clap::{Parser, Subcommand};
use colored::Colorize;
use auxin::{lock_integration, logger, progress, success, vlog, warn, BlenderProject, CommitMetadata, Config, OxenRepository, ProjectType, SketchUpMetadata, SketchUpProject, AuxinServerClient, ServerConfig, server_client, BounceManager};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "auxin")]
#[command(version)]
#[command(about = "Auxin - Version control for creative applications")]
#[command(long_about = "Auxin - Version control for creative applications

This tool provides Git-like version control specifically designed for binary
project files from Logic Pro and SketchUp. It integrates with Oxen.ai to
efficiently track and manage your creative projects with support for
application-specific metadata.

FEATURES:
  • Automatic detection and setup for Logic Pro and SketchUp projects
  • Application-specific metadata tracking
  • Draft branch workflow for auto-commits
  • Efficient handling of large binary files with block-level deduplication
  • Smart ignore patterns for cache and temporary files

BASIC WORKFLOW (Logic Pro):
  1. Initialize: auxin init --type logicpro MyProject.logicx
  2. Make changes in Logic Pro
  3. Commit: auxin commit -m \"Added drum track\" --bpm 120

BASIC WORKFLOW (SketchUp):
  1. Initialize: auxin init --type sketchup MyModel.skp
  2. Make changes in SketchUp
  3. Commit: auxin commit -m \"Added materials\" --units Inches --layers 10

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
    auxin lock acquire [--timeout <HOURS>]

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
    auxin lock acquire

    # Acquire lock with 8-hour timeout
    auxin lock acquire --timeout 8")]
    Acquire {
        #[arg(long, default_value = "4", help = "Lock timeout in hours")]
        timeout: u64,
    },

    /// Release the lock you currently hold
    #[command(long_about = "Release the lock you currently hold

USAGE:
    auxin lock release

DESCRIPTION:
    Releases the exclusive lock you currently hold on the project, allowing
    other team members to acquire it and make changes.

    You should release the lock when:
      • You're done editing for the session
      • You've committed your changes
      • You need to switch to a different task

EXAMPLES:
    # Release your lock
    auxin lock release")]
    Release,

    /// Show current lock status
    #[command(long_about = "Show current lock status

USAGE:
    auxin lock status

DESCRIPTION:
    Displays information about the current project lock:
      • Whether the project is locked
      • Who holds the lock
      • When the lock was acquired
      • When the lock expires
      • Time remaining

EXAMPLES:
    # Check lock status
    auxin lock status")]
    Status,

    /// Force break an existing lock (admin only)
    #[command(long_about = "Force break an existing lock (admin only)

USAGE:
    auxin lock break --force

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
    auxin lock break --force")]
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
    auxin daemon status

DESCRIPTION:
    Displays the current status of the Auxin background daemon, including:
      • Whether the daemon is running
      • Process ID (if running)
      • Number of monitored projects
      • Uptime information

EXAMPLES:
    # Check daemon status
    auxin daemon status")]
    Status,

    /// Start the daemon service
    #[command(long_about = "Start the daemon service

USAGE:
    auxin daemon start

DESCRIPTION:
    Starts the Auxin background daemon service using launchctl.
    The daemon provides:
      • Automatic file monitoring for Logic Pro projects
      • Auto-commit on file changes (with debounce)
      • Power management (save before sleep/shutdown)
      • Lock management for team collaboration

    The daemon runs in the background and starts automatically on login.

EXAMPLES:
    # Start the daemon
    auxin daemon start")]
    Start,

    /// Stop the daemon service
    #[command(long_about = "Stop the daemon service

USAGE:
    auxin daemon stop

DESCRIPTION:
    Stops the Auxin background daemon service.
    This will:
      • Stop file monitoring for all projects
      • Disable auto-commits
      • Stop power management hooks

    Note: Projects remain tracked; monitoring resumes when daemon restarts.

EXAMPLES:
    # Stop the daemon
    auxin daemon stop")]
    Stop,

    /// Restart the daemon service
    #[command(long_about = "Restart the daemon service

USAGE:
    auxin daemon restart

DESCRIPTION:
    Stops and then starts the daemon service.
    Useful after:
      • Updating the daemon binary
      • Changing configuration settings
      • Recovering from errors

EXAMPLES:
    # Restart the daemon
    auxin daemon restart")]
    Restart,

    /// Show daemon logs
    #[command(long_about = "Show daemon logs

USAGE:
    auxin daemon logs [--lines <N>]

DESCRIPTION:
    Displays recent entries from the daemon log file.
    Useful for debugging and monitoring daemon activity.

OPTIONS:
    --lines <N>    Number of recent log lines to show (default: 50)

EXAMPLES:
    # Show last 50 log lines
    auxin daemon logs

    # Show last 100 log lines
    auxin daemon logs --lines 100")]
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
    auxin auth login

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
    auxin auth login")]
    Login,

    /// Logout from Oxen Hub
    #[command(long_about = "Logout from Oxen Hub

USAGE:
    auxin auth logout

DESCRIPTION:
    Remove stored Oxen Hub credentials from the system.
    After logout, push/pull operations will fail until you login again.

EXAMPLES:
    # Logout
    auxin auth logout")]
    Logout,

    /// Show current authentication status
    #[command(long_about = "Show current authentication status

USAGE:
    auxin auth status

DESCRIPTION:
    Display information about the currently authenticated user:
      • Username
      • Oxen Hub URL
      • Authentication status

EXAMPLES:
    # Check auth status
    auxin auth status")]
    Status,

    /// Test authentication with Oxen Hub
    #[command(long_about = "Test authentication with Oxen Hub

USAGE:
    auxin auth test

DESCRIPTION:
    Verify that your stored credentials are valid by testing
    connection to Oxen Hub. This is useful for troubleshooting
    authentication issues.

EXAMPLES:
    # Test authentication
    auxin auth test")]
    Test,
}

#[derive(Subcommand)]
enum ServerCommands {
    /// Show server configuration and connection status
    #[command(long_about = "Show server configuration and connection status

USAGE:
    auxin server status

DESCRIPTION:
    Displays the current server configuration including:
      • Server URL
      • Connection status (healthy/unreachable)
      • Whether server locks are enabled
      • Whether server metadata storage is enabled
      • Default namespace

EXAMPLES:
    # Check server status
    auxin server status")]
    Status,

    /// Test connection to auxin-server
    #[command(long_about = "Test connection to auxin-server

USAGE:
    auxin server health

DESCRIPTION:
    Performs a health check on the configured auxin-server to verify:
      • Network connectivity
      • Server is responding
      • API is functional

EXAMPLES:
    # Test server connection
    auxin server health")]
    Health,

    /// Set server configuration value
    #[command(long_about = "Set server configuration value

USAGE:
    auxin server set <KEY> <VALUE>

DESCRIPTION:
    Updates the server configuration in .auxin/config.toml.
    Available keys:
      • url          - Server URL (e.g., http://localhost:3000)
      • namespace    - Default namespace for repositories
      • timeout      - Request timeout in seconds
      • locks        - Enable/disable server locks (true/false)
      • metadata     - Enable/disable server metadata (true/false)

EXAMPLES:
    # Set server URL
    auxin server set url http://192.168.1.100:3000

    # Set default namespace
    auxin server set namespace myteam

    # Enable server locks
    auxin server set locks true

    # Disable server metadata
    auxin server set metadata false")]
    Set {
        #[arg(value_name = "KEY", help = "Configuration key to set")]
        key: String,

        #[arg(value_name = "VALUE", help = "Value to set")]
        value: String,
    },
}

#[derive(Subcommand)]
enum BounceCommands {
    /// Add a bounce file for a commit
    #[command(long_about = "Add a bounce file for a commit

USAGE:
    auxin bounce add <FILE> [--commit <ID>] [--description <TEXT>]

DESCRIPTION:
    Attaches an audio bounce file to a commit as an audio 'screenshot' of the
    project state. Bounces are used for:
      • Quick A/B comparison between versions
      • Audio fingerprinting and semantic analysis
      • Historical record of project evolution

    Supported formats: WAV, AIFF, MP3, FLAC, M4A

    If no commit ID is specified, the bounce is attached to the most recent commit.

EXAMPLES:
    # Add bounce to latest commit
    auxin bounce add Bounces/MyMix.wav

    # Add bounce to specific commit
    auxin bounce add Bounces/MyMix.wav --commit abc123

    # Add bounce with description
    auxin bounce add Bounces/MyMix.wav --description 'Final mix before mastering'")]
    Add {
        #[arg(value_name = "FILE", help = "Path to the audio bounce file")]
        file: PathBuf,

        #[arg(long, value_name = "ID", help = "Commit ID to attach bounce to (default: latest)")]
        commit: Option<String>,

        #[arg(long, short, value_name = "TEXT", help = "Description of the bounce")]
        description: Option<String>,
    },

    /// List all bounces in the repository
    #[command(long_about = "List all bounces in the repository

USAGE:
    auxin bounce list

DESCRIPTION:
    Shows all audio bounces stored in the repository, including:
      • Commit ID
      • Original filename
      • Format and duration
      • File size
      • When it was added
      • Description (if provided)

EXAMPLES:
    # List all bounces
    auxin bounce list")]
    List,

    /// Play a bounce audio file
    #[command(long_about = "Play a bounce audio file

USAGE:
    auxin bounce play <COMMIT_ID>

DESCRIPTION:
    Plays the bounce audio file associated with a commit using the system
    audio player (afplay on macOS). This allows quick preview of how the
    project sounded at any point in history.

EXAMPLES:
    # Play bounce for a commit
    auxin bounce play abc123")]
    Play {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID of the bounce to play")]
        commit_id: String,
    },

    /// Show bounce metadata
    #[command(long_about = "Show bounce metadata

USAGE:
    auxin bounce info <COMMIT_ID>

DESCRIPTION:
    Displays detailed metadata about a bounce file:
      • Original filename
      • Audio format
      • Duration
      • Sample rate, bit depth, channels
      • File size
      • When added and by whom
      • Description

EXAMPLES:
    # Show bounce info
    auxin bounce info abc123")]
    Info {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID of the bounce")]
        commit_id: String,
    },

    /// Delete a bounce
    #[command(long_about = "Delete a bounce

USAGE:
    auxin bounce delete <COMMIT_ID>

DESCRIPTION:
    Removes the bounce audio file and metadata for a commit.
    This action cannot be undone.

EXAMPLES:
    # Delete bounce
    auxin bounce delete abc123")]
    Delete {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID of the bounce to delete")]
        commit_id: String,
    },

    /// Search and filter bounces
    #[command(long_about = "Search and filter bounces

USAGE:
    auxin bounce search [OPTIONS]

DESCRIPTION:
    Search through bounces with various filters:
      • Format (wav, mp3, flac, etc.)
      • Filename pattern (regex)
      • Duration range
      • File size range
      • Date range
      • User who added

EXAMPLES:
    # Find all WAV bounces
    auxin bounce search --format wav

    # Find bounces longer than 3 minutes
    auxin bounce search --min-duration 180

    # Find bounces with 'mix' in the filename
    auxin bounce search --pattern 'mix'

    # Find bounces from last week
    auxin bounce search --after 2024-01-01")]
    Search {
        #[arg(long, value_name = "FORMAT", help = "Filter by audio format (wav, mp3, flac, aiff, m4a)")]
        format: Option<String>,

        #[arg(long, value_name = "PATTERN", help = "Filter by filename pattern (regex)")]
        pattern: Option<String>,

        #[arg(long, value_name = "SECONDS", help = "Minimum duration in seconds")]
        min_duration: Option<f64>,

        #[arg(long, value_name = "SECONDS", help = "Maximum duration in seconds")]
        max_duration: Option<f64>,

        #[arg(long, value_name = "BYTES", help = "Minimum file size in bytes")]
        min_size: Option<u64>,

        #[arg(long, value_name = "BYTES", help = "Maximum file size in bytes")]
        max_size: Option<u64>,

        #[arg(long, value_name = "DATE", help = "Filter bounces added after this date (YYYY-MM-DD)")]
        after: Option<String>,

        #[arg(long, value_name = "DATE", help = "Filter bounces added before this date (YYYY-MM-DD)")]
        before: Option<String>,

        #[arg(long, value_name = "USER", help = "Filter by user who added the bounce")]
        user: Option<String>,
    },

    /// Compare two bounces
    #[command(long_about = "Compare two bounces

USAGE:
    auxin bounce compare <COMMIT_A> <COMMIT_B>

DESCRIPTION:
    Compares two bounces side-by-side, showing differences in:
      • Duration
      • File size
      • Format
      • Sample rate and bit depth

    Useful for A/B testing different mixes or comparing project evolution.

EXAMPLES:
    # Compare two commits
    auxin bounce compare abc123 def456")]
    Compare {
        #[arg(value_name = "COMMIT_A", help = "First commit ID")]
        commit_a: String,

        #[arg(value_name = "COMMIT_B", help = "Second commit ID")]
        commit_b: String,
    },

    /// Add multiple bounce files at once
    #[command(long_about = "Add multiple bounce files at once

USAGE:
    auxin bounce batch-add <FILES>... [--commit <ID>]

DESCRIPTION:
    Add multiple audio bounce files to commits. Each file can have its commit
    ID inferred from the filename or specified explicitly.

    If filenames match pattern 'commit_<id>_<name>.wav', the commit ID is
    extracted automatically. Otherwise, files are added to the latest commit
    or the specified commit.

EXAMPLES:
    # Add all bounces in a directory
    auxin bounce batch-add Bounces/*.wav

    # Add specific files
    auxin bounce batch-add mix1.wav mix2.wav mix3.wav --commit abc123")]
    BatchAdd {
        #[arg(value_name = "FILES", required = true, num_args = 1..)]
        files: Vec<PathBuf>,

        #[arg(long, value_name = "ID", help = "Default commit ID for files without embedded ID")]
        commit: Option<String>,
    },

    /// Delete multiple bounces at once
    #[command(long_about = "Delete multiple bounces at once

USAGE:
    auxin bounce bulk-delete [OPTIONS]

DESCRIPTION:
    Delete bounces matching filter criteria. Use with caution as this
    action cannot be undone.

    At least one filter must be specified to prevent accidental deletion
    of all bounces.

EXAMPLES:
    # Delete all MP3 bounces
    auxin bounce bulk-delete --format mp3

    # Delete bounces older than a date
    auxin bounce bulk-delete --before 2024-01-01

    # Delete bounces by user
    auxin bounce bulk-delete --user olduser")]
    BulkDelete {
        #[arg(long, value_name = "FORMAT", help = "Delete bounces with this format")]
        format: Option<String>,

        #[arg(long, value_name = "DATE", help = "Delete bounces added before this date (YYYY-MM-DD)")]
        before: Option<String>,

        #[arg(long, value_name = "USER", help = "Delete bounces added by this user")]
        user: Option<String>,

        #[arg(long, help = "Force deletion without confirmation")]
        force: bool,
    },
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
    /// Initialize a new Oxen repository for a project
    #[command(long_about = "Initialize a new Oxen repository for a project

USAGE:
    auxin init [--type <TYPE>] <PATH>

DESCRIPTION:
    Creates a new Oxen repository at the specified path with project-specific
    configuration. The project type can be auto-detected or explicitly specified.

    Supported project types:
      • auto       - Auto-detect based on file extension (default)
      • logicpro   - Logic Pro projects (.logicx)
      • sketchup   - SketchUp models (.skp)

    For Logic Pro projects:
      • Detects and validates .logicx structure
      • Creates .oxenignore with Logic-specific patterns
      • Tracks projectData, Alternatives, Resources
      • Sets up draft branch workflow

    For SketchUp projects:
      • Detects and validates .skp file
      • Creates .oxenignore with SketchUp-specific patterns
      • Tracks .skp file and asset directories (textures/, components/)
      • Sets up draft branch workflow

    The PATH can be:
      • Current directory: .
      • Relative path: MyProject.logicx or MyModel.skp
      • Absolute path: /Users/you/Projects/MyModel.skp

EXAMPLES:
    # Auto-detect project type
    auxin init MyProject.logicx

    # Explicitly specify Logic Pro
    auxin init --type logicpro MyProject.logicx

    # Initialize SketchUp project
    auxin init --type sketchup MyModel.skp

    # Auto-detect in current directory
    auxin init .")]
    Init {
        #[arg(value_name = "PATH", help = "Path to the project file or directory")]
        path: PathBuf,

        #[arg(
            long,
            value_name = "TYPE",
            help = "Project type: auto, logicpro, sketchup (default: auto)"
        )]
        r#type: Option<String>,

        /// Legacy flag for backward compatibility
        #[arg(long, hide = true)]
        logic: bool,
    },

    /// Stage changes to be committed
    #[command(long_about = "Stage changes to be committed

USAGE:
    auxin add --all
    auxin add <PATHS>...

DESCRIPTION:
    Stages changes in the working directory for the next commit. Similar to 'git add'.
    Use --all to stage all changes, or specify individual files/directories.

EXAMPLES:
    # Stage all changes in the repository
    auxin add --all

    # Stage specific files
    auxin add projectData Alternatives/Take001

    # Stage a directory
    auxin add Resources/")]
    Add {
        #[arg(value_name = "PATHS", help = "Files or directories to stage")]
        paths: Vec<PathBuf>,

        #[arg(long, short, help = "Stage all changes in the repository")]
        all: bool,
    },

    /// Create a commit with optional project metadata
    #[command(long_about = "Create a commit with optional project metadata

USAGE:
    auxin commit -m <MESSAGE> [OPTIONS]

DESCRIPTION:
    Creates a new commit with the currently staged changes. You can attach
    project-specific metadata to help track the evolution of your project.

    Logic Pro metadata:
      • BPM (tempo)
      • Sample rate (Hz)
      • Key signature
      • Tags for categorization

    SketchUp metadata:
      • Units (Inches, Feet, Meters, etc.)
      • Layer count
      • Component count
      • Group count
      • File size (bytes)
      • Tags for categorization

EXAMPLES (Logic Pro):
    # Simple commit
    auxin commit -m \"Initial project setup\"

    # Commit with audio metadata
    auxin commit -m \"Added bass line\" --bpm 120 --key \"A Minor\"

    # Full metadata commit
    auxin commit -m \"Verse 2 complete\" \\
        --bpm 128 \\
        --sample-rate 48000 \\
        --key \"C Major\" \\
        --tags \"verse,arrangement\"

EXAMPLES (SketchUp):
    # Simple commit
    auxin commit -m \"Initial model geometry\"

    # Commit with SketchUp metadata
    auxin commit -m \"Added materials and textures\" \\
        --units Inches \\
        --layers 10 \\
        --components 150

    # Full metadata commit
    auxin commit -m \"Presentation ready\" \\
        --units Feet \\
        --layers 15 \\
        --components 234 \\
        --groups 12 \\
        --tags \"presentation,milestone\"")]
    Commit {
        #[arg(short, long, help = "Commit message describing the changes")]
        message: String,

        // Logic Pro metadata
        #[arg(long, help = "[Logic Pro] Beats per minute (tempo) of the project")]
        bpm: Option<f32>,

        #[arg(long, help = "[Logic Pro] Sample rate in Hz (e.g., 44100, 48000, 96000)")]
        sample_rate: Option<u32>,

        #[arg(long, help = "[Logic Pro] Key signature (e.g., 'C Major', 'A Minor', 'F# Minor')")]
        key: Option<String>,

        // SketchUp metadata
        #[arg(long, help = "[SketchUp] Model units (e.g., Inches, Feet, Meters, Millimeters)")]
        units: Option<String>,

        #[arg(long, help = "[SketchUp] Number of layers/tags in the model")]
        layers: Option<u32>,

        #[arg(long, help = "[SketchUp] Number of component instances")]
        components: Option<u32>,

        #[arg(long, help = "[SketchUp] Number of groups")]
        groups: Option<u32>,

        #[arg(long, help = "[SketchUp] Model file size in bytes")]
        file_size: Option<u64>,

        // Common metadata
        #[arg(
            long,
            help = "Tags for categorization (comma-separated, e.g., 'mixing,draft' or 'presentation,milestone')"
        )]
        tags: Option<String>,

        // Audio bounce
        #[arg(
            long,
            value_name = "FILE",
            help = "Audio bounce file to attach (WAV, AIFF, MP3, FLAC, M4A)"
        )]
        bounce: Option<PathBuf>,
    },

    /// Show commit history
    #[command(long_about = "Show commit history

USAGE:
    auxin log [OPTIONS]

DESCRIPTION:
    Displays the commit history for the repository, showing commit IDs, authors,
    timestamps, and messages. Audio metadata (BPM, key, etc.) is displayed if
    present in the commit message.

    Filter commits by metadata, tags, or date range to find specific versions.

EXAMPLES:
    # Show all commits
    auxin log

    # Show only the last 5 commits
    auxin log --limit 5

    # Show commits with specific BPM
    auxin log --bpm 128

    # Show commits with specific tag
    auxin log --tag mixing

    # Show commits since a date
    auxin log --since \"2025-01-01\"

    # Combine filters
    auxin log --bpm 120 --tag vocals --limit 10")]
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
    auxin restore <COMMIT_ID>

DESCRIPTION:
    Restores the project to the state at the specified commit. This checks out
    the files from that commit, allowing you to return to a previous version.

    WARNING: Make sure to commit any current changes before restoring, or they
    will be lost.

    You can find commit IDs using the 'log' command.

EXAMPLES:
    # Find commit IDs
    auxin log --limit 5

    # Restore to a specific commit
    auxin restore abc123def

    # Restore to a commit (full hash)
    auxin restore abc123def456789012345678901234567890")]
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
    auxin status

DESCRIPTION:
    Displays the current state of the working directory and staging area:
      • Staged files (ready to commit)
      • Modified files (changed but not staged)
      • Untracked files (new files not yet added)

    This is similar to 'git status' and helps you see what changes are pending.

EXAMPLES:
    # Check repository status
    auxin status")]
    Status,

    /// Show detailed information about a commit
    #[command(long_about = "Show detailed information about a commit

USAGE:
    auxin show <COMMIT_ID>

DESCRIPTION:
    Displays comprehensive information about a specific commit, including:
      • Full commit message
      • Audio metadata (BPM, sample rate, key signature, tags)
      • Author and timestamp
      • Files changed
      • Commit statistics

EXAMPLES:
    # Show details of a recent commit
    auxin show abc123f

    # Show details with full hash
    auxin show abc123def456789012345678901234567890")]
    Show {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to show details for")]
        commit_id: String,
    },

    /// Show changes between commits or working directory
    #[command(long_about = "Show changes between commits or working directory

USAGE:
    auxin diff [COMMIT_ID]

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
    auxin diff

    # Show changes since specific commit
    auxin diff abc123f

    # Compare two commits (future enhancement)
    # auxin diff abc123f def456a")]
    Diff {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID to compare against (optional)")]
        commit_id: Option<String>,
    },

    /// Compare metadata between two commits
    #[command(long_about = "Compare metadata between two commits

USAGE:
    auxin compare <COMMIT_A> <COMMIT_B>
    auxin compare <COMMIT_A> <COMMIT_B> --format json

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
    auxin compare abc123f def456a

    # Compare with plain text (no colors)
    auxin compare abc123f def456a --plain

    # Compare with JSON output
    auxin compare abc123f def456a --format json

    # Compare with compact one-line summary
    auxin compare abc123f def456a --format compact")]
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
    auxin search <QUERY>
    auxin search bpm:120-140 key:minor tag:mixing

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
    auxin search \"bpm:120-140\"

    # Find commits in minor keys with mixing tag
    auxin search \"key:minor tag:mixing\"

    # Find high BPM commits (>140)
    auxin search \"bpm:>140\"

    # Find commits with 'final' in message
    auxin search \"msg:final\"

    # Combined search with limit
    auxin search \"bpm:120-140 key:minor tag:vocals limit:5\"

    # Get compact one-line summaries
    auxin search \"bpm:>128\" --format compact

    # Ranked by relevance
    auxin search \"bpm:120-140 tag:mixing\" --ranked")]
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

    /// Manage auxin-server connection and configuration
    #[command(subcommand)]
    Server(ServerCommands),

    /// Manage audio bounce files for commits
    #[command(subcommand)]
    Bounce(BounceCommands),

    /// Compare metadata between two Logic Pro project versions
    #[command(name = "metadata-diff")]
    #[command(long_about = "Compare metadata between two Logic Pro project versions

USAGE:
    auxin metadata-diff <PROJECT_A> <PROJECT_B>
    auxin metadata-diff <PROJECT_A> <PROJECT_B> --output json

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
    auxin metadata-diff Project_v1.logicx Project_v2.logicx

    # Output as JSON for programmatic use
    auxin metadata-diff Project_v1.logicx Project_v2.logicx --output json

    # Compare with colored output
    auxin metadata-diff Project_v1.logicx Project_v2.logicx --color

    # Verbose mode with technical details
    auxin metadata-diff Project_v1.logicx Project_v2.logicx --verbose")]
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
    auxin hooks init
    auxin hooks list
    auxin hooks install <HOOK_NAME>

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
    auxin hooks init

    # List all installed hooks
    auxin hooks list

    # List available built-in hooks
    auxin hooks builtins

    # Install a built-in hook
    auxin hooks install validate-metadata

    # Install a post-commit hook
    auxin hooks install backup --hook-type post-commit

    # Remove a hook
    auxin hooks remove validate-metadata")]
    #[command(subcommand)]
    Hooks(HooksCommands),

    /// Launch interactive console for real-time monitoring
    #[command(long_about = "Launch interactive console for real-time monitoring

USAGE:
    auxin console [PATH]

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
    auxin console

    # Launch console for specific project
    auxin console ~/Music/MyProject.logicx")]
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
    auxin activity [--limit <N>]

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
    auxin activity

    # Show last 20 activities
    auxin activity --limit 20")]
    Activity {
        #[arg(long, default_value = "10", help = "Number of activities to show")]
        limit: usize,
    },

    /// Show team members and their contributions
    #[command(long_about = "Show team members and their contributions

USAGE:
    auxin team

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
    auxin team")]
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
    auxin completions <SHELL>

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
    auxin completions bash > /etc/bash_completion.d/auxin

    # Bash (macOS with Homebrew)
    auxin completions bash > /usr/local/etc/bash_completion.d/auxin

    # Zsh
    auxin completions zsh > /usr/local/share/zsh/site-functions/_auxin

    # Fish
    auxin completions fish > ~/.config/fish/completions/auxin.fish

    # PowerShell
    auxin completions powershell > auxin.ps1

EXAMPLES:
    # Generate bash completions
    auxin completions bash

    # Install for current user (bash)
    auxin completions bash > ~/.local/share/bash-completion/completions/auxin")]
    Completions {
        #[arg(value_name = "SHELL", help = "Shell to generate completions for (bash, zsh, fish, powershell)")]
        shell: String,
    },

    /// View operation history and audit trail
    #[command(subcommand)]
    History(HistoryCommands),

    /// Workflow automation and smart suggestions
    #[command(subcommand)]
    Workflow(WorkflowCommands),

    /// Backup and snapshot management
    #[command(subcommand)]
    Snapshot(SnapshotCommands),

    /// Recovery guides for common scenarios
    #[command(subcommand)]
    Recovery(RecoveryCommands),

    /// Push commits to remote with progress tracking
    #[command(long_about = "Push commits to remote with progress tracking

USAGE:
    auxin push [OPTIONS]

DESCRIPTION:
    Pushes commits to the remote repository with enhanced progress tracking
    and resume capability. For large files, uploads are chunked to allow
    resuming interrupted transfers.

    Features:
      • Real-time progress display with speed/ETA
      • Automatic resume of interrupted uploads
      • Bandwidth estimation
      • Retry on network failures

    If no remote or branch is specified, uses 'origin' and the current branch.

EXAMPLES:
    # Push to default remote (origin) and current branch
    auxin push

    # Push to specific remote and branch
    auxin push --remote origin --branch main

    # Push with verbose output
    auxin push --verbose

    # Force push (use with caution)
    auxin push --force")]
    Push {
        #[arg(long, short, help = "Remote name (default: origin)")]
        remote: Option<String>,

        #[arg(long, short, help = "Branch name (default: current branch)")]
        branch: Option<String>,

        #[arg(long, help = "Force push (overwrites remote history)")]
        force: bool,

        #[arg(long, short, help = "Show detailed progress")]
        verbose: bool,
    },

    /// Check system environment and dependencies
    #[command(long_about = "Check system environment and dependencies

USAGE:
    auxin doctor

DESCRIPTION:
    Validates your environment to ensure Auxin can work correctly.
    Checks for:
      • Oxen CLI installation and version
      • Authentication status with Oxen Hub
      • Remote repository configuration
      • Project initialization status
      • Daemon status

    This is the first command to run if you're having issues.

EXAMPLES:
    # Check everything
    auxin doctor

    # Check in current project directory
    cd MyProject.logicx && auxin doctor")]
    Doctor,

    /// Manage remote repositories
    #[command(subcommand)]
    Remote(RemoteCommands),
}

#[derive(Subcommand)]
enum CommentCommands {
    /// Add a comment to a commit
    #[command(long_about = "Add a comment to a commit

USAGE:
    auxin comment add <COMMIT_ID> <TEXT>

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
    auxin comment add HEAD \"Vocals need more reverb\"

    # Add comment to specific commit
    auxin comment add abc123 \"Great mix on this version!\"")]
    Add {
        #[arg(value_name = "COMMIT_ID", help = "Commit ID or HEAD")]
        commit_id: String,

        #[arg(value_name = "TEXT", help = "Comment text")]
        text: String,
    },

    /// List comments on a commit
    #[command(long_about = "List comments on a commit

USAGE:
    auxin comment list [COMMIT_ID]

DESCRIPTION:
    Shows all comments on a specific commit. If no commit ID is provided,
    shows comments on the latest commit (HEAD).

EXAMPLES:
    # Show comments on latest commit
    auxin comment list

    # Show comments on specific commit
    auxin comment list abc123")]
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
    auxin queue status

DESCRIPTION:
    Displays all pending operations that are queued due to network unavailability.
    Shows operation type, priority, queued time, and number of retry attempts.

EXAMPLES:
    # Show all pending operations
    auxin queue status")]
    Status,

    /// Manually sync all pending operations
    #[command(long_about = "Manually sync all pending operations

USAGE:
    auxin queue sync

DESCRIPTION:
    Attempts to execute all pending operations in the queue. Operations are
    executed in priority order (highest first), then by age (oldest first).

    Only executes if network connectivity is available. Failed operations
    remain in the queue for retry.

EXAMPLES:
    # Sync all pending operations
    auxin queue sync")]
    Sync,

    /// Clear completed operations from the queue
    #[command(long_about = "Clear completed operations from the queue

USAGE:
    auxin queue clear [--all]

DESCRIPTION:
    Removes completed operations from the queue to free up disk space.
    Use --all to remove ALL operations (including pending).

EXAMPLES:
    # Clear only completed operations
    auxin queue clear

    # Clear all operations (pending and completed)
    auxin queue clear --all")]
    Clear {
        #[arg(long, help = "Clear all operations including pending")]
        all: bool,
    },

    /// Remove a specific operation from the queue
    #[command(long_about = "Remove a specific operation from the queue

USAGE:
    auxin queue remove <ENTRY_ID>

DESCRIPTION:
    Removes a specific operation from the queue by its entry ID.
    Use 'queue status' to see entry IDs.

EXAMPLES:
    # Remove a specific operation
    auxin queue remove 01234567-89ab-cdef-0123-456789abcdef")]
    Remove {
        #[arg(value_name = "ENTRY_ID", help = "Queue entry ID to remove")]
        entry_id: String,
    },
}

#[derive(Subcommand)]
enum HistoryCommands {
    /// View recent operation history
    #[command(long_about = "View recent operation history

USAGE:
    auxin history view [--limit <N>]
    auxin history view --repo <PATH>

DESCRIPTION:
    Displays a timeline of all operations performed, including:
      • Lock operations (acquire, release, renew, break)
      • Commits and pushes
      • Authentication events
      • Comments and activity views
      • Success/failure status for each operation

    Operations are stored persistently and survive restarts.

OPTIONS:
    --limit <N>     Number of recent operations to show (default: 20)
    --repo <PATH>   Filter by repository path

EXAMPLES:
    # Show last 20 operations
    auxin history view

    # Show last 50 operations
    auxin history view --limit 50

    # Show operations for specific repository
    auxin history view --repo /path/to/project.logicx")]
    View {
        #[arg(long, default_value = "20", help = "Number of operations to show")]
        limit: usize,

        #[arg(long, value_name = "PATH", help = "Filter by repository path")]
        repo: Option<PathBuf>,
    },

    /// Export history to CSV file
    #[command(long_about = "Export history to CSV file

USAGE:
    auxin history export <OUTPUT_FILE>

DESCRIPTION:
    Exports complete operation history to a CSV file for analysis,
    compliance, or reporting. CSV includes:
      • Timestamp
      • Operation type
      • User and machine
      • Result (success/failure)
      • Repository path

EXAMPLES:
    # Export to CSV
    auxin history export operations.csv

    # Export and open in Excel
    auxin history export report.csv && open report.csv")]
    Export {
        #[arg(value_name = "OUTPUT_FILE", help = "CSV file to write")]
        output: PathBuf,
    },

    /// Show operation statistics
    #[command(long_about = "Show operation statistics

USAGE:
    auxin history stats

DESCRIPTION:
    Displays statistics about all operations:
      • Total operations
      • Success rate
      • Lock operation count
      • Network operation count
      • Failure breakdown

EXAMPLES:
    # View statistics
    auxin history stats")]
    Stats,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Get smart suggestions based on repository state
    #[command(long_about = "Get smart suggestions based on repository state

USAGE:
    auxin workflow suggest [PATH]

DESCRIPTION:
    Analyzes current repository state and provides context-aware
    suggestions for next actions. Checks:
      • Lock status and expiration
      • Recent operation failures
      • Uncommitted changes
      • Recommended workflows

EXAMPLES:
    # Get suggestions for current directory
    auxin workflow suggest

    # Get suggestions for specific project
    auxin workflow suggest /path/to/project.logicx")]
    Suggest {
        #[arg(value_name = "PATH", help = "Repository path (default: current directory)")]
        path: Option<PathBuf>,
    },

    /// Run lock renewal daemon (keeps lock alive)
    #[command(long_about = "Run lock renewal daemon (keeps lock alive)

USAGE:
    auxin workflow lock-daemon <PATH>

DESCRIPTION:
    Starts a background daemon that automatically renews your lock
    before it expires. Useful for long editing sessions (>4 hours).

    The daemon:
      • Checks every 15 minutes
      • Renews when <60 minutes remaining
      • Records all renewals in history
      • Stops when lock is released

OPTIONS:
    Daemon runs in foreground. Use '&' to run in background:
    auxin workflow lock-daemon . &

EXAMPLES:
    # Start lock renewal daemon
    auxin workflow lock-daemon .

    # Run in background
    auxin workflow lock-daemon . &")]
    LockDaemon {
        #[arg(value_name = "PATH", help = "Repository path")]
        path: PathBuf,
    },

    /// Show current workflow configuration
    #[command(long_about = "Show current workflow configuration

USAGE:
    auxin workflow config

DESCRIPTION:
    Displays current workflow automation settings:
      • Auto-lock renewal (enabled/disabled)
      • Lock check interval
      • Lock renew threshold
      • Auto-pull on startup
      • Auto-push after commit
      • Confirmation prompts
      • Dry-run mode

    Configuration file: ~/.auxin/workflow_config.json

EXAMPLES:
    # View configuration
    auxin workflow config")]
    Config,
}

#[derive(Subcommand)]
enum SnapshotCommands {
    /// Create a manual backup snapshot
    #[command(long_about = "Create a manual backup snapshot

USAGE:
    auxin snapshot create <PATH> [DESCRIPTION]

DESCRIPTION:
    Creates a manual backup snapshot of the repository state.
    Snapshots include:
      • Current commit ID
      • Timestamp
      • Repository path
      • User description

    Automatic snapshots are also created before:
      • Push operations
      • Pull operations
      • Lock break operations
      • Rollback operations

EXAMPLES:
    # Create snapshot with description
    auxin snapshot create . \"Before major refactor\"

    # Create snapshot without description
    auxin snapshot create /path/to/project.logicx")]
    Create {
        #[arg(value_name = "PATH", help = "Repository path")]
        path: PathBuf,

        #[arg(value_name = "DESCRIPTION", help = "Optional description")]
        description: Option<String>,
    },

    /// List all backup snapshots
    #[command(long_about = "List all backup snapshots

USAGE:
    auxin snapshot list [OPTIONS]

DESCRIPTION:
    Lists all backup snapshots with:
      • Snapshot ID
      • Type (manual, auto-before-push, etc.)
      • Timestamp
      • Description
      • Associated commit ID
      • Repository path

OPTIONS:
    --all           Show all snapshots (default: last 20)
    --repo <PATH>   Filter by repository

EXAMPLES:
    # List recent snapshots
    auxin snapshot list

    # List all snapshots
    auxin snapshot list --all

    # List snapshots for specific repository
    auxin snapshot list --repo /path/to/project.logicx")]
    List {
        #[arg(long, help = "Show all snapshots")]
        all: bool,

        #[arg(long, value_name = "PATH", help = "Filter by repository")]
        repo: Option<PathBuf>,
    },

    /// Get restore instructions for a snapshot
    #[command(long_about = "Get restore instructions for a snapshot

USAGE:
    auxin snapshot restore <SNAPSHOT_ID>

DESCRIPTION:
    Displays step-by-step instructions for restoring from a snapshot.
    Does NOT actually perform the restore (dry-run only).

    Instructions include:
      • Snapshot details
      • Commit to restore
      • Commands to run
      • Warnings about uncommitted changes

EXAMPLES:
    # Get restore instructions
    auxin snapshot restore abc123def456")]
    Restore {
        #[arg(value_name = "SNAPSHOT_ID", help = "Snapshot ID to restore from")]
        snapshot_id: String,
    },

    /// Delete a snapshot
    #[command(long_about = "Delete a snapshot

USAGE:
    auxin snapshot delete <SNAPSHOT_ID>

DESCRIPTION:
    Permanently deletes a snapshot. Cannot be undone.

    Note: Automatic cleanup keeps only the 50 most recent snapshots.

EXAMPLES:
    # Delete snapshot
    auxin snapshot delete abc123def456")]
    Delete {
        #[arg(value_name = "SNAPSHOT_ID", help = "Snapshot ID to delete")]
        snapshot_id: String,
    },
}

#[derive(Subcommand)]
enum RecoveryCommands {
    /// Show recovery guide for failed push
    #[command(long_about = "Show recovery guide for failed push

USAGE:
    auxin recovery push

DESCRIPTION:
    Displays step-by-step recovery guide for push failures.

    Common push failure causes:
      • Network connectivity issues
      • Authentication problems
      • Missing lock
      • Diverged branches

EXAMPLES:
    # Show push recovery guide
    auxin recovery push")]
    Push,

    /// Show recovery guide for failed pull
    #[command(long_about = "Show recovery guide for failed pull

USAGE:
    auxin recovery pull

DESCRIPTION:
    Displays step-by-step recovery guide for pull failures.

    Common pull failure causes:
      • Network connectivity issues
      • Authentication problems
      • Uncommitted local changes
      • Merge conflicts

EXAMPLES:
    # Show pull recovery guide
    auxin recovery pull")]
    Pull,

    /// Show recovery guide for lock conflicts
    #[command(long_about = "Show recovery guide for lock conflicts

USAGE:
    auxin recovery lock

DESCRIPTION:
    Displays step-by-step recovery guide for lock conflicts.

    Common lock conflict scenarios:
      • Lock held by another user
      • Expired lock not auto-released
      • Stale lock after crash

EXAMPLES:
    # Show lock recovery guide
    auxin recovery lock")]
    Lock,
}

#[derive(Subcommand)]
enum RemoteCommands {
    /// Add a remote repository
    #[command(long_about = "Add a remote repository

USAGE:
    auxin remote add <NAME> <URL>

DESCRIPTION:
    Adds a remote repository URL. This is typically your Oxen Hub repository
    where you push and pull changes for collaboration.

EXAMPLES:
    # Add Oxen Hub remote
    auxin remote add origin https://hub.oxen.ai/username/myproject")]
    Add {
        #[arg(value_name = "NAME", help = "Name for the remote (typically 'origin')")]
        name: String,

        #[arg(value_name = "URL", help = "URL of the remote repository")]
        url: String,
    },

    /// List configured remotes
    #[command(long_about = "List configured remotes

USAGE:
    auxin remote list

DESCRIPTION:
    Shows all configured remote repositories for the current project.

EXAMPLES:
    # List all remotes
    auxin remote list")]
    List,

    /// Remove a remote
    #[command(long_about = "Remove a remote

USAGE:
    auxin remote remove <NAME>

DESCRIPTION:
    Removes a remote repository configuration.

EXAMPLES:
    # Remove a remote
    auxin remote remove origin")]
    Remove {
        #[arg(value_name = "NAME", help = "Name of the remote to remove")]
        name: String,
    },
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Enable verbose logging if requested
    logger::set_verbose(cli.verbose);

    match cli.command {
        Commands::Init { path, r#type, logic } => {
            vlog!("Starting initialization for path: {}", path.display());

            // Determine project type (handle backward compatibility with --logic flag)
            let project_type = if logic {
                vlog!("Using legacy --logic flag, treating as LogicPro");
                ProjectType::LogicPro
            } else if let Some(type_str) = r#type {
                ProjectType::parse(&type_str).unwrap_or_else(|| {
                    progress::error(&format!("Unknown project type: {}. Supported types: auto, logicpro, sketchup", type_str));
                    std::process::exit(1);
                })
            } else {
                // Auto-detect based on file extension
                let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                match extension {
                    "logicx" => {
                        vlog!("Auto-detected Logic Pro project (.logicx)");
                        ProjectType::LogicPro
                    }
                    "skp" => {
                        vlog!("Auto-detected SketchUp project (.skp)");
                        ProjectType::SketchUp
                    }
                    _ => {
                        vlog!("No specific project type detected, using generic init");
                        ProjectType::Auto
                    }
                }
            };

            vlog!("Project type: {:?}", project_type);

            match project_type {
                ProjectType::LogicPro => {
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
                    println!("  • Create milestone commits: auxin commit -m \"Your message\" --bpm 120");
                    println!("  • View history: auxin log");
                    println!("  • Restore to any commit: auxin restore <commit-id>");
                }
                ProjectType::SketchUp => {
                    let pb = progress::spinner("Validating SketchUp project structure...");
                    vlog!("Detecting SketchUp project...");

                    // Validate SketchUp project
                    let _skp_project = SketchUpProject::detect(&path)?;

                    vlog!("Initializing SketchUp project repository...");
                    let _repo = OxenRepository::init(&path).await?;

                    progress::finish_success(&pb, "SketchUp project repository initialized");
                    println!();
                    progress::success(&format!("Repository created at: {}", path.display()));
                    progress::success("Initial commit created on main branch");
                    progress::success("Draft branch created and checked out");
                    println!();
                    progress::info("You're all set! Start working in SketchUp:");
                    println!("  • Changes will be automatically tracked on the draft branch");
                    println!("  • Create milestone commits: auxin commit -m \"Your message\" --units Inches --layers 10");
                    println!("  • View history: auxin log");
                    println!("  • Restore to any commit: auxin restore <commit-id>");
                }
                ProjectType::Blender => {
                    let pb = progress::spinner("Validating Blender project structure...");
                    vlog!("Detecting Blender project...");

                    // Validate Blender project
                    let _blend_project = BlenderProject::detect(&path)?;

                    vlog!("Initializing Blender project repository...");
                    let _repo = OxenRepository::init(&path).await?;

                    progress::finish_success(&pb, "Blender project repository initialized");
                    println!();
                    progress::success(&format!("Repository created at: {}", path.display()));
                    progress::success("Initial commit created on main branch");
                    progress::success("Draft branch created and checked out");
                    println!();
                    progress::info("You're all set! Start working in Blender:");
                    println!("  • Changes will be automatically tracked on the draft branch");
                    println!("  • Create milestone commits: auxin commit -m \"Your message\"");
                    println!("  • View history: auxin log");
                    println!("  • Restore to any commit: auxin restore <commit-id>");
                }
                ProjectType::Auto => {
                    let pb = progress::spinner(&format!("Initializing Oxen repository at {}...", path.display()));
                    vlog!("Initializing generic Oxen repository...");
                    let _repo = OxenRepository::init(&path).await?;

                    progress::finish_success(
                        &pb,
                        &format!("Oxen repository initialized at: {}", path.display())
                    );
                }
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
                progress::info("Next step: auxin commit -m \"Your message\"");
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
            units,
            layers,
            components,
            groups,
            file_size,
            tags,
            bounce,
        } => {
            let pb = progress::spinner("Preparing commit...");
            let repo = OxenRepository::new(".");

            // Detect if we're using Logic Pro or SketchUp metadata
            let has_logic_metadata = bpm.is_some() || sample_rate.is_some() || key.is_some();
            let has_sketchup_metadata = units.is_some() || layers.is_some() || components.is_some()
                || groups.is_some() || file_size.is_some();

            let formatted_message = if has_sketchup_metadata {
                // SketchUp project - use SketchUpMetadata
                vlog!("Using SketchUp metadata");
                let mut skp_metadata = SketchUpMetadata::new(message.clone());

                if let Some(ref units_val) = units {
                    skp_metadata = skp_metadata.with_units(units_val.clone());
                }

                if let Some(layer_count) = layers {
                    skp_metadata = skp_metadata.with_layer_count(layer_count);
                }

                if let Some(component_count) = components {
                    skp_metadata = skp_metadata.with_component_count(component_count);
                }

                if let Some(group_count) = groups {
                    skp_metadata = skp_metadata.with_group_count(group_count);
                }

                if let Some(size) = file_size {
                    skp_metadata = skp_metadata.with_file_size(size);
                }

                if let Some(ref tags_str) = tags {
                    for tag in tags_str.split(',') {
                        skp_metadata = skp_metadata.with_tag(tag.trim());
                    }
                }

                skp_metadata.format_commit_message()
            } else if has_logic_metadata {
                // Logic Pro project - use CommitMetadata
                vlog!("Using Logic Pro metadata");
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

                metadata.format_commit_message()
            } else {
                // No metadata, just use message
                vlog!("No metadata provided, using plain message");
                if let Some(ref tags_str) = tags {
                    let mut msg = message.clone();
                    msg.push_str(&format!("\n\nTags: {}", tags_str));
                    msg
                } else {
                    message.clone()
                }
            };

            pb.set_message("Creating commit...");
            let commit_metadata = CommitMetadata::new(formatted_message);
            let commit_id = repo.create_commit(commit_metadata).await?;

            progress::finish_success(&pb, &format!("Commit created: {}", commit_id));

            // Store metadata on server if configured
            let config = Config::load().unwrap_or_default();
            if config.server.use_server_metadata {
                let server_config = ServerConfig {
                    url: config.server.url.clone(),
                    token: config.server.token.clone(),
                    timeout_secs: config.server.timeout_secs,
                };

                if let Ok(client) = AuxinServerClient::new(server_config) {
                    let namespace = config.server.default_namespace.clone()
                        .unwrap_or_else(|| "default".to_string());
                    let current_dir = std::env::current_dir()?;
                    let repo_name = current_dir.file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());

                    // Build server metadata
                    let server_metadata = auxin::server_client::LogicProMetadata {
                        bpm: bpm.map(|b| b as f64),
                        sample_rate,
                        key_signature: key.clone(),
                        tags: tags.as_ref().map(|t| t.split(',').map(|s| s.trim().to_string()).collect()),
                        custom: None,
                    };

                    match client.store_metadata(&namespace, &repo_name, &commit_id, &server_metadata) {
                        Ok(()) => {
                            vlog!("Metadata stored on server for commit {}", commit_id);
                        }
                        Err(e) => {
                            vlog!("Failed to store metadata on server: {}", e);
                        }
                    }
                }
            }

            // Show commit details
            println!();
            progress::info("Commit Details:");
            println!("  Message: {}", message);

            // Show Logic Pro metadata
            if let Some(ref bpm_val) = bpm {
                println!("  BPM: {}", bpm_val);
            }
            if let Some(ref sr_val) = sample_rate {
                println!("  Sample Rate: {} Hz", sr_val);
            }
            if let Some(ref key_val) = key {
                println!("  Key: {}", key_val);
            }

            // Show SketchUp metadata
            if let Some(ref units_val) = units {
                println!("  Units: {}", units_val);
            }
            if let Some(layer_count) = layers {
                println!("  Layers: {}", layer_count);
            }
            if let Some(component_count) = components {
                println!("  Components: {}", component_count);
            }
            if let Some(group_count) = groups {
                println!("  Groups: {}", group_count);
            }
            if let Some(size) = file_size {
                let size_mb = size as f64 / (1024.0 * 1024.0);
                println!("  File Size: {:.2} MB", size_mb);
            }

            // Show common metadata
            if let Some(ref tags_val) = tags {
                println!("  Tags: {}", tags_val);
            }

            // Process bounce file if provided
            if let Some(bounce_path) = bounce {
                let current_dir = std::env::current_dir()?;
                let bounce_manager = BounceManager::new(&current_dir);

                let pb = progress::spinner("Adding bounce file...");
                match bounce_manager.add_bounce(&commit_id, &bounce_path, None) {
                    Ok(metadata) => {
                        progress::finish_success(&pb, "Bounce added");
                        println!("  Bounce: {} ({}, {})",
                            metadata.original_filename,
                            metadata.format_duration(),
                            metadata.format_size());
                    }
                    Err(e) => {
                        progress::finish_error(&pb, "Failed to add bounce");
                        warn!("Bounce error: {}", e);
                    }
                }
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
                println!("  auxin add --all");
                println!("  auxin commit -m \"Initial commit\"");
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
            println!("  auxin add --all");
            println!("  auxin commit -m \"Your message\"");

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

            // Check for pending sync operations
            {
                use auxin::OfflineQueue;
                if let Ok(queue) = OfflineQueue::new() {
                    let pending_count = queue.pending().len();
                    if pending_count > 0 {
                        println!("│                                                          │");
                        println!("│  {} {} pending sync operation(s)                        │",
                            "⟳".yellow(),
                            pending_count.to_string().yellow()
                        );
                    }
                }
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
                    progress::info("Next step: auxin add --all");
                } else if !status.staged.is_empty() {
                    progress::info("Next step: auxin commit -m \"Your message\"");
                }
            }

            // Hint about pending sync operations
            {
                use auxin::OfflineQueue;
                if let Ok(queue) = OfflineQueue::new() {
                    let pending_count = queue.pending().len();
                    if pending_count > 0 {
                        progress::warning(&format!("{} operation(s) pending sync", pending_count));
                        progress::info("Sync with: auxin queue sync");
                    }
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
                progress::info(&format!("Use 'auxin restore {}' to restore to this commit", &commit.id[..7.min(commit.id.len())]));
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
                progress::info("Some changes are already staged. Use 'auxin status' for details");
            }

            Ok(())
        }

        Commands::Compare {
            commit_a,
            commit_b,
            format,
            plain,
        } => {
            use auxin::CommitMetadata;

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
                _ => {
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
            use auxin::search::SearchEngine;

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
                _ => {
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
            use auxin::lock_integration;
            use auxin::network_resilience::{check_connectivity, ConnectivityState};
            use auxin::{OfflineQueue, OfflineQueuedOperation};
            use std::env;

            let current_dir = env::current_dir()?;

            // Auto-sync pending queue if online (for all lock commands)
            if check_connectivity() == ConnectivityState::Online {
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

            match lock_cmd {
                LockCommands::Acquire { timeout } => {
                    // Load config to check if server locks are enabled
                    let config = Config::load().unwrap_or_default();

                    if config.server.use_server_locks {
                        // Use server-based locking
                        let server_config = ServerConfig {
                            url: config.server.url.clone(),
                            token: config.server.token.clone(),
                            timeout_secs: config.server.timeout_secs,
                        };

                        match AuxinServerClient::new(server_config) {
                            Ok(client) => {
                                let user = server_client::get_user_identifier();
                                let machine_id = server_client::get_machine_id();

                                // Get namespace/name from config or current directory
                                let namespace = config.server.default_namespace.clone()
                                    .unwrap_or_else(|| "default".to_string());
                                let repo_name = current_dir.file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                let pb = progress::spinner("Acquiring server lock...");
                                match client.acquire_lock(&namespace, &repo_name, &user, &machine_id, timeout as u32) {
                                    Ok(lock) => {
                                        progress::finish_success(&pb, "Lock acquired via server");
                                        println!();
                                        success!("Lock acquired successfully");
                                        println!("  {} {}", "Lock ID:".dimmed(), lock.lock_id.cyan());
                                        println!("  {} {}", "User:".dimmed(), lock.user.dimmed());
                                        println!("  {} {}", "Expires:".dimmed(), lock.expires_at.dimmed());
                                    }
                                    Err(e) => {
                                        progress::finish_error(&pb, "Failed to acquire lock");
                                        anyhow::bail!("Server lock error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to connect to server, falling back to local lock");
                                vlog!("Server error: {}", e);
                                lock_integration::handle_lock_acquire(&current_dir, timeout)?;
                            }
                        }
                    } else {
                        // Use local locking (original behavior)
                        // Check connectivity
                        match check_connectivity() {
                            ConnectivityState::Offline => {
                                // Queue the operation
                                let mut queue = OfflineQueue::new()?;
                                let user_id = lock_integration::get_user_identifier();

                                let entry_id = queue.enqueue_with_priority(OfflineQueuedOperation::AcquireLock {
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
                                progress::info("Use 'auxin queue sync' to retry manually");
                            }
                            _ => {
                                // Execute normally (online or unknown)
                                lock_integration::handle_lock_acquire(&current_dir, timeout)?;
                            }
                        }
                    }
                }

                LockCommands::Release => {
                    // Load config to check if server locks are enabled
                    let config = Config::load().unwrap_or_default();

                    if config.server.use_server_locks {
                        // Use server-based locking
                        let server_config = ServerConfig {
                            url: config.server.url.clone(),
                            token: config.server.token.clone(),
                            timeout_secs: config.server.timeout_secs,
                        };

                        match AuxinServerClient::new(server_config) {
                            Ok(client) => {
                                let user = server_client::get_user_identifier();
                                let machine_id = server_client::get_machine_id();

                                // Get namespace/name from config or current directory
                                let namespace = config.server.default_namespace.clone()
                                    .unwrap_or_else(|| "default".to_string());
                                let repo_name = current_dir.file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                // First get the current lock status to get lock_id
                                let pb = progress::spinner("Releasing server lock...");
                                match client.get_lock_status(&namespace, &repo_name) {
                                    Ok(status) => {
                                        if let Some(lock) = status.lock {
                                            match client.release_lock(&namespace, &repo_name, &lock.lock_id, &user, &machine_id) {
                                                Ok(()) => {
                                                    progress::finish_success(&pb, "Lock released via server");
                                                    success!("Lock released successfully");
                                                }
                                                Err(e) => {
                                                    progress::finish_error(&pb, "Failed to release lock");
                                                    anyhow::bail!("Server release error: {}", e);
                                                }
                                            }
                                        } else {
                                            progress::finish_error(&pb, "No lock to release");
                                            warn!("No active lock found for this repository");
                                        }
                                    }
                                    Err(e) => {
                                        progress::finish_error(&pb, "Failed to get lock status");
                                        anyhow::bail!("Server error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to connect to server, falling back to local lock");
                                vlog!("Server error: {}", e);
                                lock_integration::handle_lock_release(&current_dir)?;
                            }
                        }
                    } else {
                        // Use local locking (original behavior)
                        // Check connectivity
                        match check_connectivity() {
                            ConnectivityState::Offline => {
                                // Queue the operation
                                let mut queue = OfflineQueue::new()?;

                                // We don't know the lock_id when offline, so use a placeholder
                                // The execute_entry will look up the actual lock
                                let entry_id = queue.enqueue_with_priority(OfflineQueuedOperation::ReleaseLock {
                                    project_path: current_dir.to_string_lossy().to_string(),
                                    lock_id: "pending".to_string(), // Will be looked up during execution
                                }, 100)?; // High priority

                                warn!("Network is offline - operation queued");
                                println!();
                                println!("  {} {}", "Queued:".bold(), "Release lock".yellow());
                                println!("  {} {}", "Entry ID:".dimmed(), &entry_id[..8].dimmed());
                                println!();
                                progress::info("Lock will be released when network is available");
                                progress::info("Use 'auxin queue sync' to retry manually");
                            }
                            _ => {
                                // Execute normally (online or unknown)
                                lock_integration::handle_lock_release(&current_dir)?;
                            }
                        }
                    }
                }

                LockCommands::Status => {
                    // Load config to check if server locks are enabled
                    let config = Config::load().unwrap_or_default();

                    if config.server.use_server_locks {
                        // Use server-based locking
                        let server_config = ServerConfig {
                            url: config.server.url.clone(),
                            token: config.server.token.clone(),
                            timeout_secs: config.server.timeout_secs,
                        };

                        match AuxinServerClient::new(server_config) {
                            Ok(client) => {
                                // Get namespace/name from config or current directory
                                let namespace = config.server.default_namespace.clone()
                                    .unwrap_or_else(|| "default".to_string());
                                let repo_name = current_dir.file_name()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                let pb = progress::spinner("Checking server lock status...");

                                // ureq is a blocking client, safe to call directly
                                match client.get_lock_status(&namespace, &repo_name) {
                                    Ok(status) => {
                                        progress::finish_success(&pb, "Lock status retrieved");
                                        println!();
                                        if status.locked {
                                            if let Some(lock) = status.lock {
                                                println!("{}", "🔒 Repository is LOCKED".red().bold());
                                                println!();
                                                println!("  {} {}", "Lock ID:".dimmed(), lock.lock_id.cyan());
                                                println!("  {} {}", "Held by:".dimmed(), lock.user.yellow());
                                                println!("  {} {}", "Machine:".dimmed(), lock.machine_id.dimmed());
                                                println!("  {} {}", "Acquired:".dimmed(), lock.acquired_at.dimmed());
                                                println!("  {} {}", "Expires:".dimmed(), lock.expires_at.yellow());
                                                println!("  {} {}", "Last seen:".dimmed(), lock.last_heartbeat.dimmed());
                                            }
                                        } else {
                                            println!("{}", "🔓 Repository is UNLOCKED".green().bold());
                                            println!();
                                            progress::info("You can acquire a lock with: auxin lock acquire");
                                        }
                                    }
                                    Err(e) => {
                                        progress::finish_error(&pb, "Failed to get lock status");
                                        anyhow::bail!("Server error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to connect to server, falling back to local lock status");
                                vlog!("Server error: {}", e);
                                lock_integration::handle_lock_status(&current_dir)?;
                            }
                        }
                    } else {
                        // Status check always executes (even when offline, shows local state)
                        lock_integration::handle_lock_status(&current_dir)?;
                    }
                }

                LockCommands::Break { force } => {
                    // Break force always executes (administrative override)
                    lock_integration::handle_lock_break(&current_dir, force)?;
                }
            }

            Ok(())
        }

        Commands::Auth(auth_cmd) => {
            use auxin::AuthManager;

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
                            progress::info("Test authentication with: auxin auth test");
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
                            progress::info("Run 'auxin auth test' to verify connection");
                        }
                        Ok(None) => {
                            println!("│  Status: {} Not authenticated                        │", "○".yellow());
                            println!("│                                                          │");
                            println!("│  You need to login to use remote features               │");
                            println!("│                                                          │");
                            println!("└──────────────────────────────────────────────────────────┘");
                            println!();
                            progress::info("Login with: auxin auth login");
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
                            progress::info("Try logging in again: auxin auth login");
                            std::process::exit(1);
                        }
                    }
                }
            }

            Ok(())
        }

        Commands::Server(server_cmd) => {
            let config = Config::load().unwrap_or_default();

            match server_cmd {
                ServerCommands::Status => {
                    // Truncate URL if too long
                    let url_display = if config.server.url.len() > 43 {
                        format!("{}...", &config.server.url[..40])
                    } else {
                        config.server.url.clone()
                    };

                    println!();
                    println!("┌─ Server Configuration ──────────────────────────────────┐");
                    println!("│                                                          │");
                    println!("│  URL:        {:<43} │", url_display);
                    println!("│  Namespace:  {:<43} │",
                        config.server.default_namespace.as_deref().unwrap_or("(none)"));
                    println!("│  Timeout:    {} seconds{:<34} │",
                        config.server.timeout_secs, "");
                    println!("│  Locks:      {:<43} │",
                        if config.server.use_server_locks { "enabled" } else { "disabled" });
                    println!("│  Metadata:   {:<43} │",
                        if config.server.use_server_metadata { "enabled" } else { "disabled" });
                    println!("│                                                          │");

                    // Check connection
                    let server_config = ServerConfig {
                        url: config.server.url.clone(),
                        token: config.server.token.clone(),
                        timeout_secs: config.server.timeout_secs,
                    };

                    if let Ok(client) = AuxinServerClient::new(server_config) {
                        match client.health_check() {
                            Ok(true) => {
                                println!("│  Status:     {} Connected{:<33} │", "●".green(), "");
                            }
                            Ok(false) | Err(_) => {
                                println!("│  Status:     {} Unreachable{:<31} │", "●".red(), "");
                            }
                        }
                    } else {
                        println!("│  Status:     {} Unknown{:<35} │", "●".yellow(), "");
                    }

                    println!("│                                                          │");
                    println!("└──────────────────────────────────────────────────────────┘");
                    println!();
                }

                ServerCommands::Health => {
                    let pb = progress::spinner("Testing server connection...");

                    let server_config = ServerConfig {
                        url: config.server.url.clone(),
                        token: config.server.token.clone(),
                        timeout_secs: config.server.timeout_secs,
                    };

                    match AuxinServerClient::new(server_config) {
                        Ok(client) => {
                            match client.health_check() {
                                Ok(true) => {
                                    progress::finish_success(&pb, "Server is healthy");
                                    println!();
                                    progress::success(&format!("Connected to {}", config.server.url));
                                }
                                Ok(false) => {
                                    progress::finish_error(&pb, "Server health check failed");
                                    println!();
                                    progress::error(&format!("Server at {} is not responding", config.server.url));
                                }
                                Err(e) => {
                                    progress::finish_error(&pb, "Connection failed");
                                    println!();
                                    progress::error(&format!("Failed to connect: {}", e));
                                }
                            }
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Client error");
                            println!();
                            progress::error(&format!("Failed to create client: {}", e));
                        }
                    }
                }

                ServerCommands::Set { key, value } => {
                    let mut config = Config::load().unwrap_or_default();

                    match key.as_str() {
                        "url" => {
                            config.server.url = value.clone();
                            progress::success(&format!("Set server URL to: {}", value));
                        }
                        "namespace" => {
                            config.server.default_namespace = Some(value.clone());
                            progress::success(&format!("Set default namespace to: {}", value));
                        }
                        "timeout" => {
                            match value.parse::<u64>() {
                                Ok(timeout) => {
                                    config.server.timeout_secs = timeout;
                                    progress::success(&format!("Set timeout to: {} seconds", timeout));
                                }
                                Err(_) => {
                                    progress::error("Invalid timeout value (must be a number)");
                                    std::process::exit(1);
                                }
                            }
                        }
                        "locks" => {
                            match value.to_lowercase().as_str() {
                                "true" | "on" | "yes" | "1" => {
                                    config.server.use_server_locks = true;
                                    progress::success("Server locks enabled");
                                }
                                "false" | "off" | "no" | "0" => {
                                    config.server.use_server_locks = false;
                                    progress::success("Server locks disabled");
                                }
                                _ => {
                                    progress::error("Invalid value for locks (use true/false)");
                                    std::process::exit(1);
                                }
                            }
                        }
                        "metadata" => {
                            match value.to_lowercase().as_str() {
                                "true" | "on" | "yes" | "1" => {
                                    config.server.use_server_metadata = true;
                                    progress::success("Server metadata storage enabled");
                                }
                                "false" | "off" | "no" | "0" => {
                                    config.server.use_server_metadata = false;
                                    progress::success("Server metadata storage disabled");
                                }
                                _ => {
                                    progress::error("Invalid value for metadata (use true/false)");
                                    std::process::exit(1);
                                }
                            }
                        }
                        _ => {
                            progress::error(&format!("Unknown configuration key: {}", key));
                            progress::info("Available keys: url, namespace, timeout, locks, metadata");
                            std::process::exit(1);
                        }
                    }

                    // Save to project config
                    if let Some(project_config_path) = Config::project_config_path() {
                        match config.save_to_file(&project_config_path) {
                            Ok(()) => {
                                progress::info(&format!("Saved to {}", project_config_path.display()));
                            }
                            Err(e) => {
                                progress::error(&format!("Failed to save config: {}", e));
                            }
                        }
                    }
                }
            }

            Ok(())
        }

        Commands::Bounce(bounce_cmd) => {
            // Find repository root
            let current_dir = std::env::current_dir()
                .context("Failed to get current directory")?;

            let manager = BounceManager::new(&current_dir);

            match bounce_cmd {
                BounceCommands::Add { file, commit, description } => {
                    // Get commit ID - use latest if not specified
                    let commit_id = match commit {
                        Some(id) => id,
                        None => {
                            // Get latest commit from oxen log
                            let subprocess = auxin::OxenSubprocess::new();
                            let commits = subprocess.log(&current_dir, Some(1))
                                .context("Failed to get latest commit")?;
                            if commits.is_empty() {
                                anyhow::bail!("No commits found. Create a commit first.");
                            }
                            commits[0].id.clone()
                        }
                    };

                    let pb = progress::spinner(&format!("Adding bounce for commit {}...", &commit_id[..8.min(commit_id.len())]));

                    match manager.add_bounce(&commit_id, &file, description.as_deref()) {
                        Ok(metadata) => {
                            progress::finish_success(&pb, "Bounce added");
                            println!();
                            println!("  Commit:    {}", &metadata.commit_id[..8.min(metadata.commit_id.len())]);
                            println!("  File:      {}", metadata.original_filename);
                            println!("  Format:    {:?}", metadata.format);
                            println!("  Size:      {}", metadata.format_size());
                            println!("  Duration:  {}", metadata.format_duration());
                            if let Some(desc) = &metadata.description {
                                println!("  Note:      {}", desc);
                            }
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to add bounce");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                BounceCommands::List => {
                    let bounces = manager.list_bounces()
                        .context("Failed to list bounces")?;

                    if bounces.is_empty() {
                        println!("No bounces found.");
                        println!();
                        println!("Add a bounce with: auxin bounce add <file>");
                    } else {
                        println!();
                        println!("┌─ Audio Bounces ─────────────────────────────────────────┐");
                        println!("│                                                          │");

                        for bounce in &bounces {
                            let commit_short = &bounce.commit_id[..8.min(bounce.commit_id.len())];
                            let duration = bounce.format_duration();
                            let size = bounce.format_size();

                            println!("│  {} {} │",
                                commit_short.yellow(),
                                " ".repeat(48 - commit_short.len()));
                            println!("│    File:     {:<41} │",
                                if bounce.original_filename.len() > 41 {
                                    format!("{}...", &bounce.original_filename[..38])
                                } else {
                                    bounce.original_filename.clone()
                                });
                            println!("│    Format:   {:<41} │", format!("{:?}", bounce.format));
                            println!("│    Duration: {:<41} │", duration);
                            println!("│    Size:     {:<41} │", size);

                            if let Some(desc) = &bounce.description {
                                let desc_display = if desc.len() > 41 {
                                    format!("{}...", &desc[..38])
                                } else {
                                    desc.clone()
                                };
                                println!("│    Note:     {:<41} │", desc_display);
                            }
                            println!("│                                                          │");
                        }

                        println!("└──────────────────────────────────────────────────────────┘");
                        println!();
                        println!("{} bounce(s) total", bounces.len());
                    }
                }

                BounceCommands::Play { commit_id } => {
                    let pb = progress::spinner(&format!("Playing bounce for {}...", &commit_id[..8.min(commit_id.len())]));

                    match manager.play_bounce(&commit_id) {
                        Ok(()) => {
                            progress::finish_success(&pb, "Playback complete");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Playback failed");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                BounceCommands::Info { commit_id } => {
                    match manager.get_bounce(&commit_id)? {
                        Some(metadata) => {
                            println!();
                            println!("┌─ Bounce Info ───────────────────────────────────────────┐");
                            println!("│                                                          │");
                            println!("│  Commit:      {:<42} │", &metadata.commit_id[..8.min(metadata.commit_id.len())]);
                            println!("│  File:        {:<42} │",
                                if metadata.original_filename.len() > 42 {
                                    format!("{}...", &metadata.original_filename[..39])
                                } else {
                                    metadata.original_filename.clone()
                                });
                            println!("│  Format:      {:<42} │", format!("{:?}", metadata.format));
                            println!("│  Size:        {:<42} │", metadata.format_size());
                            println!("│  Duration:    {:<42} │", metadata.format_duration());

                            if let Some(sr) = metadata.sample_rate {
                                println!("│  Sample Rate: {:<42} │", format!("{} Hz", sr));
                            }
                            if let Some(bd) = metadata.bit_depth {
                                println!("│  Bit Depth:   {:<42} │", format!("{}-bit", bd));
                            }
                            if let Some(ch) = metadata.channels {
                                let ch_str = if ch == 1 { "Mono".to_string() } else if ch == 2 { "Stereo".to_string() } else { format!("{} channels", ch) };
                                println!("│  Channels:    {:<42} │", ch_str);
                            }

                            println!("│  Added:       {:<42} │",
                                metadata.added_at.format("%Y-%m-%d %H:%M").to_string());
                            println!("│  By:          {:<42} │", metadata.added_by);

                            if let Some(desc) = &metadata.description {
                                println!("│                                                          │");
                                println!("│  Description:                                            │");
                                // Word wrap description
                                for chunk in desc.as_bytes().chunks(54) {
                                    let line = String::from_utf8_lossy(chunk);
                                    println!("│    {:<54} │", line);
                                }
                            }

                            println!("│                                                          │");
                            println!("└──────────────────────────────────────────────────────────┘");
                            println!();
                        }
                        None => {
                            anyhow::bail!("No bounce found for commit {}", commit_id);
                        }
                    }
                }

                BounceCommands::Delete { commit_id } => {
                    let pb = progress::spinner(&format!("Deleting bounce for {}...", &commit_id[..8.min(commit_id.len())]));

                    match manager.delete_bounce(&commit_id) {
                        Ok(()) => {
                            progress::finish_success(&pb, "Bounce deleted");
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Delete failed");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                BounceCommands::Search {
                    format,
                    pattern,
                    min_duration,
                    max_duration,
                    min_size,
                    max_size,
                    after,
                    before,
                    user,
                } => {
                    use auxin::BounceFilter;
                    use chrono::NaiveDate;

                    let pb = progress::spinner("Searching bounces...");

                    // Build filter
                    let mut filter = BounceFilter::default();

                    if let Some(fmt) = format {
                        filter.format = auxin::AudioFormat::from_extension(&fmt);
                        if filter.format.is_none() {
                            progress::finish_error(&pb, "Invalid format");
                            anyhow::bail!("Unknown audio format: {}", fmt);
                        }
                    }

                    filter.filename_pattern = pattern;
                    filter.min_duration = min_duration;
                    filter.max_duration = max_duration;
                    filter.min_size = min_size;
                    filter.max_size = max_size;
                    filter.added_by = user;

                    // Parse date filters
                    if let Some(after_str) = after {
                        match NaiveDate::parse_from_str(&after_str, "%Y-%m-%d") {
                            Ok(date) => {
                                filter.after = Some(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                            }
                            Err(_) => {
                                progress::finish_error(&pb, "Invalid date");
                                anyhow::bail!("Invalid date format: {}. Use YYYY-MM-DD", after_str);
                            }
                        }
                    }

                    if let Some(before_str) = before {
                        match NaiveDate::parse_from_str(&before_str, "%Y-%m-%d") {
                            Ok(date) => {
                                filter.before = Some(date.and_hms_opt(23, 59, 59).unwrap().and_utc());
                            }
                            Err(_) => {
                                progress::finish_error(&pb, "Invalid date");
                                anyhow::bail!("Invalid date format: {}. Use YYYY-MM-DD", before_str);
                            }
                        }
                    }

                    match manager.search_bounces(&filter) {
                        Ok(bounces) => {
                            progress::finish_success(&pb, &format!("Found {} bounces", bounces.len()));

                            if bounces.is_empty() {
                                println!("No bounces match the filter criteria");
                            } else {
                                for bounce in bounces {
                                    println!("\n{}", "─".repeat(50).dimmed());
                                    println!("{}: {}", "Commit".cyan(), &bounce.commit_id[..8.min(bounce.commit_id.len())]);
                                    println!("{}: {}", "File".cyan(), bounce.original_filename);
                                    println!("{}: {:?} | {} | {}",
                                        "Info".cyan(),
                                        bounce.format,
                                        bounce.format_duration(),
                                        bounce.format_size());
                                    println!("{}: {}", "Added".cyan(), bounce.added_at.format("%Y-%m-%d %H:%M"));
                                    if let Some(desc) = &bounce.description {
                                        println!("{}: {}", "Description".cyan(), desc);
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Search failed");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                BounceCommands::Compare { commit_a, commit_b } => {
                    let pb = progress::spinner(&format!(
                        "Comparing {} and {}...",
                        &commit_a[..8.min(commit_a.len())],
                        &commit_b[..8.min(commit_b.len())]
                    ));

                    match manager.compare_bounces(&commit_a, &commit_b) {
                        Ok(comparison) => {
                            progress::finish_success(&pb, "Comparison complete");
                            println!("\n{}", comparison.format_report());
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Comparison failed");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                BounceCommands::BatchAdd { files, commit } => {
                    let pb = progress::spinner(&format!("Adding {} bounce files...", files.len()));

                    // Get latest commit if no default commit specified
                    let default_commit = if let Some(c) = commit {
                        c
                    } else {
                        let subprocess = auxin::OxenSubprocess::new();
                        match subprocess.log(&current_dir, Some(1)) {
                            Ok(commits) if !commits.is_empty() => commits[0].id.clone(),
                            _ => {
                                progress::finish_error(&pb, "No commits found");
                                anyhow::bail!("No commits found. Please specify --commit or create a commit first.");
                            }
                        }
                    };

                    let mut success_count = 0;
                    let mut error_count = 0;

                    for file in &files {
                        // Try to extract commit ID from filename (pattern: commit_<id>_name.ext)
                        let commit_id = if let Some(filename) = file.file_name().and_then(|n| n.to_str()) {
                            if filename.starts_with("commit_") {
                                let parts: Vec<&str> = filename.splitn(3, '_').collect();
                                if parts.len() >= 2 {
                                    parts[1].to_string()
                                } else {
                                    default_commit.clone()
                                }
                            } else {
                                default_commit.clone()
                            }
                        } else {
                            default_commit.clone()
                        };

                        match manager.add_bounce(&commit_id, file, None) {
                            Ok(_) => {
                                success_count += 1;
                            }
                            Err(e) => {
                                error_count += 1;
                                eprintln!("{}: {}", file.display(), e);
                            }
                        }
                    }

                    if error_count == 0 {
                        progress::finish_success(&pb, &format!("Added {} bounces", success_count));
                    } else {
                        progress::finish_error(&pb, &format!("{} added, {} failed", success_count, error_count));
                    }
                }

                BounceCommands::BulkDelete { format, before, user, force } => {
                    use auxin::BounceFilter;
                    use chrono::NaiveDate;

                    // Require at least one filter
                    if format.is_none() && before.is_none() && user.is_none() {
                        anyhow::bail!("At least one filter (--format, --before, or --user) must be specified");
                    }

                    let pb = progress::spinner("Finding bounces to delete...");

                    // Build filter
                    let mut filter = BounceFilter::default();

                    if let Some(fmt) = &format {
                        filter.format = auxin::AudioFormat::from_extension(fmt);
                    }

                    if let Some(before_str) = &before {
                        match NaiveDate::parse_from_str(before_str, "%Y-%m-%d") {
                            Ok(date) => {
                                filter.before = Some(date.and_hms_opt(23, 59, 59).unwrap().and_utc());
                            }
                            Err(_) => {
                                progress::finish_error(&pb, "Invalid date");
                                anyhow::bail!("Invalid date format: {}. Use YYYY-MM-DD", before_str);
                            }
                        }
                    }

                    filter.added_by = user.clone();

                    // Find matching bounces
                    let bounces = match manager.search_bounces(&filter) {
                        Ok(b) => b,
                        Err(e) => {
                            progress::finish_error(&pb, "Search failed");
                            anyhow::bail!("{}", e);
                        }
                    };

                    if bounces.is_empty() {
                        progress::finish_success(&pb, "No matching bounces found");
                        return Ok(());
                    }

                    progress::finish_success(&pb, &format!("Found {} bounces", bounces.len()));

                    // Confirm deletion
                    if !force {
                        println!("\n{} bounces will be deleted:", bounces.len());
                        for bounce in &bounces {
                            println!("  • {} ({})", bounce.original_filename, &bounce.commit_id[..8.min(bounce.commit_id.len())]);
                        }
                        println!("\nUse --force to delete without confirmation.");
                        return Ok(());
                    }

                    // Delete bounces
                    let pb = progress::spinner(&format!("Deleting {} bounces...", bounces.len()));
                    let mut deleted = 0;

                    for bounce in &bounces {
                        if let Err(e) = manager.delete_bounce(&bounce.commit_id) {
                            eprintln!("Failed to delete {}: {}", bounce.commit_id, e);
                        } else {
                            deleted += 1;
                        }
                    }

                    progress::finish_success(&pb, &format!("Deleted {} bounces", deleted));
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
            use auxin::{LogicParser, MetadataDiffer};

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
                _ => {
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
            use auxin::daemon_client::DaemonClient;

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
                        progress::info("Start the daemon with: auxin daemon start");
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
            use auxin::hooks::{HookManager, HookType};

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
                    progress::info("Install built-in hooks with: auxin hooks install <name>");

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
                    progress::info("Install with: auxin hooks install <name>");

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
            use auxin::console::{Console, DaemonStatus as ConsoleDaemonStatus};
            use auxin::daemon_client::DaemonClient;

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
            let status = daemon_client.status().unwrap_or(auxin::daemon_client::DaemonStatus {
                is_running: false,
                pid: None,
                project_count: None,
                version: None,
                uptime: None,
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
            use auxin::ActivityFeed;
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
                println!("  auxin add --all");
                println!("  auxin commit -m \"Initial commit\" --bpm 120");
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
            use auxin::TeamManager;
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
            use auxin::OfflineQueue;
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

                        println!("  {}", "Use 'auxin queue sync' to sync pending operations".dimmed());
                    }

                    Ok(())
                }

                QueueCommands::Sync => {
                    use auxin::network_resilience::{check_connectivity, ConnectivityState};

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
            let bin_name = "auxin";

            generate(shell_type, &mut cmd, bin_name, &mut io::stdout());

            Ok(())
        }

        Commands::Comment(comment_cmd) => {
            use auxin::CommentManager;
            use std::env;

            let current_dir = env::current_dir()?;
            let comment_mgr = CommentManager::new();

            match comment_cmd {
                CommentCommands::Add { commit_id, text } => {
                    let user = lock_integration::get_user_identifier();

                    let pb = progress::spinner("Adding comment...");

                    match comment_mgr.add_comment(&current_dir, &commit_id, &user, &text) {
                        Ok(_comment) => {
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
                        println!("  auxin comment add {} \"Your comment here\"", commit);
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

        Commands::Push { remote, branch, force, verbose } => {
            use auxin::{ChunkedUploadManager, UploadConfig};

            let current_dir = std::env::current_dir()?;
            vlog!("Push from directory: {}", current_dir.display());

            // Initialize upload manager with config
            let mut config = UploadConfig::default();
            config.verbose = verbose || cli.verbose;

            let mut manager = ChunkedUploadManager::new(config)
                .context("Failed to initialize upload manager")?;

            // Determine remote and branch
            let remote_name = remote.unwrap_or_else(|| "origin".to_string());
            let branch_name = if let Some(b) = branch {
                b
            } else {
                // Get current branch
                let subprocess = auxin::OxenSubprocess::new();
                subprocess.current_branch(&current_dir)
                    .context("Failed to get current branch")?
            };

            // Show push info
            println!();
            println!("┌─ Push to Remote ────────────────────────────────────────┐");
            println!("│                                                          │");
            println!("│  {} {}:{}",
                "→".cyan(),
                remote_name.cyan().bold(),
                branch_name.cyan().bold()
            );
            println!("│                                                          │");
            println!("└──────────────────────────────────────────────────────────┘");
            println!();

            if force {
                progress::warning("Force push enabled - this will overwrite remote history");
            }

            // Check for pending local changes
            let subprocess = auxin::OxenSubprocess::new();
            if let Ok(status) = subprocess.status(&current_dir) {
                if !status.staged.is_empty() || !status.modified.is_empty() {
                    progress::warning("You have uncommitted changes");
                    progress::info("Consider committing before push: auxin commit -m \"message\"");
                    println!();
                }
            }

            // Check for existing upload session to resume
            let is_resuming = manager.has_resumable_session(&current_dir);
            if is_resuming {
                if let Some(session_info) = manager.get_resumable_session_info(&current_dir) {
                    progress::info(&format!("Resuming interrupted upload ({:.1}% complete)",
                        session_info.percentage
                    ));
                    println!();
                }
            }

            // Execute push with progress tracking
            progress::info("Starting push...");

            match manager.upload_with_progress(&current_dir, &remote_name, &branch_name, |_progress| {
                // Progress callback - could be used for real-time display in future
            }) {
                Ok(result) => {
                    println!();

                    if is_resuming {
                        progress::success("Upload resumed and completed successfully");
                    } else {
                        progress::success("Push completed successfully");
                    }

                    // Show statistics
                    if result.bytes_uploaded > 0 {
                        let size_mb = result.bytes_uploaded as f64 / (1024.0 * 1024.0);
                        let duration_secs = result.duration.as_secs_f64();
                        let speed_mbps = if duration_secs > 0.0 {
                            size_mb / duration_secs
                        } else {
                            0.0
                        };

                        println!();
                        println!("  {} Uploaded: {:.2} MB", "•".dimmed(), size_mb);
                        println!("  {} Duration: {:.1}s", "•".dimmed(), duration_secs);
                        println!("  {} Speed: {:.2} MB/s", "•".dimmed(), speed_mbps);

                        if result.files_uploaded > 0 {
                            println!("  {} Files: {}", "•".dimmed(), result.files_uploaded);
                        }
                    }

                    println!();
                    progress::info(&format!("Branch '{}' pushed to '{}'", branch_name, remote_name));

                    Ok(())
                }
                Err(e) => {
                    println!();
                    progress::error(&format!("Push failed: {}", e));

                    // Provide recovery hints
                    println!();
                    progress::info("To retry, run the same command again");
                    progress::info("Your progress has been saved and will resume automatically");

                    // Check if it's a network error
                    let error_str = e.to_string().to_lowercase();
                    if error_str.contains("network") || error_str.contains("connection") || error_str.contains("timeout") {
                        println!();
                        progress::info("Network issue detected. Check your connection and retry.");
                        progress::info("For recovery help: auxin recovery push");
                    } else if error_str.contains("auth") || error_str.contains("permission") || error_str.contains("401") || error_str.contains("403") {
                        println!();
                        progress::info("Authentication issue. Try: auxin login");
                    }

                    Err(e)
                }
            }
        }

        Commands::Doctor => {
            println!("\n{}", "Auxin Doctor - Environment Check".cyan().bold());
            println!("{}", "=".repeat(40).dimmed());

            let mut all_good = true;

            // 1. Check Oxen CLI
            print!("\n{} ", "Checking Oxen CLI...".cyan());
            let subprocess = auxin::OxenSubprocess::new();
            match subprocess.version() {
                Ok(version) => {
                    println!("{} {}", "✓".green(), version.trim());
                }
                Err(_) => {
                    println!("{} {}", "✗".red(), "Not found");
                    println!("  {} Install with: pip install oxen-ai", "→".yellow());
                    all_good = false;
                }
            }

            // 2. Check if in a repository
            let current_dir = std::env::current_dir()?;
            let oxen_dir = current_dir.join(".oxen");

            print!("{} ", "Checking repository...".cyan());
            if oxen_dir.exists() {
                println!("{} {}", "✓".green(), "Oxen repository found");

                // 3. Check for remotes
                print!("{} ", "Checking remotes...".cyan());
                match subprocess.remote_list(&current_dir) {
                    Ok(remotes) if !remotes.is_empty() => {
                        println!("{}", "✓".green());
                        for (name, url) in &remotes {
                            println!("  {} {} → {}", "•".dimmed(), name.cyan(), url);
                        }
                    }
                    Ok(_) => {
                        println!("{} {}", "⚠".yellow(), "No remotes configured");
                        println!("  {} Add with: auxin remote add origin <URL>", "→".yellow());
                        all_good = false;
                    }
                    Err(e) => {
                        println!("{} {}", "✗".red(), e);
                        all_good = false;
                    }
                }

                // 4. Check current branch
                print!("{} ", "Checking branch...".cyan());
                match subprocess.current_branch(&current_dir) {
                    Ok(branch) => {
                        println!("{} {}", "✓".green(), branch);
                    }
                    Err(e) => {
                        println!("{} {}", "✗".red(), e);
                        all_good = false;
                    }
                }

            } else {
                println!("{} {}", "⚠".yellow(), "Not in an Oxen repository");
                println!("  {} Initialize with: auxin init <path>", "→".yellow());
            }

            // 5. Check authentication (try to see if we can list remotes without error)
            print!("{} ", "Checking authentication...".cyan());
            // We can't easily check auth status without trying to connect
            // For now, just note that auth can be set up
            println!("{} {}", "?".yellow(), "Run 'auxin auth login' to authenticate");

            // Summary
            println!("\n{}", "─".repeat(40).dimmed());
            if all_good {
                println!("{}", "All checks passed! You're ready to use Auxin.".green().bold());
            } else {
                println!("{}", "Some issues found. See suggestions above.".yellow());
            }
            println!();

            Ok(())
        }

        Commands::Remote(cmd) => {
            let current_dir = std::env::current_dir()?;
            let subprocess = auxin::OxenSubprocess::new();

            match cmd {
                RemoteCommands::Add { name, url } => {
                    let pb = progress::spinner(&format!("Adding remote '{}'...", name));

                    match subprocess.remote_add(&current_dir, &name, &url) {
                        Ok(()) => {
                            progress::finish_success(&pb, &format!("Remote '{}' added", name));
                            println!("  {} → {}", name.cyan(), url);
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to add remote");
                            anyhow::bail!("{}", e);
                        }
                    }
                }

                RemoteCommands::List => {
                    match subprocess.remote_list(&current_dir) {
                        Ok(remotes) if !remotes.is_empty() => {
                            println!("\n{}", "Configured remotes:".cyan().bold());
                            for (name, url) in &remotes {
                                println!("  {} → {}", name.cyan(), url);
                            }
                            println!();
                        }
                        Ok(_) => {
                            println!("{}", "No remotes configured".yellow());
                            println!("Add one with: auxin remote add origin <URL>");
                        }
                        Err(e) => {
                            anyhow::bail!("Failed to list remotes: {}", e);
                        }
                    }
                }

                RemoteCommands::Remove { name } => {
                    let pb = progress::spinner(&format!("Removing remote '{}'...", name));

                    match subprocess.remote_remove(&current_dir, &name) {
                        Ok(()) => {
                            progress::finish_success(&pb, &format!("Remote '{}' removed", name));
                        }
                        Err(e) => {
                            progress::finish_error(&pb, "Failed to remove remote");
                            anyhow::bail!("{}", e);
                        }
                    }
                }
            }

            Ok(())
        }

        // TODO: Implement these command handlers
        Commands::History(_) | Commands::Workflow(_) | Commands::Snapshot(_) | Commands::Recovery(_) => {
            anyhow::bail!("This command is not yet implemented")
        }
    }
}
