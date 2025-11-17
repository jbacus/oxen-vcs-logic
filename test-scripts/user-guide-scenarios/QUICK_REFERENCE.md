# Quick Reference: User Guide Test Scripts

## One-Line Command Quick Start

```bash
# Run all tests
cd test-scripts/user-guide-scenarios && ./run_all_tests.sh
```

## Individual Test Commands

```bash
cd test-scripts/user-guide-scenarios

# 1. Initialize project (~30s)
./01_initialize_project.sh

# 2. Milestone commits (~1min)
./02_milestone_commit.sh

# 3. Rollback (~1min)
./03_rollback_workflow.sh

# 4. Browse history (~1min)
./04_browse_history.sh

# 5. Auto-commits (~2min, requires LaunchAgent)
./05_auto_commit_fsevents.sh

# 6. Lock workflow (~1min)
./06_lock_workflow.sh

# 7. Remote sync (~2min)
./07_remote_sync.sh
```

## Prerequisites Checklist

```bash
# ✓ Check Oxen CLI
oxen --version

# ✓ Check auxin
./Auxin-CLI-Wrapper/target/release/auxin --version

# ○ Check LaunchAgent (optional, for test 5)
launchctl list | grep oxenvcs
```

## Expected Results by Test

| Test | Duration | Pass Criteria |
|------|----------|---------------|
| **01** | 30s | .oxen dir created, initial commit exists |
| **02** | 1m | Milestone commits with BPM/key metadata |
| **03** | 1m | Rollback restores correct project state |
| **04** | 1m | Can search/filter 9 commits by metadata |
| **05** | 2m | Auto-commit created after debounce (35s) |
| **06** | 1m | Lock blocks User 2, releases correctly |
| **07** | 2m | Push/pull syncs clone correctly |

## Troubleshooting Quick Fixes

```bash
# Missing Oxen CLI
pip3 install oxen-ai

# Missing auxin
cd Auxin-CLI-Wrapper && cargo build --release

# LaunchAgent not running
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist

# Clean up test projects
rm -rf ~/Desktop/oxenvcs-test-projects
```

## Test Output Color Legend

- 🟢 **Green ✓**: Success
- 🔴 **Red ✗**: Error/Failure
- 🟡 **Yellow STEP**: Progress indicator
- 🟣 **Magenta ℹ**: Information
- 🔵 **Blue**: Test headers

## Common Test Patterns

### Pattern 1: Standard Test Flow
```
1. Setup (create test project)
2. Execute (run scenario)
3. Verify (check results)
4. Cleanup (ask to delete)
```

### Pattern 2: Multi-Version Test
```
1. Create v1 → commit
2. Create v2 → commit
3. Create v3 → commit
4. Verify history
5. Rollback to v2
6. Verify state
```

### Pattern 3: Multi-User Test
```
1. User A acquires lock
2. User B blocked
3. User A releases
4. User B acquires
5. Verify handoff
```

## Test Locations

- **Test projects**: `~/Desktop/oxenvcs-test-projects/`
- **Logs**: `test-scripts/user-guide-scenarios/test-logs/`
- **Scripts**: `test-scripts/user-guide-scenarios/`

## Integration Points

| Test | User Guide Section | Manual Test Plan |
|------|-------------------|------------------|
| **01** | Initializing Your First Project | Phase 1: Test 1.1 |
| **02** | Creating Milestone Commits | Phase 1: Test 1.2 |
| **03** | Rolling Back | Phase 1: Test 1.5 |
| **04** | Browsing History | Phase 2: Test 2.4 |
| **05** | Automatic Versioning | Phase 2: Test 2.1 |
| **06** | Collaboration | Phase 3: Test 3.3 |
| **07** | Remote Sync | Phase 2: Test 2.5 |

## CI/CD Integration Snippet

```yaml
# Add to .github/workflows/test.yml
- name: Run User Guide Tests
  run: |
    cd test-scripts/user-guide-scenarios
    ./run_all_tests.sh
```

## Typical Test Session Timeline

```
00:00 - Prerequisites check (1 min)
00:01 - Test 1: Initialize (0.5 min)
00:02 - Test 2: Milestones (1 min)
00:03 - Test 3: Rollback (1 min)
00:04 - Test 4: Browse (1 min)
00:05 - Test 5: Auto-commit (2 min) ← includes 35s wait
00:07 - Test 6: Locks (1 min)
00:08 - Test 7: Remote (2 min)
00:10 - Complete (10 min total)
```

## Success Indicators

✅ **All tests passed** means:
- Project initialization works
- Commits include metadata
- Rollback restores state correctly
- History browsing functional
- Auto-commits working (if daemon running)
- Lock mechanism prevents conflicts
- Remote sync working

🎯 **Ready for MVP** if:
- Tests 1-4, 7 pass (core VCS functionality)
- Test 5 passes (auto-commit with daemon)
- Test 6 passes (lock logic, even if simulated)

## Next Steps After Tests Pass

1. ✓ Run manual tests with real Logic Pro projects
2. ✓ Test with actual music production workflow
3. ✓ Validate on clean macOS system
4. ✓ Performance test with large projects (10GB+)
5. ✓ Multi-user collaboration test (2+ people)

## Support

- **Full README**: [README.md](README.md)
- **User Guide**: [../../docs/USER_GUIDE.md](../../docs/USER_GUIDE.md)
- **Issues**: https://github.com/jbacus/auxin/issues

---

*Quick Reference - Last Updated: 2025-10-30*
