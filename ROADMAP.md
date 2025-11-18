# Auxin Roadmap

**Last Updated**: 2025-11-18
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
| **Phase 6** | Network Resilience | Not Started | 0% |
| **Phase 7** | Auxin Server | In Progress | 30% |
| **Phase 8** | AI-Powered Diffing | Future | 0% |

**Current Focus**: Phase 6 (Network Resilience) and Phase 7 (Server)

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

### Phase 6: Network Resilience (0%)

**Goal**: Reliable operation on unreliable networks

**Planned Features**:
- Offline mode with commit queue
- Smart retry with exponential backoff
- Partial push recovery
- Pre-pull conflict detection
- Network connectivity monitoring
- Emergency unlock protocol

**Why Critical**: Current system assumes reliable connectivity. Push failures leave repository in inconsistent state. No retry on transient errors.

**Estimated Effort**: 2-3 weeks

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

**Current Priorities**:
1. Network resilience implementation
2. macOS integration testing
3. Server development
4. Documentation improvements

---

## Resources

- **User Guide**: [docs/FOR_MUSICIANS.md](docs/FOR_MUSICIANS.md)
- **Developer Guide**: [docs/FOR_DEVELOPERS.md](docs/FOR_DEVELOPERS.md)
- **CLI Examples**: [docs/CLI_EXAMPLES.md](docs/CLI_EXAMPLES.md)
- **Feature Status**: [FEATURE_STATUS.md](FEATURE_STATUS.md)
- **Troubleshooting**: [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

---

*This roadmap reflects the current state and future direction of Auxin. Timelines are estimates based on current understanding and may change as development progresses.*
