# Auxin Roadmap

**Last Updated**: 2025-11-19
**Vision**: The definitive version control system for professional creative applications
**Priority**: Remote team collaboration (distributed teams, unreliable networks)

---

## Mission

Auxin solves the fundamental incompatibility between traditional VCS (like Git) and professional creative workflows. Creative projects have large binary files, proprietary formats, and non-mergeable content that cause Git to fail catastrophically with repository bloat and unresolvable conflicts.

**Target Users**: Audio engineers, music producers, 3D modelers, architects, and collaborative production teams managing multi-GB creative projects.

---

## Progress Overview

| Phase | Description | Status | Completion |
|-------|-------------|--------|------------|
| **Phase 1** | Core CLI & Logic Pro | Complete | 100% |
| **Phase 2** | Background Services | Code Complete | 100% |
| **Phase 3** | GUI Application | Code Complete | 100% |
| **Phase 4** | Team Collaboration | Complete | 95% |
| **Phase 5** | 3D Modeling Support | Complete | 100% |
| **Phase 6** | Network Resilience | Not Started | 0% |
| **Phase 7** | Auxin Server | In Progress | 30% |
| **Phase 8** | AI-Powered Diffing | Future | 0% |

**Current Focus**: Phase 6 (Network Resilience) - CRITICAL for remote teams

---

## End-to-End Remote Collaboration Checklist

**Use Case**: Distributed teams (e.g., Colorado ↔ London) collaborating on the same project

### What's Working Now
- [x] Core VCS operations (init, add, commit, log, restore)
- [x] Logic Pro project support with metadata
- [x] Pessimistic locking system (acquire, release, break)
- [x] Team discovery and activity feeds
- [x] Authentication with Oxen Hub
- [x] Local file monitoring and auto-commits

### What's Blocking Remote Collaboration

| Feature | Status | Why Critical |
|---------|--------|--------------|
| **Network retry logic** | Partial | Push failures over long distances leave repo in bad state |
| **Offline commit queue** | Not started | Can't work when network is temporarily down |
| **Partial push recovery** | Not started | Large sessions (2GB+) fail halfway = lost time |
| **Remote lock synchronization** | Basic | Race conditions possible with high latency |
| **Connection health monitoring** | Not started | No warning before operations fail |

### Minimum Viable Remote Collaboration (v0.2)

To get a working end-to-end solution for distributed teams:

1. **Smart Retry System** (1 week)
   - Exponential backoff for transient failures
   - Distinguish network errors from auth/permission errors
   - Resume interrupted uploads from last successful chunk

2. **Offline Mode** (1 week)
   - Queue commits locally when offline
   - Sync automatically when connection restored
   - Clear status indicators ("3 commits pending sync")

3. **Lock Robustness** (3-5 days)
   - Heartbeat system for active locks
   - Graceful handling of lock holder going offline
   - Configurable timeouts for different time zones

4. **Connection Monitoring** (2-3 days)
   - Pre-flight check before push/pull
   - Estimated transfer time for large files
   - Warning when connection is unstable

**Total Estimated Effort**: 3-4 weeks

---

## Completed Phases

### Phase 1: Core CLI & Logic Pro Support (100%)

**Delivered**: Full-featured Rust CLI wrapper with Logic Pro integration

- Logic Pro project detection and validation
- Commit metadata (BPM, sample rate, key signature)
- .oxenignore generation for Logic-specific patterns
- All VCS operations (init, add, commit, log, restore, status, diff, show)
- Oxen subprocess wrapper for reliable backend operations
- 331 unit tests, 88% code coverage

**Key Files**: `oxen_subprocess.rs`, `logic_project.rs`, `commit_metadata.rs`

---

### Phase 2: Background Services (100%)

**Delivered**: Swift LaunchAgent for automatic version control

- FSEvents monitoring with 30-60s debounce
- Power management hooks (sleep/shutdown safety commits)
- Draft commit automation
- XPC communication with GUI app
- Lock enforcement

**Status**: Code complete, needs macOS integration testing

**Key Files**: `Daemon.swift`, `FSEventsMonitor.swift`, `PowerManagement.swift`

---

### Phase 3: GUI Application (100%)

**Delivered**: Native macOS SwiftUI application

- NavigationSplitView with project sidebar
- Commit history browser
- Status bar showing daemon state
- Toolbar with project management
- Menu bar integration

**Status**: Code complete, needs integration testing

**Key Files**: `ContentView.swift`, `ProjectDetailContentView.swift`

---

### Phase 4: Team Collaboration (95%)

**Delivered**: Distributed locking and team coordination

- Authentication with Oxen Hub (login, logout, status, test)
- Pessimistic file locking (acquire, release, status, break)
- Activity feeds with timeline
- Team discovery from commit history
- Comments on commits
- 17 tests passing

**Gap**: Network resilience (no retry logic, no offline mode)

**Key Files**: `remote_lock.rs`, `collaboration.rs`, `auth.rs`

---

### Phase 5: 3D Modeling Support (100%)

**Delivered**: Support for SketchUp and Blender projects

**SketchUp (.skp)**:
- Project detection and validation
- Metadata: units, layers, components, groups
- Optimized .oxenignore patterns

**Blender (.blend)**:
- Project detection and validation
- Scene metadata support
- Optimized .oxenignore patterns

**Key Files**: `sketchup_project.rs`, `sketchup_metadata.rs`, `blender_project.rs`, `blender_metadata.rs`

---

## In Progress

### Phase 6: Network Resilience (0%) ⚠️ HIGHEST PRIORITY

**Goal**: Reliable operation on unreliable networks - REQUIRED for remote collaboration

**Why This Is Priority #1**:
- Remote teams (different cities/countries) cannot reliably use Auxin today
- Long-distance connections have higher latency and more transient failures
- Large audio files (1-5GB) are prone to upload interruption
- Without this, Auxin is limited to local/single-user workflows

**Implementation Phases**:

#### 6.1 Smart Retry System (Week 1)
- [ ] Classify errors: retryable (network) vs fatal (auth, permissions)
- [ ] Exponential backoff: 2s → 4s → 8s → 16s → fail
- [ ] Maximum retry attempts (configurable, default 4)
- [ ] Progress preservation between retries
- [ ] Clear error messages with suggested actions

#### 6.2 Offline Mode (Week 2)
- [ ] Detect network availability before operations
- [ ] Queue commits locally when offline
- [ ] Persist queue to disk (survive app restart)
- [ ] Auto-sync when connection restored
- [ ] Status command shows pending sync count
- [ ] Manual sync trigger option

#### 6.3 Large File Handling (Week 2-3)
- [ ] Chunked uploads with resume capability
- [ ] Track upload progress per-file
- [ ] Resume from last successful chunk on retry
- [ ] Bandwidth estimation and ETA display
- [ ] Abort and resume later option

#### 6.4 Lock Resilience (Week 3)
- [ ] Heartbeat system (ping every 60s while locked)
- [ ] Auto-release on missed heartbeats (configurable timeout)
- [ ] Graceful handling when lock holder goes offline
- [ ] Time zone aware timeout configuration
- [ ] Lock status includes "last seen" timestamp

#### 6.5 Connection Monitoring (Week 3-4)
- [ ] Pre-flight connectivity check
- [ ] Estimated transfer time for large pushes
- [ ] Warning when connection is unstable
- [ ] Bandwidth test option
- [ ] Network quality indicator in status

**Files to Modify**:
- `oxen_subprocess.rs` - Add retry logic and error classification
- `remote_lock.rs` - Add heartbeat system
- New: `offline_queue.rs` - Offline commit queue
- New: `network_monitor.rs` - Connection health checks

**Estimated Effort**: 3-4 weeks (prioritized over Phase 7)

---

### Phase 7: Auxin Server (30%)

**Goal**: Self-hosted collaboration server with web interface

**Architecture**: Rust backend (Axum) + React/TypeScript frontend

**Completed**:
- Project structure and build system
- Basic repository management API
- Initial frontend scaffolding
- Local development scripts

**Remaining**:
- Lock management server
- Activity aggregation
- User authentication
- Web dashboard
- Real-time notifications (WebSocket)

**Estimated Effort**: 6-8 weeks

**Key Directory**: `auxin-server/`

---

## Future Phases

### Phase 8: AI-Powered Semantic Diffing

**Goal**: Intelligent change summaries for binary files

**Planned Features**:
- Audio feature extraction (librosa, CLAP embeddings)
- FCP XML structural diffing
- Natural language change summaries
- Timeline visualization of project evolution
- Search across commit history by semantic content

**Note**: AI diffing enhances understanding but does NOT enable automatic merging of binary files.

**Dependencies**: Phase 7 (Server) for compute and storage

---

### Phase 9: Cross-DAW Expansion

**Goal**: Support additional creative applications

**Planned Support**:
- Ableton Live
- Pro Tools
- Cubase
- Premiere Pro
- After Effects

**Approach**: Application-specific detection, metadata, and .oxenignore patterns following the Logic Pro / SketchUp / Blender model.

---

### Phase 10: Enterprise Features

**Goal**: Features for professional studios and teams

**Planned Features**:
- LDAP/SSO authentication
- Audit logging
- Role-based access control
- Compliance reporting
- SLA guarantees

---

## Technical Debt

| Item | Priority | Effort |
|------|----------|--------|
| Remove liboxen_stub (deprecated fallback) | Low | 1 day |
| Fix 12 failing doctests | Low | 1 day |
| Improve XPC reconnection logic | Medium | 2 days |
| Add property-based testing | Low | 2 days |

---

## Release Plan

### v0.1 - CLI First (Ready Now)
- Full CLI functionality
- Local workflows only
- Requires `oxen` CLI
- Single user or same-location teams

### v0.2 - Remote Collaboration Ready (ETA: 4 weeks) ⭐ NEXT MILESTONE
**Target**: Distributed teams can reliably collaborate across any distance

**Week 1-2 Deliverables**:
- Smart retry with exponential backoff
- Error classification (retryable vs fatal)
- Offline commit queue with auto-sync

**Week 3-4 Deliverables**:
- Chunked uploads with resume
- Lock heartbeat system
- Connection health monitoring
- macOS integration testing

**Success Criteria**:
- Push 2GB session from Colorado to Oxen Hub with simulated packet loss → succeeds with retries
- Work offline for 1 hour, reconnect → all commits sync automatically
- Lock held for 6 hours across time zones → no false expiration
- Two users in different countries can complete full workflow without errors

### v0.3 - Server Alpha (ETA: 10 weeks)
- Self-hosted collaboration server
- Web dashboard for project overview
- Real-time activity notifications
- Centralized lock management

### v1.0 - Production (ETA: 4 months)
- All phases through 7 complete
- Comprehensive testing
- Documentation polish
- Pre-built binaries
- Homebrew tap
- Enterprise support options

---

## Key Metrics

### Current State (November 2025)

| Metric | Value |
|--------|-------|
| Rust code | 10,498 lines |
| Swift code | ~5,000 lines |
| Unit tests | 348 passing |
| Integration tests | 40+ |
| Coverage | 88% |
| Supported apps | 3 (Logic Pro, SketchUp, Blender) |

### Target State (v1.0)

| Metric | Target |
|--------|--------|
| Rust code | ~15,000 lines |
| Swift code | ~6,000 lines |
| Unit tests | 500+ |
| Integration tests | 100+ |
| Coverage | 90% |
| Supported apps | 5+ |

---

## Architecture Principles

1. **Oxen-first**: All VCS operations go through Oxen subprocess wrapper
2. **Binary-aware**: Never attempt algorithmic merge of binary files
3. **Pessimistic locking**: Prevent conflicts rather than resolve them
4. **Application-specific**: Metadata and ignore patterns per application
5. **Offline-capable**: Queue operations when network unavailable
6. **Power-safe**: Commit before sleep/shutdown

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

**Current Priorities** (in order):
1. **Network resilience implementation** - BLOCKING remote collaboration
2. Offline commit queue
3. Lock heartbeat system
4. macOS integration testing
5. Documentation for distributed teams

---

## Resources

- **User Guide**: [docs/FOR_MUSICIANS.md](docs/FOR_MUSICIANS.md)
- **Developer Guide**: [docs/FOR_DEVELOPERS.md](docs/FOR_DEVELOPERS.md)
- **CLI Examples**: [docs/CLI_EXAMPLES.md](docs/CLI_EXAMPLES.md)
- **Feature Status**: [FEATURE_STATUS.md](FEATURE_STATUS.md)
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

---

*This roadmap reflects the current state and future direction of Auxin. Timelines are estimates based on current understanding and may change as development progresses.*
