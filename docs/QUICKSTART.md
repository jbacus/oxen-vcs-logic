# Quick Start Guide - OxVCS for Logic Pro

Get started with version control for your Logic Pro projects in 5 minutes.

## Prerequisites

1. **macOS 14.0+** with Xcode installed
2. **Logic Pro 11.x** with folder projects (.logicx)
3. **Rust toolchain** (install from https://rustup.rs)
4. **Oxen.ai CLI** (install: `pip install oxen-ai`)

## Step 1: Build the Tools

### Build the CLI

```bash
cd oxen-vcs-logic/OxVCS-CLI-Wrapper
cargo build --release

# Add to PATH (optional)
export PATH="$PWD/target/release:$PATH"
```

### Build the Monitor (Optional)

```bash
cd oxen-vcs-logic/OxVCS-LaunchAgent
swift build -c release
```

## Step 2: Initialize Your First Project

Navigate to a Logic Pro project:

```bash
cd ~/Music/MyTrack.logicx

# Initialize with Logic Pro auto-detection
oxenvcs-cli init --logic .
```

Expected output:
```
Detected Logic Pro project: MyTrack
Initialized Oxen repository at: /Users/you/Music/MyTrack.logicx
Created .oxenignore file
✓ Successfully initialized Logic Pro project repository
```

## Step 3: Create Your First Commit

```bash
# Stage all files
oxenvcs-cli add --all

# Create initial commit with metadata
oxenvcs-cli commit \
  -m "Initial project setup" \
  --bpm 120 \
  --sample-rate 48000 \
  --key "C Major"
```

## Step 4: Work and Save Versions

After making changes in Logic Pro:

```bash
# Check what changed
oxenvcs-cli status

# Stage and commit
oxenvcs-cli add --all
oxenvcs-cli commit -m "Added bass line" --tags "bass,recording"
```

## Step 5: View History

```bash
# See all commits
oxenvcs-cli log

# See last 5 commits
oxenvcs-cli log --limit 5
```

## Step 6: Restore a Previous Version (If Needed)

```bash
# List commits to find the ID
oxenvcs-cli log

# Restore to a specific commit
oxenvcs-cli restore <commit-id>
```

## Optional: Enable File Monitoring

Watch for changes automatically:

```bash
# From the LaunchAgent directory
cd oxen-vcs-logic/OxVCS-LaunchAgent

# Run the monitor
.build/release/oxvcs-monitor ~/Music/MyTrack.logicx
```

The monitor will log file changes and show when it would trigger an auto-commit (after 30 seconds of inactivity).

## Daily Workflow Example

```bash
# Morning: Start a new session
cd ~/Music/MyTrack.logicx
oxenvcs-cli status

# Work in Logic Pro...
# (Make changes, record, edit, mix)

# After recording session
oxenvcs-cli add --all
oxenvcs-cli commit \
  -m "Recorded guitar parts for verse" \
  --bpm 128 \
  --tags "recording,guitar"

# Before lunch break
oxenvcs-cli commit -m "WIP: Working on mix" --tags "wip"

# End of day: Final commit
oxenvcs-cli commit -m "Final mix adjustments" --tags "mixing,eod"
```

## What Gets Tracked?

✅ **Tracked:**
- `projectData` - Main project file
- `Alternatives/` - Alternative takes
- `Resources/` - Audio files

❌ **Ignored (Automatic):**
- `Bounces/` - Exported audio
- `Freeze Files/` - Track freezes
- `Autosave/` - Auto-saves
- `.DS_Store` - System files

## Common Commands Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `init --logic <path>` | Initialize project | `oxenvcs-cli init --logic .` |
| `add --all` | Stage all changes | `oxenvcs-cli add --all` |
| `commit -m <msg>` | Create commit | `oxenvcs-cli commit -m "Done"` |
| `status` | Check changes | `oxenvcs-cli status` |
| `log` | View history | `oxenvcs-cli log --limit 10` |
| `restore <id>` | Restore version | `oxenvcs-cli restore a1b2c3d4` |

## Commit Metadata Options

Enhance your commits with metadata:

```bash
oxenvcs-cli commit \
  -m "Your message" \
  --bpm 120 \              # Tempo
  --sample-rate 48000 \    # Sample rate (Hz)
  --key "C Major" \        # Musical key
  --tags "draft,wip"       # Tags (comma-separated)
```

## Tips for Success

1. **Commit often**: After each significant change
2. **Use descriptive messages**: Explain what you did
3. **Add metadata**: Helps search and organize later
4. **Tag milestones**: Use tags like `final`, `mix`, `master`
5. **Check status first**: Always run `status` before committing

## Troubleshooting

### "Repository not found"

You need to run `init` first:
```bash
oxenvcs-cli init --logic .
```

### "Path is not a Logic Pro folder project"

Make sure you're in a `.logicx` directory with a `projectData` file.

### Permission errors

Ensure you have write access:
```bash
chmod -R u+w .
```

## Next Steps

- Read the full [Usage Guide](../OxVCS-CLI-Wrapper/USAGE.md)
- See [Phase 1 Completion Report](PHASE1_COMPLETE.md) for technical details
- Review [Implementation Plan](IMPLEMENTATION_PLAN.md) for upcoming features

## Getting Help

For issues or questions:
1. Check the [Usage Guide](../OxVCS-CLI-Wrapper/USAGE.md)
2. Review error messages carefully
3. Verify prerequisites are installed
4. Check file permissions

## What's Coming in Phase 2

- Automatic commits via LaunchAgent
- Power-safe operation (pre-sleep commits)
- Draft branch tracking
- Background monitoring service
- XPC-based inter-process communication

---

**You're all set!** Start versioning your Logic Pro projects with confidence.
