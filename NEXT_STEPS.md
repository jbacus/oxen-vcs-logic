# Auxin Development - Next Steps

**Created**: 2025-11-20
**Updated**: 2025-11-22
**Current Phase**: Phase 7 (Auxin Server) at 85%
**Goal**: Complete Phase 7 and prepare for v0.3 Server Alpha release

---

## Executive Summary

This document outlines the remaining steps to complete Phase 7 (Auxin Server) and move toward the v0.3 release. **Major progress update:** Most VCS operations and web dashboard features are complete. Focus areas are now documentation, production deployment, and comprehensive testing.

---

## Phase 7 Completion Tasks

### ✅ Week 1-2: VCS Operations Integration - **COMPLETE**

**Status**: All critical VCS operations are implemented and working

#### Task 1.1: Repository Cloning Endpoint - ✅ COMPLETE
- ✅ Implemented `POST /api/repos/{namespace}/{name}/clone` endpoint
- ✅ Added clone URL validation and sanitization
- ✅ Execute `oxen clone` via RepositoryOps wrapper
- ✅ Handle clone failures with proper error responses
- ✅ Integration tests exist

**Files completed**:
- `auxin-server/src/api/repo_ops.rs` (lines 546-615)
- `auxin-server/src/main.rs` (route registered)

#### Task 1.2: Push/Pull Operations - ✅ COMPLETE
- ✅ Implemented `POST /api/repos/{namespace}/{name}/push` endpoint
- ✅ Implemented `POST /api/repos/{namespace}/{name}/pull` endpoint
- ✅ Implemented `POST /api/repos/{namespace}/{name}/fetch` endpoint
- ✅ Authentication checks via AuthService
- ✅ Activity logging via log_activity()
- ✅ WebSocket broadcasting via WsHub

**Files completed**:
- `auxin-server/src/api/repo_ops.rs` (push: lines 94-145, pull: lines 147-187)
- `auxin-server/src/extensions/activity.rs`

#### Task 1.3: Branch Management - ✅ COMPLETE
- ✅ Implemented `GET /api/repos/{namespace}/{name}/branches` endpoint
- ✅ Implemented `POST /api/repos/{namespace}/{name}/branches` endpoint
- ✅ Implemented `DELETE /api/repos/{namespace}/{name}/branches/{branch}` endpoint
- ✅ Branch operations logged to activity feed

**Files completed**:
- `auxin-server/src/api/repo_ops.rs` (list: lines 189-211, create: lines 213-242, delete: lines 617-645)

#### Task 1.4: Commit History API - ✅ COMPLETE
- ✅ Implemented `GET /api/repos/{namespace}/{name}/commits` endpoint
- ✅ Supports pagination via query parameter (limit)
- ✅ Includes commit metadata support via separate endpoint
- ✅ Authentication and authorization checks

**Files completed**:
- `auxin-server/src/api/repo_ops.rs` (lines 59-81)

---

### ✅ Week 3: Web Dashboard - **COMPLETE**

**Status**: React/TypeScript frontend is functional and built

#### Task 2.1: Project Overview Dashboard - ✅ COMPLETE
- ✅ Repository cards with status indicators (RepositoryCard.tsx)
- ✅ Home page with repository listing (HomePage.tsx)
- ✅ Create repository modal (CreateRepoModal.tsx)
- ✅ Clone instructions component (CloneInstructions.tsx)

**Files completed**:
- `auxin-server/frontend/src/components/repos/RepositoryCard.tsx`
- `auxin-server/frontend/src/pages/HomePage.tsx`
- `auxin-server/frontend/src/components/repos/CreateRepoModal.tsx`

#### Task 2.2: Repository Detail View - ✅ COMPLETE
- ✅ Commit history timeline (CommitList.tsx)
- ✅ Repository detail page (RepoPage.tsx)
- ✅ Lock status panel integrated
- ✅ Activity feed sidebar (ActivityFeed.tsx)

**Files completed**:
- `auxin-server/frontend/src/pages/RepoPage.tsx`
- `auxin-server/frontend/src/components/commits/CommitList.tsx`
- `auxin-server/frontend/src/components/activity/ActivityFeed.tsx`

#### Task 2.3: Lock Management UI - ✅ COMPLETE
- ✅ Lock manager component (LockManager.tsx)
- ✅ Acquire/release lock functionality
- ✅ Lock status display

**Files completed**:
- `auxin-server/frontend/src/components/locks/LockManager.tsx`

#### Task 2.4: Real-time Updates - ✅ COMPLETE
- ✅ WebSocket connection management
- ✅ Real-time activity updates
- ✅ Authentication store (authStore.ts)
- ✅ Theme management (themeStore.ts)

**Files completed**:
- `auxin-server/frontend/src/stores/authStore.ts`
- `auxin-server/frontend/src/stores/themeStore.ts`
- Frontend built and ready: `auxin-server/frontend/dist/`

---

### 🔄 Week 4: Testing & Documentation - **IN PROGRESS**

**Goal**: Ensure quality and prepare for deployment

**Status**: Basic integration tests exist. Need E2E tests with real Oxen and production documentation.

#### Task 3.1: End-to-End Testing - ⏳ PARTIAL
- ✅ Integration tests exist (api_tests.rs, auth_integration_tests.rs, collaboration_e2e_tests.rs)
- ⏳ Need E2E tests with real Oxen operations (currently using mocks)
- [ ] Multi-user collaboration scenario with real server
- [ ] Lock contention handling end-to-end
- [ ] Network failure recovery testing
- [ ] Large file upload (1GB+) testing

**Test scenarios**:
```
Scenario 1: Basic Workflow
1. User A registers and creates repository
2. User A clones locally
3. User A makes changes and commits
4. User A pushes to server
5. Verify: commit appears in web UI

Scenario 2: Collaboration
1. User A creates repo and pushes content
2. User B clones repo
3. User A acquires lock
4. User B attempts to acquire lock (should fail)
5. User A releases lock
6. User B acquires lock (should succeed)
7. Verify: WebSocket notifications sent to both users

Scenario 3: Network Resilience
1. User starts large file push
2. Simulate network interruption
3. User resumes push
4. Verify: upload completes from last chunk
```

#### Task 3.2: Production Deployment Documentation - ⏳ PARTIAL
- ✅ Docker Compose setup exists (`auxin-server/docker-compose.yml`)
- ✅ Dockerfile exists with multi-stage build
- ✅ CONFIGURATION.md documents Docker deployment
- [ ] **NEEDED:** Comprehensive deployment guide (`docs/deployment/PRODUCTION.md`)
- [ ] **NEEDED:** Nginx reverse proxy configuration (`docs/deployment/nginx.conf.example`)
- [ ] **NEEDED:** SSL/TLS setup with Let's Encrypt guide
- [ ] **NEEDED:** Systemd service file (`docs/deployment/auxin-server.service`)
- [ ] **NEEDED:** Backup and restore procedures
- [ ] **NEEDED:** Monitoring with Prometheus/Grafana setup

**Files to create**:
- `auxin-server/docs/deployment/PRODUCTION.md` (comprehensive guide)
- `auxin-server/docs/deployment/nginx.conf.example`
- `auxin-server/docs/deployment/auxin-server.service`
- `auxin-server/docs/deployment/prometheus.yml`
- `auxin-server/docs/deployment/grafana-dashboard.json`

#### Task 3.3: API Documentation - [ ] NOT STARTED
- [ ] **NEEDED:** OpenAPI/Swagger specification (`docs/api/openapi.yaml`)
- [ ] **NEEDED:** Authentication flow documentation
- [ ] **NEEDED:** WebSocket message format reference
- [ ] **NEEDED:** Error codes and handling guide

---

### 🔄 Week 5: Swift Component Testing - **IN PROGRESS**

**Goal**: Validate macOS desktop experience and increase test coverage

**Current Coverage**: LaunchAgent ~30%, App <10%

#### Task 4.1: LaunchAgent Integration Testing - ⏳ PARTIAL
- ✅ Basic tests exist for LockManager
- [ ] **NEEDED:** FSEventsMonitor tests (426 lines, 0% coverage)
- [ ] **NEEDED:** PowerManagement tests (319 lines, 0% coverage)
- [ ] **NEEDED:** NetworkMonitor tests (276 lines, 0% coverage)
- [ ] **NEEDED:** Daemon orchestration tests (426 lines, 0% coverage)
- [ ] Test XPC communication reliability
- [ ] Stress test with rapid file changes
- [ ] Memory and CPU profiling

#### Task 4.2: GUI App Testing - ⏳ PARTIAL
- ✅ Basic Project model tests exist
- ✅ ProjectListViewModel tests exist (5 passing)
- [ ] **NEEDED:** SwiftUI view tests (ProjectDetailContentView.swift - 306 lines)
- [ ] **NEEDED:** Additional ViewModel tests
- [ ] **NEEDED:** Service layer tests
- [ ] Test all UI workflows end-to-end
- [ ] Verify commit dialog with metadata
- [ ] Test restore/rollback functionality

#### Task 4.3: Performance Optimization - [ ] NOT STARTED
- [ ] Profile daemon memory usage
- [ ] Optimize file change debouncing
- [ ] Reduce XPC call latency
- [ ] Cache frequently accessed data

---

## Technical Debt Items

### During Phase 7 (address as encountered)
- [x] ✅ Fix failing doctests in auxin-oxen (all 7 doctests now passing)
- [ ] Improve error messages in API responses
- [ ] Add request validation middleware

### Post-Phase 7
- [ ] Remove deprecated `liboxen_stub/` code
- [ ] Improve XPC reconnection logic
- [ ] Add property-based testing for parsers

---

## Success Criteria for v0.3

### Must Have
- [ ] User can register, login, and manage repositories via web UI
- [ ] User can clone, push, and pull via CLI connected to Auxin Server
- [ ] Locks are enforced server-side and visible in web UI
- [ ] Activity feed shows all operations in real-time
- [ ] Deployment documentation allows self-hosting

### Should Have
- [ ] File browser in web UI
- [ ] Commit metadata displayed in history
- [ ] Email notifications for lock events

### Nice to Have
- [ ] Dark mode in web UI
- [ ] Mobile-responsive design
- [ ] Slack/Discord webhook integration

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Oxen subprocess integration issues | Test early with real repositories |
| WebSocket scalability | Load test with 50+ concurrent users |
| Large file handling | Verify chunked upload resume works end-to-end |
| Swift component bugs | Allocate dedicated testing time on macOS hardware |

---

## Resource Requirements

### Development Environment
- macOS for Swift component testing
- Linux/Docker for server development
- Oxen CLI installed (`pip install oxenai`)
- Node.js 18+ for frontend development

### Testing Infrastructure
- Test Oxen Hub account
- Large Logic Pro project (2-5GB)
- Multiple test user accounts

---

## Revised Timeline Summary (2025-11-22)

| Original Week | Focus | Status | Remaining Work |
|---------------|-------|--------|----------------|
| ✅ Weeks 1-2 | VCS Integration | **COMPLETE** | None |
| ✅ Week 3 | Dashboard UI | **COMPLETE** | None |
| ✅ Week 4 | Testing & Docs | **95% COMPLETE** | E2E test implementation |
| 🔄 Week 5 | Swift Testing | **20% COMPLETE** | Comprehensive unit/integration tests |

**Updated completion estimate**: Phase 7 at 100% in 1-2 weeks
**Target release**: v0.3 Server Alpha in 3-4 weeks

**Recent Progress (2025-11-22)**:
- ✅ Created auxin-oxen shared crate (unblocks E2E tests)
- ✅ Updated E2E test spec (removed async blocker)
- ✅ Fixed all doctests in auxin-oxen (7 passing)
- ✅ Deployment docs complete (PRODUCTION.md, nginx, systemd, prometheus)
- ✅ API docs complete (OpenAPI 3.0 spec, 30+ endpoints)

---

## Remaining High-Priority Tasks

### 1. Production Deployment Documentation - ✅ COMPLETE
- [x] Comprehensive deployment guide (`auxin-server/docs/deployment/PRODUCTION.md`)
- [x] Nginx reverse proxy configuration (`nginx.conf.example`)
- [x] SSL/TLS setup with Let's Encrypt (documented in PRODUCTION.md)
- [x] Systemd service file (`auxin-server.service`)
- [x] Backup and restore procedures (`backup-auxin.sh`)
- [x] Monitoring with Prometheus/Grafana (`prometheus.yml`)

### 2. API Documentation - ✅ COMPLETE
- [x] OpenAPI/Swagger specification (`auxin-server/docs/api/openapi.yaml`, 30+ endpoints)
- [x] Authentication flow documentation (included in OpenAPI spec)
- [x] WebSocket message format reference (documented in README.md)
- [x] Error codes and handling guide (documented in OpenAPI spec)

### 3. End-to-End Testing - 🔄 IN PROGRESS
- [x] ✅ E2E test spec updated and unblocked (auxin-oxen refactoring complete)
- [x] ✅ Improved repo_full.rs to use OxenSubprocess API
- [ ] ⏳ Complete E2E test implementation (needs missing RepositoryOps methods)
- [ ] Multi-user collaboration scenarios
- [ ] Lock contention handling
- [ ] Large file upload testing

**Note**: E2E tests are partially implemented but need additional methods in auxin-oxen (status, fetch, delete_branch) to complete.

### 4. Swift Component Testing (Est: 3-5 days)
- [ ] FSEventsMonitor unit tests
- [ ] PowerManagement unit tests
- [ ] NetworkMonitor unit tests
- [ ] SwiftUI view tests
- [ ] Additional ViewModel tests

---

*This plan will be updated as development progresses. Check ROADMAP.md for overall project status.*
