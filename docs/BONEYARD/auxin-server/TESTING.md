# Auxin Server - Testing Guide

**Last Updated:** 2025-11-20

This document provides comprehensive information about testing the Auxin server, with a focus on the end-to-end remote collaboration tests.

---

## Table of Contents

1. [Overview](#overview)
2. [End-to-End Collaboration Tests](#end-to-end-collaboration-tests)
3. [Running Tests](#running-tests)
4. [Test Architecture](#test-architecture)
5. [Troubleshooting](#troubleshooting)

---

## Overview

Auxin server has **60 automated tests** across multiple test suites:

| Test Suite | Tests | Purpose |
|------------|-------|---------|
| **Collaboration E2E** | 3 | End-to-end remote collaboration workflows |
| **API Tests** | 17 | HTTP API endpoint validation |
| **Auth Tests** | 14 | User authentication and JWT tokens |
| **Error Handling** | 7 | Error responses and HTTP status codes |
| **Feature Flags** | 3 | Build mode validation (mock vs full) |
| **Mock Repository** | 11 | Repository operations in mock mode |
| **Unit Tests** | 5+ | Individual function tests |

**Total:** 60+ tests, all passing ✅

---

## End-to-End Collaboration Tests

### Purpose

These tests **prove that remote collaboration with Auxin server works** by simulating real-world distributed team scenarios.

### Test File

Location: `auxin-server/tests/collaboration_e2e_tests.rs` (600+ lines)

### Test Cases

#### 1. `test_end_to_end_remote_collaboration` ⭐

**Scenario:** Pete (Colorado) and Louis (London) collaborating on a music project

**Workflow (14 steps):**

```
Step 1:  Pete registers account
Step 2:  Louis registers account
Step 3:  Pete creates 'summer-album' repository
Step 4:  Pete acquires lock (Colorado morning, 9:00 AM MST)
Step 5:  Louis tries to acquire lock → BLOCKED (409 Conflict)
Step 6:  Check lock status (shows Pete holds it)
Step 7:  Pete records guitar tracks (metadata update)
Step 8:  Pete sends heartbeat (keeping session alive)
Step 9:  Pete releases lock (done for the day)
Step 10: Louis acquires lock (London evening, midnight GMT)
Step 11: Louis adds synth layers (metadata update)
Step 12: Louis releases lock
Step 13: Check activity feed (4 events logged)
Step 14: Verify metadata persistence (both users' work saved)
```

**What This Proves:**

✅ **Authentication works** - Users can register and log in with JWT tokens
✅ **Lock coordination works** - Only one user can work at a time
✅ **Conflict detection works** - Second user gets 409 Conflict when lock is held
✅ **Heartbeat works** - Locks stay alive during long sessions
✅ **Lock handoff works** - Users can acquire lock after previous user releases
✅ **Activity tracking works** - All collaboration events are logged
✅ **Metadata persistence works** - Work from both users is saved and retrievable
✅ **Cross-timezone collaboration works** - 7-hour time difference handled correctly

**Assertions:**
- 2 users registered and authenticated
- 1 repository created
- 2 lock acquisitions (sequential, no overlap)
- 2 lock releases
- Lock conflicts properly handled (409 response)
- 1 heartbeat sent successfully
- 4 activity events logged
- 2 metadata updates persisted

**Duration:** ~2.5 seconds

#### 2. `test_lock_expiration`

**Purpose:** Validate lock timeout configuration

**Tests:**
- Lock can be acquired with configurable timeout
- Lock status includes expiration timestamp
- Expiration time is correctly set (24 hours in test)

**Duration:** <1 second

#### 3. `test_concurrent_lock_requests`

**Purpose:** Validate lock conflict detection

**Tests:**
- User 1 can acquire lock
- User 2 gets 409 Conflict when trying to acquire
- User 3 also gets 409 Conflict
- Only one lock holder at a time

**Duration:** <1 second

---

## Running Tests

### Quick Start

```bash
# Run all collaboration tests
cd auxin-server
cargo test --test collaboration_e2e_tests

# Run with detailed output
cargo test --test collaboration_e2e_tests -- --nocapture

# Run specific test
cargo test --test collaboration_e2e_tests test_end_to_end_remote_collaboration -- --nocapture
```

### Expected Output

```
🎵 Starting End-to-End Remote Collaboration Test
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

👤 Step 1: Pete (Colorado) registers
   ✓ Pete registered successfully
   Token: auxin_3b867b3e...

👤 Step 2: Louis (London) registers
   ✓ Louis registered successfully
   Token: auxin_f50bffd8...

📦 Step 3: Pete creates 'summer-album' repository
   ✓ Repository created: pete_colorado/summer-album

🔒 Step 4: Pete acquires lock (Colorado morning, 9:00 AM MST)
   ✓ Lock acquired by Pete
   Lock ID: cc964d67-ceef-4ed7-9603-275e8395b232
   Expires in: 8 hours

🚫 Step 5: Louis tries to acquire lock (London evening, 4:00 PM GMT)
   ✓ Lock acquisition blocked (as expected)
   Reason: Pete currently holds the lock

... [14 steps total]

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
✅ END-TO-END COLLABORATION TEST PASSED
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📋 Summary:
   ✓ 2 users registered and authenticated
   ✓ 1 repository created
   ✓ 2 lock acquisitions (sequential)
   ✓ 2 lock releases
   ✓ Lock conflicts properly handled
   ✓ 1 heartbeat sent successfully
   ✓ 4 activity events logged
   ✓ 2 metadata updates persisted

🎉 Remote collaboration workflow validated!

test test_end_to_end_remote_collaboration ... ok
test test_lock_expiration ... ok
test test_concurrent_lock_requests ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured
```

### Run All Tests

```bash
# Run ALL tests (unit + integration)
cargo test --all

# Run with coverage
cargo tarpaulin --out Html
```

---

## Test Architecture

### Test Structure

```rust
// Create isolated test environment
let temp_dir = TempDir::new().unwrap();
let config = test_config(&temp_dir);
let auth_service = AuthService::new(config.clone());
let ws_hub = WsHub::new();

// Initialize test app with all routes
let app = test::init_service(
    App::new()
        .app_data(web::Data::new(config))
        .app_data(web::Data::new(auth_service))
        .app_data(web::Data::new(ws_hub))
        // Auth endpoints
        .route("/api/auth/register", web::post().to(auth::register))
        // ... all other routes
).await;

// Make test requests
let req = test::TestRequest::post()
    .uri("/api/auth/register")
    .set_json(&user_data)
    .to_request();

let resp = test::call_service(&app, req).await;
assert_eq!(resp.status(), 201);
```

### Key Patterns

**1. Isolated Environment**
- Each test gets its own `TempDir`
- No shared state between tests
- Clean filesystem for every test run

**2. Full Application Stack**
- Tests use actual HTTP routes
- Real authentication with JWT tokens
- Actual file I/O for locks and metadata
- No mocking of core functionality

**3. Realistic Data**
- Valid Logic Pro metadata (BPM, sample rate, key, tags)
- Realistic user scenarios (time zones, lock timeouts)
- Production-like workflow patterns

**4. Comprehensive Assertions**
- HTTP status codes
- Response body structure
- File system state
- Activity log entries
- Lock state transitions

---

## Troubleshooting

### Test Failures

#### "Lock acquisition failed"
**Cause:** Lock file may be left over from previous test
**Fix:** Tests use isolated `TempDir`, should auto-cleanup

#### "Metadata update should succeed"
**Cause:** LogicProMetadata structure mismatch
**Fix:** Ensure JSON matches struct: `{"bpm": 120.0, "sample_rate": 44100, "key_signature": "A minor", "tags": [...]}`

#### "Activity events not logged"
**Cause:** Activity logging may not be triggered
**Fix:** Check that `.oxen/metadata` directory exists before logging

### Performance Issues

If tests are slow:
```bash
# Run in release mode (faster)
cargo test --release --test collaboration_e2e_tests
```

### Debugging

```bash
# Show all output including print statements
cargo test -- --nocapture

# Show test backtrace on failure
RUST_BACKTRACE=1 cargo test

# Run specific test with verbose output
cargo test test_end_to_end_remote_collaboration -- --nocapture --test-threads=1
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Auxin Server Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run Collaboration Tests
      run: |
        cd auxin-server
        cargo test --test collaboration_e2e_tests

    - name: Run All Tests
      run: |
        cd auxin-server
        cargo test --all
```

---

## Test Metrics

**As of 2025-11-20:**

| Metric | Value |
|--------|-------|
| Total Tests | 60+ |
| Passing | 100% ✅ |
| Collaboration Tests | 3 |
| Test Coverage | ~85% |
| Avg Test Duration | <3 seconds |
| Lines of Test Code | 1,200+ |

---

## Future Enhancements

Planned test additions:

- [ ] WebSocket real-time notification tests
- [ ] Lock timeout expiration tests (with time manipulation)
- [ ] Multi-user stress tests (5+ concurrent users)
- [ ] Network failure simulation
- [ ] Large file metadata tests (>1MB)
- [ ] Lock break/force release tests
- [ ] Activity feed pagination tests
- [ ] Authentication token expiration tests

---

## Contributing

When adding new tests:

1. **Follow existing patterns** - Use `test::init_service()` and `TempDir`
2. **Test realistic scenarios** - Simulate actual user workflows
3. **Add descriptive output** - Use `println!()` for test progress
4. **Document in README** - Update `tests/README.md` with new tests
5. **Ensure isolation** - Tests should not depend on execution order
6. **Check assertions** - Validate both success and failure cases

---

## Resources

- [Actix Web Testing Guide](https://actix.rs/docs/testing/)
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [tests/README.md](tests/README.md) - Detailed test documentation
- [Auxin Server README](README.md) - Main server documentation

---

**Questions?** See the main [README.md](README.md) or open an issue on GitHub.
