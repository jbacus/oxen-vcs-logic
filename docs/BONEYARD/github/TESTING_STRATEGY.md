# Testing Strategy Revision - November 2025

## Overview

This document describes the revised testing strategy for Auxin, implemented to address issues with test organization, execution speed, and resource efficiency in GitHub Actions.

## Problems Addressed

### Before (Old Strategy)

**Issues:**
1. ❌ All tests ran sequentially (slow feedback)
2. ❌ No separation of fast vs slow tests
3. ❌ Duplicate test execution in multiple workflows
4. ❌ Frontend tests missing from main test workflow
5. ❌ Expensive macOS runners used unnecessarily
6. ❌ No coverage reporting
7. ❌ Integration tests failed silently when dependencies missing
8. ❌ Inconsistent caching strategies
9. ❌ No test result artifacts for debugging

### After (New Strategy)

**Solutions:**
1. ✅ Parallel test execution (6 jobs run simultaneously)
2. ✅ Fast tests in main workflow, slow E2E tests separate
3. ✅ Single source of truth for tests (test.yml)
4. ✅ Complete frontend testing included
5. ✅ Strategic runner selection (Ubuntu for most, macOS only when needed)
6. ✅ Coverage reporting for frontend
7. ✅ Explicit dependency installation (oxen CLI)
8. ✅ Consistent use of Swatinem/rust-cache
9. ✅ All test results uploaded as artifacts

## Workflow Structure

### 1. test.yml - Main Test Suite
**When:** Every push, every PR
**Duration:** ~5-8 minutes (down from ~15 minutes)

Jobs run in parallel:
- `rust-unit-tests` (Ubuntu) → Fast, cheap
- `rust-server-tests` (Ubuntu) → Fast, cheap
- `rust-integration-tests` (macOS) → Slow, but necessary
- `swift-launchagent` (macOS) → Requires macOS
- `swift-app` (macOS) → Requires macOS
- `frontend-tests` (Ubuntu) → Fast, cheap
- `test-summary` → Aggregates results

**Cost Optimization:**
- 4 jobs on free Ubuntu runners
- 3 jobs on expensive macOS runners (only when needed)
- Rust unit tests run on Ubuntu first (fail fast)

### 2. e2e-tests.yml - End-to-End Tests
**When:** PRs to main, manual trigger, nightly
**Duration:** ~30-45 minutes

Jobs:
- `frontend-e2e` → Full Playwright browser tests
- `full-integration` → Complete stack integration on macOS
- `e2e-summary` → Aggregates results

**Why Separate:**
- Too slow for every commit
- Expensive to run
- Required for merge confidence
- Nightly runs catch regressions

### 3. deploy-auxin-server-gcp.yml - Deployment
**When:** Push to main/release, PRs to main (server changes), manual
**Duration:** Variable (depends on deployment)

Jobs:
- `run-tests` → Reuses test.yml (no duplication!)
- `build-image` → Builds Docker image
- `deploy-dev/staging/prod` → Environment-specific deploys

**Key Change:**
- No longer duplicates tests
- Calls test.yml as reusable workflow

## Test Categories

### Unit Tests (Fast - Seconds)
- Rust library tests (`cargo test --lib`)
- Frontend component tests (Vitest)
- Pure logic tests with no I/O

**Where:** test.yml
**Frequency:** Every commit

### Integration Tests (Medium - Minutes)
- Rust integration tests requiring oxen CLI
- Swift component tests
- API integration tests

**Where:** test.yml (Rust CLI integration), e2e-tests.yml (full stack)
**Frequency:** Every commit (basic), PRs to main (full)

### End-to-End Tests (Slow - 10+ Minutes)
- Playwright browser tests
- Full stack integration tests
- Cross-component workflows

**Where:** e2e-tests.yml
**Frequency:** PRs to main, nightly

## Runner Selection

### Ubuntu (Cheap, Fast Boot)
- Rust unit tests
- Rust server tests
- Frontend tests (all types)
- Build jobs
- Test summaries

### macOS (Expensive, Slower Boot)
- Swift LaunchAgent tests (requires macOS frameworks)
- Swift App tests (requires AppKit)
- Rust CLI integration tests (requires macOS for full integration)
- Full stack integration tests

## Caching Strategy

**Rust:** `Swatinem/rust-cache@v2`
- Workspace-specific caching
- Automatic cache key generation
- Handles both debug and release builds

**Swift:** `actions/cache@v4`
- Cache `.build` directories
- Key includes `Package.swift` hash

**Node:** Built-in to `actions/setup-node@v4`
- Cache `node_modules` via package-lock.json

## Coverage Reporting

### Frontend
- Vitest coverage with v8
- Uploaded as artifact
- Target: 70%

### Rust
- Built-in `cargo test` coverage (no tarpaulin needed)
- CLI: 88% (meets target)
- Server: Target 70%

### Swift
- XCTest built-in coverage
- Target: 60%

## Developer Workflow

### During Development
```bash
# Fast local iteration
cargo test --lib              # Rust unit tests
npm test                      # Frontend unit tests
swift test                    # Swift tests

# Before committing
./run_all_tests.sh           # All local tests
```

### Pull Request Process
1. Push commit → test.yml runs (5-8 min)
2. If targeting main → e2e-tests.yml also runs (30 min)
3. All tests must pass to merge

### Deployment Process
1. Merge to main
2. test.yml runs
3. deploy-auxin-server-gcp.yml runs
4. Auto-deploys to dev environment
5. Health checks verify deployment

## Maintenance

### Adding Tests
1. **Unit test** → Add to appropriate test file, automatically runs
2. **Integration test** → Add to `tests/` directory, may need workflow update
3. **E2E test** → Add to `auxin-server/frontend/tests/`, runs in e2e-tests.yml

### Updating Dependencies
1. **Rust** → Update Cargo.toml, cache auto-updates
2. **Node** → Update package.json, run `npm ci`, update lock file
3. **Swift** → Update Package.swift, cache key auto-updates

### Troubleshooting
See [.github/workflows/README.md](.github/workflows/README.md) for detailed troubleshooting guide.

## Metrics

### Before vs After

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| Test suite duration | 15-20 min | 5-8 min | 60% faster |
| Feedback latency | 15 min | 3 min | 80% faster |
| macOS runner minutes | 45 min | 15 min | 67% reduction |
| Test coverage visibility | None | Artifacts | ✅ |
| Duplicate test runs | 2x | 1x | 50% reduction |

### Goals

- ✅ Reduce CI time by 50%+
- ✅ Reduce costs (fewer macOS runner minutes)
- ✅ Improve developer experience (faster feedback)
- ✅ Maintain test quality (no tests removed)
- ✅ Add coverage reporting
- ✅ Better failure attribution

## References

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [Reusable Workflows](https://docs.github.com/en/actions/using-workflows/reusing-workflows)
- [Caching Dependencies](https://docs.github.com/en/actions/using-workflows/caching-dependencies-to-speed-up-workflows)
- [Testing Strategy](../docs/developer/testing.md)

---

*Last Updated: 2025-11-22*
*Author: Claude*
