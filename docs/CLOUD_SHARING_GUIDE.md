# OxVCS Cloud Sharing Guide

**Last Updated**: 2025-11-15
**Status**: Phase 1 (Authentication) Complete, Phases 2-4 In Development

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

OxVCS Cloud Sharing enables GitHub-like collaboration for Logic Pro projects through Oxen Hub. Key features include:

- **Remote repository hosting** via Oxen Hub (https://hub.oxen.ai)
- **Distributed pessimistic locking** to prevent merge conflicts
- **Team collaboration** with access control
- **Activity feeds** for project tracking
- **Network resilience** with automatic retry

### Why Cloud Sharing?

Logic Pro projects consist of large binary files that cannot be merged algorithmically. OxVCS solves this with:
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

3. **OxVCS CLI** installed:
   ```bash
   ./install.sh
   # or manually build
   cd OxVCS-CLI-Wrapper && cargo build --release
   ```

### Quick Start (5 minutes)

```bash
# 1. Authenticate with Oxen Hub
oxenvcs-cli auth login
# Enter username and API key when prompted

# 2. Initialize project (if not already done)
cd MyProject.logicx
oxenvcs-cli init --logic .

# 3. Add remote repository (create repo on hub.oxen.ai first)
oxen remote add origin https://hub.oxen.ai/username/my-project

# 4. Push to cloud
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial commit" --bpm 120
oxen push origin main

# 5. Verify
oxenvcs-cli auth status
```

---

## Authentication

### Login to Oxen Hub

```bash
oxenvcs-cli auth login
```

You will be prompted for:
- **Username**: Your Oxen Hub username
- **API Key**: Generated from https://hub.oxen.ai/settings/api-keys

#### Where Credentials Are Stored

OxVCS uses **dual storage** for maximum compatibility:

1. **Primary**: Oxen CLI config (`~/.oxen/user_config.toml`)
   - Leverages Oxen's built-in credential management
   - Shared with `oxen` CLI commands

2. **Fallback**: OxVCS config (`~/.oxenvcs/credentials`)
   - Stores username and hub URL
   - API key stored only in Oxen config
   - File permissions: `0600` (user read/write only)

### Check Authentication Status

```bash
oxenvcs-cli auth status
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

Run 'oxenvcs-cli auth test' to verify connection
```

### Test Connection

```bash
oxenvcs-cli auth test
```

Verifies that your stored credentials are valid by testing connection to Oxen Hub.

### Logout

```bash
oxenvcs-cli auth logout
```

Removes credentials from both Oxen CLI config and OxVCS config.

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
oxenvcs-cli add --all

# Create initial commit
oxenvcs-cli commit -m "Initial project setup" --bpm 120

# Push to Oxen Hub
oxen push origin main
```

**Note**: First push may take several minutes for large projects (GB-scale audio files). Oxen uses block-level deduplication, so subsequent pushes are much faster.

### Clone Existing Project

To collaborate on an existing project:

```bash
oxen clone https://hub.oxen.ai/username/my-logic-project MyProject.logicx
cd MyProject.logicx
oxenvcs-cli auth status  # Verify authentication
```

---

## Team Collaboration

### Best Practices

1. **Always acquire lock before editing**:
   ```bash
   oxenvcs-cli lock acquire --timeout 4
   # Open project in Logic Pro
   # Make changes
   # Commit and push
   oxenvcs-cli lock release
   ```

2. **Pull latest changes before starting work**:
   ```bash
   oxen pull origin main
   ```

3. **Commit frequently with descriptive messages**:
   ```bash
   oxenvcs-cli commit -m "Added bass track, adjusted mix" --bpm 128
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
oxenvcs-cli lock acquire --timeout 8   # Lock for 8 hours
# ... work in Logic Pro ...
oxenvcs-cli add --all
oxenvcs-cli commit -m "Recorded vocals" --bpm 120
oxen push origin main
oxenvcs-cli lock release
```

**Team Member B** (Mixer):
```bash
# Afternoon: Continue work
cd MyProject.logicx
oxen pull origin main                  # Get A's vocal recordings
oxenvcs-cli lock acquire --timeout 4
# ... mixing work ...
oxenvcs-cli commit -m "Mixed vocals, added reverb" --bpm 120
oxen push origin main
oxenvcs-cli lock release
```

---

## Lock Management

### Overview

**Pessimistic locking** prevents merge conflicts by allowing only one editor at a time. This is essential for binary Logic Pro files which cannot be merged algorithmically.

### Acquire Lock

```bash
oxenvcs-cli lock acquire --timeout 4
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
Remember to release the lock when done: oxenvcs-cli lock release
```

### Check Lock Status

```bash
oxenvcs-cli lock status
```

Shows who holds the lock and when it expires.

### Release Lock

```bash
oxenvcs-cli lock release
```

**When to release**:
- Done editing for the session
- Committed your changes
- Switching to a different task

### Break Lock (Emergency)

```bash
oxenvcs-cli lock break --force
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
oxenvcs-cli auth login      # Authenticate
oxenvcs-cli auth status     # Check status
oxenvcs-cli auth test       # Verify connection
oxenvcs-cli auth logout     # Sign out
```

### 🚧 Phase 2: Distributed Lock Management (IN DEVELOPMENT)

**Planned Features:**
- Remote lock storage (`.oxen/locks.json` tracked file)
- Atomic lock acquisition via commit + force-push
- Lock heartbeat mechanism (renew every 10 minutes)
- Race condition detection
- Stale lock cleanup (>48h)

**Target**: 2-3 weeks

### 🚧 Phase 3: Collaboration Features (PLANNED)

**Planned Features:**
- Activity feed with commit timeline
- Team member roster
- Comment system on commits
- @mention notifications
- Branch visualization

**Target**: 3-4 weeks

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

**Problem**: `oxenvcs-cli auth test` fails with "Authentication failed"

**Solutions:**
1. Verify credentials:
   ```bash
   oxen config user.name
   oxen config user.api_key
   ```

2. Re-login:
   ```bash
   oxenvcs-cli auth logout
   oxenvcs-cli auth login
   ```

3. Check API key validity at https://hub.oxen.ai/settings/api-keys

### Push Failures

**Problem**: `oxen push origin main` fails with "401 Unauthorized"

**Solution**:
```bash
oxenvcs-cli auth test  # Verify authentication
oxenvcs-cli auth login # Re-authenticate if needed
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
   oxenvcs-cli auth status
   ```

### Lock Conflicts

**Problem**: Cannot acquire lock (already locked by someone else)

**Solution**:
```bash
# Check lock status
oxenvcs-cli lock status

# Wait for lock to expire OR
# Contact lock holder to release

# Emergency only (with team approval):
oxenvcs-cli lock break --force
```

---

## Best Practices

### 1. Commit Messages

Use descriptive messages that explain **what** and **why**:

✅ Good:
```bash
oxenvcs-cli commit -m "Added drum automation, fixed timing issues in chorus" --bpm 128
```

❌ Bad:
```bash
oxenvcs-cli commit -m "updates"
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

### Q: How is OxVCS different from Git-LFS?

**A**: Key differences:
- **Storage efficiency**: Oxen uses block-level deduplication vs. file-level in Git-LFS (10-100x better)
- **Locking**: Built-in pessimistic locking for binary files
- **Performance**: Optimized for large binary datasets (GB-TB scale)
- **Native integration**: Designed for DAW workflows, not generic version control

### Q: Can I use OxVCS with self-hosted Oxen server?

**A**: Yes! Configure custom hub URL during login:
```bash
# After login, manually configure:
oxen config remote.hub_url https://your-oxen-server.com
```

### Q: What happens if I edit without acquiring a lock?

**A**: You risk creating conflicts that cannot be automatically merged. **Always acquire lock before editing** Logic Pro projects in collaborative workflows.

### Q: How much does Oxen Hub cost?

**A**: Check https://hub.oxen.ai/pricing for current plans. Free tier available for small projects.

### Q: Can I migrate from Git to OxVCS?

**A**: Yes, but history won't transfer. Workflow:
1. Archive Git repository
2. Initialize OxVCS repository
3. Create initial commit with current state
4. Push to Oxen Hub
5. Team clones from Oxen Hub

---

## Resources

### Documentation
- [OxVCS User Guide](USER_GUIDE.md)
- [Oxen.ai Documentation](https://docs.oxen.ai)
- [Architecture Overview](FOR_DEVELOPERS.md)

### Getting Help
- **GitHub Issues**: https://github.com/jbacus/oxen-vcs-logic/issues
- **Oxen Community**: hello@oxen.ai
- **Documentation**: Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)

### Related Tools
- [Oxen CLI](https://github.com/Oxen-AI/Oxen) - Core version control engine
- [Logic Pro](https://www.apple.com/logic-pro/) - Digital Audio Workstation

---

## Roadmap

### Completed
- ✅ Phase 1: Authentication system (2025-11-15)

### In Development
- 🚧 Phase 2: Distributed lock management
- 🚧 Phase 3: Collaboration features
- 🚧 Phase 4: Network resilience

### Future
- 📅 Phase 5: Advanced features (semantic diff, audio preview, automated merge)
- 📅 Web UI for project management
- 📅 Mobile app for project browsing
- 📅 CI/CD integrations

---

*Last updated: 2025-11-15 | OxVCS v0.1.0 | Oxen integration via subprocess wrapper*
