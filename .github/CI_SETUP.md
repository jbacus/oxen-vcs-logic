# GitHub Actions CI/CD Setup

## Overview

This repository uses GitHub Actions to automatically run tests on every commit and pull request. The test suite ensures code quality and prevents regressions.

## Test Workflow

**File**: `.github/workflows/test.yml`

### When Tests Run

Tests run automatically on:
- ✅ **Push** to `main`, `develop`, or `claude/**` branches
- ✅ **Pull requests** targeting `main` or `develop`
- ✅ **Manual trigger** via GitHub Actions UI (workflow_dispatch)

### Test Jobs

The workflow includes 5 parallel jobs:

#### 1. **All Tests** (Primary)
- Uses `./run_all_tests.sh` - the same script you run locally
- Runs all 487 tests across all components
- **This is the main quality gate**

#### 2. **Rust Tests (Detailed)**
- 194 unit tests
- 30 Oxen integration tests
- 34 CLI integration tests
- Code formatting check (`cargo fmt`)
- Linting check (`cargo clippy`)

#### 3. **Swift LaunchAgent Tests**
- 229 tests including new Daemon and ServiceManager tests
- Builds on macOS-14 with Xcode 15.2

#### 4. **Swift App Tests**
- UI component tests
- Mock XPC client validation

#### 5. **Build All Components**
- Verifies all components build successfully in release mode
- Runs after all tests pass

#### 6. **Test Summary**
- Aggregates results from all jobs
- Displays formatted summary
- Fails the build if critical tests fail

## Viewing Test Results

### GitHub UI

1. Go to your repository on GitHub
2. Click **Actions** tab
3. Click on any workflow run
4. See status of all test jobs

### Status Badge

The README includes a status badge that shows current test status:

[![Test Suite](https://github.com/jbacus/oxen-vcs-logic/actions/workflows/test.yml/badge.svg)](https://github.com/jbacus/oxen-vcs-logic/actions/workflows/test.yml)

- ✅ **Green (passing)**: All tests passed
- ❌ **Red (failing)**: Some tests failed
- 🟡 **Yellow (pending)**: Tests are running

### Pull Request Checks

When you create a pull request:
1. Tests run automatically
2. PR shows "Checks" section with status
3. You can merge only when tests pass (recommended)

## Test Environment

### Requirements

- **Runner**: macOS-14 (required for Swift/Xcode)
- **Rust**: Stable toolchain
- **Xcode**: 15.2
- **Oxen CLI**: Installed via `pip3 install oxen-ai`

### Caching

The workflow caches:
- Cargo registry and build artifacts
- Speeds up subsequent runs by ~50%

## Manual Testing

To run the same tests locally that CI runs:

```bash
# Install Oxen CLI (if not installed)
pip3 install oxen-ai
oxen config --name "Your Name" --email "your@email.com"

# Run the full test suite
./run_all_tests.sh

# Or run individual components
cd OxVCS-CLI-Wrapper && cargo test
cd OxVCS-LaunchAgent && swift test
cd OxVCS-App && swift test
```

## Troubleshooting

### Tests Pass Locally But Fail in CI

1. **Oxen CLI not configured**: CI installs and configures Oxen automatically
2. **File paths**: Use relative paths, not absolute paths in tests
3. **Timing issues**: CI may be slower; increase timeouts if needed

### Workflow Not Triggering

1. Check branch name matches `push.branches` patterns
2. Verify `.github/workflows/test.yml` is committed
3. Check GitHub Actions is enabled for your repo

### Slow Test Runs

- First run: ~5-10 minutes (building dependencies)
- Cached runs: ~3-5 minutes (cached dependencies)
- Consider disabling `cargo clippy` if too slow

## Branch Protection (Recommended)

To require tests to pass before merging:

1. Go to **Settings** → **Branches**
2. Add rule for `main` branch
3. Enable **Require status checks to pass**
4. Select:
   - ✅ All Tests (run_all_tests.sh)
   - ✅ Rust Tests (Detailed)
   - ✅ Swift LaunchAgent Tests
   - ✅ Swift App Tests
   - ✅ Build All Components

## Test Coverage Report

Current coverage (as of October 2025):
- **Overall**: ~65%
- **Rust CLI**: ~90%
- **Swift LaunchAgent**: ~85%
- **Swift App**: <5%

See `CLAUDE.md` for detailed coverage breakdown.

## Adding New Tests

When you add new tests:

1. **Add to appropriate test file**:
   - Rust: `tests/*.rs`
   - Swift LaunchAgent: `Tests/*Tests.swift`
   - Swift App: `Tests/*Tests.swift`

2. **Run locally first**:
   ```bash
   ./run_all_tests.sh
   ```

3. **Commit and push**:
   ```bash
   git add .
   git commit -m "Add tests for X feature"
   git push
   ```

4. **Check GitHub Actions**:
   - Tests run automatically
   - Fix any failures before merging

## Cost Considerations

- **macOS runners**: 10x cost of Linux runners
- **Current usage**: ~5-10 min per run
- **Free tier**: 20,000 minutes/month for macOS (plenty!)

If you exceed limits:
- Reduce test frequency (e.g., only on PR to main)
- Use `workflow_dispatch` for manual runs only
- Cache more aggressively

## Support

For issues with CI/CD:
1. Check workflow run logs in Actions tab
2. Review this documentation
3. Run tests locally to reproduce
4. Create an issue with logs attached

---

**Last Updated**: October 29, 2025
**Workflow Version**: 2.0 (with comprehensive test suite)
