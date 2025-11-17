# Testing Quick Start Guide

**For macOS Validation** | Last Updated: November 17, 2025

This is a quick-reference checklist for running integration tests on macOS. For detailed procedures, see [INTEGRATION_TEST_PLAN.md](INTEGRATION_TEST_PLAN.md).

---

## Prerequisites (15 minutes)

### 1. Environment Setup

```bash
# Verify macOS version
sw_vers  # Should be 14.0+

# Install Oxen CLI
pip3 install oxen-ai
# OR
cargo install oxen

# Verify installation
oxen --version  # Should show version number
```

### 2. Build Auxin CLI

```bash
cd /path/to/auxin/Auxin-CLI-Wrapper
cargo build --release

# Test CLI works
./target/release/auxin --help
```

### 3. Create Oxen Hub Test Account

1. Visit https://hub.oxen.ai
2. Sign up for free account
3. Go to Settings → API Keys
4. Generate new API key
5. Copy key (you'll need it soon)

### 4. Create Test Repository

1. On hub.oxen.ai, click "New Repository"
2. Name: `auxin-test-project`
3. Visibility: Private (recommended)
4. Click Create

### 5. Set Environment Variables

```bash
# Add to ~/.zshrc or ~/.bash_profile
export RUN_INTEGRATION_TESTS=1
export OXEN_TEST_USERNAME='your-username'
export OXEN_TEST_API_KEY='your-api-key'
export OXEN_TEST_REPO_URL='https://hub.oxen.ai/your-username/auxin-test-project'

# Reload shell
source ~/.zshrc  # or source ~/.bash_profile
```

---

## Day 1: Authentication Tests (2-3 hours)

### Quick Test

```bash
cd Auxin-CLI-Wrapper

# Test 1: Login
./target/release/auxin auth login
# Enter your username and API key when prompted
# Expected: Success message

# Test 2: Status
./target/release/auxin auth status
# Expected: Shows "● Authenticated" with your username

# Test 3: Connection test
./target/release/auxin auth test
# Expected: "Connection successful" or similar

# Test 4: Logout
./target/release/auxin auth logout
# Expected: Success message

# Test 5: Verify logged out
./target/release/auxin auth status
# Expected: "○ Not Authenticated"

# Re-login for next tests
./target/release/auxin auth login
```

### Run Automated Tests

```bash
# Run auth integration tests
cargo test test_auth_login_flow -- --ignored --nocapture
cargo test test_auth_logout_flow -- --ignored --nocapture
```

**✅ Pass Criteria:** All 5 manual tests work, no errors

---

## Day 2: Lock Basics (3-4 hours)

### Setup Test Project

```bash
# Create test Logic Pro project
mkdir -p ~/Desktop/TestProject.logicx/Audio\ Files
mkdir -p ~/Desktop/TestProject.logicx/Alternatives
echo "test project data" > ~/Desktop/TestProject.logicx/projectData

cd ~/Desktop/TestProject.logicx

# Initialize Auxin
auxin init --logic .

# Configure remote
oxen remote add origin $OXEN_TEST_REPO_URL

# Create initial commit
auxin add --all
auxin commit -m "Initial test project" --bpm 120
oxen push origin main
```

### Test Lock Operations

```bash
cd ~/Desktop/TestProject.logicx

# Test 1: Check status (should be unlocked)
auxin lock status
# Expected: "No lock" or "unlocked"

# Test 2: Acquire lock
auxin lock acquire --timeout 4
# Expected: Success with lock ID

# Test 3: Check status (should show locked)
auxin lock status
# Expected: "● Locked" with your username

# Test 4: Try to acquire again (should fail)
auxin lock acquire --timeout 4
# Expected: Error "already locked"

# Test 5: Release lock
auxin lock release
# Expected: Success message

# Test 6: Verify released
auxin lock status
# Expected: "No lock"
```

### Run Automated Tests

```bash
cd Auxin-CLI-Wrapper
cargo test test_lock_acquire_release -- --ignored --nocapture
cargo test test_lock_force_break -- --ignored --nocapture
```

**✅ Pass Criteria:** Can acquire, check status, and release locks successfully

---

## Day 3: Lock Advanced (3-4 hours)

### Test Two-User Collision (Requires 2 machines or 2 repos)

**Machine A:**
```bash
cd ~/Desktop/TestProject.logicx
auxin lock acquire --timeout 4
# Note the lock ID and expiration time
```

**Machine B (or separate terminal with different user):**
```bash
cd /path/to/cloned/TestProject.logicx
oxen pull origin locks  # Get latest lock state
auxin lock acquire --timeout 4
# Expected: ERROR "Project locked by [user A]"

# Check lock status
auxin lock status
# Expected: Shows lock held by user A
```

**Machine A:**
```bash
auxin lock release
oxen push origin locks
```

**Machine B:**
```bash
oxen pull origin locks
auxin lock acquire --timeout 4
# Expected: SUCCESS (lock now available)
```

### Test Lock Renewal

```bash
# Acquire lock
auxin lock acquire --timeout 2
# Note expiration time

# Wait 1 hour (or manually edit lock file to simulate time passing)

# Renew lock
auxin lock renew --additional 2
# Expected: New expiration 2 hours from now

# Verify
auxin lock status
# Expected: Updated expiration time
```

### Test Force Break

```bash
# Acquire lock
auxin lock acquire --timeout 4

# Try break without --force
auxin lock break
# Expected: Error requiring --force

# Force break
auxin lock break --force
# Expected: Success with warning

# Verify
auxin lock status
# Expected: No lock
```

**✅ Pass Criteria:** Lock collision detected, renewal works, force break works

---

## Day 4: Collaboration Features (2-3 hours)

### Test Activity Feed

```bash
cd ~/Desktop/TestProject.logicx

# Create some commits with metadata
echo "track 1" > "Audio Files/track1.wav"
auxin add --all
auxin commit -m "First track" --bpm 120 --key "C Major"

echo "track 2" > "Audio Files/track2.wav"
auxin add --all
auxin commit -m "Added drums" --bpm 128 --tags "drums,tracking"

echo "track 3" > "Audio Files/track3.wav"
auxin add --all
auxin commit -m "Final mix" --bpm 128 --key "C Major" --tags "mixing,final"

# View activity feed
auxin activity --limit 10
# Expected: Shows all 3 commits with metadata (BPM, key, tags)
```

### Test Team Discovery

```bash
# View team members
auxin team
# Expected: Shows contributors with commit counts and percentages
```

### Test Comments

```bash
# Get latest commit hash
COMMIT=$(auxin log --limit 1 | grep -o '[a-f0-9]\{7,\}' | head -1)

# Add comment
auxin comment add $COMMIT "Great mix on this one!"
# Expected: Success message

# List comments
auxin comment list $COMMIT
# Expected: Shows the comment with author and timestamp

# Verify comment file created
ls .oxen/comments/
# Expected: File named ${COMMIT}.json

# To share comment with team (manual sync)
oxen add .oxen/comments/
oxen commit -m "Add review comment"
oxen push origin main
```

### Run Automated Tests

```bash
cd Auxin-CLI-Wrapper
cargo test test_activity_feed -- --ignored --nocapture
cargo test test_team_discovery -- --ignored --nocapture
cargo test test_comment_system -- --ignored --nocapture
```

**✅ Pass Criteria:** Activity feed shows commits, team stats accurate, comments work

---

## Day 5: End-to-End Workflow (3-4 hours)

### Complete Team Workflow

**Scenario:** Producer and Mixer collaborate

#### Producer (Morning Session)

```bash
cd ~/Desktop/MusicProject.logicx

# Start of day - get latest
oxen pull origin main

# Acquire lock
auxin lock acquire --timeout 8

# Simulate work (copy real audio or create dummy files)
echo "vocal recording" > "Audio Files/vocals.wav"

# Commit work
auxin add --all
auxin commit -m "Recorded vocals for chorus" --bpm 120 --tags "recording,vocals"

# Push to share
oxen push origin main

# Release lock
auxin lock release

# Verify lock released
auxin lock status
```

#### Mixer (Afternoon Session)

```bash
cd ~/Desktop/MusicProject.logicx

# Check what happened
auxin activity --limit 10
# Should see Producer's commit

# See team contributions
auxin team

# Get latest work
oxen pull origin main

# Check files (vocals.wav should be present)
ls "Audio Files/"

# Acquire lock
auxin lock acquire --timeout 4

# Simulate mixing work
echo "processed vocals" > "Audio Files/vocals_processed.wav"

# Commit work
auxin add --all
auxin commit -m "Mixed vocals, added reverb" --bpm 120 --tags "mixing"

# Add feedback on Producer's commit
PREV_COMMIT=$(auxin log --limit 2 | grep -o '[a-f0-9]\{7,\}' | tail -1)
auxin comment add $PREV_COMMIT "Great vocal take, worked perfectly!"

# Push everything
oxen add .oxen/comments/
oxen commit -m "Add review feedback"
oxen push origin main

# Release lock
auxin lock release
```

#### Producer (Next Day)

```bash
# Get updates
oxen pull origin main

# See Mixer's activity
auxin activity --limit 10

# Check comments on my commit
COMMIT=$(auxin log --limit 3 | grep -o '[a-f0-9]\{7,\}' | tail -1)
auxin comment list $COMMIT
```

**✅ Pass Criteria:** Complete workflow succeeds, no conflicts, changes sync properly

---

## Performance Testing (Optional - Day 6)

### Large Project Test

```bash
# Create project with ~5GB of audio
mkdir -p ~/Desktop/LargeProject.logicx/Audio\ Files
cd ~/Desktop/LargeProject.logicx

# Generate large audio files (5GB total)
for i in {1..5}; do
    dd if=/dev/zero of="Audio Files/track_${i}.wav" bs=1m count=1024
done

# Initialize and push
auxin init --logic .
oxen remote add origin $OXEN_TEST_REPO_URL
auxin add --all

# Time the commit
time auxin commit -m "Large project initial commit" --bpm 120

# Time the push
time oxen push origin main
# RECORD: Push time = ___

# From another machine, time the clone
time oxen clone $OXEN_TEST_REPO_URL LargeProject.logicx
# RECORD: Clone time = ___
```

**Expected Times:**
- Push: 5-10 minutes (first time)
- Clone: 5-10 minutes
- Subsequent pushes: <2 minutes (due to deduplication)

---

## Troubleshooting

### Auth Fails

```bash
# Check Oxen config
cat ~/.oxen/user_config.toml

# Verify API key
oxen config user.api_key

# Try Oxen CLI directly
oxen login

# Check Auxin credentials
cat ~/.auxin/credentials
```

### Lock Operations Fail

```bash
# Check remote configured
oxen remote -v

# Fetch locks branch manually
oxen fetch origin locks

# Check locks branch
oxen branch -a
# Should see remotes/origin/locks

# Inspect lock files
oxen checkout locks
cat .oxen/locks/*.json
```

### Push/Pull Fails

```bash
# Check authentication
auxin auth status

# Check network connectivity
ping hub.oxen.ai

# Try with verbose output
oxen push origin main --verbose
```

### Tests Hang

```bash
# Kill hung processes
pkill -f auxin
pkill -f oxen

# Clean up test directories
rm -rf ~/Desktop/*TestProject*
```

---

## Test Results Template

Copy this for each day's testing:

```markdown
## Day [N]: [Test Name]

**Date:** YYYY-MM-DD
**Tester:** [Your Name]
**Duration:** [X] hours

### Results Summary
- Total Tests: [N]
- Passed: [N] ✅
- Failed: [N] ❌
- Partial: [N] ⚠️

### Detailed Results

#### Test 1: [Name]
- **Result:** ✅ PASS / ❌ FAIL / ⚠️ PARTIAL
- **Time:** [X] seconds
- **Notes:** [Observations]

#### Test 2: [Name]
- **Result:** ✅ PASS / ❌ FAIL / ⚠️ PARTIAL
- **Time:** [X] seconds
- **Notes:** [Observations]

### Bugs Found
1. **[Bug Title]**
   - Severity: Critical / High / Medium / Low
   - Reproduction: [Steps]
   - Expected: [What should happen]
   - Actual: [What happened]

### Performance Metrics
- Lock acquire: [X]ms
- Lock release: [X]ms
- Push (5GB): [X]min
- Clone (5GB): [X]min

### Recommendations
- [Recommendation 1]
- [Recommendation 2]
```

---

## Success Criteria

**Week 1 Complete When:**
- ✅ All authentication tests pass
- ✅ Lock acquire/release works reliably
- ✅ Two-user collision properly detected
- ✅ Activity feed shows correct data
- ✅ Team discovery accurate
- ✅ Comments work and sync
- ✅ Complete workflow succeeds
- ✅ No data loss in any scenario
- ✅ All bugs documented with reproduction steps

**Known Acceptable Issues for v0.1:**
- Network failures require manual retry
- No automatic lock heartbeat
- Race conditions may occur rarely
- Comments require manual push to sync

---

## Next Steps After Testing

1. **Create Bug Reports** - File issues on GitHub
2. **Update Documentation** - Fix any inaccuracies discovered
3. **Begin Phase 4** - Implement network resilience (Week 2-3)

---

**Need Help?**
- Full test plan: [INTEGRATION_TEST_PLAN.md](INTEGRATION_TEST_PLAN.md)
- Troubleshooting: [TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)
- Cloud guide: [CLOUD_SHARING_GUIDE.md](docs/CLOUD_SHARING_GUIDE.md)

*Last Updated: 2025-11-17*
