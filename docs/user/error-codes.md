# Auxin Error Codes & Troubleshooting

**Last Updated**: 2025-11-20
**Purpose**: Reference for error messages, codes, and recovery steps

---

## Quick Reference

| Exit Code | Category | Common Cause |
|-----------|----------|--------------|
| 0 | Success | Operation completed |
| 1 | General Error | Various issues |
| 2 | Invalid Arguments | Wrong command syntax |
| 3 | Authentication | Login required |
| 4 | Lock Conflict | Project locked by another user |
| 5 | Network Error | Connection issues |
| 6 | Not Repository | Not in an Auxin project |

---

## Error Categories

### Authentication Errors

#### "Authentication required"
**Code**: 3

**Cause**: Not logged in to Oxen Hub

**Solution**:
```bash
auxin auth login
```

Then enter your username and API key from https://hub.oxen.ai/settings

---

#### "Invalid credentials"
**Cause**: Wrong username or API key

**Solution**:
1. Get your API key from https://hub.oxen.ai/settings
2. Try logging in again:
   ```bash
   auxin auth logout
   auxin auth login
   ```

---

#### "Token expired"
**Cause**: API token needs refresh

**Solution**:
```bash
auxin auth test  # Check status
auxin auth login  # Re-authenticate
```

---

### Lock Errors

#### "Project is locked by another user"
**Code**: 4

**Cause**: Someone else has the exclusive lock

**Shows**: Lock holder, when acquired, expiration time

**Solution**:
1. Wait for them to release:
   ```bash
   auxin lock status  # Check when it expires
   ```

2. Contact the lock holder (shown in status)

3. If urgent and they're unavailable:
   ```bash
   auxin lock break --force
   ```
   **Warning**: They may lose unsaved work!

---

#### "Lock expired while editing"
**Cause**: Your lock timed out

**Solution**:
1. Acquire a new lock:
   ```bash
   auxin lock acquire --timeout 8
   ```

2. Use longer timeout for long sessions:
   ```bash
   auxin lock acquire --timeout 12
   ```

---

#### "Cannot break lock: not authorized"
**Cause**: Trying to break someone else's lock without `--force`

**Solution**:
```bash
auxin lock break --force
```

---

### Network Errors

#### "Network error: connection refused"
**Code**: 5

**Cause**: Cannot connect to Oxen Hub or Auxin server

**Solution**:
1. Check internet connection
2. Check if Oxen Hub is up: https://hub.oxen.ai
3. If using private server, verify it's running:
   ```bash
   auxin server health
   ```

---

#### "Network error: timeout"
**Cause**: Server took too long to respond

**Solution**:
1. Try again (may be temporary)
2. For large files, increase timeout:
   ```bash
   export AUXIN_NETWORK_TIMEOUT=120
   auxin push
   ```

3. Check network quality:
   ```bash
   auxin network check
   ```

---

#### "Upload interrupted"
**Cause**: Connection lost during push

**Solution**: Auxin automatically resumes uploads:
```bash
auxin push  # Will resume from last chunk
```

If repeated failures:
1. Check connection stability
2. Try smaller commits
3. Use offline mode:
   ```bash
   auxin commit -m "My changes"  # Queued locally
   # Later when connection stable:
   auxin queue sync
   ```

---

#### "Rate limited"
**Cause**: Too many requests to Oxen Hub

**Solution**: Wait and retry. Auxin automatically backs off:
- 2s, 4s, 8s, 16s delays between retries

---

### Repository Errors

#### "Not an Auxin repository"
**Code**: 6

**Cause**: No `.oxen` directory found

**Solution**:
1. Navigate to your project:
   ```bash
   cd ~/Music/MyProject.logicx
   ```

2. Initialize if needed:
   ```bash
   auxin init .
   ```

---

#### "Invalid project structure"
**Cause**: Project folder doesn't match expected format

**For Logic Pro**:
- Missing `projectData` file
- Not a `.logicx` bundle

**For SketchUp**:
- Not a `.skp` file
- File is corrupted

**Solution**:
1. Verify file exists and opens in the application
2. Check you're in the right directory:
   ```bash
   ls -la  # Should show project files
   ```

---

#### "Already initialized"
**Cause**: Repository already exists

**Solution**: Repository is ready to use. Check status:
```bash
auxin status
```

---

### Commit Errors

#### "Nothing to commit"
**Cause**: No staged changes

**Solution**:
1. Check what changed:
   ```bash
   auxin status
   ```

2. Stage your changes:
   ```bash
   auxin add --all
   ```

---

#### "Pre-commit hook failed"
**Cause**: Hook script returned non-zero

**Solution**:
1. Check hook output for details
2. Fix the issue (e.g., add required metadata)
3. Or skip hooks temporarily:
   ```bash
   auxin commit -m "Message" --no-verify
   ```

**Common hook failures**:
- Missing BPM: Add `--bpm 120`
- File too large: Reduce before commit

---

#### "Invalid metadata format"
**Cause**: Metadata value doesn't match expected type

**Examples**:
- BPM must be number: `--bpm 120` not `--bpm "fast"`
- Sample rate must be standard: `44100`, `48000`, `96000`
- Units must be recognized: `Inches`, `Feet`, `Meters`

---

### Daemon Errors

#### "Daemon not running"
**Cause**: Background service is stopped

**Solution**:
```bash
auxin daemon start
```

If it won't start:
```bash
auxin daemon logs --lines 100  # Check errors
```

---

#### "Cannot connect to daemon"
**Cause**: XPC communication failed

**Solution**:
1. Restart daemon:
   ```bash
   auxin daemon restart
   ```

2. If still failing, check system permissions:
   - System Settings > General > Login Items
   - Enable "Auxin Daemon"

---

#### "Daemon requires approval"
**Cause**: First-time installation needs permission

**Solution**:
1. Open System Settings
2. Go to General > Login Items & Extensions
3. Find "Auxin Daemon" and toggle ON

---

### Server Errors

#### "Server unreachable"
**Cause**: Cannot connect to Auxin server

**Solution**:
```bash
# Check configuration
auxin server status

# Test connection
auxin server health

# Update URL if wrong
auxin server set url http://correct-server:3000
```

---

#### "Server error: 500"
**Cause**: Internal server error

**Solution**:
1. Try again (may be temporary)
2. Check server logs (admin)
3. Report if persistent

---

## Recovery Procedures

### Recovering from Failed Push

If push fails repeatedly:

```bash
# Check what's queued
auxin queue list

# Force sync when ready
auxin queue sync --force

# Or clear and re-commit
auxin queue clear
auxin add --all
auxin commit -m "Re-commit after failure"
auxin push
```

---

### Recovering from Corrupted Repository

If repository seems corrupted:

```bash
# Verify repository integrity
auxin fsck

# If issues found, try repair
auxin fsck --repair

# Nuclear option: re-clone
cd ..
mv MyProject.logicx MyProject.logicx.backup
oxen clone https://hub.oxen.ai/user/repo MyProject.logicx
```

---

### Recovering Lost Work

If you need to restore previous version:

```bash
# Find the commit you want
auxin log --limit 20

# Restore to that commit
auxin restore abc123f

# Or cherry-pick files from commit
auxin show abc123f  # See what files
# Then manually restore specific files
```

---

## Getting Help

### Diagnostic Information

When reporting issues, include:

```bash
# Version information
auxin --version

# System information
uname -a

# Repository status
auxin status

# Recent logs
auxin daemon logs --lines 100 2>&1

# Network status
auxin network check
```

### Support Channels

1. **GitHub Issues**: https://github.com/jbacus/auxin/issues
2. **Documentation**: Check [troubleshooting guide](troubleshooting.md)
3. **Logs**: `/tmp/com.auxin.daemon.stdout` and `.stderr`

---

*Last Updated: 2025-11-20*
