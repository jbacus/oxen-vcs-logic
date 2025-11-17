# Auxin Cloud Sharing Guide

**Last Updated**: 2025-11-15
**Status**: Phases 1-3 Complete (Authentication, Locks, Collaboration), Phase 4+ In Development

---

## Table of Contents

1. [Overview](#overview)
2. [Getting Started](#getting-started)
3. [Authentication](#authentication)
4. [Remote Repository Setup](#remote-repository-setup)
5. [Team Collaboration](#team-collaboration)
6. [Lock Management](#lock-management)
7. [Troubleshooting](#troubleshooting)
8. [Implementation Status](#implementation-status)

---

## Overview

Auxin Cloud Sharing enables GitHub-like collaboration for Logic Pro projects through Oxen Hub. Key features include:

- **Remote repository hosting** via Oxen Hub (https://hub.oxen.ai)
- **Distributed pessimistic locking** to prevent merge conflicts
- **Team collaboration** with access control
- **Activity feeds** for project tracking
- **Network resilience** with automatic retry

### Why Cloud Sharing?

Logic Pro projects consist of large binary files that cannot be merged algorithmically. Auxin solves this with:
- **Block-level deduplication** (10-100x more efficient than Git-LFS)
- **Pessimistic locking** (one editor at a time, no conflicts)
- **Automatic sync** with progress tracking

---

## Getting Started

### Prerequisites

1. **Oxen CLI** must be installed:
   ```bash
   pip install oxen-ai
   # or
   cargo install oxen
   ```

2. **Oxen Hub account**:
   - Visit https://hub.oxen.ai
   - Sign up for a free account
   - Generate an API key from Settings → API Keys

3. **Auxin CLI** installed:
   ```bash
   ./install.sh
   # or manually build
   cd Auxin-CLI-Wrapper && cargo build --release
   ```

### Quick Start (5 minutes)

```bash
# 1. Authenticate with Oxen Hub
auxin auth login
# Enter username and API key when prompted

# 2. Initialize project (if not already done)
cd MyProject.logicx
auxin init --logic .

# 3. Add remote repository (create repo on hub.oxen.ai first)
oxen remote add origin https://hub.oxen.ai/username/my-project

# 4. Push to cloud
auxin add --all
auxin commit -m "Initial commit" --bpm 120
oxen push origin main

# 5. Verify
auxin auth status
```

---

## Authentication

### Login to Oxen Hub

```bash
auxin auth login
```

You will be prompted for:
- **Username**: Your Oxen Hub username
- **API Key**: Generated from https://hub.oxen.ai/settings/api-keys

#### Where Credentials Are Stored

Auxin uses **dual storage** for maximum compatibility:

1. **Primary**: Oxen CLI config (`~/.oxen/user_config.toml`)
   - Leverages Oxen's built-in credential management
   - Shared with `oxen` CLI commands

2. **Fallback**: Auxin config (`~/.auxin/credentials`)
   - Stores username and hub URL
   - API key stored only in Oxen config
   - File permissions: `0600` (user read/write only)

### Check Authentication Status

```bash
auxin auth status
```

**Output:**
```
┌─ Authentication Status ─────────────────────────────────┐
│                                                          │
│  Status: ● Authenticated                                │
│                                                          │
│  Username: johndoe                                       │
│  Hub URL:  https://hub.oxen.ai                          │
│                                                          │
└──────────────────────────────────────────────────────────┘

Run 'auxin auth test' to verify connection
```

### Test Connection

```bash
auxin auth test
```

Verifies that your stored credentials are valid by testing connection to Oxen Hub.

### Logout

```bash
auxin auth logout
```

Removes credentials from both Oxen CLI config and Auxin config.

---

## Remote Repository Setup

### Create Repository on Oxen Hub

1. Go to https://hub.oxen.ai
2. Click **"New Repository"**
3. Enter repository name (e.g., `my-logic-project`)
4. Choose **Private** (recommended for production work)
5. Click **Create**

### Connect Local Project to Remote

```bash
# In your .logicx directory
oxen remote add origin https://hub.oxen.ai/username/my-logic-project
```

Verify remote configuration:
```bash
oxen remote -v
```

### First Push

```bash
# Stage all files
auxin add --all

# Create initial commit
auxin commit -m "Initial project setup" --bpm 120

# Push to Oxen Hub
oxen push origin main
```

**Note**: First push may take several minutes for large projects (GB-scale audio files). Oxen uses block-level deduplication, so subsequent pushes are much faster.

### Clone Existing Project

To collaborate on an existing project:

```bash
oxen clone https://hub.oxen.ai/username/my-logic-project MyProject.logicx
cd MyProject.logicx
auxin auth status  # Verify authentication
```

---

## Team Collaboration

### Best Practices

1. **Always acquire lock before editing**:
   ```bash
   auxin lock acquire --timeout 4
   # Open project in Logic Pro
   # Make changes
   # Commit and push
   auxin lock release
   ```

2. **Pull latest changes before starting work**:
   ```bash
   oxen pull origin main
   ```

3. **Commit frequently with descriptive messages**:
   ```bash
   auxin commit -m "Added bass track, adjusted mix" --bpm 128
   ```

4. **Push milestone commits to share with team**:
   ```bash
   oxen push origin main
   ```

### Workflow Example

**Team Member A** (Producer):
```bash
# Morning: Start work
cd MyProject.logicx
oxen pull origin main                  # Get latest changes
auxin lock acquire --timeout 8   # Lock for 8 hours
# ... work in Logic Pro ...
auxin add --all
auxin commit -m "Recorded vocals" --bpm 120
oxen push origin main
auxin lock release
```

**Team Member B** (Mixer):
```bash
# Afternoon: Continue work
cd MyProject.logicx
oxen pull origin main                  # Get A's vocal recordings
auxin lock acquire --timeout 4
# ... mixing work ...
auxin commit -m "Mixed vocals, added reverb" --bpm 120
oxen push origin main
auxin lock release
```

### Communication & Tracking

**View Project Activity:**
```bash
# See what the team has been working on
auxin activity --limit 20
```

**Output:**
```
┌─ Recent Activity ────────────────────────────────────────┐
│                                                           │
│  ● Commit by john@laptop                                 │
│    "Mixed vocals, added reverb"                          │
│    BPM: 120                                              │
│                                                           │
│  🔒 Lock Acquired by sarah@studio                        │
│    Locked for editing (4 hours)                          │
│                                                           │
│  ● Commit by sarah@studio                                │
│    "Recorded vocals"                                     │
│    BPM: 120, Sample Rate: 48000Hz                        │
└───────────────────────────────────────────────────────────┘
```

**Discover Team Members:**
```bash
# See who's contributing to the project
auxin team
```

**Output:**
```
┌─ Team Members ───────────────────────────────────────────┐
│                                                           │
│  sarah@studio                                            │
│    15 commits  ████████████████████████░░░░░  62%       │
│    Last active: 2 hours ago                              │
│                                                           │
│  john@laptop                                             │
│    9 commits   ██████████████░░░░░░░░░░░░░░░  38%       │
│    Last active: 5 hours ago                              │
└───────────────────────────────────────────────────────────┘
```

**Add Comments to Commits:**
```bash
# Provide feedback on a specific commit
auxin comment add abc123 "Love the vocal processing! Can we try more compression?"

# View comments on a commit
auxin comment list abc123

# Share comments with team (must commit and push)
oxen add .oxen/comments/
oxen commit -m "Add review comments"
oxen push origin main
```

---

## Lock Management

### Overview

**Pessimistic locking** prevents merge conflicts by allowing only one editor at a time. This is essential for binary Logic Pro files which cannot be merged algorithmically.

### Acquire Lock

```bash
auxin lock acquire --timeout 4
```

**Parameters:**
- `--timeout <HOURS>`: Lock expiration time (default: 4 hours)

**Output:**
```
┌─ Lock Acquired ─────────────────────────────────────────┐
│                                                          │
│  ✓ You now have exclusive editing rights                │
│                                                          │
│  Lock expires in: 4 hours                                │
│                                                          │
└──────────────────────────────────────────────────────────┘

You can now safely edit the project in Logic Pro
Remember to release the lock when done: auxin lock release
```

### Check Lock Status

```bash
auxin lock status
```

Shows who holds the lock and when it expires.

### Release Lock

```bash
auxin lock release
```

**When to release**:
- Done editing for the session
- Committed your changes
- Switching to a different task

### Break Lock (Emergency)

```bash
auxin lock break --force
```

**WARNING**: Only use in emergencies:
- Lock holder is unavailable
- Lock expired but wasn't auto-released
- Urgent access needed

Breaking someone else's lock may cause them to lose unsaved work!

---

## Implementation Status

### ✅ Phase 1: Authentication (COMPLETE)

**Features Implemented:**
- ✅ Oxen Hub login/logout
- ✅ API key management (secure storage via Oxen CLI config)
- ✅ Authentication status checking
- ✅ Connection testing
- ✅ CLI commands: `auth login`, `auth logout`, `auth status`, `auth test`
- ✅ Dual credential storage (Oxen config + fallback)
- ✅ 12 unit tests passing

**Usage:**
```bash
auxin auth login      # Authenticate
auxin auth status     # Check status
auxin auth test       # Verify connection
auxin auth logout     # Sign out
```

### ✅ Phase 2: Distributed Lock Management (COMPLETE)

**Features Implemented:**
- ✅ Remote lock storage in dedicated "locks" branch
- ✅ Atomic lock acquisition via fetch → check → commit → push → verify
- ✅ Lock heartbeat mechanism (renew to extend expiration)
- ✅ Race condition detection (polls after push to verify ownership)
- ✅ Stale lock detection (>1 hour no heartbeat)
- ✅ Automatic expiration (configurable timeout)
- ✅ Force break for emergencies
- ✅ CLI commands: `lock acquire`, `lock release`, `lock status`, `lock break`
- ✅ 10+ unit tests passing

**Usage:**
```bash
# Acquire exclusive lock (4 hour timeout)
auxin lock acquire --timeout 4

# Check lock status
auxin lock status

# Release lock when done
auxin lock release

# Emergency break (with confirmation)
auxin lock break --force
```

**Lock Storage:**
- Locks stored in separate `locks` branch (orphan)
- Lock files: `.oxen/locks/<project>.json`
- JSON schema: lock_id, user, machine_id, timestamps
- Atomic operations via Git/Oxen commit+push

**Completed**: 2025-11-15

### ✅ Phase 3: Collaboration Features (COMPLETE)

**Features Implemented:**
- ✅ Activity feed with commit timeline
- ✅ Team member discovery from commit history
- ✅ Comment system on commits (stored in `.oxen/comments/`)
- ✅ CLI commands: `activity`, `team`, `comment add`, `comment list`
- ✅ 7 unit tests passing

**Usage:**
```bash
# View recent project activity
auxin activity --limit 10

# Discover team members and contributions
auxin team

# Add comment to a commit
auxin comment add <commit-id> "Great mix on the drums!"

# View comments on a commit
auxin comment list <commit-id>
```

**Activity Feed Features:**
- Commit timeline with author, message, and metadata (BPM, sample rate)
- Activity type icons (● commits, 🔒 locks, 💬 comments)
- Configurable limit for recent activities

**Team Discovery:**
- Automatic extraction from commit history
- Contribution statistics (commit count, percentage)
- Last activity timestamp per member
- Sorted by most active contributors

**Comment System:**
- Comments stored locally in `.oxen/comments/<commit_hash>.json`
- Must be committed and pushed to share with team
- Supports multiple comments per commit
- Includes author, timestamp, and unique comment ID

**Completed**: 2025-11-15

### 🚧 Phase 4: Network Resilience & Safety (PLANNED)

**Planned Features:**
- Offline mode with commit queue
- Smart retry with exponential backoff
- Partial push recovery
- Pre-pull conflict detection
- Emergency unlock protocol

**Target**: 2-3 weeks

---

## Troubleshooting

### Authentication Issues

**Problem**: `auxin auth test` fails with "Authentication failed"

**Solutions:**
1. Verify credentials:
   ```bash
   oxen config user.name
   oxen config user.api_key
   ```

2. Re-login:
   ```bash
   auxin auth logout
   auxin auth login
   ```

3. Check API key validity at https://hub.oxen.ai/settings/api-keys

### Push Failures

**Problem**: `oxen push origin main` fails with "401 Unauthorized"

**Solution**:
```bash
auxin auth test  # Verify authentication
auxin auth login # Re-authenticate if needed
```

**Problem**: Push times out on large files

**Solution**:
```bash
# Push with verbose output to monitor progress
oxen push origin main --verbose

# For very large projects (>10GB), consider:
# 1. Exclude generated files via .oxenignore
# 2. Push over reliable network connection
# 3. Use staged pushes (push subsets of files)
```

### Clone Issues

**Problem**: `oxen clone` fails with "404 Not Found"

**Solutions:**
1. Verify repository URL:
   ```bash
   # Correct format:
   https://hub.oxen.ai/username/repo-name
   ```

2. Check repository access permissions (private repos require authentication)

3. Ensure you're authenticated:
   ```bash
   auxin auth status
   ```

### Lock Conflicts

**Problem**: Cannot acquire lock (already locked by someone else)

**Solution**:
```bash
# Check lock status
auxin lock status

# Wait for lock to expire OR
# Contact lock holder to release

# Emergency only (with team approval):
auxin lock break --force
```

---

## Best Practices

### 1. Commit Messages

Use descriptive messages that explain **what** and **why**:

✅ Good:
```bash
auxin commit -m "Added drum automation, fixed timing issues in chorus" --bpm 128
```

❌ Bad:
```bash
auxin commit -m "updates"
```

### 2. Lock Hygiene

- **Acquire locks for realistic durations** (don't lock overnight if not needed)
- **Release locks promptly** after committing
- **Renew locks** if work session extends beyond timeout
- **Communicate** with team about lock intentions

### 3. .oxenignore Configuration

Exclude generated/temporary files to reduce repository size:

```gitignore
# Generated audio (regenerable from project)
Bounces/
Freeze Files/

# Temporary Logic Pro files
Autosave/
*.nosync
*.logictemp

# macOS system files
.DS_Store
._*
```

### 4. Push Frequency

- **Milestone commits**: Push after completing significant work
- **End of day**: Push before leaving for the day
- **Before lock release**: Always push committed changes

### 5. Pull Frequency

- **Start of day**: Pull latest changes before starting work
- **Before acquiring lock**: Ensure you have latest state
- **After notifications**: Pull when team members push updates

---

## FAQ

### Q: How is Auxin different from Git-LFS?

**A**: Key differences:
- **Storage efficiency**: Oxen uses block-level deduplication vs. file-level in Git-LFS (10-100x better)
- **Locking**: Built-in pessimistic locking for binary files
- **Performance**: Optimized for large binary datasets (GB-TB scale)
- **Native integration**: Designed for DAW workflows, not generic version control

### Q: Can I use Auxin with self-hosted Oxen server?

**A**: Yes! Configure custom hub URL during login:
```bash
# After login, manually configure:
oxen config remote.hub_url https://your-oxen-server.com
```

### Q: What happens if I edit without acquiring a lock?

**A**: You risk creating conflicts that cannot be automatically merged. **Always acquire lock before editing** Logic Pro projects in collaborative workflows.

### Q: How much does Oxen Hub cost?

**A**: Check https://hub.oxen.ai/pricing for current plans. Free tier available for small projects.

### Q: Can I migrate from Git to Auxin?

**A**: Yes, but history won't transfer. Workflow:
1. Archive Git repository
2. Initialize Auxin repository
3. Create initial commit with current state
4. Push to Oxen Hub
5. Team clones from Oxen Hub

---

## Resources

### Documentation
- [Auxin User Guide](USER_GUIDE.md)
- [Oxen.ai Documentation](https://docs.oxen.ai)
- [Architecture Overview](FOR_DEVELOPERS.md)

### Getting Help
- **GitHub Issues**: https://github.com/jbacus/auxin/issues
- **Oxen Community**: hello@oxen.ai
- **Documentation**: Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### Related Tools
- [Oxen CLI](https://github.com/Oxen-AI/Oxen) - Core version control engine
- [Logic Pro](https://www.apple.com/logic-pro/) - Digital Audio Workstation

---

## Roadmap

### Completed
- ✅ Phase 1: Authentication system (2025-11-15)
- ✅ Phase 2: Distributed lock management (2025-11-15)
- ✅ Phase 3: Collaboration features (2025-11-15)

### In Development
- 🚧 Phase 4: Network resilience

### Future
- 📅 Phase 5: Advanced features (semantic diff, audio preview, automated merge)
- 📅 Web UI for project management
- 📅 Mobile app for project browsing
- 📅 CI/CD integrations

---

*Last updated: 2025-11-15 | Auxin v0.1.0 | Oxen integration via subprocess wrapper*
