# Auxin CLI Wrapper - Usage Guide

## Overview

The Auxin CLI wrapper provides a command-line interface for managing Logic Pro projects with Oxen version control. It includes specialized features for Logic Pro folder projects (.logicx) with automatic ignore file generation and structured commit metadata.

## Installation

```bash
cd Auxin-CLI-Wrapper
cargo build --release

# Binary will be at: target/release/auxin
# Optionally, copy to your PATH:
sudo cp target/release/auxin /usr/local/bin/
```

## Commands

### Initialize a Repository

Initialize a standard Oxen repository:

```bash
auxin init /path/to/project
```

Initialize for a Logic Pro project (auto-detects .logicx, creates .oxenignore):

```bash
auxin init --logic /path/to/MyProject.logicx
```

This will:
- Verify the path is a valid Logic Pro folder project
- Initialize an Oxen repository
- Create a `.oxenignore` file with Logic Pro-specific patterns
- Print confirmation

### Stage Changes

Stage specific files:

```bash
auxin add projectData Resources/file.wav
```

Stage all changes:

```bash
auxin add --all
```

### Create a Commit

Simple commit:

```bash
auxin commit -m "Initial mix"
```

Commit with metadata:

```bash
auxin commit \
  -m "Finished verse arrangement" \
  --bpm 120 \
  --sample-rate 48000 \
  --key "C Major" \
  --tags "draft,verse"
```

This creates a structured commit message:

```
Finished verse arrangement

BPM: 120
Sample Rate: 48000 Hz
Key: C Major
Tags: draft, verse
```

### View History

Show all commits:

```bash
auxin log
```

Show last 5 commits:

```bash
auxin log --limit 5
```

Output format:

```
Commit: a1b2c3d4...
Author: user@example.com
Date:   2024-01-15 14:30:22

    Finished verse arrangement

    BPM: 120
    Sample Rate: 48000 Hz
    Key: C Major

────────────────────────────────────────────────────────────────────────────────
```

### Check Status

```bash
auxin status
```

Output shows:

```
Repository Status:

Staged files:
  + projectData
  + Resources/vocals.wav

Modified files:
  M Alternatives/000/DisplayState.plist

Untracked files:
  ? Resources/new-bass.wav
```

### Restore to a Previous Commit

```bash
auxin restore a1b2c3d4
```

This will restore the entire project to the specified commit.

## Workflow Examples

### Initial Project Setup

```bash
# Initialize Logic Pro project with Oxen
auxin init --logic ~/Music/MyTrack.logicx

# Stage all initial files
auxin add --all

# Create initial commit
auxin commit \
  -m "Initial project setup" \
  --bpm 128 \
  --sample-rate 48000 \
  --key "A Minor"
```

### After a Recording Session

```bash
# Check what changed
auxin status

# Stage all changes
auxin add --all

# Commit with metadata
auxin commit \
  -m "Recorded guitar parts for chorus" \
  --bpm 128 \
  --tags "recording,guitar,chorus"
```

### Before Mixing

```bash
# Create a milestone before mixing
auxin add --all
auxin commit \
  -m "Pre-mix checkpoint - all parts recorded" \
  --tags "milestone,pre-mix"
```

### Rolling Back to a Previous Version

```bash
# View history to find the commit
auxin log --limit 10

# Restore to a specific commit
auxin restore a1b2c3d4
```

## .oxenignore File

When you initialize with `--logic`, a `.oxenignore` file is created with these patterns:

```
# Volatile/Generated Files
Bounces/
Freeze Files/
*.nosync
Autosave/
Media.localized/

# System Files
.DS_Store
*.smbdelete*
.TemporaryItems
.Trashes
.fseventsd

# Cache and Temporary Files
*.cache
*.tmp
*~
```

You can edit this file to customize which files are ignored.

## Commit Metadata Fields

| Field | Flag | Type | Example |
|-------|------|------|---------|
| Message | `-m, --message` | Required | `"Finished mix"` |
| BPM | `--bpm` | Optional | `120` or `128.5` |
| Sample Rate | `--sample-rate` | Optional | `48000`, `96000` |
| Key Signature | `--key` | Optional | `"C Major"`, `"A Minor"` |
| Tags | `--tags` | Optional | `"draft,wip"` (comma-separated) |

## Integration with Logic Pro

### Tracked Files

The CLI automatically tracks:

- `projectData` - Main project file
- `Alternatives/` - Alternative takes and comps
- `Resources/` - Audio files and samples

### Ignored Files (Not Tracked)

- `Bounces/` - Exported audio files
- `Freeze Files/` - Track freeze files
- `Autosave/` - Auto-save backups
- System files (`.DS_Store`, etc.)

## Tips

1. **Commit often**: Use commits to mark significant points in your workflow
2. **Use metadata**: Add BPM, sample rate, and key to make commits searchable
3. **Tag milestones**: Use tags like `milestone`, `pre-mix`, `final` for important versions
4. **Check status before committing**: Always run `status` to see what will be committed
5. **Descriptive messages**: Write clear commit messages that explain what changed

## Troubleshooting

### "Repository not found"

Make sure you're in a directory that has been initialized with `auxin init`.

### "Path is not a Logic Pro folder project"

The `--logic` flag requires a path ending in `.logicx` with a valid `projectData` file inside.

### Permission denied errors

Ensure you have read/write permissions to the project directory.

## Next Steps

For automatic background monitoring, see the FSEvents monitor:

```bash
# In Auxin-LaunchAgent directory
swift build
.build/debug/auxin-monitor ~/Music/MyProject.logicx
```

This will watch for changes and trigger auto-commits after 30 seconds of inactivity (Phase 2 feature).
