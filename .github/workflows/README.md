# GitHub Actions Workflows

This directory contains automated CI/CD workflows for the Auxin project.

## Workflow Overview

### 🧪 [test.yml](test.yml) - Test Suite (Main)
**Triggers:** Push to main/develop/claude/**, PRs to main/develop, manual

Runs all automated tests in parallel for fast feedback:

- **Rust CLI Unit Tests** (Ubuntu) - Fast unit tests, linting, formatting
- **Rust CLI Integration Tests** (macOS) - Integration tests requiring oxen CLI
- **Rust Server Tests** (Ubuntu) - Server-side Rust tests
- **Swift LaunchAgent Tests** (macOS) - macOS daemon tests
- **Swift App Tests** (macOS) - macOS GUI application tests
- **Frontend Tests** (Ubuntu) - React frontend tests with coverage

**Key Features:**
- ✅ Jobs run in parallel for speed
- ✅ Fast unit tests run first on cheap Ubuntu runners
- ✅ Expensive macOS tests only for components that need them
- ✅ Automatic test result artifact uploads
- ✅ Coverage reporting for frontend
- ✅ Concurrency control (cancels old runs when new commits pushed)

### 🚀 [e2e-tests.yml](e2e-tests.yml) - End-to-End Tests
**Triggers:** PRs to main, manual, nightly at 2 AM UTC

Runs expensive, slow end-to-end tests:

- **Frontend E2E Tests** - Playwright browser tests with full backend
- **Full Integration Tests** - Complete stack integration on macOS

**Key Features:**
- ✅ Only runs on important events (PRs to main, manual, nightly)
- ✅ Full stack testing with real backend
- ✅ Screenshot capture on failure
- ✅ Timeout protection (30-45 min limits)

### 🚢 [deploy-auxin-server-gcp.yml](deploy-auxin-server-gcp.yml) - Server Deployment
**Triggers:** Push to main/release/**, PRs to main (server changes), manual

Deploys the Auxin server to Google Cloud Platform:

- **Run Tests** - Reuses test.yml workflow (no duplication!)
- **Build Image** - Builds Docker image
- **Deploy Dev** - Auto-deploy to dev on main branch
- **Deploy Staging** - Auto-deploy to staging on release/** branches
- **Deploy Prod** - Manual only (workflow_dispatch)

**Key Features:**
- ✅ No test duplication (reuses test.yml)
- ✅ Separate environments (dev/staging/prod)
- ✅ Workload Identity Federation for secure GCP auth
- ✅ Smoke tests after deployment
- ✅ Environment-specific configurations

### 🤖 [claude.yml](claude.yml) - Claude Code Assistant
**Triggers:** Issue/PR comments with @claude

Enables Claude AI to assist with GitHub issues and PRs.

**Usage:**
- Comment `@claude <your request>` on any issue or PR
- Claude will respond with code changes, reviews, or answers
- Provides on-demand assistance without automatic triggers

## Test Organization Strategy

### Fast Tests (Run on Every Push)
Located in `test.yml`:
- Rust unit tests (~30s)
- Frontend unit tests (~1min)
- Linting and formatting (~10s)

### Slow Tests (Run on PRs to Main)
Located in `e2e-tests.yml`:
- Integration tests requiring oxen CLI (~5min)
- Full stack integration (~10min)
- Playwright E2E tests (~15min)

### Why This Organization?

1. **Developer Feedback Speed**
   - Fast tests give immediate feedback on every commit
   - Developers don't wait for slow tests during iteration

2. **Cost Efficiency**
   - Ubuntu runners are cheaper than macOS
   - macOS runners only used when necessary

3. **Resource Efficiency**
   - Parallel execution reduces total time
   - Concurrency control prevents waste
   - Test caching speeds up repeated runs

4. **Clear Failure Attribution**
   - Separate jobs make it clear what failed
   - Test summary provides overview
   - Artifacts preserved for debugging

## Running Tests Locally

### All Tests
```bash
./run_all_tests.sh
```

### Individual Components

**Rust CLI:**
```bash
cd Auxin-CLI-Wrapper
cargo test --lib                    # Unit tests only
cargo test --test '*'               # Integration tests (requires oxen CLI)
cargo fmt -- --check                # Format check
cargo clippy -- -D warnings         # Linting
```

**Rust Server:**
```bash
cd auxin-server
cargo test --features mock-oxen     # All tests
cargo fmt -- --check                # Format check
cargo clippy -- -D warnings         # Linting
```

**Swift LaunchAgent:**
```bash
cd Auxin-LaunchAgent
swift test --parallel               # All tests
swift build -c release              # Build check
```

**Swift App:**
```bash
cd Auxin-App
swift test --parallel               # All tests
swift build -c release              # Build check
```

**Frontend:**
```bash
cd auxin-server/frontend
npm run lint                        # Linting
npm run type-check                  # TypeScript check
npm test                            # Unit tests
npm run test:coverage               # With coverage
npm run test:e2e                    # E2E tests (requires backend)
```

## Adding New Tests

### Unit Tests
Add to the appropriate test file in your component. These will automatically run in `test.yml`.

### Integration Tests
Add to the `tests/` directory in your component. May need to update the workflow if new dependencies are required.

### E2E Tests
Add to `auxin-server/frontend/tests/` for Playwright tests. These will run in `e2e-tests.yml`.

## Troubleshooting

### Tests Failing Locally but Passing in CI
- Check your Rust/Swift/Node versions match CI
- Ensure dependencies are up to date
- Look for environment-specific issues

### Tests Passing Locally but Failing in CI
- Check for timing issues (tests too fast/slow)
- Look for hardcoded paths or assumptions
- Review CI logs for environment differences

### macOS Runner Availability Issues
- macOS runners can sometimes be in short supply
- Consider if your test really needs macOS
- Use Ubuntu runners when possible

### Cache Issues
- Workflows use different cache keys per component
- Clear cache by changing cache key if needed
- Caches are scoped to branch by default

## Workflow Maintenance

### When to Update test.yml
- Adding new test categories
- Changing test infrastructure
- Updating dependencies that affect testing

### When to Update e2e-tests.yml
- Adding new E2E test suites
- Changing integration test requirements
- Updating timeout values for slow tests

### When to Update deploy-auxin-server-gcp.yml
- Changing deployment process
- Adding new environments
- Updating GCP configuration

## Best Practices

1. **Keep Tests Fast**
   - Unit tests should complete in seconds
   - Move slow tests to integration category
   - Use mocks/stubs where appropriate

2. **Keep Tests Isolated**
   - No shared state between tests
   - Clean up after tests
   - Don't depend on test order

3. **Keep Tests Deterministic**
   - No flaky tests
   - No timing dependencies
   - Seed random values

4. **Keep Tests Maintainable**
   - Clear test names
   - Good error messages
   - Minimal duplication

## Coverage Goals

- **Rust CLI:** 88% (currently met) ✅
- **Rust Server:** 70% target
- **Swift Components:** 60% target
- **Frontend:** 70% target

Coverage reports are uploaded as artifacts in the test runs.
