# Migration Guide: Auxin-VCS → Auxin

This guide helps you migrate from Auxin-VCS (v0.1.0) to Auxin (v0.2.0).

## What Changed?

Version 0.2.0 includes a complete rebranding from "Auxin-VCS" to "Auxin":

| Component | Old Name | New Name |
|-----------|----------|----------|
| **Binary** | `auxin-cli` | `auxin` |
| **Package** | `auxin-cli` | `auxin` |
| **Repository** | `jbacus/oxen-vcs-logic` | `jbacus/auxin` |
| **Config Directory** | `~/.auxin` | `~/.auxin` |
| **LaunchAgent Service** | `com.oxen.logic.daemon` | `com.auxin.daemon` |
| **Rust Daemon Client** | `com.auxenvcs.agent` | `com.auxin.agent` |
| **App Bundle ID** | `com.auxenvcs.app` | `com.auxin.app` |
| **Environment Variables** | `OXENVCS_*` | `AUXIN_*` |

## Migration Steps

### 1. Stop the Old LaunchAgent (macOS only)

```bash
# Stop and unload the old daemon
launchctl unload ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Remove old plist file
rm ~/Library/LaunchAgents/com.oxen.logic.daemon.plist
```

### 2. Backup Your Configuration

```bash
# Backup old config if it exists
if [ -d ~/.auxin ]; then
  cp -r ~/.auxin ~/.auxin-backup-$(date +%Y%m%d)
  echo "Config backed up to ~/.auxin-backup-$(date +%Y%m%d)"
fi
```

### 3. Uninstall Old Version

```bash
# Remove old binaries
sudo rm -f /usr/local/bin/auxin-cli
sudo rm -f /usr/local/bin/auxvcs-daemon

# Remove old app bundle if installed
if [ -d "/Applications/Auxin.app" ]; then
  rm -rf "/Applications/Auxin.app"
fi
```

### 4. Install New Version

```bash
# Clone or pull the latest code
cd /path/to/auxin
git pull origin main

# Run the installer
./install.sh
```

The installer will:
- Build and install the `auxin` binary
- Build and install the `auxin-daemon` binary
- Install the new LaunchAgent with updated service ID
- Create/update configuration files
- Install shell completions for the new binary name

### 5. Update Your Configuration

Your existing configuration at `~/.auxin/config.toml` should work without changes, but if you have environment variables set, update them:

**Old environment variables (no longer recognized):**
```bash
export OXENVCS_VERBOSE=true
export OXENVCS_COLOR=always
export OXENVCS_LOCK_TIMEOUT=8
export OXENVCS_MAX_RETRIES=10
export OXENVCS_QUEUE_DIR=~/custom-queue
```

**New environment variables:**
```bash
export AUXIN_VERBOSE=true
export AUXIN_COLOR=always
export AUXIN_LOCK_TIMEOUT=8
export AUXIN_MAX_RETRIES=10
export AUXIN_QUEUE_DIR=~/custom-queue
```

**Update your shell profile** (~/.zshrc, ~/.bashrc, etc.):
```bash
# Find and replace OXENVCS_ with AUXIN_
sed -i '' 's/OXENVCS_/AUXIN_/g' ~/.zshrc
```

### 6. Update Shell Completions

**Zsh:**
```bash
# Remove old completions
rm -f ~/.zsh/completions/_auxin-cli

# The new completions are installed automatically at ~/.zsh/completions/_auxin
# Refresh completion cache
rm -f ~/.zcompdump*
compinit
```

**Bash:**
```bash
# Old completion will be automatically replaced by installer
# Restart your shell or source the new completion
source /usr/local/etc/bash_completion.d/auxin
```

**Fish:**
```bash
# Old completion at ~/.config/fish/completions/auxin-cli.fish
# is automatically replaced with auxin.fish
# Restart Fish or run:
fish_update_completions
```

### 7. Verify Installation

```bash
# Check binary version
auxin --version
# Should output: auxin 0.2.0

# Check daemon status
auxin-daemon --status

# Check LaunchAgent is loaded
launchctl list | grep com.auxin
# Should show: com.auxin.daemon

# Verify configuration
auxin config show
```

### 8. Test with a Project

```bash
# Navigate to an existing project
cd ~/Music/MyProject.logicx

# Check status (should work with new binary)
auxin status

# Create a test commit
echo "test" > test-file.txt
auxin add test-file.txt
auxin commit -m "Testing Auxin v0.2.0" --bpm 120

# View history
auxin log
```

### 9. Update Git Remotes (if applicable)

If you were using the GitHub repository:

```bash
# Update remote URL
git remote set-url origin https://github.com/jbacus/auxin.git

# Verify
git remote -v
```

## Compatibility Notes

### Repositories Created with Old Version

**Good news:** Your existing repositories will work seamlessly with Auxin v0.2.0! The underlying Oxen.ai repository format has not changed.

- All commit history is preserved
- All branches, tags, and locks remain intact
- No data migration required
- The `.oxen` directory in your projects is untouched

### Shell Scripts and Automation

If you have shell scripts that call `auxin-cli`, update them to use `auxin`:

```bash
# Old
auxin-cli init --logic .
auxin-cli commit -m "message"

# New
auxin init --logic .
auxin commit -m "message"
```

### CI/CD Pipelines

If you use Auxin in CI/CD:

1. Update binary name in all scripts: `auxin-cli` → `auxin`
2. Update environment variables: `OXENVCS_*` → `AUXIN_*`
3. Update GitHub URLs: `jbacus/oxen-vcs-logic` → `jbacus/auxin`

## Troubleshooting

### "auxin-cli: command not found"

This is expected after upgrading. Use `auxin` instead:
```bash
auxin --help
```

### "LaunchAgent failed to load"

The new daemon uses a different service ID. Make sure you:
1. Unloaded the old daemon: `launchctl unload ~/Library/LaunchAgents/com.oxen.logic.daemon.plist`
2. Removed the old plist: `rm ~/Library/LaunchAgents/com.oxen.logic.daemon.plist`
3. Installed the new version: `./install.sh`

Grant permission in **System Settings → General → Login Items & Extensions**.

### "Environment variables not working"

If you had `OXENVCS_*` variables set, rename them to `AUXIN_*`:
```bash
# Check current variables
env | grep OXENVCS
env | grep AUXIN

# Update shell profile
sed -i '' 's/OXENVCS_/AUXIN_/g' ~/.zshrc
source ~/.zshrc
```

### Shell Completions Not Working

```bash
# Zsh
rm -f ~/.zcompdump* && compinit

# Bash
source /usr/local/etc/bash_completion.d/auxin

# Fish
fish_update_completions
```

### Old Binary Still in PATH

```bash
# Check which binary is being used
which auxin-cli  # Should return nothing
which auxin      # Should return /usr/local/bin/auxin

# If auxin-cli is still present, remove it manually
sudo rm $(which auxin-cli)
```

## Rollback Instructions

If you need to roll back to v0.1.0:

```bash
# Stop new daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.daemon.plist

# Restore old daemon
launchctl load ~/Library/LaunchAgents/com.oxen.logic.daemon.plist

# Restore old binaries from backup (if you created one)
# or reinstall v0.1.0:
git checkout v0.1.0
./install.sh

# Restore configuration
if [ -d ~/.auxin-backup-* ]; then
  mv ~/.auxin ~/.auxin-v0.2.0
  cp -r ~/.auxin-backup-* ~/.auxin
fi
```

## What's New in v0.2.0?

Besides the rebranding, v0.2.0 includes:

- **Improved UX**: Simpler binary name (`auxin` vs `auxin-cli`)
- **Cleaner Service IDs**: Unified `com.auxin.*` namespace
- **Better Documentation**: All docs updated and consolidated
- **Enhanced Consistency**: Matching naming across all components

For a complete list of changes, see [CHANGELOG.md](CHANGELOG.md).

## Getting Help

- **Documentation**: See [README.md](README.md) and [docs/](docs/)
- **Issues**: Report at https://github.com/jbacus/auxin/issues
- **Community**: hello@oxen.ai

---

**Estimated Migration Time**: 5-10 minutes

**Data Safety**: Your projects and commit history are 100% safe. This update only changes the tool names, not the data format.
