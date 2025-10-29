# Changelog

All notable changes to the Oxen-VCS for Logic Pro project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Documentation consolidation and cleanup (2025-10-29)
  - Removed 12 outdated files: development plans (ACTION_PLAN_REVISED.md, WORK_PLAN_2025-10-29.md), build issues (BUILD_ISSUES.md), test evaluation (TEST_STRATEGY_EVALUATION.md), phase completion reports (PHASE1_COMPLETE.md, PHASE2_COMPLETE.md, PHASE3_COMPLETE.md), and PR descriptions
  - Consolidated docs/QUICKSTART.md into docs/USER_GUIDE.md with new "Quick Start (5 Minutes)" section
  - Updated all references to deleted documentation files
  - Streamlined from 42 markdown files to 23 essential documents
  - Updated IMPLEMENTATION_PLAN.md with clear "ALL PHASES COMPLETE" status
  - Refreshed all "Last Updated" dates to 2025-10-29

### Added
- Comprehensive documentation update (2025-10-27)
  - Completely rewrote OxVCS-CLI-Wrapper/README.md (62→435 lines) with architecture, features, usage, testing, and troubleshooting
  - Completely rewrote OxVCS-LaunchAgent/README.md (49→558 lines) with daemon details, XPC API, configuration, and development guide
  - Completely rewrote OxVCS-App/README.md (47→637 lines) with UI features, MVVM architecture, and complete user guide
  - Updated main README.md to reflect all three phases complete
  - Updated CONTRIBUTING.md to reflect production-ready status

## [3.0.0] - 2025-10-25

### Added - Phase 3: UI Application & Collaboration
- Native macOS AppKit UI application (OxVCS-App)
- Repository browser with real-time project status
- Project initialization wizard
- Milestone commit interface with rich metadata (BPM, sample rate, key, time signature, tags)
- Commit history browser with rollback capability
- Exclusive file locking system for team collaboration
  - Lock acquisition/release with configurable timeout
  - Lock status indicators in UI
  - Force-break mechanism for admin override
- Manual merge protocol documentation
- Merge helper window with step-by-step FCP XML workflow
- Settings panel for daemon configuration
- Lock management view
- Pre-flight cleanup for milestone commits (remove Bounces/, Freeze Files/)
- XPC client library for daemon communication

### Documentation
- Phase 3 Completion Report (PHASE3_COMPLETE.md)
- Phase 3 Quick Reference (PHASE3_QUICK_REFERENCE.md)
- Merge Protocol Guide (docs/MERGE_PROTOCOL.md)
- Updated README with Phase 3 status

### Testing
- LockManager unit tests (90%+ coverage)
- ViewModel unit tests with mock XPC client
- Manual testing checklist

## [2.0.0] - 2025-10-25

### Added - Phase 2: Service Architecture & Resilience
- LaunchAgent integration with SMAppService
  - Automatic daemon startup on user login
  - KeepAlive for crash recovery
  - Resource limits (512MB RAM, 50% CPU)
- Power management integration
  - Emergency commits before sleep/shutdown
  - Battery awareness (<5% skips commits)
  - System load detection
  - IOKit power assertions
- Auto-commit workflow
  - Draft branch tracking
  - 30-second debounce threshold
  - Automatic staging and commit
  - Multi-project monitoring support
- XPC communication service
  - Mach service (com.oxen.logic.daemon.xpc)
  - Protocol for UI integration
  - Async operations support
- Draft branch management
  - Automatic draft branch creation on init
  - Configurable pruning threshold (max commits)
  - Statistics tracking
- Daemon (OxenDaemon)
  - Main coordinator for all background operations
  - FSEventsMonitor integration
  - CommitOrchestrator for auto-commits
  - ServiceManager for LaunchAgent registration

### Documentation
- Phase 2 Completion Report (docs/PHASE2_COMPLETE.md)
- Phase 2 Installation Guide (docs/PHASE2_INSTALLATION.md)
- Updated README with Phase 2 status

## [1.0.0] - 2025-10-24

### Added - Phase 1: Core Data Management (MVP)
- Logic Pro project detection and validation
  - .logicx folder structure verification
  - projectData file detection
  - Required directories check (Alternatives/, Resources/)
- .oxenignore template generation
  - Automatic creation with Logic Pro-specific patterns
  - Filters for temporary files (Bounces/, Freeze Files/, Autosave/)
  - System file exclusions (.DS_Store, etc.)
- Oxen initialization wrapper
  - Repository setup
  - Template installation
- Core VCS operations (oxenvcs-cli)
  - `init --logic` - Initialize Logic Pro project
  - `add` - Stage changes
  - `commit` - Create commits with metadata
  - `log` - View commit history
  - `status` - Check working tree state
  - `restore` - Rollback to previous commits
  - `branch` - Manage branches
- Structured commit metadata
  - BPM (tempo)
  - Sample rate (Hz)
  - Key signature
  - Time signature
  - Custom tags (comma-separated)
- FSEvents monitoring proof of concept
  - Real-time file change detection
  - Debounce logic (30-second threshold)
- Rust CLI wrapper (OxVCS-CLI-Wrapper)
  - Direct liboxen integration
  - Logic project detection (logic_project.rs)
  - Oxen operations wrapper (oxen_ops.rs)
  - Commit metadata handling (commit_metadata.rs)
  - Draft manager module (draft_manager.rs)

### Documentation
- Phase 1 Completion Report (docs/PHASE1_COMPLETE.md)
- Quick Start Guide (docs/QUICKSTART.md)
- Usage Guide (OxVCS-CLI-Wrapper/USAGE.md)
- Implementation Plan (docs/IMPLEMENTATION_PLAN.md)
- Contributing Guidelines (CONTRIBUTING.md)
- Testing Strategy (docs/TESTING_STRATEGY.md)
- Test Implementation Plan (docs/TEST_IMPLEMENTATION_PLAN.md)

### Testing
- Unit tests for Logic Pro project detection
- Integration tests for Oxen operations
- Manual testing with real Logic Pro projects

## Project Milestones

### Phase 1 (MVP) - COMPLETE ✅
- **Goal**: Prove versioning model works with Logic Pro
- **Duration**: Initial development phase
- **Lines of Code**: ~2,000 (Rust: 1,500, Swift: 500)

### Phase 2 (Service Architecture) - COMPLETE ✅
- **Goal**: Build production-grade background service
- **Duration**: Second development phase
- **Lines of Code**: ~1,600 (Swift: 1,200, Rust: 400)

### Phase 3 (UI & Collaboration) - COMPLETE ✅
- **Goal**: Complete user-facing application and team features
- **Duration**: Third development phase
- **Lines of Code**: ~3,750 (Swift: 2,500, Swift tests: 400, Docs: 850)

### Total Project
- **Production Code**: ~5,500 lines (Rust: 1,900, Swift: 3,600)
- **Test Code**: ~400 lines
- **Documentation**: ~10,000+ lines across all markdown files

## Known Issues

### Current Limitations
1. Draft pruning has placeholder implementation (awaits liboxen API enhancements)
2. Lock files stored in plain JSON (spoofable in local network)
3. No automatic merge for binary Logic Pro files (manual FCP XML workflow required)
4. XPC connection requires manual daemon restart if daemon crashes
5. Lock status polling-based (not real-time push notifications)

### Platform Requirements
- macOS 14.0+ (Sonoma or later)
- Logic Pro 11.x recommended
- Xcode 15+ for building from source
- Rust 1.70+ for CLI wrapper

## Security Considerations

- Daemon runs in user context (not privileged)
- No elevated permissions required
- XPC service restricted to same user
- Lock files are plain JSON (consider encryption for production)
- No authentication for local XPC calls (local trust model)

## Future Enhancements

See individual component READMEs and PHASE3_COMPLETE.md for detailed future enhancement lists.

### High Priority
- Real-time lock notifications via NSXPCConnection delegates
- Automated FCP XML diff tool
- Centralized lock server with authentication
- Visual diff viewer for project metadata

### Medium Priority
- Timeline visualization of commits
- Integration with Slack/Teams for lock notifications
- Multi-window support
- Preferences panel for all settings
- Dark mode optimizations

### Low Priority
- Conflict detection UI
- Localization (internationalization)
- Plugin-specific merge handlers
- Advanced filtering and search

## Contributors

This project was developed with assistance from Claude Code (Anthropic).

## License

MIT License - See [LICENSE](LICENSE) for details.

---

**Note**: This CHANGELOG was created on 2025-10-27 to document the project's development history. Previous changes were reconstructed from git history and phase completion reports.
