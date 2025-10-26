# Phase 2 Installation Guide

Quick installation guide for the Oxen VCS Daemon (Phase 2).

## Automated Installation (Recommended)

The easiest way to install all components:

```bash
cd oxen-vcs-logic
./install.sh
```

See [Installation Guide](../INSTALL.md) for detailed instructions.

## Prerequisites

- macOS 14.0 or later
- Xcode Command Line Tools
- Rust toolchain (1.70+)
- Swift 5.9+

## Manual Installation

For manual installation or development setup:

### 1. Build Components

```bash
# Build Rust CLI
cd OxVCS-CLI-Wrapper
cargo build --release

# Build Swift daemon
cd ../OxVCS-LaunchAgent
swift build -c release
```

### 2. Install Binaries

```bash
# Install to /usr/local/bin
sudo cp OxVCS-CLI-Wrapper/target/release/oxenvcs-cli /usr/local/bin/
sudo cp OxVCS-LaunchAgent/.build/release/oxvcs-daemon /usr/local/bin/

# Verify installation
which oxenvcs-cli
which oxvcs-daemon
```

### 3. Install LaunchAgent

```bash
# Copy plist to LaunchAgents
mkdir -p ~/Library/LaunchAgents
cp OxVCS-LaunchAgent/Resources/com.oxen.logic.daemon.plist \
   ~/Library/LaunchAgents/

# Register service
oxvcs-daemon --install
```

### 4. Approve Service (First Time)

If you see "requires approval" message:

1. Open **System Settings**
2. Go to **General** → **Login Items & Extensions**
3. Find "Oxen VCS Daemon" in the list
4. Toggle it **ON**
5. Run `oxvcs-daemon --status` to verify

### 5. Verify Installation

```bash
# Check daemon status
oxvcs-daemon --status

# Should show:
# ✓ Enabled and running
# XPC Service: ✓ Listening
# Monitored Projects: 0

# Test with a Logic Pro project
oxenvcs-cli init ~/Music/YourProject.logicx

# Wait 30 seconds after making changes
# Check for auto-commit
oxenvcs-cli log ~/Music/YourProject.logicx
```

## Testing the Installation

### Test Auto-Commits

```bash
# 1. Initialize a project
oxenvcs-cli init ~/Music/TestProject.logicx

# 2. Open project in Logic Pro
open ~/Music/TestProject.logicx

# 3. Make some changes in Logic Pro
# 4. Wait 30 seconds
# 5. Check commits
oxenvcs-cli log ~/Music/TestProject.logicx

# You should see auto-commit(s)
```

### Test Power Management

```bash
# 1. Make changes without committing
# 2. Put system to sleep
pmset sleepnow

# 3. Wake system
# 4. Check for emergency commit
oxenvcs-cli log ~/Music/TestProject.logicx | grep "Emergency"
```

## Troubleshooting

### Daemon Not Starting

```bash
# Check launchctl
launchctl list | grep com.oxen.logic.daemon

# View logs
tail -f /tmp/com.oxen.logic.daemon.stdout
tail -f /tmp/com.oxen.logic.daemon.stderr

# Manually start for debugging
oxvcs-daemon --daemon
```

### Permission Issues

```bash
# Ensure binaries are executable
chmod +x /usr/local/bin/oxenvcs-cli
chmod +x /usr/local/bin/oxvcs-daemon

# Check plist permissions
ls -l ~/Library/LaunchAgents/com.oxen.logic.daemon.plist
# Should be: -rw-r--r--
```

### Auto-Commits Not Working

1. Check daemon is running: `oxvcs-daemon --status`
2. Verify project is registered (daemon scans on startup)
3. Check debounce time (default: 30 seconds)
4. View FSEvents output in logs

### Uninstalling

```bash
# Unregister service
oxvcs-daemon --uninstall

# Remove binaries
sudo rm /usr/local/bin/oxenvcs-cli
sudo rm /usr/local/bin/oxvcs-daemon

# Remove plist
rm ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Remove logs (optional)
rm /tmp/com.oxen.logic.daemon.*
```

## Advanced Configuration

### Custom Debounce Time

Edit `Daemon.swift` before building:

```swift
let daemon = OxenDaemon(
    debounceThreshold: 60.0  // 60 seconds instead of 30
)
```

### Custom Draft Branch Name

Edit `DraftManager.rs` before building:

```rust
pub const DEFAULT_DRAFT_BRANCH: &'static str = "auto-save"; // instead of "draft"
```

### Resource Limits

Edit `com.oxen.logic.daemon.plist`:

```xml
<key>HardResourceLimits</key>
<dict>
    <key>MemoryLimit</key>
    <integer>1073741824</integer>  <!-- 1GB instead of 512MB -->
</dict>
```

Then reload:

```bash
launchctl unload ~/Library/LaunchAgents/com.oxen.logic.daemon.plist
launchctl load ~/Library/LaunchAgents/com.oxen.logic.daemon.plist
```

## Support

For issues, check:

1. Logs: `/tmp/com.oxen.logic.daemon.stdout`
2. Documentation: `docs/PHASE2_COMPLETE.md`
3. GitHub Issues: https://github.com/Oxen-AI/oxen-vcs-logic/issues
