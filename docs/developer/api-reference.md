# Auxin CLI API Reference

**Last Updated**: 2025-11-20
**Purpose**: Complete reference for all CLI commands, options, and return codes

---

## Overview

The `auxin` CLI provides version control specifically designed for creative applications. All commands follow the pattern:

```bash
auxin [OPTIONS] <COMMAND> [SUBCOMMAND] [ARGS]
```

**Global Options**:
- `-v, --verbose` - Enable verbose debug output
- `--help` - Show help for any command
- `--version` - Show version information

---

## Core Commands

### auxin init

Initialize a new Oxen repository for a project.

```bash
auxin init [--type <TYPE>] <PATH>
```

**Arguments**:
- `PATH` - Path to the project file or directory

**Options**:
- `--type <TYPE>` - Project type: `auto`, `logicpro`, `sketchup`, `blender` (default: auto)

**Examples**:
```bash
auxin init MyProject.logicx              # Auto-detect Logic Pro
auxin init --type sketchup MyModel.skp   # Explicit SketchUp
auxin init .                             # Current directory
```

**Exit Codes**:
- `0` - Success
- `1` - Invalid project structure
- `2` - Already initialized

---

### auxin add

Stage changes to be committed.

```bash
auxin add [--all] [PATHS...]
```

**Options**:
- `-a, --all` - Stage all changes in the repository

**Arguments**:
- `PATHS` - Specific files or directories to stage

**Examples**:
```bash
auxin add --all                          # Stage everything
auxin add projectData Resources/         # Stage specific paths
```

---

### auxin commit

Create a commit with optional project metadata.

```bash
auxin commit -m <MESSAGE> [OPTIONS]
```

**Required**:
- `-m, --message <MESSAGE>` - Commit message

**Logic Pro Metadata**:
- `--bpm <BPM>` - Beats per minute (tempo)
- `--sample-rate <HZ>` - Sample rate (44100, 48000, 96000)
- `--key <KEY>` - Key signature (e.g., "C Major", "A Minor")

**SketchUp Metadata**:
- `--units <UNITS>` - Model units (Inches, Feet, Meters, Millimeters)
- `--layers <N>` - Number of layers/tags
- `--components <N>` - Number of component instances
- `--groups <N>` - Number of groups
- `--file-size <BYTES>` - Model file size

**Common Options**:
- `--tags <TAGS>` - Comma-separated tags (e.g., "mixing,draft")
- `--bounce <FILE>` - Audio bounce file to attach

**Examples**:
```bash
# Logic Pro
auxin commit -m "Vocal tracking done" --bpm 120 --key "A Minor"

# SketchUp
auxin commit -m "Added materials" --units Feet --layers 15 --components 200
```

---

### auxin status

Show repository status.

```bash
auxin status
```

Shows:
- Staged files (ready to commit)
- Modified files (changed but not staged)
- Untracked files (new files)

---

### auxin log

Show commit history.

```bash
auxin log [OPTIONS]
```

**Options**:
- `-l, --limit <N>` - Maximum commits to display
- `--bpm <BPM>` - Filter by BPM
- `--tag <TAG>` - Filter by tag
- `--key <KEY>` - Filter by key signature
- `--since <DATE>` - Show commits since date (YYYY-MM-DD)

**Examples**:
```bash
auxin log --limit 10
auxin log --bpm 128 --tag vocals
auxin log --since "2025-01-01"
```

---

### auxin show

Show detailed information about a commit.

```bash
auxin show <COMMIT_ID>
```

Displays:
- Full commit message
- Audio/model metadata
- Author and timestamp
- Files changed

---

### auxin diff

Show changes between commits or working directory.

```bash
auxin diff [COMMIT_ID]
```

**Arguments**:
- `COMMIT_ID` - Compare against this commit (optional)

---

### auxin restore

Restore project to a previous commit.

```bash
auxin restore <COMMIT_ID>
```

**Warning**: Uncommitted changes will be lost.

---

### auxin compare

Compare metadata between two commits.

```bash
auxin compare <COMMIT_A> <COMMIT_B> [OPTIONS]
```

**Options**:
- `--format <FORMAT>` - Output format: `text`, `colored`, `json`, `compact`
- `--plain` - Disable colored output

---

### auxin search

Search commit history with advanced filtering.

```bash
auxin search <QUERY> [OPTIONS]
```

**Query Syntax**:
- `bpm:120-140` - BPM range
- `bpm:>120` - BPM greater than
- `key:minor` - Key contains "minor"
- `tag:mixing` - Has specific tag
- `message:vocals` - Message contains text

**Options**:
- `--ranked` - Sort by relevance score
- `--limit <N>` - Maximum results

**Examples**:
```bash
auxin search "bpm:120-140 key:minor tag:mixing"
auxin search "bpm:>128 tag:vocals,final" --ranked
```

---

## Lock Commands

### auxin lock acquire

Acquire exclusive lock for editing.

```bash
auxin lock acquire [--timeout <HOURS>]
```

**Options**:
- `--timeout <HOURS>` - Lock expiration time (default: 4)

---

### auxin lock release

Release the lock you currently hold.

```bash
auxin lock release
```

---

### auxin lock status

Show current lock status.

```bash
auxin lock status
```

Shows:
- Lock holder
- When acquired
- When expires
- Time remaining

---

### auxin lock break

Force break an existing lock (admin only).

```bash
auxin lock break --force
```

**Warning**: May cause lock holder to lose unsaved work.

---

## Auth Commands

### auxin auth login

Login to Oxen Hub with API credentials.

```bash
auxin auth login
```

Prompts for username and API key interactively.

---

### auxin auth logout

Remove stored credentials.

```bash
auxin auth logout
```

---

### auxin auth status

Show current authentication status.

```bash
auxin auth status
```

---

### auxin auth test

Test authentication with Oxen Hub.

```bash
auxin auth test
```

---

## Daemon Commands

### auxin daemon status

Check daemon status.

```bash
auxin daemon status
```

---

### auxin daemon start

Start the daemon service.

```bash
auxin daemon start
```

---

### auxin daemon stop

Stop the daemon service.

```bash
auxin daemon stop
```

---

### auxin daemon restart

Restart the daemon service.

```bash
auxin daemon restart
```

---

### auxin daemon logs

Show daemon logs.

```bash
auxin daemon logs [--lines <N>]
```

**Options**:
- `--lines <N>` - Number of log lines (default: 50)

---

## Hooks Commands

### auxin hooks init

Initialize hooks directory.

```bash
auxin hooks init
```

Creates `.auxin/hooks/` directory structure.

---

### auxin hooks list

List all installed hooks.

```bash
auxin hooks list
```

---

### auxin hooks builtins

List available built-in hooks.

```bash
auxin hooks builtins
```

---

### auxin hooks install

Install a built-in hook.

```bash
auxin hooks install <HOOK_NAME> [--type <TYPE>]
```

**Options**:
- `--type <TYPE>` - Hook type: `pre-commit`, `post-commit` (default: pre-commit)

**Built-in Hooks**:
- `validate-metadata` - Validate commit metadata
- `backup` - Create backup after commit
- `notify` - Send notifications
- `check-size` - Verify file sizes

---

### auxin hooks remove

Remove an installed hook.

```bash
auxin hooks remove <HOOK_NAME> [--type <TYPE>]
```

---

## Bounce Commands

Audio bounces are "audio screenshots" attached to commits.

### auxin bounce add

Add a bounce file for a commit.

```bash
auxin bounce add <FILE> [--commit <ID>] [--description <TEXT>]
```

**Supported Formats**: WAV, AIFF, MP3, FLAC, M4A

---

### auxin bounce list

List all bounces in the repository.

```bash
auxin bounce list
```

---

### auxin bounce play

Play a bounce audio file.

```bash
auxin bounce play <COMMIT_ID>
```

---

### auxin bounce info

Show bounce metadata.

```bash
auxin bounce info <COMMIT_ID>
```

---

### auxin bounce delete

Delete a bounce.

```bash
auxin bounce delete <COMMIT_ID>
```

---

### auxin bounce search

Search and filter bounces.

```bash
auxin bounce search [OPTIONS]
```

**Options**:
- `--format <FORMAT>` - Filter by audio format
- `--pattern <REGEX>` - Filter by filename
- `--min-duration <SECONDS>` - Minimum duration
- `--max-duration <SECONDS>` - Maximum duration
- `--after <DATE>` - Added after date
- `--before <DATE>` - Added before date
- `--user <USER>` - Added by user

---

### auxin bounce compare

Compare two bounces.

```bash
auxin bounce compare <COMMIT_A> <COMMIT_B>
```

---

## Server Commands

### auxin server status

Show server configuration and connection status.

```bash
auxin server status
```

---

### auxin server health

Test connection to auxin-server.

```bash
auxin server health
```

---

### auxin server set

Set server configuration value.

```bash
auxin server set <KEY> <VALUE>
```

**Available Keys**:
- `url` - Server URL
- `namespace` - Default namespace
- `timeout` - Request timeout (seconds)
- `locks` - Enable server locks (true/false)
- `metadata` - Enable server metadata (true/false)

---

## Rust Library API

The `auxin` crate exposes a rich library API for programmatic use. Key types and modules include:

### Network Resilience

**RetryPolicy** - Configurable retry behavior for network operations:

```rust
use auxin::RetryPolicy;

let policy = RetryPolicy::default();

// Get delay for retry attempt (1-indexed)
let delay = policy.delay_for_attempt(3); // Exponential backoff

// Check if should retry
if policy.should_retry(attempt) {
    // Check if error is retryable
    if policy.is_retryable(&error_message) {
        // Retry operation
    }
}

// Access configuration
let max = policy.max_attempts();
let base = policy.base_delay_ms();
let max_delay = policy.max_delay_ms();
```

**Additional Types**: `CircuitBreaker`, `CircuitState`, `NetworkHealth`, `NetworkQuality`, `ConnectivityState`, `AdaptiveRetryPolicy`

### Console TUI

**Console** - Interactive terminal interface for monitoring and control:

```rust
use auxin::{Console, ConsoleMode, LogLevel, DaemonStatus};
use std::path::PathBuf;

let mut console = Console::new(PathBuf::from("/path/to/project"));

// Log messages
console.log(LogLevel::Info, "Status message");
console.log(LogLevel::Success, "Operation completed");
console.log(LogLevel::Warning, "Warning message");
console.log(LogLevel::Error, "Error occurred");

// Update daemon status
console.set_daemon_status(DaemonStatus::Running);

// Update repository status
console.set_repo_status(5, 3, 2); // staged, modified, untracked

// Access state
let mode = console.mode; // ConsoleMode::Normal, Help, etc.
let should_quit = console.should_quit;
```

**ConsoleMode** variants: `Normal`, `CommitDialog`, `RestoreBrowser`, `Compare`, `Search`, `Hooks`, `Help`

### Bounce (Audio Snapshots)

**BounceFilter** - Search criteria for audio bounces:

```rust
use auxin::{BounceFilter, BounceManager, AudioFormat};

let mut filter = BounceFilter::default();

// Pattern field (alias for filename_pattern)
filter.pattern = Some("mix.*".to_string());

// Other filter options
filter.format = Some(AudioFormat::Wav);
filter.min_duration = Some(30.0);
filter.max_duration = Some(300.0);
filter.added_by = Some("user".to_string());

// Use filter
let manager = BounceManager::new(&repo_path);
let results = manager.search_bounces(&filter)?;
```

### Hooks

**HookManager** - Workflow automation hooks:

```rust
use auxin::hooks::{HookManager, HookType};

let manager = HookManager::new(&repo_path);
manager.init()?;

// Install built-in hooks
manager.install_builtin("validate-metadata", HookType::PreCommit)?;
manager.install_builtin("backup", HookType::PostCommit)?;

// List all hooks (returns tuples)
let all_hooks = manager.list_hooks()?;  // Vec<(HookType, String)>

// List hooks filtered by type
let pre_commit_hooks = manager.list_hooks_by_type(HookType::PreCommit)?; // Vec<String>
let post_commit_hooks = manager.list_hooks_by_type(HookType::PostCommit)?;

// Remove hooks
manager.remove_hook("validate-metadata", HookType::PreCommit)?;

// Run hooks during commit
let metadata = CommitMetadata::new("My commit");
let success = manager.run_hooks(HookType::PreCommit, &metadata)?;
```

**Built-in Hooks**:
- `validate-metadata` - Ensure BPM/sample rate are set (pre-commit)
- `check-file-sizes` - Warn about files >100MB (pre-commit)
- `notify` - Send notifications (post-commit)
- `backup` - Create timestamped backups (post-commit)

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Authentication required |
| 4 | Lock held by another user |
| 5 | Network error |
| 6 | Not an Auxin repository |

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `AUXIN_CONFIG` | Config file path | `~/.auxin/config.toml` |
| `AUXIN_LOG_LEVEL` | Log level (debug, info, warn, error) | `info` |
| `OXEN_AUTH_TOKEN` | Oxen Hub auth token | (from keychain) |

---

*Last Updated: 2025-11-20*
