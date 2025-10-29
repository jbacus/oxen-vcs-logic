# Quick Start Guide

**Get started with Oxen-VCS in 5 minutes**

---

## Overview

This guide gets you up and running with Oxen-VCS for Logic Pro in the fastest way possible.

**What You'll Do:**
1. Install Oxen CLI (2 minutes)
2. Install & Launch Oxen-VCS.app (1 minute)
3. Initialize Your First Project (1 minute)
4. Verify Auto-Commits Work (1 minute)

**Total Time:** ~5 minutes

---

## Prerequisites

Before starting, ensure you have:
- ✅ macOS 14.0 (Sonoma) or later
- ✅ Logic Pro 11.x installed
- ✅ A Logic Pro project in **folder format** (`.logicx` folder, not package)
- ✅ Internet connection (for installation)

---

## Step 1: Install Oxen CLI (2 minutes)

### Option A: Using pip (Recommended)

Open Terminal and run:

```bash
pip3 install oxen-ai
```

**Verify installation:**
```bash
oxen --version
```

Should output: `oxen 0.x.x`

✅ **Done!** Move to Step 2.

---

### Option B: If pip3 Not Found

**Install Homebrew first:**
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

**Install Python 3:**
```bash
brew install python3
```

**Install Oxen:**
```bash
pip3 install oxen-ai
```

**Verify:**
```bash
oxen --version
```

✅ **Done!** Move to Step 2.

---

## Step 2: Install Oxen-VCS.app (1 minute)

1. **Download** `Oxen-VCS.app` from [GitHub Releases](https://github.com/jbacus/oxen-vcs-logic/releases)

2. **Drag to Applications** folder

3. **Right-click** and select **"Open"** (first time only, to bypass Gatekeeper)

4. **Click "Open"** in security dialog

5. **Grant Permissions** when prompted:
   - Full Disk Access
   - Accessibility (if requested)

**Grant Full Disk Access:**
- System Preferences → Security & Privacy → Privacy
- Select "Full Disk Access"
- Click the lock to make changes
- Click "+" and add Oxen-VCS-LaunchAgent

✅ **Done!** Move to Step 3.

---

## Step 3: Initialize Your First Project (1 minute)

**Important:** Your Logic Pro project must be in **folder format**.

To check:
- Right-click your `.logicx` project in Finder
- If you see "Show Package Contents" → It's packaged (won't work)
- If it opens as a folder → It's folder-based ✅ (correct format)

**If Packaged:** Convert in Logic Pro:
1. Open project
2. File → Save As...
3. Choose "Folder" format
4. Save

---

**Initialize:**

1. **Launch Oxen-VCS.app**
   - You should see the menu bar icon
   - Or launch from Applications

2. **Click "Add Project"** (or ⌘N)

3. **Navigate to your `.logicx` folder**
   - Example: `/Users/yourusername/Music/Logic/MyProject.logicx`

4. **Click "Initialize"**

**What Happens:**
- `.oxen/` directory created (version control data)
- `.oxenignore` file generated (exclusion patterns)
- Initial commit made
- Monitoring starts

**Expected Time:** 10-30 seconds

✅ **Done!** Your project is now version controlled.

---

## Step 4: Verify Auto-Commits (1 minute)

Let's test that automatic versioning works:

1. **Open your project in Logic Pro**
   - File → Open Recent → MyProject

2. **Make a small change**
   - Move a region
   - Change a plugin parameter
   - Add a track
   - (Anything that modifies the project)

3. **Save** (Cmd+S)

4. **Wait 60 seconds**
   - Oxen-VCS uses a debounce timer
   - Prevents commit spam while you're actively editing

5. **Check Oxen-VCS.app**
   - Select your project in sidebar
   - Click "History" tab
   - You should see a new "draft" commit with recent timestamp

**Expected Result:**
```
draft_2025-10-29_14-32-15
Created: Just now
Message: Auto-saved changes
```

✅ **Success!** Auto-commits are working.

---

## What's Next?

You're all set! Oxen-VCS is now:
- ✅ Monitoring your project for changes
- ✅ Creating automatic draft commits every 30-60 seconds after you save
- ✅ Saving before system sleep/shutdown
- ✅ Ready for milestone commits and collaboration

### Recommended Next Steps:

#### 1. Create Your First Milestone Commit

Milestones mark important versions with metadata:

1. **In Oxen-VCS.app, click "Milestone Commit"** (or ⌘K)
2. **Fill in the form:**
   ```
   Message: Initial version - project setup complete
   BPM: 120
   Sample Rate: 48000
   Key: C Major
   Tags: initial, tracking
   ```
3. **Click "Commit"**

**Result:** Your current state is saved as a named milestone.

---

#### 2. Try Rolling Back

Experience the power of version control:

1. **Make a significant change in Logic Pro**
   - Delete a track
   - Change arrangement
   - (Something you can easily verify later)

2. **Save and create a milestone commit**
   - "Experimental change - deleting bass track"

3. **In Oxen-VCS.app:**
   - Click "History" tab
   - Select the commit **before** your change
   - Click "Rollback"

4. **Reopen in Logic Pro**
   - Your bass track is back!
   - Project restored to earlier state

5. **To get your changes back:**
   - Go to History
   - Find `draft_rollback_backup_<timestamp>`
   - Rollback to that commit
   - You're back to where you were

---

#### 3. Learn More

**Essential Reading:**
- **[User Guide](USER_GUIDE.md)** - Comprehensive documentation (30 min read)
- **[FAQ](FAQ.md)** - Common questions answered (15 min read)

**Key Concepts:**
- **Draft Commits** - Automatic, granular safety net
- **Milestone Commits** - Manual, named important versions
- **Rollback** - Restore to any previous state (non-destructive)
- **.oxenignore** - What files are excluded (Bounces, Freeze Files, etc.)

**Advanced Topics:**
- Collaboration (locks, feature branches)
- Remote sync (Oxen Hub)
- Manual merge protocol (FCP XML)

---

## Common First-Time Issues

### "oxen: command not found"

**Problem:** Oxen CLI not installed or not in PATH.

**Solution:**
```bash
pip3 install oxen-ai
```

Then verify:
```bash
oxen --version
```

---

### Auto-commits not working

**Check daemon status:**
```bash
launchctl list | grep oxenvcs
```

Should show process with PID.

**If empty:**
```bash
launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist
```

---

### "Project is not a folder-based .logicx"

**Problem:** Your project is in package format.

**Solution:** Convert in Logic Pro:
1. File → Save As...
2. Select "Folder" format
3. Choose new location
4. Initialize the new folder project with Oxen-VCS

---

### Initialization fails

**Common causes:**

1. **No ProjectData file:**
   - Open project in Logic Pro
   - Make a change
   - Save (Cmd+S)
   - Try initializing again

2. **Permission error:**
   - Ensure project folder is on local drive (not network)
   - Check file permissions: `ls -la MyProject.logicx`

3. **Oxen CLI issue:**
   - Test: `oxen init /tmp/test`
   - If fails, reinstall: `pip3 install --force-reinstall oxen-ai`

---

## Tips for Success

### 1. Use Descriptive Milestone Messages

**Good:**
```
"Vocal tracking complete - 12 takes recorded"
"Mix v3 - increased bass, client feedback applied"
"Final master - ready for distribution"
```

**Bad:**
```
"Update"
"Changes"
"asdf"
```

### 2. Create Milestones at Key Points

- **Before major changes** (so you can undo)
- **After significant progress** (tracking session, mix complete)
- **Client deliverables** (what you send to clients)
- **End of work session** (before closing Logic Pro)

### 3. Understand What's Tracked

**Tracked (versioned):**
- `Alternatives/` - Project state, MIDI, automation
- `Resources/Audio Files/` - Your recordings

**Not Tracked (ignored):**
- `Bounces/` - Exported audio (regenerable)
- `Freeze Files/` - Track freezes (volatile)
- `Autosave/` - Automatic backups (noisy)

**Why?** To prevent repository bloat and conflicts.

### 4. Use Branches for Experiments

Instead of duplicating entire project:

```bash
cd MyProject.logicx

# Create experimental branch
oxen checkout -b experiment/orchestral-intro

# Make changes in Logic Pro
# They're tracked on this branch

# Switch back to main
oxen checkout main

# Project reverts to main branch state
```

### 5. Collaborate with Locks

If working with a team:

1. **Acquire lock before editing** (Oxen-VCS.app → "Acquire Lock")
2. **Make your changes**
3. **Commit and release lock**
4. **Next person can acquire**

Prevents conflicts and data loss.

---

## Getting Help

### Documentation

- **[User Guide](USER_GUIDE.md)** - Complete documentation
- **[FAQ](FAQ.md)** - Frequently asked questions
- **[Troubleshooting](TROUBLESHOOTING.md)** - Fix common problems

### Support

- **GitHub Issues:** [https://github.com/jbacus/oxen-vcs-logic/issues](https://github.com/jbacus/oxen-vcs-logic/issues)
- **Discord Community:** (Coming soon)
- **Email:** support@oxen-vcs.com

### Report Bugs

Include:
1. macOS version
2. Logic Pro version
3. Oxen version
4. Error message
5. Steps to reproduce

---

## Summary

**You've successfully:**
- ✅ Installed Oxen CLI
- ✅ Installed Oxen-VCS.app
- ✅ Initialized your first project
- ✅ Verified auto-commits work

**Oxen-VCS is now:**
- Monitoring your project 24/7
- Auto-saving changes every 30-60 seconds
- Ready for milestone commits
- Protecting your work

**Next:** Read the [User Guide](USER_GUIDE.md) to learn about:
- Collaboration workflows
- Remote synchronization
- Advanced features
- Best practices

---

**Welcome to safer Logic Pro production!** 🎵

**Questions?** Check [FAQ.md](FAQ.md) or [open an issue](https://github.com/jbacus/oxen-vcs-logic/issues).

---

**Last Updated:** 2025-10-29
**Version:** 0.1-beta
