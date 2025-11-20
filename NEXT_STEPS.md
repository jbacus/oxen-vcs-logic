# Auxin Development - Next Steps

**Created**: 2025-11-20
**Current Phase**: Phase 7 (Auxin Server) at 60%
**Goal**: Complete Phase 7 and prepare for v0.3 Server Alpha release

---

## Executive Summary

This document outlines the concrete next steps to complete Phase 7 (Auxin Server) and move toward the v0.3 release. The plan is organized into 5 weeks of focused development.

---

## Phase 7 Completion Tasks

### Week 1-2: VCS Operations Integration (Critical Path)

**Goal**: Connect the Auxin Server to actual Oxen VCS operations

#### Task 1.1: Repository Cloning Endpoint
- [ ] Implement `POST /api/repos/{name}/clone` endpoint
- [ ] Add clone URL validation and sanitization
- [ ] Execute `oxen clone` via subprocess wrapper
- [ ] Return progress updates via WebSocket
- [ ] Handle clone failures with proper error responses
- [ ] Add integration tests

**Files to modify**:
- `auxin-server/src/api/repo_ops.rs`
- `auxin-server/src/api/mod.rs`

#### Task 1.2: Push/Pull Operations
- [ ] Implement `POST /api/repos/{name}/push` endpoint
- [ ] Implement `POST /api/repos/{name}/pull` endpoint
- [ ] Integrate with chunked upload system from Phase 6
- [ ] Add authentication checks (only repo collaborators)
- [ ] Track operation progress in activity log
- [ ] Broadcast completion via WebSocket

**Files to modify**:
- `auxin-server/src/api/repo_ops.rs`
- `auxin-server/src/extensions/activity.rs`

#### Task 1.3: Branch Management
- [ ] Implement `GET /api/repos/{name}/branches` endpoint
- [ ] Implement `POST /api/repos/{name}/branches` endpoint
- [ ] Implement `DELETE /api/repos/{name}/branches/{branch}` endpoint
- [ ] Add branch protection rules (prevent main deletion)
- [ ] Log branch operations to activity feed

#### Task 1.4: Commit History API
- [ ] Implement `GET /api/repos/{name}/commits` endpoint
- [ ] Support pagination (limit, offset)
- [ ] Include commit metadata (BPM, sample rate, key, tags)
- [ ] Support filtering by branch, author, date range

**Estimated effort**: 8-10 days

---

### Week 3: Web Dashboard Polish

**Goal**: Complete the React frontend for a usable web interface

#### Task 2.1: Project Overview Dashboard
- [ ] Repository cards with status indicators
- [ ] Last commit info and active locks display
- [ ] Team member avatars
- [ ] Quick action buttons (clone, lock, view history)

**Files to create/modify**:
- `auxin-server/frontend/src/components/RepoCard.tsx`
- `auxin-server/frontend/src/pages/Dashboard.tsx`

#### Task 2.2: Repository Detail View
- [ ] Commit history timeline
- [ ] Branch selector dropdown
- [ ] File browser (tree view)
- [ ] Lock status panel
- [ ] Activity feed sidebar

**Files to create/modify**:
- `auxin-server/frontend/src/pages/RepoDetail.tsx`
- `auxin-server/frontend/src/components/CommitHistory.tsx`
- `auxin-server/frontend/src/components/LockPanel.tsx`

#### Task 2.3: Lock Management UI
- [ ] Active locks table with owner, file, duration
- [ ] Acquire/release lock buttons
- [ ] Break lock with confirmation (admin only)
- [ ] Lock history view

#### Task 2.4: Real-time Updates
- [ ] WebSocket connection management
- [ ] Toast notifications for lock events
- [ ] Auto-refresh on push/pull completion
- [ ] Connection status indicator

**Estimated effort**: 5-7 days

---

### Week 4: Testing & Documentation

**Goal**: Ensure quality and prepare for deployment

#### Task 3.1: End-to-End Testing
- [ ] Full workflow: register → create repo → clone → commit → push
- [ ] Multi-user collaboration scenario
- [ ] Lock contention handling
- [ ] Network failure recovery
- [ ] Large file upload (1GB+)

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

#### Task 3.2: Production Deployment Documentation
- [ ] Docker Compose setup
- [ ] Environment variables reference
- [ ] Nginx reverse proxy configuration
- [ ] SSL/TLS setup with Let's Encrypt
- [ ] Systemd service file
- [ ] Backup and restore procedures
- [ ] Monitoring with Prometheus/Grafana

**Files to create**:
- `auxin-server/docs/deployment.md`
- `auxin-server/docker-compose.yml`
- `auxin-server/nginx.conf.example`

#### Task 3.3: API Documentation
- [ ] OpenAPI/Swagger specification
- [ ] Authentication flow documentation
- [ ] WebSocket message format reference
- [ ] Error codes and handling

**Estimated effort**: 5-7 days

---

### Week 5: Swift Component Testing

**Goal**: Validate macOS desktop experience

#### Task 4.1: LaunchAgent Integration Testing
- [ ] Test FSEvents monitoring with real Logic Pro projects
- [ ] Verify power management hooks (sleep/shutdown commits)
- [ ] Test XPC communication reliability
- [ ] Stress test with rapid file changes
- [ ] Memory and CPU profiling

#### Task 4.2: GUI App Testing
- [ ] Test all UI workflows end-to-end
- [ ] Verify commit dialog with metadata
- [ ] Test restore/rollback functionality
- [ ] Check lock status updates
- [ ] Test with multiple projects open

#### Task 4.3: Performance Optimization
- [ ] Profile daemon memory usage
- [ ] Optimize file change debouncing
- [ ] Reduce XPC call latency
- [ ] Cache frequently accessed data

**Estimated effort**: 5-7 days

---

## Technical Debt Items

### During Phase 7 (address as encountered)
- [ ] Fix 12 failing doctests
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

## Timeline Summary

| Week | Focus | Deliverables |
|------|-------|--------------|
| 1-2 | VCS Integration | Clone, push, pull, branch APIs working |
| 3 | Dashboard UI | Functional web interface |
| 4 | Testing & Docs | E2E tests passing, deployment docs |
| 5 | Swift Testing | Desktop app validated on macOS |

**Target completion**: Phase 7 at 100% in 5 weeks
**Target release**: v0.3 Server Alpha in ~10 weeks

---

## Next Immediate Actions

1. **Start with Task 1.1**: Repository cloning endpoint
   - This unblocks the entire VCS integration workflow
   - Provides immediate value for testing

2. **Set up test environment**:
   - Create test Oxen Hub repository
   - Prepare 1GB Logic Pro project for upload testing

3. **Review existing server code**:
   - `auxin-server/src/api/repo_ops.rs`
   - `auxin-server/src/main.rs`

---

*This plan will be updated as development progresses. Check ROADMAP.md for overall project status.*
