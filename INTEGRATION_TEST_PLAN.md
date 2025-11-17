# Integration Test Plan: Collaboration Features

**Created:** November 17, 2025
**Status:** Week 1 - Validation Phase
**Goal:** Validate all collaboration features with real Oxen Hub before building Phase 4

---

## Overview

This document outlines the integration testing strategy for OxVCS collaboration features. Tests are designed to run against **real Oxen Hub** to validate production readiness.

### Test Environment Requirements

- macOS 14.0+ with Xcode 15+
- Oxen CLI installed (`pip install oxen-ai` or `cargo install oxen`)
- Test Oxen Hub account (https://hub.oxen.ai)
- Network connectivity to hub.oxen.ai
- Test repository created on Oxen Hub

---

## Test Categories

### 1. Authentication Integration Tests (Day 1)
### 2. Remote Lock Integration Tests (Days 2-3)
### 3. Collaboration Features Integration Tests (Day 4)
### 4. End-to-End Workflow Tests (Day 5)

---

## 1. Authentication Integration Tests

**Duration:** Day 1 (4-6 hours)
**Prerequisites:** Oxen Hub test account, API key

### Test 1.1: Login Flow (Happy Path)

**Objective:** Verify complete login workflow with real Oxen Hub

**Steps:**
1. Run `oxenvcs-cli auth login`
2. Enter test username when prompted
3. Enter valid API key when prompted
4. Verify success message displayed
5. Check `~/.oxen/user_config.toml` contains credentials
6. Check `~/.oxenvcs/credentials` contains fallback data
7. Verify file permissions are 0600

**Expected Result:**
- Login succeeds
- Credentials stored in both locations
- No errors or warnings

**Failure Modes to Test:**
- Invalid username
- Invalid API key
- Network timeout during auth
- Corrupted config file

---

### Test 1.2: Authentication Status Check

**Objective:** Verify status command shows correct authentication state

**Steps:**
1. Run `oxenvcs-cli auth status` (when authenticated)
2. Verify output shows "● Authenticated"
3. Verify username displayed correctly
4. Verify hub URL displayed
5. Run `oxenvcs-cli auth logout`
6. Run `oxenvcs-cli auth status` (when not authenticated)
7. Verify output shows "○ Not Authenticated"

**Expected Result:**
- Status accurately reflects authentication state
- All metadata displayed correctly

---

### Test 1.3: Connection Testing

**Objective:** Verify connection test validates credentials

**Steps:**
1. Login with valid credentials
2. Run `oxenvcs-cli auth test`
3. Verify success message
4. Manually corrupt API key in config
5. Run `oxenvcs-cli auth test`
6. Verify failure with clear error message

**Expected Result:**
- Valid credentials → test succeeds
- Invalid credentials → test fails with actionable error

---

### Test 1.4: Logout Flow

**Objective:** Verify logout clears all credentials

**Steps:**
1. Login with valid credentials
2. Verify credentials exist in both config files
3. Run `oxenvcs-cli auth logout`
4. Verify success message
5. Check `~/.oxen/user_config.toml` - credentials removed
6. Check `~/.oxenvcs/credentials` - file removed or empty
7. Run `oxenvcs-cli auth status` - should show not authenticated

**Expected Result:**
- Logout clears all stored credentials
- No sensitive data remains

---

### Test 1.5: Credential Persistence

**Objective:** Verify credentials persist across CLI invocations

**Steps:**
1. Login with valid credentials
2. Exit terminal/restart shell
3. Run `oxenvcs-cli auth status`
4. Verify still authenticated
5. Run any command requiring auth (e.g., lock operations)
6. Verify command succeeds without re-login

**Expected Result:**
- Credentials persist after shell restart
- No need to re-authenticate

---

## 2. Remote Lock Integration Tests

**Duration:** Days 2-3 (12-16 hours)
**Prerequisites:** Authenticated, test repository with remote configured

### Test 2.1: Lock Acquisition (Happy Path)

**Objective:** Verify single-user lock acquisition succeeds

**Setup:**
```bash
cd /path/to/test-project.logicx
oxen remote add origin https://hub.oxen.ai/testuser/test-project
oxenvcs-cli init --logic .
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial commit" --bpm 120
oxen push origin main
```

**Steps:**
1. Run `oxenvcs-cli lock status` - should show "No lock"
2. Run `oxenvcs-cli lock acquire --timeout 4`
3. Verify success message with lock ID
4. Verify expiration shown (4 hours from now)
5. Run `oxenvcs-cli lock status`
6. Verify status shows "● Locked" with your user
7. Verify lock stored remotely (check locks branch)

**Expected Result:**
- Lock acquired successfully
- Lock visible in status
- Lock stored in remote `locks` branch

**Verification:**
```bash
# Manually verify lock in remote
oxen fetch origin locks
oxen checkout locks
cat .oxen/locks/*.json  # Should show your lock
```

---

### Test 2.2: Lock Release

**Objective:** Verify lock release clears remote lock

**Steps:**
1. Acquire lock (see Test 2.1)
2. Run `oxenvcs-cli lock release`
3. Verify success message
4. Run `oxenvcs-cli lock status`
5. Verify status shows "No lock"
6. Check remote locks branch
7. Verify lock file removed

**Expected Result:**
- Lock released successfully
- Remote lock file deleted
- Status shows unlocked

---

### Test 2.3: Lock Collision (Two Users)

**Objective:** Verify second user cannot acquire lock while first holds it

**Setup:** Requires two machines or two separate repos

**Machine A (User A):**
1. Acquire lock: `oxenvcs-cli lock acquire --timeout 4`
2. Verify success

**Machine B (User B):**
1. Pull latest: `oxen pull origin main`
2. Attempt to acquire lock: `oxenvcs-cli lock acquire --timeout 4`
3. **Expected:** Failure with message "Project locked by userA@machineA until [timestamp]"
4. Run `oxenvcs-cli lock status`
5. Verify status shows lock held by User A

**Machine A:**
1. Release lock: `oxenvcs-cli lock release`
2. Push: `oxen push origin locks`

**Machine B:**
1. Run `oxenvcs-cli lock acquire --timeout 4`
2. **Expected:** Success (lock is now available)

**Expected Result:**
- Second user blocked while first holds lock
- Second user can acquire after first releases
- Clear error messages about lock holder

---

### Test 2.4: Race Condition Handling

**Objective:** Verify system detects race conditions when two users acquire simultaneously

**Setup:** Requires precise timing or script coordination

**Machine A & B (simultaneously):**
1. Both users run `oxenvcs-cli lock acquire --timeout 4` at same time
2. **Expected Behavior:**
   - One user succeeds (first to push)
   - Other user fails after verification (detects race)
   - No silent failures

**Steps to Test:**
```bash
# On both machines, run this script simultaneously:
#!/bin/bash
echo "Acquiring lock in 3..."
sleep 1
echo "2..."
sleep 1
echo "1..."
sleep 1
oxenvcs-cli lock acquire --timeout 4
```

**Expected Result:**
- Exactly one user gets the lock
- Other user receives clear error
- No data corruption in locks branch

---

### Test 2.5: Lock Expiration

**Objective:** Verify locks expire after timeout

**Steps:**
1. Acquire lock with short timeout: `oxenvcs-cli lock acquire --timeout 1` (1 hour)
2. Wait 61 minutes (or manually adjust system clock)
3. Run `oxenvcs-cli lock status`
4. Verify status shows "○ Expired"
5. From second machine, acquire lock
6. **Expected:** Success (expired lock can be overwritten)

**Alternative (Fast Test):**
Manually edit lock file to set `expires_at` to past:
```bash
oxen checkout locks
# Edit .oxen/locks/*.json - set expires_at to 1 hour ago
oxen commit -m "Expire lock for testing"
oxen push origin locks
# Now try to acquire from another user
```

**Expected Result:**
- Expired locks can be overwritten
- Status correctly shows "Expired"
- No blocking on expired locks

---

### Test 2.6: Lock Staleness Detection

**Objective:** Verify stale locks (no heartbeat >1hr) are detected

**Steps:**
1. Acquire lock: `oxenvcs-cli lock acquire --timeout 4`
2. Manually edit lock file to set `last_heartbeat` to 2 hours ago
3. Commit and push
4. Run `oxenvcs-cli lock status`
5. Verify status shows "◐ Stale" warning
6. Verify message: "No heartbeat for >1 hour (may be abandoned)"

**Expected Result:**
- Stale locks detected
- Warning displayed to user
- Lock can still be force-broken

---

### Test 2.7: Lock Renewal/Heartbeat

**Objective:** Verify lock renewal extends expiration

**Steps:**
1. Acquire lock: `oxenvcs-cli lock acquire --timeout 2`
2. Note expiration time (2 hours from now)
3. Wait 1 hour (or adjust clock)
4. Renew lock: `oxenvcs-cli lock renew --additional 2`
5. Run `oxenvcs-cli lock status`
6. Verify new expiration is 2 hours from current time
7. Verify `last_heartbeat` updated

**Expected Result:**
- Lock expiration extended
- Heartbeat timestamp updated
- Changes pushed to remote

---

### Test 2.8: Force Break Lock

**Objective:** Verify force break works and shows warnings

**Setup:**
- Lock held by User A
- User B needs to break it

**Steps (User B):**
1. Run `oxenvcs-cli lock break` (without --force)
2. **Expected:** Error "Must use --force flag"
3. Run `oxenvcs-cli lock break --force`
4. Verify warning message about data loss
5. Confirm break (if prompted)
6. Verify success message
7. Run `oxenvcs-cli lock status`
8. Verify lock removed

**Expected Result:**
- Requires --force flag (safety)
- Shows clear warnings
- Breaks lock successfully
- User A's lock is invalidated

---

### Test 2.9: Network Failure During Lock Ops

**Objective:** Verify behavior when network fails mid-operation

**Test 2.9a: Network failure during acquire**
1. Start lock acquisition
2. Disconnect network after fetch but before push
3. **Expected:** Error "Failed to push lock to remote"
4. Verify no local lock created
5. Verify remote unchanged

**Test 2.9b: Network failure during release**
1. Acquire lock successfully
2. Start lock release
3. Disconnect network during push
4. **Expected:** Error "Failed to push lock deletion"
5. Verify local lock file removed
6. Verify remote still has lock (inconsistent state)
7. **Document:** This is a known issue (Phase 4 will fix)

**Expected Result:**
- Clear error messages
- No silent failures
- Document inconsistent state issues

---

## 3. Collaboration Features Integration Tests

**Duration:** Day 4 (6-8 hours)
**Prerequisites:** Repository with commit history

### Test 3.1: Activity Feed

**Objective:** Verify activity feed parses real commit history

**Setup:**
```bash
# Create commits with metadata
oxenvcs-cli commit -m "First track" --bpm 120 --key "C Major"
oxenvcs-cli commit -m "Added drums" --bpm 128
oxenvcs-cli commit -m "Mixed" --bpm 128 --key "C Major" --tags "mixing,final"
oxen push origin main
```

**Steps:**
1. Run `oxenvcs-cli activity --limit 10`
2. Verify all commits appear in timeline
3. Verify metadata displayed (BPM, key, tags)
4. Verify icons shown (● for commits)
5. Verify authors shown correctly
6. Verify timestamps in chronological order

**Expected Result:**
- All commits appear in activity feed
- Metadata parsed correctly
- Beautiful formatted output

---

### Test 3.2: Team Discovery

**Objective:** Verify team members discovered from commit history

**Setup:** Repository with commits from multiple users

**Steps:**
1. Run `oxenvcs-cli team`
2. Verify all contributors listed
3. Verify commit counts accurate
4. Verify contribution percentages sum to 100%
5. Verify activity bars shown
6. Verify "Last active" timestamps

**Expected Result:**
- All team members discovered
- Statistics accurate
- Sorted by most active

---

### Test 3.3: Comment System

**Objective:** Verify comments can be added and synced

**Steps:**
1. Get commit hash: `oxenvcs-cli log --limit 1`
2. Add comment: `oxenvcs-cli comment add abc123 "Great mix!"`
3. Verify success message
4. List comments: `oxenvcs-cli comment list abc123`
5. Verify comment appears with author and timestamp
6. Check local file: `cat .oxen/comments/abc123.json`
7. Commit and push comments:
   ```bash
   oxen add .oxen/comments/
   oxen commit -m "Add review comments"
   oxen push origin main
   ```
8. From second machine, pull and list comments
9. Verify comments synced

**Expected Result:**
- Comments stored locally
- Comments sync via manual push
- Multiple comments per commit supported

---

### Test 3.4: Activity Feed with Lock Events

**Objective:** Verify lock events appear in activity feed

**Steps:**
1. Acquire lock: `oxenvcs-cli lock acquire --timeout 4`
2. Run `oxenvcs-cli activity --limit 10`
3. **Expected:** Lock acquisition event shown with 🔒 icon
4. Release lock: `oxenvcs-cli lock release`
5. Run `oxenvcs-cli activity --limit 10`
6. **Expected:** Lock release event shown with 🔓 icon

**Expected Result:**
- Lock events integrated into activity timeline
- Clear icons distinguish event types

---

## 4. End-to-End Workflow Tests

**Duration:** Day 5 (6-8 hours)
**Prerequisites:** All previous tests passing

### Test 4.1: Complete Collaboration Workflow (Two Users)

**Objective:** Simulate real-world team collaboration

**Scenario:** Producer (User A) and Mixer (User B) collaborate on project

**Morning - User A (Producer):**
```bash
# Start of day
cd MyProject.logicx
oxen pull origin main                    # Get latest
oxenvcs-cli lock acquire --timeout 8      # Lock for day
# ... work in Logic Pro (simulated) ...
# Modify some files
touch "Audio Files/vocals.wav"
oxenvcs-cli add --all
oxenvcs-cli commit -m "Recorded vocals" --bpm 120 --tags "recording"
oxen push origin main
oxenvcs-cli lock release                  # Done for now
```

**Afternoon - User B (Mixer):**
```bash
# Check what happened
oxenvcs-cli activity --limit 10           # See A's work
oxenvcs-cli team                          # Check team stats
oxen pull origin main                     # Get A's vocals
oxenvcs-cli lock acquire --timeout 4      # Lock for mixing
# ... mixing work (simulated) ...
touch "Audio Files/vocals_processed.wav"
oxenvcs-cli add --all
oxenvcs-cli commit -m "Mixed vocals, added reverb" --bpm 120 --tags "mixing"
oxenvcs-cli comment add <prev_commit> "Great vocal take!"  # Comment on A's commit
oxen add .oxen/comments/
oxen commit -m "Add review comment"
oxen push origin main
oxenvcs-cli lock release
```

**Next Day - User A:**
```bash
# Check updates
oxen pull origin main
oxenvcs-cli activity --limit 10           # See B's work
oxenvcs-cli comment list <commit>         # See B's comment
# Continue work...
```

**Expected Result:**
- Complete workflow succeeds
- Lock prevents conflicts
- Activity feed shows all work
- Comments sync properly
- No data loss or corruption

---

### Test 4.2: Large Project Workflow

**Objective:** Test with realistic Logic Pro project size

**Setup:** Create project with ~5 GB of audio files

**Steps:**
1. Initialize OxVCS: `oxenvcs-cli init --logic .`
2. Add all: `oxenvcs-cli add --all`
3. Commit: `oxenvcs-cli commit -m "Initial project" --bpm 120`
4. Push to hub: `oxen push origin main` (note duration)
5. From second machine, clone: `oxen clone <url> Project.logicx` (note duration)
6. Acquire lock, make changes, commit, push
7. From first machine, pull changes

**Metrics to Record:**
- Initial push time: ___ minutes
- Clone time: ___ minutes
- Lock acquisition time: ___ seconds
- Pull time after changes: ___ minutes

**Expected Result:**
- All operations complete successfully
- Times are reasonable (<10 min for push/clone)
- No timeouts or failures

---

### Test 4.3: Recovery from Network Interruption

**Objective:** Document current behavior when network fails

**Test 4.3a: During Push**
1. Start large push: `oxen push origin main`
2. Disconnect network mid-push
3. **Document:** What happens? Error message? Partial upload?
4. Reconnect network
5. Retry push: `oxen push origin main`
6. **Document:** Does it resume or restart?

**Test 4.3b: During Lock Acquire**
1. Start lock acquisition
2. Disconnect network during operation
3. **Document:** Error message? State of lock?

**Expected Result:**
- Document all failure modes
- Identify gaps for Phase 4 work

---

## Test Execution Checklist

### Prerequisites Setup

- [ ] macOS 14.0+ environment ready
- [ ] Oxen CLI installed and working: `oxen --version`
- [ ] OxVCS CLI built: `cd OxVCS-CLI-Wrapper && cargo build --release`
- [ ] Test Oxen Hub account created
- [ ] API key generated from hub.oxen.ai
- [ ] Test repository created on hub
- [ ] Network connectivity verified

### Day 1: Authentication (4-6 hours)

- [ ] Test 1.1: Login flow (happy path)
- [ ] Test 1.2: Authentication status check
- [ ] Test 1.3: Connection testing
- [ ] Test 1.4: Logout flow
- [ ] Test 1.5: Credential persistence
- [ ] Document any bugs found
- [ ] Create bug report issues if needed

### Day 2: Lock Basics (6-8 hours)

- [ ] Test 2.1: Lock acquisition (happy path)
- [ ] Test 2.2: Lock release
- [ ] Test 2.3: Lock collision (two users)
- [ ] Test 2.4: Race condition handling
- [ ] Document any bugs found

### Day 3: Lock Advanced (6-8 hours)

- [ ] Test 2.5: Lock expiration
- [ ] Test 2.6: Lock staleness detection
- [ ] Test 2.7: Lock renewal/heartbeat
- [ ] Test 2.8: Force break lock
- [ ] Test 2.9: Network failure scenarios
- [ ] Document all failure modes

### Day 4: Collaboration (6-8 hours)

- [ ] Test 3.1: Activity feed
- [ ] Test 3.2: Team discovery
- [ ] Test 3.3: Comment system
- [ ] Test 3.4: Activity feed with lock events
- [ ] Document any bugs found

### Day 5: End-to-End (6-8 hours)

- [ ] Test 4.1: Complete collaboration workflow (2 users)
- [ ] Test 4.2: Large project workflow (5+ GB)
- [ ] Test 4.3: Recovery from network interruption
- [ ] Document performance metrics
- [ ] Create summary report

---

## Test Results Template

For each test, record:

```markdown
### Test [Number]: [Name]

**Date:** YYYY-MM-DD
**Tester:** [Name]
**Environment:** macOS [version], Oxen [version]

**Result:** ✅ PASS / ❌ FAIL / ⚠️ PARTIAL

**Execution Time:** [X] minutes

**Observations:**
- [What worked well]
- [What didn't work]
- [Unexpected behavior]

**Bugs Found:**
- [Bug #1: Description]
- [Bug #2: Description]

**Performance Metrics:**
- [Metric 1]: [Value]
- [Metric 2]: [Value]

**Screenshots/Logs:**
[Attach relevant output]
```

---

## Known Issues to Watch For

Based on code review, these areas are most likely to have issues:

### High Priority

1. **Race Condition Detection**
   - Location: `remote_lock.rs:241-242`
   - Issue: Fixed 2s sleep may not be sufficient
   - Test: Test 2.4

2. **Network Failure Handling**
   - Location: Throughout remote_lock.rs
   - Issue: No retry logic
   - Test: Test 2.9

3. **Lock Heartbeat**
   - Location: `remote_lock.rs:287-334`
   - Issue: No automatic heartbeat daemon
   - Test: Test 2.7

### Medium Priority

4. **Comment Sync**
   - Location: `collaboration.rs:318-440`
   - Issue: Manual commit+push required
   - Test: Test 3.3

5. **Large File Handling**
   - Location: Oxen subprocess calls
   - Issue: Timeouts not tuned for large projects
   - Test: Test 4.2

---

## Success Criteria

**Week 1 integration testing is successful if:**

1. ✅ **80%+ tests pass** with real Oxen Hub
2. ✅ **All critical bugs documented** with reproduction steps
3. ✅ **Performance metrics recorded** for large projects
4. ✅ **Failure modes documented** for Phase 4 planning
5. ✅ **No data loss** in any scenario
6. ✅ **Clear error messages** for all failure cases

**Blockers for v0.1:**
- Authentication must work 100%
- Lock acquisition/release must work reliably
- No data corruption under any scenario

**Acceptable for v0.1 (fix in v0.2):**
- Network failures require manual intervention
- Race conditions occur occasionally
- No automatic lock heartbeat

---

## Phase 4 Requirements (Informed by Testing)

After Week 1 testing, create Phase 4 backlog:

### P0 (Critical)
- [ ] Network retry logic (Test 2.9, 4.3)
- [ ] Better race condition handling (Test 2.4)
- [ ] Graceful degradation on network loss

### P1 (Important)
- [ ] Automatic lock heartbeat daemon (Test 2.7)
- [ ] Stale lock cleanup (Test 2.6)
- [ ] Partial push recovery (Test 4.3)

### P2 (Nice to Have)
- [ ] Automatic comment sync (Test 3.3)
- [ ] Performance optimization for large projects (Test 4.2)
- [ ] Offline mode with commit queue

---

## Next Steps After Testing

1. **Triage bugs** - Categorize by severity
2. **Fix critical bugs** - Blockers for v0.1
3. **Update documentation** - Based on real-world behavior
4. **Plan Phase 4** - Prioritize based on test findings
5. **Begin Phase 4 implementation** - Week 2-3

---

*Created: 2025-11-17*
*Target Start: When macOS environment available*
*Expected Duration: 5 days (30-40 hours)*
