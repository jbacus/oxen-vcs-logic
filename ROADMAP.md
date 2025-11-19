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
| **Phase 6** | Network Resilience | In Progress | 70% |
| **Phase 7** | Auxin Server | In Progress | 30% |
| **Phase 8** | AI-Powered Diffing | Future | 0% |

**Current Focus**: Phase 6 (Network Resilience) - CRITICAL for remote teams

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
| **Offline commit queue** | Framework ready | Queue infrastructure in place, needs CLI integration |
| **Partial push recovery** | Not started | Large sessions (2GB+) fail halfway = lost time |
| **Remote lock synchronization** | ✅ Complete | Heartbeat system keeps locks alive during sessions |
| **Connection health monitoring** | ✅ Complete | Network quality check with latency measurement |

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

### Phase 6: Network Resilience (70%) 🚧 IN PROGRESS

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

#### 6.2 Offline Mode (Week 2) 🚧 MOSTLY COMPLETE
- [x] Detect network availability before operations
- [x] Queue commits locally when offline
- [x] Persist queue to disk (survive app restart)
- [ ] Auto-sync when connection restored (needs background daemon)
- [x] Status command shows pending sync count
- [x] Manual sync trigger option (auxin queue sync)

#### 6.3 Large File Handling (Week 2-3) ⏳ NOT STARTED
- [ ] Chunked uploads with resume capability
- [ ] Track upload progress per-file
- [ ] Resume from last successful chunk on retry
- [ ] Bandwidth estimation and ETA display
- [ ] Abort and resume later option

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

**Remaining Effort**: 1-2 weeks (offline queue CLI integration, large file chunking)

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
