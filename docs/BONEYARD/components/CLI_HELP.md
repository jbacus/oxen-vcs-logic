# auxin Help Documentation

This document shows the enhanced help text for the auxin tool.

## Main Help

```bash
$ auxin --help
```

**Output:**
```
Oxen.ai CLI wrapper for Logic Pro version control

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
  1. Initialize: auxin init --logic .
  2. Make changes in Logic Pro
  3. Stage: auxin add --all
  4. Commit: auxin commit -m "Added drum track" --bpm 120
  5. View history: auxin log

Usage: auxin <COMMAND>

Commands:
  init     Initialize a new Oxen repository for a Logic Pro project
  add      Stage changes to be committed
  commit   Create a commit with optional audio metadata
  log      Show commit history
  restore  Restore project to a previous commit
  status   Show repository status
  help     Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

## Command-Specific Help

### Init Command

```bash
$ auxin init --help
```

**Output:**
```
Initialize a new Oxen repository for a Logic Pro project

USAGE:
    auxin init --logic <PATH>
    auxin init <PATH>

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
    auxin init --logic .

    # Initialize from parent directory
    auxin init --logic Demo_Project.logicx

    # Initialize a generic Oxen repository (not Logic-specific)
    auxin init /path/to/folder

Usage: auxin init [OPTIONS] <PATH>

Arguments:
  <PATH>  Path to the project directory

Options:
      --logic  Initialize for Logic Pro project (auto-detect and configure)
  -h, --help   Print help
```

### Add Command

```bash
$ auxin add --help
```

**Output:**
```
Stage changes to be committed

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
    auxin add Resources/

Usage: auxin add [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  Files or directories to stage

Options:
  -a, --all   Stage all changes in the repository
  -h, --help  Print help
```

### Commit Command

```bash
$ auxin commit --help
```

**Output:**
```
Create a commit with optional audio metadata

USAGE:
    auxin commit -m <MESSAGE> [OPTIONS]

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
    auxin commit -m "Initial project setup"

    # Commit with audio metadata
    auxin commit -m "Added bass line" --bpm 120 --key "A Minor"

    # Commit with multiple tags
    auxin commit -m "Final mix" --sample-rate 44100 --tags "mixing,mastered"

    # Full metadata commit
    auxin commit -m "Verse 2 complete" \
        --bpm 128 \
        --sample-rate 48000 \
        --key "C Major" \
        --tags "verse,arrangement"

Usage: auxin commit [OPTIONS] --message <MESSAGE>

Options:
  -m, --message <MESSAGE>          Commit message describing the changes
      --bpm <BPM>                  Beats per minute (tempo) of the project
      --sample-rate <SAMPLE_RATE>  Sample rate in Hz (e.g., 44100, 48000, 96000)
      --key <KEY>                  Key signature (e.g., 'C Major', 'A Minor', 'F# Minor')
      --tags <TAGS>                Tags for categorization (comma-separated, e.g., 'mixing,draft')
  -h, --help                       Print help
```

### Log Command

```bash
$ auxin log --help
```

**Output:**
```
Show commit history

USAGE:
    auxin log [--limit <N>]

DESCRIPTION:
    Displays the commit history for the repository, showing commit IDs, authors,
    timestamps, and messages. Audio metadata (BPM, key, etc.) is displayed if
    present in the commit message.

EXAMPLES:
    # Show all commits
    auxin log

    # Show only the last 5 commits
    auxin log --limit 5

    # Show the last 10 commits
    auxin log -l 10

Usage: auxin log [OPTIONS]

Options:
  -l, --limit <LIMIT>  Maximum number of commits to display
  -h, --help           Print help
```

### Restore Command

```bash
$ auxin restore --help
```

**Output:**
```
Restore project to a previous commit

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
    auxin restore abc123def456789012345678901234567890

Usage: auxin restore <COMMIT_ID>

Arguments:
  <COMMIT_ID>  Commit ID to restore to (from 'log' command)

Options:
  -h, --help  Print help
```

### Status Command

```bash
$ auxin status --help
```

**Output:**
```
Show repository status

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
    auxin status

Usage: auxin status

Options:
  -h, --help  Print help
```

## Quick Reference

| Command | Description | Example |
|---------|-------------|---------|
| `init --logic <PATH>` | Initialize Logic Pro project | `auxin init --logic .` |
| `add --all` | Stage all changes | `auxin add --all` |
| `commit -m <MSG>` | Create commit | `auxin commit -m "Added drums"` |
| `log` | View history | `auxin log --limit 10` |
| `status` | Check status | `auxin status` |
| `restore <ID>` | Restore to commit | `auxin restore abc123` |
