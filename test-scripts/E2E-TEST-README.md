# End-to-End Full System Test

Comprehensive test covering CLI, daemon, server, GUI, and collaboration workflows.

## What This Test Covers

### Components Tested
- ✅ **Auxin CLI** - All major commands
- ✅ **Auxin Server** - REST API, WebSocket, locks, metadata
- ✅ **Collaboration Workflow** - Two users (Pete & Louis) working together
- ✅ **Web Dashboard** - All API endpoints
- ⚠️  **macOS Daemon** - Acknowledged (requires LaunchAgent setup)
- ⚠️  **macOS GUI** - Acknowledged (requires manual verification)

### Workflow Tested
1. **Pete creates a project** - Initializes Logic Pro project, makes commits
2. **Server setup** - Auxin server running with Pete's repository
3. **Louis clones project** - Gets Pete's work from server
4. **Parallel work** - Both users make changes independently
5. **Locking** - Pete acquires lock, Louis sees it's taken
6. **Restore** - Pete restores to previous milestone
7. **Metadata & bounces** - Upload and retrieve bounce files
8. **Activity tracking** - All actions logged

## Prerequisites

```bash
# 1. Build auxin CLI
cd Auxin-CLI-Wrapper
cargo build --release
cargo install --path .

# 2. Build auxin-server
cd ../auxin-server
cargo build --release --features mock-oxen

# 3. Ensure oxen is installed
brew install oxen

# 4. (Optional) Build macOS app
cd ../Auxin-App
./build-app.sh
```

## Running the Test

### Quick Run
```bash
cd /path/to/auxin
./test-scripts/e2e-full-system-test.sh
```

### With Output Logging
```bash
./test-scripts/e2e-full-system-test.sh 2>&1 | tee e2e-test-output.log
```

### Watch Mode (for development)
```bash
# In one terminal - watch server logs
tail -f /tmp/auxin-e2e-test-*/server.log

# In another terminal - run test
./test-scripts/e2e-full-system-test.sh
```

## Test Phases

### Phase 1: Start Auxin Server
- Starts server on port 3333
- Verifies health endpoint
- Tests repository API

### Phase 2: Pete Creates Initial Project
- Creates Logic Pro project structure
- Initializes Auxin repository
- Makes milestone commit with metadata
- Sets up remote
- Creates repository on server

### Phase 3: Pete's Daemon (Acknowledged)
- Notes daemon functionality
- In full test, would monitor for auto-commits

### Phase 4: Louis Clones Project
- Louis gets project from server
- Verifies clone completeness

### Phase 5: Collaboration Workflow
- Pete adds drums track
- Louis adds vocals track
- Both commit independently
- Test status command

### Phase 6: Locking Workflow
- Pete acquires lock
- Checks lock status
- Louis attempts to acquire (should see taken)
- Pete releases lock

### Phase 7: Restore Workflow
- View commit history
- Restore to previous milestone via API
- Verify activity logging

### Phase 8: Metadata and Bounce Files
- Upload bounce file via API
- List bounces
- Retrieve bounce metadata

### Phase 9: Web Dashboard Integration
- List repositories
- Get repository details
- Fetch commits via API
- Get activity feed

### Phase 10: CLI Command Coverage
- `auxin --version`
- `auxin --help`
- `auxin status`
- `auxin log`
- `auxin branch`
- Branch creation

### Phase 11: GUI Testing (Manual)
- Provides checklist for manual GUI verification

## Expected Results

### Success Output
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ✓ ALL TESTS PASSED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Total Tests Run:    45+
Tests Passed:       45+
Tests Failed:       0

Server is still running at: http://localhost:3333
```

The test will keep the server running so you can:
- Browse web UI: `open http://localhost:3333`
- Inspect Pete's project
- Inspect Louis's project
- Review server logs

Press `Ctrl+C` to cleanup and exit.

### Failure Output
If any test fails, you'll see:
- Red `[FAIL]` messages
- Expected vs actual values
- Server logs location for debugging

## Test Data

All test data is created in `/tmp/auxin-e2e-test-<PID>/`:
- `server-data/` - Server repository storage
- `pete/` - Pete's workspace
- `louis/` - Louis's workspace
- `server.log` - Server output
- `test-bounce.wav` - Sample bounce file

Cleanup happens automatically on exit (success or failure).

## Customization

### Change Server Port
Edit the script:
```bash
SERVER_PORT=3333  # Change this line
```

### Skip Phases
Comment out phases you don't want to test:
```bash
# log_section "Phase 7: Restore Workflow"
# ... skip this phase
```

### Add Custom Scenarios
Add new phases at the end:
```bash
log_section "Phase 12: My Custom Test"
cd "$PETE_WORKSPACE/$PROJECT_NAME"
# ... your test code
```

## Troubleshooting

### "auxin CLI not found in PATH"
```bash
cd Auxin-CLI-Wrapper
cargo install --path .
```

### "auxin-server binary not found"
```bash
cd auxin-server
cargo build --release --features mock-oxen
```

### "Server failed to start"
- Check if port 3333 is already in use: `lsof -i :3333`
- Kill existing process: `kill -9 $(lsof -ti:3333)`
- Check server log: `cat /tmp/auxin-e2e-test-*/server.log`

### "Tests fail with mock-oxen"
Some operations (clone, push, pull) return `501 Not Implemented` with mock-oxen mode.
This is expected. The test handles these gracefully.

For full VCS operations, build with `--features full-oxen` (requires async refactoring).

### "Permission denied"
Make script executable:
```bash
chmod +x test-scripts/e2e-full-system-test.sh
```

## Integration with CI/CD

### GitHub Actions
```yaml
name: E2E Test
on: [push, pull_request]

jobs:
  e2e-test:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
      - name: Install oxen
        run: brew install oxen
      - name: Build components
        run: |
          cd Auxin-CLI-Wrapper && cargo build --release && cargo install --path .
          cd ../auxin-server && cargo build --release --features mock-oxen
      - name: Run E2E test
        run: ./test-scripts/e2e-full-system-test.sh
```

## Next Steps

After successful E2E test:
1. Run on macOS with full LaunchAgent daemon
2. Test GUI manually using Auxin.app
3. Test with real Logic Pro projects
4. Stress test with large files
5. Test network failures and recovery

## Related Tests

- `user-guide-scenarios/` - Individual scenario tests
- `run_all_tests.sh` - Runs all scenario tests
- Unit tests in each component

## Contributing

To add new test scenarios:
1. Add a new phase to the script
2. Use `log_info`, `log_success`, `log_error` for output
3. Use `assert_*` functions for validation
4. Update this README with the new phase
5. Test in isolation before adding to main script
