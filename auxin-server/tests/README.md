# Auxin Server Tests

This directory contains comprehensive tests for the Auxin server, including end-to-end collaboration scenarios.

## Test Structure

### Unit Tests (`src/`)
- Embedded within source files
- Test individual functions and components
- Run with: `cargo test --lib`

### Integration Tests (`tests/`)
- Test full workflows and API interactions
- Run with: `cargo test --test <test_name>`

## Test Files

### `collaboration_e2e_tests.rs`
**Purpose**: End-to-end remote collaboration testing

Simulates real-world scenarios of distributed teams using Auxin server for collaborative music production.

**Test Scenarios**:

1. **`test_end_to_end_remote_collaboration`**
   - Comprehensive workflow test simulating Pete (Colorado) and Louis (London) collaborating on a music project
   - Tests 14 steps including:
     - User registration and authentication
     - Repository creation
     - Lock acquisition and coordination
     - Metadata updates (Logic Pro project metadata)
     - Heartbeat system
     - Lock handoff between users
     - Activity feed tracking
     - Metadata persistence
   - **Duration**: ~2.5 seconds
   - **Validations**: 2 users, 2 locks, 4 activity events, 2 metadata updates

2. **`test_lock_expiration`**
   - Tests lock timeout configuration
   - Validates lock status includes expiration time
   - **Duration**: <1 second

3. **`test_concurrent_lock_requests`**
   - Tests that only one user can hold a lock at a time
   - Validates 409 Conflict responses for concurrent lock attempts
   - **Duration**: <1 second

**Key Features Tested**:
- ✅ User authentication with JWT tokens
- ✅ Repository management
- ✅ Distributed locking with conflict detection
- ✅ Lock heartbeat system
- ✅ Activity logging and feeds
- ✅ Logic Pro metadata storage and retrieval
- ✅ WebSocket integration (infrastructure)

**Running the Tests**:
```bash
# Run all collaboration tests
cargo test --test collaboration_e2e_tests

# Run with output
cargo test --test collaboration_e2e_tests -- --nocapture

# Run specific test
cargo test --test collaboration_e2e_tests test_end_to_end_remote_collaboration -- --nocapture
```

### `api_tests.rs`
Basic API endpoint tests including:
- Health checks
- Repository CRUD operations
- Authentication endpoints
- Activity endpoints

### `error_handling_tests.rs`
Tests error responses and HTTP status codes

### `feature_flag_tests.rs`
Tests feature flag behavior (mock-oxen vs full-oxen)

### `mock_repository_tests.rs`
Tests mock Oxen repository operations

## Test Coverage

As of 2025-11-20:
- **57 total tests** (22 unit + 35 integration)
- **All passing** ✅
- **Coverage**: Mock mode functionality fully tested

## Changes Made (2025-11-20)

### New Features
1. Added `AppError::Conflict` variant for proper 409 Conflict HTTP status codes
2. Implemented comprehensive end-to-end collaboration test suite
3. Enhanced error handling for lock conflicts

### Files Modified
- `src/error.rs`: Added `Conflict` error variant
- `src/repo_mock.rs`: Updated lock acquisition to return `Conflict` on `AlreadyExists`
- `src/repo_full.rs`: Updated lock acquisition to return `Conflict` on `AlreadyExists`
- `tests/collaboration_e2e_tests.rs`: **NEW** - 600+ line comprehensive collaboration test

### Bug Fixes
- Lock conflict now returns correct HTTP 409 status instead of 400
- Lock status endpoint returns proper nested structure: `{"locked": true, "lock": {...}}`

## CI/CD Integration

These tests are designed to run in CI/CD pipelines:
```bash
# In CI, run all tests
cargo test --all

# Generate coverage report
cargo tarpaulin --out Xml
```

## Future Enhancements

Potential additions:
- [ ] WebSocket notification tests (currently infrastructure only)
- [ ] Long-running lock tests with real timeouts
- [ ] Multi-user concurrent access stress tests
- [ ] Network failure simulation tests
- [ ] Large file metadata tests

## Contributing

When adding new tests:
1. Follow the existing test structure
2. Use descriptive test names
3. Add documentation comments
4. Ensure tests are deterministic
5. Add new tests to this README
