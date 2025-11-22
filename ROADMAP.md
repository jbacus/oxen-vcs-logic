# Auxin Roadmap

**Last Updated**: 2025-11-22
**Vision**: The definitive version control system for professional creative applications
**Priority**: Production-ready v0.3 with proven remote collaboration
**Status**: Phases 1-6 Complete, Phase 7 at 98%

---

## Strategic Vision

### What is Auxin?

**Auxin is professional version control for creative applications**, combining:
- ✅ **Git's power** - Full version history, branching, tagging, restore
- ✅ **Splice's creativity focus** - Application metadata (BPM, key, layers), auto-commit
- ✅ **Perforce's conflict prevention** - Pessimistic locking, no binary merges
- ✅ **Local-first architecture** - Works offline, you own your data
- ✅ **Open source** - MIT license, community-driven, transparent development

### Who is Auxin For?

**Primary Audience**: Solo creators and small creative teams (2-10 people)
- Music producers working with Logic Pro (1-10GB projects)
- 3D modelers using SketchUp for architecture (5-50GB projects)
- Artists creating with Blender for animation

**Secondary Audience**: Professional production studios
- Teams needing collaboration without merge conflicts
- Studios wanting self-hosted version control (data ownership)
- Organizations tired of $900+/user/year enterprise VCS pricing

### Strategic Positioning

**Auxin is NOT**:
- ❌ A Splice clone (we're open source, local-first, no subscription)
- ❌ Git with plugins (built from ground-up for large binaries)
- ❌ Enterprise-only (complex setup, expensive licensing)
- ❌ Cloud-only SaaS (works completely offline)

**Auxin IS**:
- ✅ The **open-source, local-first alternative to Splice** for cost-conscious creators
- ✅ **Git for large binaries, done right** - no bloat, no conflicts, application-aware
- ✅ **Perforce for indie creators** - same workflow, without enterprise complexity
- ✅ **Professional tool at indie price** (free, self-hosted, optional managed hosting)

See [COMPETITIVE_POSITIONING.md](COMPETITIVE_POSITIONING.md) for detailed market analysis.

### Business Model

**Free Forever** (MIT License):
- Core CLI, LaunchAgent daemon, GUI app, self-hosted Auxin Server
- All features for solo creators and teams
- No vendor lock-in, you own your data

**Future Optional Services** (Sustainability):
- Auxin Cloud - Managed hosting (convenience, not capability)
- Enterprise support - SLA-backed contracts for studios
- Training & consulting - Professional onboarding services

**Commitment**: Core software stays free. We make money by making Auxin so good that users *want* to support it, not because they're forced to.

See [BUSINESS_MODEL.md](BUSINESS_MODEL.md) for complete sustainability strategy.

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
| **Phase 6** | Network Resilience | Complete | 100% |
| **Phase 7** | Auxin Server | In Progress | 98% |
| **Phase 8** | AI-Powered Diffing | Future | 0% |

**Current Focus**: Phase 7 (Auxin Server) - Self-hosted collaboration

---

## End-to-End Remote Collaboration Checklist

**Use Case**: Distributed teams (e.g., Colorado ↔ London) collaborating on the same project

### Real-World Requirements: Pete & Louis

**Who**: Two Berklee College of Music students starting a music production company
- Pete: Based in Colorado
- Louis: Based in London
- Distance: ~4,500 miles / 7,200 km

**What They Need**:
- Manage Logic Pro sessions (1-5GB typical, up to 10GB for large projects)
- Coordinate who's working on what (no simultaneous edits)
- Track versions: "mix v1", "vocals done", "client revision 3"
- Work across 7-hour time zone difference (MST ↔ GMT)
- Handle unreliable transatlantic internet connections

**Their Workflow**:
```
Pete (Colorado, morning):
  → Acquires lock
  → Records guitar tracks
  → Commits: "Guitar tracking complete - 12 takes"
  → Pushes to Oxen Hub
  → Releases lock

Louis (London, evening):
  → Sees Pete released lock
  → Pulls latest changes
  → Acquires lock
  → Adds synth layers
  → Commits: "Synth pads and bass - A minor"
  → Pushes to Oxen Hub
  → Releases lock
```

**Critical Requirements**:
1. **Reliable sync over long distance** - Can't lose work to failed uploads
2. **Clear lock status** - Must know if partner is working before starting
3. **Offline resilience** - Internet drops shouldn't block local work
4. **Time zone awareness** - Lock timeouts must account for different schedules
5. **Large file handling** - 2-5GB sessions need chunked upload with resume

---

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
| **Network retry logic** | ✅ Complete | Adaptive retry with circuit breaker, exponential backoff |
| **Offline commit queue** | ✅ Complete | Queue with auto-sync on network reconnect |
| **Partial push recovery** | ✅ Complete | Chunked uploads with resume capability |
| **Remote lock synchronization** | ✅ Complete | Heartbeat system keeps locks alive during sessions |
| **Connection health monitoring** | ✅ Complete | Network quality check with latency measurement |

### Minimum Viable Remote Collaboration (v0.2) ✅ COMPLETE

All features delivered for distributed team collaboration:

1. **Smart Retry System** ✅
   - Exponential backoff for transient failures
   - Distinguish network errors from auth/permission errors
   - Resume interrupted uploads from last successful chunk

2. **Offline Mode** ✅
   - Queue commits locally when offline
   - Sync automatically when connection restored
   - Clear status indicators ("3 commits pending sync")

3. **Lock Robustness** ✅
   - Heartbeat system for active locks
   - Graceful handling of lock holder going offline
   - Configurable timeouts for different time zones

4. **Connection Monitoring** ✅
   - Pre-flight check before push/pull
   - Estimated transfer time for large files
   - Warning when connection is unstable

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

**Note**: Network resilience gap filled by Phase 6

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

### Phase 6: Network Resilience (100%) ✅ COMPLETE

**Goal**: Reliable operation on unreliable networks - REQUIRED for remote collaboration

**Why This Is Priority #1**:
- Remote teams (different cities/countries) cannot reliably use Auxin today
- Long-distance connections have higher latency and more transient failures
- Large audio files (1-5GB) are prone to upload interruption
- Without this, Auxin is limited to local/single-user workflows

**Implementation Phases**:

#### 6.1 Smart Retry System (Week 1) ✅ COMPLETE
- [x] Classify errors: retryable (network) vs fatal (auth, permissions)
- [x] Exponential backoff: 2s → 4s → 8s → 16s → fail
- [x] Maximum retry attempts (configurable, default 4)
- [x] Progress preservation between retries
- [x] Clear error messages with suggested actions
- [x] Circuit breaker pattern for cascading failure prevention
- [x] Adaptive retry policy (linear for rate limits, exponential for network)

#### 6.2 Offline Mode (Week 2) ✅ COMPLETE
- [x] Detect network availability before operations
- [x] Queue commits locally when offline
- [x] Persist queue to disk (survive app restart)
- [x] Auto-sync when connection restored (NetworkMonitor in daemon)
- [x] Status command shows pending sync count
- [x] Manual sync trigger option (auxin queue sync)

#### 6.3 Large File Handling (Week 2-3) ✅ COMPLETE
- [x] Chunked uploads with resume capability (ChunkedUploadManager)
- [x] Track upload progress per-file (FileUploadState)
- [x] Resume from last successful chunk on retry (session persistence)
- [x] Bandwidth estimation and ETA display (moving average)
- [x] Abort and resume later option (abort/clear_session methods)
- [x] CLI integration with push command (auxin push with progress tracking)

#### 6.4 Lock Resilience (Week 3) ✅ COMPLETE
- [x] Heartbeat system (configurable interval, default 10 min)
- [x] Auto-release on missed heartbeats (configurable timeout)
- [x] Graceful handling when lock holder goes offline
- [x] Lock health status reporting
- [x] Warning when lock expiring soon
- [x] Lock status includes "last seen" timestamp

#### 6.5 Connection Monitoring (Week 3-4) ✅ COMPLETE
- [x] Pre-flight connectivity check
- [x] Estimated transfer time for large pushes
- [x] Network quality rating (Excellent/Good/Fair/Poor/Offline)
- [x] Latency measurement to hub.oxen.ai
- [ ] Bandwidth test option (future enhancement)

**Files Modified**:
- `oxen_subprocess.rs` - Enhanced error classification (RateLimited, ServerError, DnsError, SslError, Conflict)
- `network_resilience.rs` - Circuit breaker, adaptive retry, network health monitor
- `remote_lock.rs` - Heartbeat system, lock health status
- `chunked_upload.rs` - Chunked upload manager with progress tracking and resume (879 lines)
- `NetworkMonitor.swift` - NWPathMonitor for connectivity detection and auto-sync

---

### Phase 7: Auxin Server (98%) ⭐

**Goal**: Self-hosted collaboration server with web interface

**Status**: **v0.3 API validation complete - Core functionality ready** ✅

**Architecture**: Rust backend (Actix Web) + React/TypeScript frontend

**Completed**:
- Project structure and build system ✅
- Basic repository management API ✅
- Frontend scaffolding (loads at http://localhost:3000) ✅
- Local development scripts ✅
- Lock management server ✅
- User authentication with bcrypt password hashing ✅
- Activity logging and aggregation ✅
- Real-time notifications (WebSocket) ✅
- **End-to-end tests with real Oxen (5 passing)** ✅
- **HTTP 409 Conflict responses for lock conflicts** ✅
- **API validation complete (2025-11-22)** ✅
- **Production deployment documentation** ✅
- **OpenAPI 3.0 specification (30+ endpoints)** ✅

**Remaining** (2%):
- Web UI polish and additional features
- CLI→Server integration testing

**Estimated Effort**: 1-2 weeks (UI enhancements)

**Key Directory**: `auxin-server/`

**Key Files Added**:
- `src/auth.rs` - User registration, login, token management (567 lines)
- `src/extensions/activity.rs` - Activity logging system (262 lines)
- `src/websocket.rs` - Real-time WebSocket notifications (282 lines)
- `tests/e2e_real_oxen.rs` - **5 passing E2E tests with real Oxen** ⭐
- `docs/deployment/PRODUCTION.md` - **Comprehensive deployment guide** ⭐
- `docs/api/openapi.yaml` - **OpenAPI 3.0 spec (30+ endpoints)** ⭐
- `V0.3_VALIDATION.md` - **API validation results** ⭐

**Test Coverage**: 431 tests passing (426 Rust CLI + 5 E2E)
**API Validation**: User auth, repos, locks, activity feed - all working

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

### v0.2 - Remote Collaboration Ready ✅ COMPLETE
**Target**: Distributed teams can reliably collaborate across any distance

**Delivered**:
- Smart retry with exponential backoff
- Error classification (retryable vs fatal)
- Offline commit queue with auto-sync
- Chunked uploads with resume
- Lock heartbeat system
- Connection health monitoring

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
1. **Phase 7: Auxin Server** - Web dashboard polish, VCS integration
2. macOS integration testing (Swift components)
3. End-to-end testing for remote collaboration
4. Documentation for distributed teams
5. Technical debt cleanup

---

## Resources

- **User Guide**: [docs/user/for-musicians.md](docs/user/for-musicians.md)
- **Developer Guide**: [docs/developer/architecture.md](docs/developer/architecture.md)
- **CLI Reference**: [docs/user/cli-reference.md](docs/user/cli-reference.md)
- **Feature Status**: [FEATURE_STATUS.md](FEATURE_STATUS.md)
- **Troubleshooting**: [docs/user/troubleshooting.md](docs/user/troubleshooting.md)

---

*This roadmap reflects the current state and future direction of Auxin. Timelines are estimates based on current understanding and may change as development progresses.*
