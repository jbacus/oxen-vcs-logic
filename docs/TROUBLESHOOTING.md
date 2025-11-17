# Troubleshooting Guide

**Last Updated:** 2025-10-29
**Version:** 0.1-beta

This guide helps you diagnose and fix common problems with Oxen-VCS for Logic Pro.

---

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Installation Issues](#installation-issues)
- [Daemon Problems](#daemon-problems)
- [Auto-Commit Issues](#auto-commit-issues)
- [Performance Problems](#performance-problems)
- [Lock Management Issues](#lock-management-issues)
- [Data & Corruption Issues](#data--corruption-issues)
- [Getting Help](#getting-help)

---

## Quick Diagnostics

Run these commands to gather diagnostic information:

```bash
# 1. System Info
sw_vers
system_profiler SPSoftwareDataType | head -20

# 2. Oxen CLI Status
which oxen
oxen --version

# 3. Daemon Status
launchctl list | grep oxenvcs

# 4. Recent Logs (last 30 minutes)
log show --predicate 'process == "Auxin-LaunchAgent"' --last 30m

# 5. Disk Space
df -h

# 6. Project Repository Size
du -sh MyProject.logicx/.oxen
```

**Save output to file:**
```bash
# Run all diagnostics and save
{
    echo "=== System Info ==="
    sw_vers
    echo -e "\n=== Oxen Status ==="
    which oxen
    oxen --version
    echo -e "\n=== Daemon Status ==="
    launchctl list | grep oxenvcs
    echo -e "\n=== Disk Space ==="
    df -h
} > ~/Desktop/oxenvcs-diagnostics.txt
```

---

## Installation Issues

### Oxen CLI Not Found

**Symptom:**
```
oxen: command not found
```

**Diagnosis:**
```bash
which oxen
# If empty, oxen is not installed
```

**Solution 1: Install via pip**
```bash
# Install Python 3 if needed
brew install python3

# Install oxen
pip3 install oxen-ai

# Verify
oxen --version
```

**Solution 2: Check PATH**
```bash
# Find where oxen was installed
python3 -m pip show oxen-ai | grep Location

# Add to PATH (add to ~/.zshrc)
export PATH="/Users/yourusername/Library/Python/3.x/bin:$PATH"

# Reload shell
source ~/.zshrc

# Verify
oxen --version
```

**Solution 3: Reinstall**
```bash
pip3 uninstall oxen-ai
pip3 install oxen-ai --user
```

### Oxen-VCS.app Won't Open

**Symptom:** App bounces in dock and closes, or shows error dialog

**Cause 1: Unsigned App**
```
"Oxen-VCS.app" cannot be opened because the developer cannot be verified
```

**Solution:**
1. Right-click `Oxen-VCS.app` in Applications
2. Select "Open"
3. Click "Open" in dialog
4. App will now open normally

**Cause 2: Gatekeeper Block**
```
"Oxen-VCS.app" is damaged and can't be opened
```

**Solution:**
```bash
# Remove quarantine attribute
xattr -cr /Applications/Oxen-VCS.app

# Try opening again
open /Applications/Oxen-VCS.app
```

**Cause 3: Missing Dependencies**

Check Console.app for crash logs:
1. Open Console.app
2. Search for "Oxen-VCS"
3. Look for errors like "dylib not found"

**Solution:** Reinstall Xcode Command Line Tools:
```bash
xcode-select --install
```

### Permission Errors During Installation

**Symptom:**
```
Permission denied: '/path/to/LaunchAgents/'
```

**Solution:**
```bash
# Create directory if missing
mkdir -p ~/Library/LaunchAgents

# Verify ownership
ls -la ~/Library/ | grep LaunchAgents
# Should show your username

# Fix permissions if needed
chmod 755 ~/Library/LaunchAgents
```

---

## Daemon Problems

### Daemon Not Running

**Symptom:**
- Auto-commits don't happen
- Oxen-VCS.app shows "Daemon Offline"
- Menu bar icon missing or grayed out

**Diagnosis:**
```bash
launchctl list | grep oxenvcs
# If empty, daemon is not running
```

**Solution 1: Manual Start**
```bash
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

**Solution 2: Check Plist File**
```bash
# Verify plist exists
ls -la ~/Library/LaunchAgents/com.auxin.agent.plist

# If missing, reinstall Oxen-VCS.app
```

**Solution 3: Check Logs for Crash**
```bash
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --info

# Look for:
# - "Fatal error"
# - "Segmentation fault"
# - "Terminated"
```

**Solution 4: Reinstall Daemon**
```bash
# Unload
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist

# Delete plist
rm ~/Library/LaunchAgents/com.auxin.agent.plist

# Reinstall Oxen-VCS.app (will recreate plist)
# Then load:
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

### Daemon Keeps Crashing

**Symptom:**
```bash
launchctl list | grep oxenvcs
# Shows PID, then disappears after a few seconds
```

**Diagnosis:**
```bash
# Check crash logs
log show --predicate 'process == "Auxin-LaunchAgent" AND eventMessage CONTAINS "error"' --last 1h
```

**Common Causes:**

**1. Out of Memory**
```
# Check memory usage before crash
vm_stat | grep "Pages free"
```

**Solution:** Close other apps, upgrade RAM

**2. Permission Denied**
```
# Check Full Disk Access
System Preferences → Security & Privacy → Privacy → Full Disk Access
```

**Solution:** Add Oxen-VCS-LaunchAgent to Full Disk Access list

**3. Corrupted Configuration**
```bash
# Reset configuration
rm -rf ~/.auxin/config
# Restart daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

**4. Oxen CLI Issues**
```bash
# Test oxen directly
cd /path/to/project.logicx
oxen status
# If this fails, daemon will fail too
```

**Solution:** Fix oxen CLI installation (see above)

### Daemon Running but Not Responding

**Symptom:**
- Daemon shows as running in launchctl
- But auto-commits don't happen
- UI shows "Daemon Not Responding"

**Diagnosis:**
```bash
# Check if process is hung
ps aux | grep Auxin-LaunchAgent

# Check CPU usage (should be <5%)
top -pid <PID> -stats cpu
```

**Solution:**
```bash
# Force kill
killall Auxin-LaunchAgent

# Restart
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

---

## Auto-Commit Issues

### Auto-Commits Not Happening

**Symptom:** You save in Logic Pro, wait 60+ seconds, but no commit appears in history.

**Step 1: Verify Daemon**
```bash
launchctl list | grep oxenvcs
# Should show process with PID
```
If not running, see [Daemon Not Running](#daemon-not-running)

**Step 2: Check Project Registration**
```bash
# Verify project is monitored
cat ~/.auxin/monitored_projects
# Should list your project path
```

If missing:
1. Open Oxen-VCS.app
2. Click "Add Project"
3. Select your .logicx folder

**Step 3: Verify Repository**
```bash
cd /path/to/project.logicx
ls -la | grep .oxen
# Should show .oxen directory
```

If missing:
```bash
oxen init
```

**Step 4: Check Logs**
```bash
log show --predicate 'process == "Auxin-LaunchAgent"' --last 10m --info

# Look for:
# - "File change detected"
# - "Debounce timer started"
# - "Committing changes"
```

**Step 5: Test Manually**
```bash
# Make a change
cd /path/to/project.logicx
echo "test" >> test.txt

# Wait 60 seconds
# Check if commit was created
oxen log | head -5
```

If manual test works, but Logic Pro saves don't:
- FSEvents may not be monitoring correct path
- Check that you're saving the **folder** project, not a packaged file

### Auto-Commits Too Frequent

**Symptom:** Commits every few seconds, filling history with noise.

**Cause:** Debounce timer too short or disabled.

**Solution:**
```bash
# Check current debounce setting
cat ~/.auxin/config | grep debounce

# Should be 30-60 seconds
# If not, edit config:
nano ~/.auxin/config

# Set:
debounce_seconds = 60

# Restart daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

### Auto-Commits Too Slow

**Symptom:** Takes 2-5 minutes after saving before commit happens.

**Cause:** Large project, slow disk, or CPU overload.

**Diagnosis:**
```bash
# Check disk I/O
iostat -w 5
# If "MB/s" is low (<50), disk is slow

# Check CPU
top -l 1 | grep "CPU usage"
# If >80%, CPU is overloaded
```

**Solutions:**

1. **Upgrade to SSD** (if using HDD)
2. **Close other apps** (especially Time Machine, Dropbox during sessions)
3. **Increase debounce** (reduce commit frequency):
   ```bash
   # Edit config
   nano ~/.auxin/config
   # Set debounce_seconds = 120
   ```
4. **Verify .oxenignore** (ensure Bounces/ and Freeze Files/ are excluded)

---

## Performance Problems

### Commits Take Too Long (30+ seconds)

**Expected Times:**
- Small project (<1GB): 3-10 seconds
- Medium project (1-10GB): 10-30 seconds
- Large project (10-50GB): 30-120 seconds

**If slower than expected:**

**Diagnosis:**
```bash
# 1. Check project size
du -sh MyProject.logicx
du -sh MyProject.logicx/.oxen

# 2. Check what's being tracked
cd MyProject.logicx
oxen status
# If Bounces/ or Freeze Files/ appear, they shouldn't be tracked

# 3. Check disk speed
diskutil info / | grep "Solid State"
# Should say "Yes" for SSD
```

**Solutions:**

**1. Ensure .oxenignore is Correct**
```bash
cat MyProject.logicx/.oxenignore | grep -E "(Bounces|Freeze Files)"
# Should show:
# Bounces/
# Freeze Files/
```

If missing:
```bash
# Add to .oxenignore
echo "Bounces/" >> MyProject.logicx/.oxenignore
echo "Freeze Files/" >> MyProject.logicx/.oxenignore

# Remove from tracking
cd MyProject.logicx
oxen rm --cached -r "Bounces/"
oxen rm --cached -r "Freeze Files/"
```

**2. Clean Up Project**
```bash
# Delete large temporary files
rm -rf MyProject.logicx/Bounces/*
rm -rf "MyProject.logicx/Freeze Files"/*
```

**3. Upgrade Hardware**
- Use SSD (not HDD)
- More RAM (8GB minimum, 16GB+ recommended)
- Faster CPU

**4. Reduce Commit Frequency**
```bash
# Increase debounce to reduce how often commits happen
nano ~/.auxin/config
# Set: debounce_seconds = 90
```

### High Memory Usage

**Symptom:** Activity Monitor shows Oxen-VCS-LaunchAgent using >1GB RAM.

**Expected:** 50-200MB typical

**Diagnosis:**
```bash
# Check current usage
ps aux | grep Auxin-LaunchAgent | awk '{print $4, $6}'
# Shows % and KB
```

**Causes:**

**1. Memory Leak**
- Check if usage grows over time
- If so, report bug with logs

**Temporary Fix:**
```bash
# Restart daemon daily
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

**2. Monitoring Too Many Projects**
- Each monitored project uses ~10-50MB RAM
- Solution: Remove unused projects from monitoring

**3. Large Project**
- 50GB+ projects require more memory
- Expected: ~500MB for 50GB project
- Solution: Upgrade RAM or reduce project size

### High CPU Usage

**Symptom:** Oxen-VCS-LaunchAgent constantly uses 20-50%+ CPU.

**Expected:** <1% idle, <5% during commit

**Diagnosis:**
```bash
# Monitor CPU over time
top -pid $(pgrep Auxin-LaunchAgent) -stats cpu -l 10
```

**Causes:**

**1. Stuck in Commit Loop**
```bash
# Check logs for repeated commits
log show --predicate 'process == "Auxin-LaunchAgent"' --last 5m | grep "Committing"
# If you see many rapid commits, this is the problem
```

**Solution:** Fix debounce (see "Auto-Commits Too Frequent")

**2. Large File Scanning**
- FSEvents scanning huge Bounces/ or Freeze Files/
- Solution: Ensure these are in .oxenignore

**3. Runaway Process**
```bash
# Check for errors in logs
log show --predicate 'process == "Auxin-LaunchAgent" AND messageType == error' --last 10m
```

**Solution:** Restart daemon

---

## Lock Management Issues

### Cannot Acquire Lock

**Symptom:**
```
Lock held by: other.user@email.com
Acquired: 2025-10-29 09:00 AM
```

**Solutions:**

**1. Wait for Release**
- Contact lock holder (email/Slack)
- Wait for timeout (4 hours default)

**2. Force Break (Emergency)**
1. Try contacting holder first
2. In Oxen-VCS.app, click "Force Break"
3. Confirm action
4. You now hold the lock

**Warning:** Force-breaking can cause lost work if holder was editing.

### Lock Won't Release

**Symptom:** You click "Release Lock" but it remains held.

**Diagnosis:**
```bash
# Check lock status
cd MyProject.logicx
oxen lock status
```

**Solution 1: Manual Release**
```bash
oxen lock release
```

**Solution 2: Force Release**
```bash
oxen lock release --force
```

**Solution 3: Delete Lock File**
```bash
# Last resort
rm MyProject.logicx/.oxen/lock
```

### Lock Timeout Not Working

**Symptom:** Lock held for >4 hours, still not expired.

**Diagnosis:**
```bash
# Check lock timestamp
cd MyProject.logicx
cat .oxen/lock
# Shows timestamp and holder
```

**Calculate age:**
```bash
# If timestamp is UNIX epoch:
# Current time - lock time > 14400 seconds (4 hours)?
```

**Solution:** Force release (see above)

---

## Data & Corruption Issues

### "Repository Corrupted" Error

**Symptom:**
```
Error: oxen repository appears to be corrupted
```

**Diagnosis:**
```bash
cd MyProject.logicx
oxen fsck
# Checks repository integrity
```

**Solution 1: Repair**
```bash
oxen fsck --repair
```

**Solution 2: Restore from Backup**
1. Locate most recent Time Machine backup
2. Restore entire `.oxen/` directory
3. Test: `oxen status`

**Solution 3: Re-initialize (Last Resort)**
```bash
# WARNING: Loses all history
mv .oxen .oxen.backup
oxen init
oxen add .
oxen commit -m "Re-initialized after corruption"
```

### Files Missing After Rollback

**Symptom:** After rolling back, some files are gone.

**Cause:** Files were in `.oxenignore` (not tracked).

**Diagnosis:**
```bash
# Check what was ignored
cat .oxenignore

# Check if missing files match patterns
```

**Solution:**
1. **Restore from Time Machine** (if available)
2. **Check Autosave/**:
   ```bash
   ls -la Autosave/
   # Logic Pro may have backup
   ```
3. **Prevention:** Don't ignore essential files

### Commits Not Syncing to Remote

**Symptom:** Local commits exist, but `oxen push` fails or doesn't upload them.

**Diagnosis:**
```bash
cd MyProject.logicx

# Check remote configuration
oxen remote -v
# Should show remote URL

# Test connection
oxen remote show origin
```

**Solution 1: Fix Remote URL**
```bash
# If remote is incorrect
oxen remote set-url origin https://hub.oxen.ai/yourname/project
```

**Solution 2: Authentication**
```bash
# Re-authenticate with Oxen Hub
oxen login
# Enter credentials
```

**Solution 3: Force Push**
```bash
oxen push origin main --force
# WARNING: Overwrites remote history
```

**Solution 4: Network Issues**
```bash
# Test internet connection
ping hub.oxen.ai

# Check firewall settings
# Ensure ports 22 (SSH) and 443 (HTTPS) are open
```

---

## Getting Help

### Collect Diagnostic Information

Before reporting issues, gather:

```bash
#!/bin/bash
# diagnostic-script.sh

OUTPUT=~/Desktop/oxenvcs-diagnostics-$(date +%Y%m%d-%H%M%S).txt

{
    echo "=== Oxen-VCS Diagnostics ==="
    echo "Generated: $(date)"
    echo ""

    echo "=== System Info ==="
    sw_vers
    system_profiler SPSoftwareDataType | head -20
    echo ""

    echo "=== Oxen CLI ==="
    which oxen
    oxen --version 2>&1
    echo ""

    echo "=== Daemon Status ==="
    launchctl list | grep oxenvcs
    echo ""

    echo "=== Recent Logs (last 30 min) ==="
    log show --predicate 'process == "Auxin-LaunchAgent"' --last 30m --style compact
    echo ""

    echo "=== Disk Space ==="
    df -h
    echo ""

    echo "=== Memory ==="
    vm_stat | grep -E "(Pages free|Pages active)"
    echo ""

    echo "=== Monitored Projects ==="
    cat ~/.auxin/monitored_projects 2>/dev/null || echo "None"
    echo ""

} > "$OUTPUT"

echo "Diagnostics saved to: $OUTPUT"
open "$OUTPUT"
```

**Run:**
```bash
chmod +x diagnostic-script.sh
./diagnostic-script.sh
```

### Reporting Bugs

**GitHub Issues:** [https://github.com/jbacus/auxin/issues](https://github.com/jbacus/auxin/issues)

**Include:**
1. macOS version
2. Logic Pro version
3. Oxen version
4. Oxen-VCS.app version
5. Project size
6. Error message (exact text)
7. Steps to reproduce
8. Diagnostic file (from above)

### Emergency Recovery

**If all else fails:**

1. **Your Logic Pro project is safe** - it's just a folder on disk
2. **Version history is in `.oxen/`** - back it up
3. **Worst case:** Copy project folder to backup location
4. **Reinstall:**
   ```bash
   # Uninstall
   rm -rf /Applications/Oxen-VCS.app
   launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
   rm ~/Library/LaunchAgents/com.auxin.agent.plist

   # Reinstall
   # Download latest from GitHub
   # Drag to Applications
   # Relaunch
   ```

---

## Additional Resources

- **User Guide:** [USER_GUIDE.md](USER_GUIDE.md)
- **FAQ:** [FAQ.md](FAQ.md)
- **Quick Start:** [USER_GUIDE.md](USER_GUIDE.md)
- **GitHub Issues:** [https://github.com/jbacus/auxin/issues](https://github.com/jbacus/auxin/issues)

**Emergency Contact:** support@oxen-vcs.com

---

**Last Updated:** 2025-10-29
**Version:** 0.1-beta
