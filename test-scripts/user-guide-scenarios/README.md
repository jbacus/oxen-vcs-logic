# User Guide Test Scripts

Automated test scripts for all scenarios described in the [USER_GUIDE.md](../../docs/USER_GUIDE.md).

## Overview

These scripts test the complete Oxen-VCS workflow for Logic Pro projects, covering:

1. **Project initialization** - Setting up version control
2. **Milestone commits** - Creating commits with metadata
3. **Rollback workflow** - Restoring previous versions
4. **History browsing** - Viewing and searching commits
5. **Auto-commits** - FSEvents monitoring and draft commits
6. **Lock management** - Multi-user collaboration
7. **Remote sync** - Push/pull with Oxen remote

## Prerequisites

### Required
- **macOS 14.0+** with terminal access
- **Oxen CLI** installed: `pip3 install oxen-ai` or `cargo install oxen`
- **Auxin-CLI** built: `cd Auxin-CLI-Wrapper && cargo build --release`

### Optional (for full testing)
- **LaunchAgent** running (for auto-commit tests)
- **Oxen-VCS.app** installed (for lock UI tests)
- **Logic Pro 11.x** (for real-world validation)

### Verify Setup

```bash
# Check Oxen CLI
oxen --version

# Check Auxin-CLI
./Auxin-CLI-Wrapper/target/release/auxin --version

# Check LaunchAgent (optional)
launchctl list | grep oxenvcs
```

## Quick Start

### Run All Tests

```bash
cd test-scripts/user-guide-scenarios

# Make scripts executable
chmod +x *.sh

# Run all tests (interactive)
./run_all_tests.sh
```

### Run Individual Tests

Each script can be run independently:

```bash
# Test 1: Initialize project
./01_initialize_project.sh

# Test 2: Milestone commits
./02_milestone_commit.sh

# Test 3: Rollback workflow
./03_rollback_workflow.sh

# Test 4: Browse history
./04_browse_history.sh

# Test 5: Auto-commits (requires LaunchAgent)
./05_auto_commit_fsevents.sh

# Test 6: Lock management
./06_lock_workflow.sh

# Test 7: Remote synchronization
./07_remote_sync.sh
```

## Test Scripts

### 01_initialize_project.sh
**Duration**: 30-60 seconds
**User Guide Section**: "Initializing Your First Project"

Tests:
- Create Logic Pro project structure
- Initialize with `auxin init`
- Verify `.oxen` directory created
- Verify `.oxenignore` configured correctly
- Verify initial commit created
- Verify tracked/ignored files working

**Expected Output**: Project initialized with all structures in place

---

### 02_milestone_commit.sh
**Duration**: 1-2 minutes
**User Guide Section**: "Creating Milestone Commits"

Tests:
- Create commits with structured metadata
- Include BPM, sample rate, key signature
- Add tags for organization
- Verify metadata embedded in commit message
- Test commit message best practices
- Search history by metadata

**Expected Output**: Multiple milestone commits with rich metadata

---

### 03_rollback_workflow.sh
**Duration**: 1-2 minutes
**User Guide Section**: "Rolling Back to Previous Versions"

Tests:
- Create project with 4 versions
- Rollback to previous commit
- Verify correct state restoration
- Verify files added/removed correctly
- Test non-destructive rollback (can return to latest)
- Test rollback with uncommitted changes (safety)
- Validate use case: undo bad mix decision

**Expected Output**: Successful rollback and restoration to any version

---

### 04_browse_history.sh
**Duration**: 1-2 minutes
**User Guide Section**: "Browsing Project History"

Tests:
- Create rich commit history (9 commits)
- View complete history
- Search by commit message
- Filter by metadata (tags, BPM, key)
- View specific commit details
- Compare commits
- Timeline view
- Export history report

**Expected Output**: Complete history browsing capabilities verified

---

### 05_auto_commit_fsevents.sh
**Duration**: 2-3 minutes (includes debounce waits)
**User Guide Section**: "Automatic Versioning (Draft Commits)"

**Prerequisites**: LaunchAgent must be running

Tests:
- Check daemon running
- Register project for monitoring
- Make change and wait for auto-commit
- Verify debounce (multiple changes → 1 commit)
- Verify no commits when no changes
- Verify ignored files don't trigger commits
- Check draft branch creation

**Expected Output**: Auto-commits working with proper debounce

---

### 06_lock_workflow.sh
**Duration**: 1-2 minutes
**User Guide Section**: "Collaboration - Acquiring/Releasing Locks"

Tests:
- Simulate multi-user scenario
- User 1 acquires lock
- User 2 blocked from lock (shown dialog)
- Verify read-only access for User 2
- User 1 releases lock
- User 2 acquires lock
- Test lock timeout mechanism
- Test force-break lock (emergency)

**Expected Output**: Lock workflow preventing conflicts

**Note**: Full implementation requires LaunchAgent + remote sync

---

### 07_remote_sync.sh
**Duration**: 2-3 minutes
**User Guide Section**: "Remote Synchronization"

Tests:
- Create local project
- Setup remote repository (local bare repo for testing)
- Configure remote
- Push to remote
- Clone to simulate second machine
- Make changes in clone and push
- Pull changes to original
- Test large file push (10MB audio)
- Verify remote integrity
- Test fetch vs pull

**Expected Output**: Full remote sync workflow working

**Note**: Uses local directory as remote for testing

---

## Test Output

Each script provides:
- ✓ Success indicators (green)
- ✗ Error messages (red)
- ℹ Informational messages (magenta)
- Step-by-step progress (yellow)
- Final summary with statistics

### Example Output

```
========================================
TEST: Initialize Project
========================================

STEP 1: Checking prerequisites
✓ Oxen CLI is available
✓ auxin is available

STEP 2: Creating test Logic Pro project
✓ Test project created at: /Users/.../TestProject.logicx
  - ProjectData: 512B
  - Audio files: 2 files, 1.5M

STEP 3: Initializing project with auxin
✓ .oxen directory created

...

========================================
TEST PASSED: Initialize Project
========================================

Summary:
  - Project initialized: /Users/.../TestProject.logicx
  - .oxen directory: created
  - .oxenignore: configured correctly
  - Initial commit: created
  - Tracked files: 5
  - Ignored files: working correctly
```

## Cleanup

Each script asks if you want to delete test projects:

```
Delete test project? (y/n)
```

- **y**: Delete test project (recommended for clean testing)
- **n**: Keep test project (useful for manual inspection)

Test projects are created in: `~/Desktop/oxenvcs-test-projects/`

### Manual Cleanup

```bash
# Remove all test projects
rm -rf ~/Desktop/oxenvcs-test-projects

# Keep them for inspection
cd ~/Desktop/oxenvcs-test-projects
ls -lh
```

## Troubleshooting

### Script Fails: "oxen command not found"

```bash
# Install Oxen CLI
pip3 install oxen-ai

# Verify
oxen --version
```

### Script Fails: "auxin not found"

```bash
# Build CLI
cd Auxin-CLI-Wrapper
cargo build --release

# Verify
./target/release/auxin --version
```

### Auto-Commit Test Fails: "LaunchAgent not running"

```bash
# Start LaunchAgent
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist

# Verify
launchctl list | grep oxenvcs
```

### Tests Running Slow

- Disk I/O may be bottleneck (check Time Machine, Spotlight)
- Large projects take longer (expected)
- Auto-commit tests include 30-35s debounce waits (cannot speed up)

### Lock Test Limitations

Full lock implementation requires:
- LaunchAgent for file permission enforcement
- Remote lock manifest for multi-machine sync
- Oxen-VCS.app UI for lock management

This test simulates lock logic only.

## Integration with Manual Test Plan

These automated scripts correspond to scenarios in:
- [MANUAL_TEST_PLAN.md](../MANUAL_TEST_PLAN.md)
- [docs/MACOS_TEST_PLAN.md](../../docs/MACOS_TEST_PLAN.md)

Use together:
1. Run automated scripts first (fast iteration)
2. Then run manual tests with real Logic Pro projects
3. Validate with actual Logic Pro workflows

## Contributing

To add new test scenarios:

1. Follow naming convention: `##_test_name.sh`
2. Use provided functions:
   - `print_header()` - Test title
   - `print_step(n, description)` - Progress
   - `print_success(message)` - Success indicator
   - `print_error(message)` - Error (exits script)
   - `print_info(message)` - Information
3. Add cleanup trap: `trap cleanup EXIT`
4. Document in this README

## Running in CI/CD

These scripts can be integrated into GitHub Actions:

```yaml
# .github/workflows/test-user-guide.yml
name: User Guide Tests

on: [push, pull_request]

jobs:
  test-scenarios:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Oxen CLI
        run: pip3 install oxen-ai
      - name: Build Auxin-CLI
        run: cd Auxin-CLI-Wrapper && cargo build --release
      - name: Run tests
        run: cd test-scripts/user-guide-scenarios && ./run_all_tests.sh
```

## License

See [LICENSE](../../LICENSE) in root directory.

## Support

- **Issues**: https://github.com/jbacus/auxin/issues
- **Docs**: https://github.com/jbacus/auxin/docs
- **Email**: support@oxen-vcs.com

---

*Last Updated: 2025-10-30*
*Corresponding to USER_GUIDE.md v0.1-beta*
