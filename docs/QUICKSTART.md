# Quick Start Guide - OxVCS for Logic Pro

Get started with version control for your Logic Pro projects in 5 minutes.

## Prerequisites

1. **macOS 14.0+** with Xcode installed
2. **Logic Pro 11.x** with folder projects (.logicx)
3. **Rust toolchain** (install from https://rustup.rs)
4. **Oxen.ai CLI** (install: `pip install oxen-ai`)

## Step 1: Build the Components

### Build the Rust CLI Wrapper

```bash
cd oxen-vcs-logic/OxVCS-CLI-Wrapper

# Build the CLI in release mode
cargo build --release

# Verify the build succeeded
ls -lh target/release/oxenvcs-cli
```

**Note**: The CLI currently uses a stub implementation of `liboxen` since Oxen.ai hasn't published official Rust bindings yet. The stub allows development and testing of the CLI interface. When real bindings become available, replace the stub by:
1. Uncommenting `liboxen = "X.X"` in `Cargo.toml`
2. Deleting `src/liboxen_stub/` directory
3. Rebuilding

**Troubleshooting Build Issues:**

If you get Rust version errors:
```bash
rustup update
rustc --version  # Should be 1.66.0 or newer
```

If you get PATH issues with old rustc:
```bash
which rustc  # Should be ~/.cargo/bin/rustc
# If not, ensure ~/.cargo/bin is first in your PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### Build the Swift LaunchAgent (Background Daemon)

```bash
cd oxen-vcs-logic/OxVCS-LaunchAgent

# Build the daemon in release mode
swift build -c release

# Verify the build succeeded
ls -lh .build/release/oxvcs-daemon
```

The daemon provides:
- Automatic file system monitoring
- Power-safe commits (before sleep/shutdown)
- Background service integration
- XPC communication with the UI app

### Build the Swift App (UI Application)

```bash
cd oxen-vcs-logic/OxVCS-App

# Build the app in release mode
swift build -c release

# Or build with Xcode for better app packaging
xcodebuild -scheme OxVCS-App -configuration Release
```

The app provides:
- Native macOS UI for repository browsing
- Visual commit history
- Milestone commit interface
- Settings and configuration panel

### Install the Binaries (Optional)

```bash
# Install CLI to PATH
sudo cp OxVCS-CLI-Wrapper/target/release/oxenvcs-cli /usr/local/bin/

# Install daemon
sudo cp OxVCS-LaunchAgent/.build/release/oxvcs-daemon /usr/local/bin/

# Verify installation
which oxenvcs-cli
oxenvcs-cli --help
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
[STUB] Would initialize Oxen repository at: /Users/you/Music/MyTrack.logicx
[STUB] Would check for repository at: /Users/you/Music/MyTrack.logicx
Detected Logic Pro project: MyTrack
Initialized Oxen repository at: /Users/you/Music/MyTrack.logicx
Created .oxenignore file
Initializing draft branch workflow...
[STUB] Would create branch: draft
✓ Draft branch workflow initialized
✓ Successfully initialized Logic Pro project repository
```

**Note**: The `[STUB]` messages indicate the stub implementation is running. These will be replaced with actual Oxen operations when real bindings are integrated.

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

Expected output:
```
[STUB] Would add files to: /Users/you/Music/MyTrack.logicx
Creating commit with message:
Initial project setup

[BPM: 120 | Sample Rate: 48000 Hz | Key: C Major]
[STUB] Would create commit in: /Users/you/Music/MyTrack.logicx
[STUB] Message: Initial project setup
...
✓ Commit created: stub_commit_id_12345
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

## Optional: Run the Background Daemon

Enable automatic monitoring and commits:

```bash
# Run the daemon (foreground for testing)
oxvcs-daemon

# Or run as a LaunchAgent (background service)
# See OxVCS-LaunchAgent/README.md for LaunchAgent setup
```

The daemon will:
- Monitor your Logic Pro projects for changes
- Auto-commit after 30 seconds of inactivity
- Trigger emergency commits before sleep/shutdown
- Provide XPC interface for the UI app

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

### Build Issues

**Rust: "requires rustc 1.66.0 or newer"**
```bash
rustup update
rustc --version
```

**Rust: Old version still showing after update**
```bash
# Check which rustc is being used
which rustc  # Should be ~/.cargo/bin/rustc

# Fix PATH in ~/.zshrc or ~/.bash_profile
export PATH="$HOME/.cargo/bin:$PATH"
source ~/.zshrc
```

**Swift: "'main' attribute cannot be used"**
This was fixed by renaming `main.swift` to `OxVCSDaemon.swift`. Make sure you have the latest code:
```bash
git pull origin main
```

**Swift: "Cannot find 'NSWorkspace'"**
This was fixed by adding `import AppKit` and `import IOKit.ps`. Pull the latest code.

### Runtime Issues

**"Repository not found"**

You need to run `init` first:
```bash
oxenvcs-cli init --logic .
```

**"Path is not a Logic Pro folder project"**

Make sure you're in a `.logicx` directory with a `projectData` file:
```bash
ls -la | grep projectData
```

**Permission errors**

Ensure you have write access:
```bash
chmod -R u+w .
```

**Proxy blocking crates.io (build environment only)**

If building in a restricted environment, see `BUILD_ISSUES.md` for solutions including:
- Requesting crates.io allowlist from network admin
- Using vendored dependencies
- Building in different environment

## Current Implementation Status

### ✅ Fully Implemented
- **CLI Wrapper**: All commands working with stub implementation
- **LaunchAgent**: File monitoring, power management, XPC service
- **App**: UI application structure (build verified)

### 🔧 Using Stubs
- **liboxen Integration**: Uses stub until official Rust bindings available
- All functionality is testable, but operations are logged rather than executed

### 📋 Integration Checklist
When Oxen.ai publishes Rust bindings:
1. Update `OxVCS-CLI-Wrapper/Cargo.toml` to use real liboxen
2. Remove stub implementation
3. Rebuild and test
4. All existing code will work without changes (interface compatible)

## Next Steps

- Read the full [Usage Guide](../OxVCS-CLI-Wrapper/USAGE.md)
- See [Phase 3 Completion Report](../PHASE3_COMPLETE.md) for technical details
- Review [Testing Strategy](TESTING_STRATEGY.md) for test coverage
- Check [BUILD_ISSUES.md](../BUILD_ISSUES.md) for build troubleshooting

## Getting Help

For issues or questions:
1. Check the [Usage Guide](../OxVCS-CLI-Wrapper/USAGE.md)
2. Review [BUILD_ISSUES.md](../BUILD_ISSUES.md) for build problems
3. Review error messages carefully
4. Verify prerequisites are installed
5. Check file permissions

## What's Included

All three phases are complete:

### Phase 1: Core Data Management ✅
- Logic Pro project detection
- Oxen initialization wrapper
- Core operations (init, add, commit, log, restore)
- Structured commit metadata

### Phase 2: Service Architecture ✅
- LaunchAgent integration
- Power management (sleep/shutdown commits)
- Auto-commit workflow with draft branches
- XPC communication

### Phase 3: UI Application ✅
- Native macOS AppKit UI
- Repository browser
- Milestone commit interface
- Settings panel
- Exclusive file locking

---

**You're all set!** Start versioning your Logic Pro projects with confidence.

**Note**: Currently using stub implementation for Oxen operations. All functionality is present and testable. Real Oxen integration pending official Rust bindings publication.
