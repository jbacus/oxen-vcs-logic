# OxVCS Installation Guide

Complete installation guide for OxVCS (Oxen Version Control System for Logic Pro).

## Quick Installation (Recommended)

The easiest way to install OxVCS is using the automated installation script:

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/oxen-vcs-logic.git
cd oxen-vcs-logic

# Run the installer
./install.sh
```

That's it! The script will:
- ✓ Check all prerequisites
- ✓ Build all components (CLI, daemon, app)
- ✓ Install binaries to `/usr/local/bin`
- ✓ Configure and install LaunchAgent
- ✓ Register the daemon service
- ✓ Verify the installation

## Prerequisites

Before running the installer, ensure you have:

### Required
- **macOS 14.0+** (Sonoma or later)
- **Xcode Command Line Tools**
  ```bash
  xcode-select --install
  ```
- **Rust toolchain** (1.66.0 or newer)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Swift 5.9+** (included with Xcode)

### Optional
- **Oxen.ai CLI** (for full functionality)
  ```bash
  pip install oxen-ai
  ```

## Installation Options

### Standard Installation

Install all components including the UI app:

```bash
./install.sh
```

### CLI and Daemon Only

Skip building the UI app:

```bash
./install.sh --skip-app
```

### Skip Prerequisite Checks

If you're certain all prerequisites are installed:

```bash
./install.sh --skip-checks
```

## Post-Installation Setup

### 1. Grant System Permissions

After installation, you may need to approve the daemon in System Settings:

1. Open **System Settings**
2. Go to **General** → **Login Items & Extensions**
3. Find **Oxen VCS Daemon** in the list
4. Toggle it **ON**

### 2. Verify Installation

Check that everything is working:

```bash
# Check CLI
oxenvcs-cli --help

# Check daemon status
oxvcs-daemon --status

# Should show:
# ✓ Enabled and running
# XPC Service: ✓ Listening
# Monitored Projects: 0
```

### 3. Initialize Your First Project

```bash
cd ~/Music/YourProject.logicx
oxenvcs-cli init --logic .
```

See [User Guide](docs/USER_GUIDE.md) for detailed usage instructions.

## What Gets Installed

The installation script installs the following components:

### Binaries
- `/usr/local/bin/oxenvcs-cli` - Command-line interface for Oxen operations
- `/usr/local/bin/oxvcs-daemon` - Background daemon for automatic tracking

### Application
- `/Applications/OxVCS.app` - Native macOS GUI application (if installed)

### Configuration
- `~/Library/LaunchAgents/com.oxen.logic.daemon.plist` - LaunchAgent configuration
- `/Applications/OxVCS.app/Contents/Info.plist` - App bundle metadata

### Logs (created at runtime)
- `/tmp/com.oxen.logic.daemon.stdout` - Standard output log
- `/tmp/com.oxen.logic.daemon.stderr` - Error log

## Manual Installation

If you prefer to install manually, follow these steps:

### 1. Build Components

```bash
# Build Rust CLI
cd OxVCS-CLI-Wrapper
cargo build --release

# Build Swift daemon
cd ../OxVCS-LaunchAgent
swift build -c release

# Build Swift app (optional)
cd ../OxVCS-App
swift build -c release
./create-app-bundle.sh  # Creates OxVCS.app bundle
```

**Note**: The app requires a proper `.app` bundle structure to render correctly. The `create-app-bundle.sh` script creates the bundle with the necessary `Info.plist` and directory structure.

**SwiftUI Migration**: As of October 29, 2025, OxVCS-App uses SwiftUI instead of AppKit for improved window management and simplified UI code. No additional configuration is needed.

### 2. Install Binaries

```bash
# Install to /usr/local/bin
sudo cp OxVCS-CLI-Wrapper/target/release/oxenvcs-cli /usr/local/bin/
sudo cp OxVCS-LaunchAgent/.build/release/oxvcs-daemon /usr/local/bin/

# Set permissions
sudo chmod +x /usr/local/bin/oxenvcs-cli
sudo chmod +x /usr/local/bin/oxvcs-daemon
```

### 3. Configure LaunchAgent

```bash
# Create LaunchAgents directory
mkdir -p ~/Library/LaunchAgents

# Copy plist
cp OxVCS-LaunchAgent/Resources/com.oxen.logic.daemon.plist \
   ~/Library/LaunchAgents/

# Update the username in the plist
sed -i.bak "s|<string><!-- Will be dynamically set during installation --></string>|<string>$USER</string>|g" \
   ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Set permissions
chmod 644 ~/Library/LaunchAgents/com.oxen.logic.daemon.plist
```

### 4. Register Service

```bash
# Load with launchctl
launchctl load ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Register with SMAppService
oxvcs-daemon --install
```

## Troubleshooting

### Installation Fails: Missing Prerequisites

**Error**: "Rust toolchain not found" or "Swift not found"

**Solution**: Install missing prerequisites:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install Xcode Command Line Tools
xcode-select --install
```

### Build Fails: Rust Version Too Old

**Error**: "requires rustc 1.66.0 or newer"

**Solution**: Update Rust:

```bash
rustup update
rustc --version  # Verify version
```

### Build Fails: Old rustc in PATH

**Error**: Still shows old rustc version after update

**Solution**: Fix PATH:

```bash
which rustc  # Should be ~/.cargo/bin/rustc

# Add to ~/.zshrc or ~/.bash_profile
export PATH="$HOME/.cargo/bin:$PATH"

# Reload shell configuration
source ~/.zshrc
```

### Daemon Requires Approval

**Error**: "Service requires user approval in System Settings"

**Solution**: This is normal for first-time installation:

1. Open **System Settings**
2. Go to **General** → **Login Items & Extensions**
3. Find **Oxen VCS Daemon** and toggle **ON**
4. Run `oxvcs-daemon --status` to verify

### Permission Denied: Can't Write to /usr/local/bin

**Error**: Permission denied when copying binaries

**Solution**: The installer will automatically request sudo privileges. If you run commands manually, use sudo:

```bash
sudo cp OxVCS-CLI-Wrapper/target/release/oxenvcs-cli /usr/local/bin/
```

### Service Not Starting

**Error**: Daemon shows as "Not running" after installation

**Solution**: Check the logs:

```bash
# View logs
tail -f /tmp/com.oxen.logic.daemon.stdout
tail -f /tmp/com.oxen.logic.daemon.stderr

# Check launchctl status
launchctl list | grep com.oxen.logic.daemon

# Try manual start for debugging
oxvcs-daemon --daemon
```

### Build Fails: Proxy Blocking crates.io

**Error**: Can't download Rust dependencies

**Solutions**:
1. Request crates.io allowlist from network admin
2. Use vendored dependencies: `cargo vendor` in the Rust project directory
3. Build in a different environment (home network, etc.)
4. Configure cargo to use a proxy: add to `~/.cargo/config.toml`:
   ```toml
   [http]
   proxy = "http://your-proxy:port"
   ```

## Uninstallation

To completely remove OxVCS:

```bash
./install.sh --uninstall
```

Or manually:

```bash
# Stop and unregister service
oxvcs-daemon --uninstall
launchctl unload ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Remove binaries
sudo rm /usr/local/bin/oxenvcs-cli
sudo rm /usr/local/bin/oxvcs-daemon

# Remove plist
rm ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Remove logs (optional)
rm /tmp/com.oxen.logic.daemon.*
```

**Note**: This does not remove repository data (`.oxen` directories in your projects).

## Updating

To update to a new version:

```bash
# Pull latest changes
git pull origin main

# Reinstall
./install.sh
```

The installer will rebuild all components and replace the existing binaries.

## Installation Script Options

The `install.sh` script supports several options:

```bash
# Show help
./install.sh --help

# Full installation (default)
./install.sh

# Skip UI app build
./install.sh --skip-app

# Skip prerequisite checks
./install.sh --skip-checks

# Uninstall everything
./install.sh --uninstall
```

## Verification Checklist

After installation, verify everything is working:

- [ ] `oxenvcs-cli --help` shows help message
- [ ] `oxvcs-daemon --status` shows "✓ Enabled and running"
- [ ] Binaries exist: `ls -l /usr/local/bin/oxenvcs-*`
- [ ] Plist exists: `ls -l ~/Library/LaunchAgents/com.oxen.logic.daemon.plist`
- [ ] Can initialize project: `oxenvcs-cli init --logic ~/Music/TestProject.logicx`

## Next Steps

Once installation is complete:

1. **Read the Quick Start Guide**: [docs/USER_GUIDE.md](docs/USER_GUIDE.md)
2. **Initialize your first project**: `oxenvcs-cli init --logic ~/Music/YourProject.logicx`
3. **Learn the CLI**: [OxVCS-CLI-Wrapper/USAGE.md](OxVCS-CLI-Wrapper/USAGE.md)
4. **Understand the architecture**: [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)

## Getting Help

If you encounter issues:

1. Check the [Troubleshooting](#troubleshooting) section above
2. Review the logs: `/tmp/com.oxen.logic.daemon.stdout` and `.stderr`
3. Run with verbose output: `oxvcs-daemon --verify`
4. Open an issue: https://github.com/YOUR_USERNAME/oxen-vcs-logic/issues

## Additional Resources

- [Quick Start Guide](docs/QUICKSTART.md) - Get started in 5 minutes
- [Usage Guide](OxVCS-CLI-Wrapper/USAGE.md) - Complete CLI reference
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Development and testing
- [Architecture](docs/ARCHITECTURE.md) - Technical details

---

**Installation complete?** Head to the [Quick Start Guide](docs/QUICKSTART.md) to start using OxVCS!
