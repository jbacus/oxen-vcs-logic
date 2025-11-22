# CI/CD Guide for Auxin

This document provides a comprehensive overview of Auxin's continuous integration and deployment infrastructure.

## Table of Contents

- [Overview](#overview)
- [Workflows](#workflows)
  - [Test Suite](#test-suite)
  - [E2E Tests](#e2e-tests)
  - [Security Scanning](#security-scanning)
  - [Code Coverage](#code-coverage)
  - [Performance Testing](#performance-testing)
  - [Release](#release)
  - [GCP Deployment](#gcp-deployment)
- [Dependency Management](#dependency-management)
- [Secrets Configuration](#secrets-configuration)
- [Monitoring & Notifications](#monitoring--notifications)

---

## Overview

Auxin uses GitHub Actions for all CI/CD operations, with workflows organized by concern:

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `test.yml` | Push, PR | Comprehensive test suite across all components |
| `e2e-tests.yml` | PR to main, nightly | End-to-end integration testing |
| `security.yml` | Push, PR, weekly | Security scanning with CodeQL and Trivy |
| `coverage.yml` | Push, PR | Code coverage reporting to Codecov |
| `performance.yml` | Push to main, weekly | Benchmarks and load testing |
| `release.yml` | Version tags | Automated release artifact creation |
| `deploy-auxin-server-gcp.yml` | Push to main/release | GCP Cloud Run deployment |

---

## Workflows

### Test Suite

**File:** `.github/workflows/test.yml`

**Runs on:** Every push and PR to main/develop branches

**Components Tested:**
- ✅ Rust CLI (unit + integration tests)
- ✅ Rust Server (unit tests with mock-oxen feature)
- ✅ Swift LaunchAgent (macOS tests)
- ✅ Swift App (macOS tests)
- ✅ Frontend (lint, type-check, unit tests)

**Key Features:**
- Parallel test execution for speed
- Caching of dependencies (Rust, Swift, npm)
- Test artifact uploads
- Comprehensive test summary

**Manual Trigger:**
```bash
# Via GitHub CLI
gh workflow run test.yml
```

---

### E2E Tests

**File:** `.github/workflows/e2e-tests.yml`

**Runs on:**
- Pull requests to main (required for merge)
- Manual dispatch
- Nightly at 2 AM UTC

**Tests:**
1. **Frontend E2E** - Playwright tests with real backend
2. **Full Integration** - Complete stack test on macOS

**Duration:** 30-45 minutes (expensive)

**Artifacts:**
- Playwright test reports
- Screenshots on failure
- Integration test results

---

### Security Scanning

**File:** `.github/workflows/security.yml`

**Runs on:**
- Push/PR to main/develop
- Weekly on Sundays at 3 AM UTC
- Manual dispatch

**Scans:**

#### CodeQL Analysis
- **Rust** - Security and quality queries
- **JavaScript/TypeScript** - Frontend security
- **Swift** - macOS component security

Results appear in GitHub Security tab

#### Docker Security
- **Trivy** - Container vulnerability scanning
- **SARIF** upload to GitHub Security
- Checks for CRITICAL and HIGH severity issues

#### Dependency Audits
- **cargo-audit** - Rust dependency vulnerabilities
- **npm audit** - Frontend dependency vulnerabilities

**Viewing Results:**
- Go to repository → Security → Code scanning alerts
- Review artifact uploads for detailed reports

---

### Code Coverage

**File:** `.github/workflows/coverage.yml`

**Runs on:** Push/PR to main/develop

**Coverage Tools:**
- **Rust:** cargo-llvm-cov
- **Swift:** Xcode code coverage + llvm-cov export
- **Frontend:** Jest with istanbul

**Uploaded to:** [Codecov](https://codecov.io)

**Configuration:** `codecov.yml` at repository root

**Targets:**
- Project overall: 80%
- New code (patches): 70%

**Viewing Coverage:**
1. Check PR comment from Codecov bot
2. Visit Codecov dashboard
3. View badges in README

**Setup Required:**
```bash
# Add CODECOV_TOKEN to repository secrets
# Get token from: https://codecov.io/gh/YOUR_ORG/auxin/settings
```

---

### Performance Testing

**File:** `.github/workflows/performance.yml`

**Runs on:**
- Push to main
- Pull requests to main
- Weekly on Saturdays at 4 AM UTC

**Tests:**

#### Rust CLI Benchmarks
- Uses `cargo bench`
- Tracks performance over time
- Alerts on >150% regression

#### Server Load Testing
- Uses k6 for load testing
- Simulates 10-50 concurrent users
- Tests performance thresholds:
  - p95 latency < 500ms
  - Error rate < 5%

#### Frontend Performance
- Lighthouse CI for web vitals
- Tests Core Web Vitals

**Artifacts:**
- Benchmark results (graphed over time)
- Load test JSON reports
- Lighthouse reports

---

### Release

**File:** `.github/workflows/release.yml`

**Trigger:** Push tag matching `v*` (e.g., `v0.3.0`)

**Creates:**
1. **Linux CLI** - `auxin-cli-linux.tar.gz`
2. **macOS CLI** - `auxin-cli-macos.tar.gz`
3. **macOS Installer** - `Auxin-vX.Y.Z.pkg`
4. **macOS DMG** - `Auxin-vX.Y.Z.dmg`

**Process:**
```bash
# Create and push a version tag
git tag v0.3.0
git push origin v0.3.0

# Workflow automatically:
# 1. Builds all artifacts
# 2. Creates GitHub Release
# 3. Uploads all files
# 4. Generates release notes
```

**Release Notes:** Edit the generated release on GitHub to add changelog

---

### GCP Deployment

**File:** `.github/workflows/deploy-auxin-server-gcp.yml`

**Environments:**
- **Development** - Auto-deploy from `main` branch
- **Staging** - Auto-deploy from `release/*` branches
- **Production** - Manual workflow_dispatch only

**Process:**
1. Run tests via `test.yml`
2. Build Docker image
3. Push to GCP Artifact Registry
4. Deploy to Cloud Run
5. Run smoke tests
6. Send Slack notification

**Resources by Environment:**

| Environment | Memory | CPU | Min Instances | Max Instances |
|-------------|--------|-----|---------------|---------------|
| Development | 2Gi    | 2   | 0             | 10            |
| Staging     | 4Gi    | 2   | 1             | 20            |
| Production  | 8Gi    | 4   | 2             | 50            |

**Manual Production Deploy:**
```bash
gh workflow run deploy-auxin-server-gcp.yml \
  -f environment=prod
```

---

## Dependency Management

**Tool:** Dependabot

**Configuration:** `.github/dependabot.yml`

**Managed Ecosystems:**
- Rust (Cargo) - CLI, Server, Config
- Swift (SPM) - LaunchAgent, App
- npm - Frontend
- Docker - Server Dockerfile
- GitHub Actions - Workflow updates

**Schedule:**
- Dependencies: Weekly on Mondays
- GitHub Actions: Monthly

**Process:**
1. Dependabot creates PRs automatically
2. CI runs on PR
3. Review and merge if tests pass
4. Max 5-10 PRs open at once

---

## Secrets Configuration

Required secrets in GitHub repository settings:

### Codecov
```
CODECOV_TOKEN - From codecov.io
```

### GCP Deployment
```
# Development/Staging
GCP_WORKLOAD_IDENTITY_PROVIDER
GCP_SERVICE_ACCOUNT
GCP_PROJECT_ID

# Production (separate)
GCP_WORKLOAD_IDENTITY_PROVIDER_PROD
GCP_SERVICE_ACCOUNT_PROD
GCP_PROJECT_ID_PROD
```

### Notifications
```
SLACK_WEBHOOK_URL - For deployment notifications
```

**Setup Workload Identity:**
```bash
# Follow GCP guide:
# https://github.com/google-github-actions/auth#setup
```

---

## Monitoring & Notifications

### Slack Notifications

Deployment notifications are sent to Slack for:
- ✅ Successful deployments (Development, Production)
- ❌ Failed deployments (with workflow link)

**Message Format:**
- Environment
- Commit SHA
- Deployed URL
- Deployed by (GitHub actor)

**Setup:**
1. Create Slack webhook: https://api.slack.com/messaging/webhooks
2. Add to repository secrets as `SLACK_WEBHOOK_URL`

### Discord Integration

To use Discord instead of Slack, modify `deploy-auxin-server-gcp.yml`:

```yaml
- uses: sarisia/actions-status-discord@v1
  with:
    webhook: ${{ secrets.DISCORD_WEBHOOK }}
    status: ${{ job.status }}
    title: "Auxin Deployment"
```

### GitHub Notifications

All workflows support:
- Email notifications (configured in GitHub profile)
- GitHub mobile push notifications
- Web notifications in GitHub UI

---

## Workflow Status Badges

Add to README.md:

```markdown
![Tests](https://github.com/jbacus/auxin/workflows/Test%20Suite/badge.svg)
![Security](https://github.com/jbacus/auxin/workflows/Security%20Scanning/badge.svg)
![Coverage](https://codecov.io/gh/jbacus/auxin/branch/main/graph/badge.svg)
```

---

## Troubleshooting

### Test Failures

**View logs:**
```bash
gh run list --workflow=test.yml
gh run view <run-id> --log
```

**Re-run failed jobs:**
```bash
gh run rerun <run-id> --failed
```

### Coverage Issues

**Swift coverage not appearing:**
- Ensure tests are running with `--enable-code-coverage`
- Check that `.profdata` file is generated
- Verify llvm-cov export command

**Rust coverage missing:**
- Install llvm-tools-preview component
- Use cargo-llvm-cov (not tarpaulin)

### Deployment Failures

**Check Cloud Run logs:**
```bash
gcloud logging read \
  "resource.type=cloud_run_revision" \
  --limit 50
```

**Rollback production:**
```bash
gcloud run services update-traffic auxin-server \
  --to-revisions=PREVIOUS_REVISION=100 \
  --region us-central1
```

---

## Best Practices

1. **Always run tests locally before pushing:**
   ```bash
   cargo test
   swift test
   npm test
   ```

2. **Use feature branches for development:**
   ```bash
   git checkout -b feature/my-feature
   ```

3. **Keep PRs small and focused**
   - One feature per PR
   - Tests included
   - Documentation updated

4. **Monitor CI execution times**
   - Workflows should complete in <15 minutes
   - Use caching aggressively
   - Parallelize where possible

5. **Review security alerts promptly**
   - Check Security tab weekly
   - Update dependencies with vulnerabilities
   - Don't ignore Dependabot PRs

---

## Contributing to CI/CD

When adding new workflows:

1. **Use existing patterns:**
   - Follow naming conventions
   - Include concurrency groups
   - Add timeout limits

2. **Add documentation:**
   - Update this guide
   - Add inline comments
   - Document required secrets

3. **Test thoroughly:**
   - Use workflow_dispatch for testing
   - Test on feature branch first
   - Verify artifacts and outputs

4. **Consider costs:**
   - macOS runners: $0.08/minute
   - Linux runners: $0.008/minute
   - Minimize macOS usage where possible

---

*Last Updated: 2025-11-22*
