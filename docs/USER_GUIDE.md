# Oxen-VCS User Guide

**Version:** 0.1-beta
**Last Updated:** 2025-10-29
**For:** macOS 14.0+ with Logic Pro 11.x

---

## Table of Contents

1. [Introduction](#introduction)
2. [What is Oxen-VCS?](#what-is-oxen-vcs)
3. [Why Use Oxen-VCS for Logic Pro?](#why-use-oxen-vcs-for-logic-pro)
4. [Getting Started](#getting-started)
5. [Daily Workflows](#daily-workflows)
6. [Collaboration](#collaboration)
7. [Best Practices](#best-practices)
8. [Understanding .oxenignore](#understanding-oxenignore)
9. [Advanced Topics](#advanced-topics)
10. [Troubleshooting](#troubleshooting)

---

## Introduction

Welcome to Oxen-VCS, a macOS-native version control system designed specifically for Apple Logic Pro projects. This guide will help you understand how to use Oxen-VCS effectively to version control your music productions, collaborate with team members, and never lose work again.

### Who Is This For?

- **Solo Music Producers** - Track your project history, experiment safely, and revert when needed
- **Collaborative Teams** - Work on the same project without conflicts or data loss
- **Professional Studios** - Maintain reliable version history for client projects
- **Educators & Students** - Track learning progress and submit versioned assignments

---

## What is Oxen-VCS?

Oxen-VCS is a version control system built on [Oxen.ai](https://oxen.ai) that solves the fundamental problem of versioning Logic Pro projects. Unlike Git, which struggles with large binary files and can't merge DAW projects, Oxen-VCS is designed specifically for the unique challenges of music production.

### Key Features

**Automatic Versioning**
- Changes are automatically saved to version control
- No need to manually create commits for every edit
- 30-60 second debounce prevents commit spam
- Power management ensures commits before sleep/shutdown

**Milestone Commits**
- Mark important versions with descriptive metadata
- Include BPM, sample rate, key signature, and tags
- Easy browsing of project milestones
- Semantic meaning in your history

**Safe Rollback**
- Restore your project to any previous state
- Non-destructive (your current work is preserved)
- Preview commit details before rolling back
- Instant restoration (<10 seconds for typical projects)

**Collaboration Support**
- Exclusive lock system prevents conflicts
- One person edits at a time (pessimistic locking)
- Manual merge workflow for feature branches
- FCP XML-based track integration

### How It's Different from Git

| Feature | Git + Git-LFS | Oxen-VCS |
|---------|---------------|----------|
| **Binary File Handling** | Stores entire files on change | Block-level deduplication |
| **Storage Efficiency** | 10-100x bloat common | Minimal storage overhead |
| **Merge Conflicts** | Unresolvable for binary files | Prevented via locking |
| **DAW Integration** | None | Native Logic Pro support |
| **Audio File Tracking** | Slow, inefficient | Optimized for large media |

---

## Why Use Oxen-VCS for Logic Pro?

### The Problem with Traditional VCS

Logic Pro projects consist of:
- **Binary ProjectData files** - Non-human-readable, non-mergeable
- **Large audio files** - Often multi-GB per project
- **Generated assets** - Bounces and freeze files that bloat repositories
- **Frequent changes** - Every edit modifies binary data

**Git fails because:**
1. Stores full file copies on every change → massive bloat
2. Cannot merge binary Logic Pro projects → data loss on conflicts
3. Git-LFS is slow and still stores full files
4. No understanding of DAW workflows

**Oxen-VCS solves this by:**
1. Block-level deduplication → only changed data stored
2. Pessimistic locking → prevents merge conflicts entirely
3. Smart .oxenignore → excludes regenerable files
4. Automatic draft tracking → safety net without manual work

### Real-World Benefits

**For Solo Producers:**
- Experiment fearlessly (can always undo)
- Track project evolution over time
- Never lose work to crashes or mistakes
- Compare different mix versions

**For Teams:**
- No more "Who's editing the project?"
- Clear ownership via locks
- Structured handoff workflow
- Full audit trail of changes

**For Studios:**
- Client version history
- Revert to any delivered version
- Track project milestones
- Backup and disaster recovery

---

## Getting Started

### System Requirements

**Required:**
- macOS 14.0 (Sonoma) or later
- Logic Pro 11.x (folder-based projects, `.logicx`)
- 500MB free disk space (plus space for your projects)
- Internet connection (for remote sync, optional)

**Recommended:**
- 8GB+ RAM
- SSD storage
- Regular backups (Oxen-VCS is not a backup solution!)

### Installation

#### Step 1: Install Oxen CLI

Open Terminal and run:
```bash
pip3 install oxen-ai
```

Verify installation:
```bash
oxen --version
# Should output: oxen 0.x.x
```

If `pip3` is not found, install it via:
```bash
# Install Homebrew (if not already installed)
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install Python 3
brew install python3
```

#### Step 2: Install Oxen-VCS.app

1. Download `Oxen-VCS.app` from [GitHub Releases](https://github.com/jbacus/oxen-vcs-logic/releases)
2. Drag to `/Applications`
3. Right-click and select "Open" (to allow unsigned app)
4. Grant permissions when prompted:
   - Full Disk Access (for monitoring Logic Pro files)
   - Accessibility (for UI integration)

#### Step 3: Launch Oxen-VCS

1. Open `Oxen-VCS.app` from Applications
2. The daemon starts automatically in the background
3. You should see the Oxen-VCS menu bar icon

**Verify Daemon Status:**
```bash
launchctl list | grep oxenvcs
# Should show: com.oxenvcs.agent with PID
```

---

## Daily Workflows

### Initializing Your First Project

**Prerequisites:**
- Logic Pro project must be in **folder format** (.logicx)
- Not package format (`.logicx` file, not folder)

**Steps:**

1. **Open Oxen-VCS.app**
2. **Click "Add Project"**
3. **Navigate to your .logicx folder** (e.g., `MyProject.logicx`)
4. **Click "Initialize"**

Oxen-VCS will:
- Create `.oxen/` directory (version control data)
- Generate `.oxenignore` file (exclusion patterns)
- Make initial commit
- Start monitoring for changes

**Expected Time:** 10-30 seconds depending on project size

**Troubleshooting:**
- If initialization fails, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- Ensure project is saved and contains `Alternatives/###/ProjectData`

---

### Automatic Versioning (Draft Commits)

Once initialized, Oxen-VCS automatically versions your project as you work.

#### How It Works

1. You edit your project in Logic Pro
2. You save (Cmd+S)
3. FSEvents detects the file change
4. 30-60 second debounce timer starts
5. If no more changes, a "draft commit" is created

**Draft Commits:**
- Automatic, no user action required
- Saved to local `draft` branch
- Provides granular safety net
- Pruned when you create milestone commits

#### Viewing Draft Commits

In Oxen-VCS.app:
1. Select your project
2. Click "History" tab
3. Filter by "Draft" branch

You'll see timestamps like:
```
draft_2025-10-29_14-32-15
draft_2025-10-29_14-45-22
draft_2025-10-29_15-01-08
```

#### When Draft Commits Are Created

**Automatically:**
- After 30-60 seconds of inactivity (no more file changes)
- Before system sleep/shutdown (emergency commit)
- When you manually create a milestone (draft branch rebased)

**NOT Created:**
- During active editing (debounce prevents spam)
- When no changes have been made
- If another user holds the lock (collaboration mode)

---

### Creating Milestone Commits

Milestone commits mark important versions with rich metadata.

#### When to Create Milestones

- **Mix versions:** "Rough Mix v1", "Client Revision 3"
- **Production phases:** "Tracking Complete", "Ready for Mastering"
- **Feature additions:** "Added strings section", "New intro"
- **Before major changes:** "Pre-arrangement change"
- **Client deliverables:** "Master v1.0 - Delivered 2025-10-29"

#### How to Create a Milestone

1. **In Oxen-VCS.app, click "Milestone Commit"**
2. **Fill in the form:**
   ```
   Message: Final mix - ready for mastering
   BPM: 128
   Sample Rate: 48000 Hz
   Key: A Minor
   Tags: mix, final, v3
   ```
3. **Click "Commit"**

**Result:**
- Current draft commits are rebased onto main branch
- Your milestone commit is created on `main`
- Draft branch is reset
- Monitoring continues

#### Commit Message Best Practices

**Good Messages:**
- "Vocal tracking complete - 12 takes recorded"
- "Mix v3 - increased bass, reduced reverb on vox"
- "Client revision: moved chorus earlier per feedback"

**Bad Messages:**
- "Update" (what changed?)
- "Changes" (not descriptive)
- "asdf" (meaningless)

**Structure:**
```
<What> - <Why>

BPM: <tempo>
Sample Rate: <rate>
Key: <key signature>
Tags: <relevant tags>
```

---

### Browsing Project History

View your project's version history to understand its evolution.

#### In Oxen-VCS.app

1. Select your project in the sidebar
2. Click "History" tab
3. Filter by branch (main, draft, feature branches)

**Information Displayed:**
- Commit ID (short hash)
- Message and metadata
- Timestamp
- Author
- Branch

#### Comparing Versions

1. Select two commits in history
2. Click "Compare"
3. View changed files list

**Note:** Binary diff is not human-readable, but you can see:
- Which files changed
- File size differences
- Timestamp of changes

#### Searching History

**By Message:**
```
Search: "mix"
Results: All commits mentioning "mix"
```

**By Metadata:**
```
Filter: BPM = 128
Filter: Key = A Minor
Filter: Tag contains "final"
```

**By Date:**
```
From: 2025-10-01
To: 2025-10-29
```

---

### Rolling Back to Previous Versions

Restore your project to any commit in history.

#### Safety First

**Important:** Rollback is **non-destructive** if you have uncommitted changes:
1. Current state is auto-committed to draft branch
2. Project is restored to selected commit
3. Your recent work is preserved and can be restored

**Recommendation:** Always create a milestone commit before rolling back.

#### How to Rollback

1. **In Oxen-VCS.app, select your project**
2. **Click "History" tab**
3. **Find the commit you want to restore to**
4. **Click the commit, then "Rollback"**
5. **Review the confirmation dialog:**
   ```
   Rolling back to:
   Commit: abc123f
   Message: Mix v2 - before bass boost
   Date: 2025-10-28 14:32:15

   Your current work will be saved to draft branch.

   [Cancel] [Rollback]
   ```
6. **Click "Rollback"**

**Expected Time:** 5-15 seconds depending on project size

**What Happens:**
- Current state committed to `draft_rollback_backup_<timestamp>`
- Files restored to selected commit state
- Logic Pro project is now at that version
- You can re-open in Logic Pro

#### After Rolling Back

**If you like the old version:**
- Continue working from this point
- New commits build on this version

**If you want your recent work back:**
1. Go to History
2. Find `draft_rollback_backup_<timestamp>`
3. Rollback to that commit
4. You're back where you started

---

## Collaboration

Oxen-VCS uses **pessimistic locking** to enable safe multi-user workflows.

### How Collaboration Works

**Key Concept:** Only one person can edit the project at a time.

1. **Acquire Lock** - You request exclusive edit rights
2. **Edit Project** - Make your changes in Logic Pro
3. **Commit & Push** - Save your work and release lock
4. **Release Lock** - Others can now acquire it

### Acquiring a Lock

**Before editing a shared project:**

1. **In Oxen-VCS.app, click "Acquire Lock"**
2. **Wait for confirmation (usually <1 second)**
3. **Begin editing in Logic Pro**

**If lock is held by someone else:**
```
Lock currently held by: john@studio.com
Acquired: 2025-10-29 10:30 AM
Duration: 2 hours 15 minutes

The project is read-only until the lock is released.

[Contact john@studio.com] [Force Break (Admin Only)]
```

### While You Hold the Lock

**Your Responsibilities:**
- Edit the project as needed
- Don't hold the lock overnight (others are waiting!)
- Commit your work when done
- Release the lock when finished

**Lock Timeout:** 4 hours by default
- After 4 hours, lock auto-expires
- Others can acquire it
- You'll receive a warning at 3.5 hours

### Releasing the Lock

**Normal Release:**
1. Create a milestone commit (to save your work)
2. Click "Release Lock" in Oxen-VCS.app
3. Others can now acquire it

**Auto-Release:**
- Lock times out after 4 hours
- You close Logic Pro and commit (optional feature)
- You quit Oxen-VCS.app (commits + releases)

### Force-Breaking a Lock (Emergency Only)

**When to use:**
- Lock holder is unreachable
- Emergency edit needed
- Lock holder forgot to release (beyond timeout)

**How:**
1. Contact lock holder first (via email/chat shown in dialog)
2. If no response, click "Force Break"
3. Confirm in dialog
4. Lock is transferred to you

**Warning:** Force-breaking can cause:
- Lost work if lock holder was editing
- Conflicts if they commit later
- Team friction

**Best Practice:** Only force-break if absolutely necessary.

### Collaboration Best Practices

**Communication:**
- Use Slack/Discord to coordinate editing sessions
- Announce when you're acquiring lock
- Estimate how long you'll need
- Notify team when releasing

**Workflow:**
1. Morning standup: Who needs to edit today?
2. Schedule lock acquisition times
3. Quick edits: Hold lock for <1 hour
4. Long sessions: Coordinate in advance

**Branch Workflow:**
Instead of fighting over main branch lock:
1. Create feature branch for your work
2. Edit on your branch (no lock needed)
3. When done, merge to main (manual merge)
4. See [Manual Merge Protocol](#manual-merge-protocol)

---

## Best Practices

### Project Organization

**Folder Structure:**
```
Projects/
├── MyProject.logicx/           # Version controlled
│   ├── .oxen/                  # Oxen data (automatic)
│   ├── .oxenignore             # Exclusion rules
│   ├── Alternatives/           # Tracked
│   │   └── 001/
│   │       └── ProjectData     # Core project state
│   ├── Resources/              # Tracked
│   │   └── Audio Files/        # Your recordings
│   ├── Bounces/                # IGNORED
│   ├── Freeze Files/           # IGNORED
│   └── Autosave/               # IGNORED
```

**Do:**
- Keep projects in folder format (`.logicx` folders)
- Use descriptive project names
- Organize by client/album/genre

**Don't:**
- Use package format (cannot be versioned)
- Nest projects inside projects
- Store projects on network drives (performance issues)

### Commit Message Conventions

**Template:**
```
<Action> - <Description>

BPM: <tempo if relevant>
Sample Rate: <rate if changed>
Key: <key signature if known>
Tags: <phase>, <version>, <status>
```

**Actions:**
- **Add:** New element added (e.g., "Add bass guitar track")
- **Update:** Existing element modified (e.g., "Update drum mix levels")
- **Fix:** Correction made (e.g., "Fix timing on chorus vocals")
- **Remove:** Element deleted (e.g., "Remove unused synth tracks")
- **Mix:** Mixing changes (e.g., "Mix - v3 client feedback")
- **Master:** Mastering adjustments
- **Track:** Recording session

**Examples:**
```
Add - Guitar solo in bridge section

BPM: 120
Key: E Minor
Tags: tracking, lead-guitar, bridge

---

Mix - v2.1: Increased vocal presence, reduced reverb

BPM: 128
Sample Rate: 48000
Tags: mix, revision, client-feedback

---

Fix - Timing correction on kick drum in verse 2

Tags: editing, drums, timing
```

### When to Use Draft vs Milestone Commits

**Draft Commits (Automatic):**
- Continuous safety net
- Every save while working
- Temporary, will be pruned
- Don't require thought

**Milestone Commits (Manual):**
- Important versions
- Before major changes
- After significant progress
- Client deliverables
- End of session
- Before sharing project

**Anti-Pattern:** Creating milestone commits every 5 minutes
**Better:** Let drafts handle frequent saves, milestones for significant points

### Managing Large Projects (10GB+)

**Challenges:**
- Longer commit times (30-60 seconds)
- More disk space needed
- Slower rollback operations

**Optimization Tips:**

1. **Clean Up Regularly:**
   ```bash
   # Remove old Bounces (they're ignored anyway)
   rm -rf MyProject.logicx/Bounces/*

   # Remove Freeze Files (regenerable)
   rm -rf "MyProject.logicx/Freeze Files"/*
   ```

2. **Prune Old Drafts:**
   - Oxen-VCS automatically keeps last 50 drafts
   - Older drafts are pruned when you create milestones

3. **Use .oxenignore Wisely:**
   - Ensure Bounces/ and Freeze Files/ are ignored
   - Add custom patterns for temp files

4. **Monitor Disk Space:**
   ```bash
   # Check Oxen repository size
   du -sh MyProject.logicx/.oxen
   ```

5. **Remote Sync:**
   - Push to Oxen Hub or self-hosted remote
   - Offload old commits to remote storage
   - Local disk space freed up

---

## Understanding .oxenignore

The `.oxenignore` file controls which files are excluded from version control.

### Default Exclusions

**Volatile/Generated Files:**
```gitignore
Bounces/              # User-exported audio
Freeze Files/         # Track freezes
Autosave/             # Automatic backups
*.nosync              # iCloud exclusions
Media.localized/      # System localized names
```

**Why excluded:**
- Large files that bloat repository
- Easily regenerable from project state
- Change frequently (cause conflicts)
- Not needed for project restoration

**System Files:**
```gitignore
.DS_Store             # Finder metadata
*.smbdelete*          # Network share markers
.TemporaryItems       # System temp files
.Trashes              # Trash metadata
.fseventsd            # Filesystem events
```

**Why excluded:**
- macOS-specific, no value in VCS
- Change constantly
- User/machine specific

**Cache/Temporary:**
```gitignore
*.cache
*.tmp
*~                    # Backup file marker
```

### Customizing .oxenignore

**Location:** `MyProject.logicx/.oxenignore`

**Format:** Same as `.gitignore`
- One pattern per line
- `#` for comments
- `*` wildcard
- `/` for directories
- `!` to negate (include)

**Common Additions:**

```gitignore
# Custom Ignore Patterns

# Exclude specific plugin cache
**/PluginData/UAD/*

# Exclude large sample libraries (if not essential)
Samples/Orchestral/            # 50GB+ library

# Exclude video files (if doing post-production)
*.mov
*.mp4

# Exclude personal notes
NOTES_PRIVATE.md
```

**Important:** If you modify `.oxenignore`:
1. Existing tracked files remain tracked
2. To un-track: Remove from Oxen, then add to `.oxenignore`
3. New files matching patterns will be automatically ignored

### What Gets Tracked

By default, Oxen-VCS tracks:

**Essential:**
- `Alternatives/###/ProjectData` (core project state)
- `Resources/Audio Files/` (your recordings)
- Plugin states (embedded in ProjectData)
- Automation data
- MIDI regions

**Optional but Recommended:**
- Custom impulse responses (if small)
- Project-specific samples (if used)
- `README.md` or project notes

---

## Advanced Topics

### Branch-Based Workflows

Create separate branches for experimental work.

**Use Cases:**
- Trying different arrangements
- Alternative mixes
- Collaboration on different sections

**Example:**
```bash
cd MyProject.logicx

# Create feature branch
oxen checkout -b feature/orchestral-intro

# Make changes in Logic Pro
# ... edit, save, auto-commits to draft ...

# Create milestone
# (via Oxen-VCS.app: Milestone Commit)

# Switch back to main
oxen checkout main

# Merge (manual via FCP XML)
# See Manual Merge Protocol below
```

### Manual Merge Protocol

When branches diverge, use FCP XML for track-level merging.

**Scenario:** You worked on `feature/vocal-harmonies`, colleague worked on `main` with drum changes.

**Steps:**

1. **Export your branch tracks:**
   - Open project on `feature/vocal-harmonies`
   - File → Export → All Tracks as Audio Files (FCP XML)
   - Save as `vocal-harmonies.xml`

2. **Switch to main branch:**
   ```bash
   oxen checkout main
   ```

3. **Import your tracks:**
   - Open project (now on main)
   - File → Import → FCP XML
   - Select `vocal-harmonies.xml`
   - Logic Pro creates new tracks

4. **Manually reconcile:**
   - Review both sets of tracks
   - Copy/move regions as needed
   - Delete redundant tracks
   - Mix appropriately

5. **Commit merged result:**
   - Milestone Commit: "Merge feature/vocal-harmonies into main"

**Note:** This is manual, but prevents data loss and gives full control.

### Remote Synchronization

Push your project to Oxen Hub or self-hosted remote.

**Benefits:**
- Cloud backup
- Collaboration from different locations
- Disaster recovery
- Share with collaborators

**Setup:**

1. **Create Oxen Hub account:** [https://oxen.ai/signup](https://oxen.ai/signup)

2. **Create remote repository:**
   ```bash
   # Via web UI or CLI
   oxen config --set remote.origin.url https://hub.oxen.ai/yourname/myproject
   ```

3. **Push your project:**
   ```bash
   cd MyProject.logicx
   oxen push origin main
   ```

**Ongoing:**
- Milestone commits are automatically pushed (optional setting)
- Or manually push: `oxen push origin main`
- Pull collaborator changes: `oxen pull origin main`

**Data Privacy:**
- Your audio files are encrypted in transit
- Oxen Hub uses industry-standard security
- Self-hosted option available for sensitive projects

---

## Troubleshooting

For comprehensive troubleshooting, see [TROUBLESHOOTING.md](TROUBLESHOOTING.md).

### Quick Fixes

**Daemon Not Running:**
```bash
launchctl list | grep oxenvcs
# If empty, load manually:
launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist
```

**Auto-Commits Not Working:**
1. Check daemon status (above)
2. Verify project is initialized: `ls MyProject.logicx/.oxen`
3. Check logs: `log show --predicate 'process == "OxVCS-LaunchAgent"' --last 30m`

**Commits Taking Too Long:**
- Expected for large projects (10GB+)
- Check disk I/O: Is Time Machine running?
- Ensure Bounces/ and Freeze Files/ are ignored

**Lock Held by Unknown User:**
- Check lock status in app
- Contact listed user
- If unreachable, force-break (with caution)

---

## Getting Help

### Resources

- **Documentation:** [https://github.com/jbacus/oxen-vcs-logic/docs](https://github.com/jbacus/oxen-vcs-logic/docs)
- **FAQ:** [FAQ.md](FAQ.md)
- **Troubleshooting:** [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- **Quick Start:** [QUICKSTART_GUIDE.md](QUICKSTART_GUIDE.md)

### Support Channels

- **GitHub Issues:** [https://github.com/jbacus/oxen-vcs-logic/issues](https://github.com/jbacus/oxen-vcs-logic/issues)
- **Discord:** [Join Server](#) (Coming soon)
- **Email:** support@oxen-vcs.com

### Reporting Bugs

When reporting issues, include:
1. macOS version (`sw_vers`)
2. Logic Pro version (Logic Pro → About Logic Pro)
3. Oxen version (`oxen --version`)
4. Project size (`du -sh MyProject.logicx`)
5. Error messages (screenshots or copy-paste)
6. Steps to reproduce

### Feature Requests

We welcome feature requests! Submit via GitHub Issues with:
- Use case description
- Expected behavior
- Why it would help your workflow

---

## Appendix

### Keyboard Shortcuts (Oxen-VCS.app)

| Action | Shortcut |
|--------|----------|
| New Project | ⌘N |
| Milestone Commit | ⌘K |
| Refresh History | ⌘R |
| Acquire Lock | ⌘L |
| Release Lock | ⌘⇧L |
| Settings | ⌘, |

### Command-Line Reference

```bash
# Initialize repository
oxen init /path/to/project.logicx

# Check status
cd /path/to/project.logicx
oxen status

# View history
oxen log

# Create commit
oxen commit -m "Message here"

# List branches
oxen branch

# Switch branch
oxen checkout branch-name

# Push to remote
oxen push origin main

# Pull from remote
oxen pull origin main
```

### File Structure Reference

```
MyProject.logicx/
├── .oxen/                      # Oxen repository data
│   ├── objects/                # Deduplicated file blocks
│   ├── refs/                   # Branch pointers
│   └── config                  # Repository configuration
├── .oxenignore                 # Exclusion patterns
├── Alternatives/               # TRACKED
│   └── 001/
│       └── ProjectData         # Core project file
├── Resources/                  # TRACKED
│   ├── Audio Files/            # Your recordings
│   └── Media/                  # Imported media
├── Bounces/                    # IGNORED
├── Freeze Files/               # IGNORED
└── Autosave/                   # IGNORED
```

---

**End of User Guide**

*For more information, see [FAQ.md](FAQ.md) or [TROUBLESHOOTING.md](TROUBLESHOOTING.md).*
