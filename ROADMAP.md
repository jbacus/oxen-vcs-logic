# Auxin Roadmap

**Last Updated**: 2025-11-19
**Vision**: The definitive version control system for professional creative applications

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
| **Phase 5.5** | Security & Performance | Complete | 100% |
| **Phase 6** | Network Resilience | Infrastructure Complete | 40% |
| **Phase 7** | Auxin Server | In Progress | 30% |
| **Phase 8** | AI-Powered Diffing | Future | 0% |

**Current Focus**: Phase 6 (Network Resilience integration) and Phase 7 (Server)

---

## Completed Phases

### Phase 1: Core CLI & Logic Pro Support (100%)

**Delivered**: Full-featured Rust CLI wrapper with Logic Pro integration

- Logic Pro project detection and validation
- Commit metadata (BPM, sample rate, key signature)
- .oxenignore generation for Logic-specific patterns
- All VCS operations (init, add, commit, log, restore, status, diff, show)
- Oxen subprocess wrapper for reliable backend operations
- 434 unit tests, 88% code coverage
- Output caching (1s TTL) for 10-100x faster repeated queries
- Automatic file batching (1000 files/batch)
- Configurable timeouts (30s default, 120s network)

**Key Files**: `oxen_subprocess.rs`, `logic_project.rs`, `commit_metadata.rs`, `oxen_backend.rs`

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
- Audit logging for all lock operations
- Lock lifecycle management (expiration, staleness, emergency unlock)
- 17 tests passing

**Gap**: Full network resilience integration (retry logic infrastructure exists, needs offline mode)

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

### Phase 5.5: Security & Performance (100%)

**Delivered**: Critical security hardening and performance optimization

**Security Improvements**:
- Input sanitization (8 dangerous patterns, path traversal prevention)
- XPC code signature verification (SecCode API)
- Commit message validation (10k char limit, null byte detection)
- Audit logging for all sensitive operations
- Version verification (requires Oxen 0.19+)

**Performance Improvements**:
- OxenBackend trait abstraction for FFI migration
- Liboxen FFI backend (10-100x faster, feature-gated)
- Output caching with 1s TTL
- Automatic file batching (1000 files/batch)
- Improved error categorization (8 error types, retryable detection)

**Test Coverage**: 12 dedicated security tests, 434 total tests

**Key Files**: `oxen_subprocess.rs`, `oxen_backend.rs`, `remote_lock.rs`, `XPCService.swift`

---

## In Progress

### Phase 6: Network Resilience (40%)

**Goal**: Reliable operation on unreliable networks

**Completed Infrastructure**:
- RetryPolicy with configurable exponential backoff
- NetworkResilienceManager for operation queuing
- Persistent queue on disk (`~/.auxin/operation_queue.json`)
- Transient error detection (timeout, connection refused, 502/503/504)
- Network availability checking (ping hub.oxen.ai)
- OxenError.is_retryable() for error classification

**Remaining Integration**:
- Offline mode with commit queue (infrastructure ready)
- Automatic retry on push/pull failures
- Partial push recovery
- Pre-pull conflict detection

**Why Critical**: Infrastructure exists but not integrated into core workflows. Push failures now detectable but not automatically retried.

**Estimated Effort**: 1-2 weeks (integration only)

**Key File**: `network_resilience.rs`

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

| Item | Priority | Effort | Status |
|------|----------|--------|--------|
| ~~Remove liboxen_stub (deprecated fallback)~~ | ~~Low~~ | ~~1 day~~ | DONE |
| Implement write-ahead logging (WAL) | Medium | 3 days | Pending |
| Replace force-push locks with CAS | Medium | 2 days | Pending |
| Enable strict XPC validation in production | Low | 1 day | Pending |
| Fix 12 failing doctests | Low | 1 day | Pending |
| Improve XPC reconnection logic | Medium | 2 days | Pending |
| Add property-based testing | Low | 2 days | Pending |

---

## Release Plan

### v0.1 - CLI First (Ready Now)
- Full CLI functionality
- Local workflows
- Requires `oxen` CLI
- Early adopter release

### v0.2 - Network Ready (ETA: 3 weeks)
- Network resilience
- macOS integration testing
- Stable for team use

### v0.3 - Server Alpha (ETA: 8 weeks)
- Self-hosted server
- Web dashboard
- Real-time collaboration

### v1.0 - Production (ETA: 4 months)
- All phases through 7 complete
- Comprehensive testing
- Documentation polish
- Pre-built binaries
- Homebrew tap

---

## Key Metrics

### Current State (November 2025)

| Metric | Value |
|--------|-------|
| Rust code | 11,000+ lines |
| Swift code | ~5,000 lines |
| Unit tests | 434 passing |
| Integration tests | 40+ |
| Coverage | 88% |
| Supported apps | 3 (Logic Pro, SketchUp, Blender) |
| Security tests | 12 dedicated |
| Error types | 8 categorized |

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

1. **Oxen-first**: All VCS operations go through OxenBackend trait (subprocess or FFI)
2. **Binary-aware**: Never attempt algorithmic merge of binary files
3. **Pessimistic locking**: Prevent conflicts rather than resolve them
4. **Application-specific**: Metadata and ignore patterns per application
5. **Offline-capable**: Queue operations when network unavailable
6. **Power-safe**: Commit before sleep/shutdown
7. **Secure-by-default**: Input sanitization, audit logging, code signature verification
8. **Performance-optimized**: Caching, batching, FFI migration path

---

## Contributing

See [docs/developer/contributing.md](docs/developer/contributing.md) for development guidelines.

**Current Priorities**:
1. Network resilience integration (infrastructure complete)
2. macOS integration testing for Swift components
3. Server development (Phase 7)
4. Enable FFI backend by default when liboxen stabilizes

---

## Resources

### User Documentation
- **Getting Started**: [docs/user/getting-started.md](docs/user/getting-started.md)
- **For Musicians**: [docs/user/for-musicians.md](docs/user/for-musicians.md)
- **For Modelers**: [docs/user/for-modelers.md](docs/user/for-modelers.md)
- **CLI Reference**: [docs/user/cli-reference.md](docs/user/cli-reference.md)
- **Troubleshooting**: [docs/user/troubleshooting.md](docs/user/troubleshooting.md)

### Developer Documentation
- **Architecture**: [docs/developer/architecture.md](docs/developer/architecture.md)
- **Architectural Review**: [docs/developer/architectural-review.md](docs/developer/architectural-review.md)
- **Development Setup**: [docs/developer/development-setup.md](docs/developer/development-setup.md)
- **Contributing**: [docs/developer/contributing.md](docs/developer/contributing.md)
- **Testing**: [docs/developer/testing.md](docs/developer/testing.md)

### Project Status
- **Feature Status**: [FEATURE_STATUS.md](FEATURE_STATUS.md)
- **Changelog**: [CHANGELOG.md](CHANGELOG.md)

---

*This roadmap reflects the current state and future direction of Auxin. Timelines are estimates based on current understanding and may change as development progresses.*
