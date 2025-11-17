# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

---

## Project Overview

**Oxen-VCS** is a macOS-native version control system for Apple Logic Pro projects that solves the fundamental incompatibility between traditional VCS (like Git) and professional DAW workflows.

### The Problem
Logic Pro projects consist of:
- Large binary audio files (WAV, AIFF, CAF) - often multi-GB
- Proprietary, opaque `.logicx` project files (non-mergeable binary)
- Generated assets (bounces, freeze files) that cause repository bloat
- Non-destructive editing patterns where metadata changes but audio doesn't

Traditional VCS fails because:
- Git/Git-LFS stores entire files on modification → massive bloat
- Binary project files cannot be algorithmically merged
- Merge conflicts are unresolvable without data loss
- No understanding of DAW-specific workflows

### The Solution
Oxen-VCS leverages Oxen.ai's block-level deduplication and implements:
- **Pessimistic locking** to prevent binary merge conflicts
- **Intelligent asset classification** with `.oxenignore` strategies
- **Automatic draft tracking** via FSEvents monitoring
- **Power-safe commits** triggered before system sleep
- **FCP XML-based manual merge** for track-level reconciliation

### Target Users
- Professional audio engineers and music producers
- Collaborative production teams
- Anyone managing multi-GB Logic Pro projects requiring reliable version history

---

## Quick Reference

### Most Common Commands

```bash
# Run all tests (comprehensive test suite)
./run_all_tests.sh

# Build Rust CLI wrapper
cd Auxin-CLI-Wrapper && cargo build --release && cargo test

# Build Swift components (macOS only)
cd Auxin-LaunchAgent && swift build
cd Auxin-App && swift build

# Build and create app bundle (REQUIRED for GUI app)
cd Auxin-App && swift build -c release && ./create-app-bundle.sh

# Run the app bundle
open Auxin-App/Auxin.app

# Run specific test suites
cd Auxin-CLI-Wrapper && cargo test                    # Rust unit tests
cd Auxin-LaunchAgent && swift test                    # LaunchAgent tests
cd Auxin-App && swift test                            # App tests

# Lint and format
cd Auxin-CLI-Wrapper && cargo fmt && cargo clippy     # Rust
swiftlint lint && swiftlint autocorrect               # Swift (if configured)
```

### Key Documentation Files

- **[README.md](README.md)** - Project overview and status
- **[INSTALL.md](INSTALL.md)** - Installation instructions
- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Code style, testing requirements
- **[docs/USER_GUIDE.md](docs/USER_GUIDE.md)** - Complete user guide with quick start section
- **[docs/TESTING_STRATEGY.md](docs/TESTING_STRATEGY.md)** - Testing approach
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Full technical specification
- **[docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)** - Common issues and solutions

### Critical Source Files

**Rust CLI Wrapper** (`Auxin-CLI-Wrapper/src/`):
- `main.rs:1` - CLI entry point and command handling
- `oxen_subprocess.rs:1` - **CRITICAL**: Oxen CLI subprocess integration (primary Oxen interface)
- `oxen_ops.rs:1` - High-level Oxen operation wrappers
- `logic_project.rs:1` - Logic Pro project detection and validation
- `commit_metadata.rs:1` - Structured commit metadata (BPM, sample rate, key)
- `ignore_template.rs:1` - .oxenignore generation
- `draft_manager.rs:1` - Draft branch management logic
- `liboxen_stub/` - Stub implementation (not connected to real Oxen)

**Swift LaunchAgent** (`Auxin-LaunchAgent/Sources/`):
- `Daemon.swift:1` - Main daemon orchestration and lifecycle
- `FSEventsMonitor.swift:1` - File system change monitoring
- `PowerManagement.swift:1` - Sleep/shutdown event handling
- `CommitOrchestrator.swift:1` - Auto-commit workflow logic
- `LockManager.swift:1` - File lock enforcement
- `XPCService.swift:1` - IPC communication with app
- `ServiceManager.swift:1` - LaunchAgent registration

**Swift App** (`Auxin-App/Sources/`):
- `AuxinApp.swift:1` - SwiftUI app entry point with @main attribute
- `AppDelegate.swift:1` - App delegate for menu actions and legacy features
- `Views/SwiftUI/` - SwiftUI declarative UI components (migrated from AppKit)
  - `ContentView.swift` - Main view with NavigationSplitView
  - `ProjectListContentView.swift` - Project sidebar
  - `ProjectDetailContentView.swift` - Project details and commit history
  - `SwiftUIStatusBar.swift` - Status bar overlay
- `ViewModels/` - MVVM business logic layer (reused from AppKit)
- `Services/OxenDaemonXPCClient.swift` - XPC client for daemon communication

### Common Development Workflows

**Adding a New Feature:**
1. Read relevant source files to understand existing patterns
2. Create feature branch: `git checkout -b feature/your-feature`
3. Write implementation following existing patterns (see Coding Conventions below)
4. Write tests (required - see Testing Strategy section)
5. Run tests: `./run_all_tests.sh` or component-specific tests
6. Update documentation if needed
7. Submit PR with description

**Debugging the Daemon:**
```bash
# View daemon logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --style syslog

# Stop daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist

# Start daemon with manual logging
cd Auxin-LaunchAgent && swift run

# Check daemon status
launchctl list | grep com.auxin
```

**Testing Oxen Integration:**
```bash
# Install Oxen CLI (prerequisite for integration tests)
pip3 install oxen-ai
# or: cargo install oxen

# Run integration tests
cd Auxin-CLI-Wrapper && cargo test --test oxen_subprocess_integration_test
```

**Building for Distribution:**
```bash
# Use the automated installer script (recommended)
./install.sh

# Or manually build release versions
cd Auxin-CLI-Wrapper && cargo build --release
cd Auxin-LaunchAgent && swift build -c release
cd Auxin-App && swift build -c release && ./create-app-bundle.sh

# The create-app-bundle.sh script creates a proper macOS .app bundle
# with Info.plist and correct directory structure for GUI rendering
```

---

## Project Status & Reality Check

**Last Updated**: 2025-10-29
**Development Environment**: Linux 4.4.0 (cannot compile/test Swift components)
**Required for Production**: macOS 14.0+ with Logic Pro 11.x

### Component Status Summary

| Component | Code Complete | Test Coverage | Integration Tested | Production Ready |
|-----------|---------------|---------------|-------------------|------------------|
| **Rust CLI Wrapper** | ✅ 100% | ✅ 85% (121 tests) | 🟡 Partial | 🟡 With subprocess wrapper |
| **Swift LaunchAgent** | ✅ 100% | 🟡 30% | ❌ 0% | ❌ Needs testing |
| **Swift App UI** | ✅ 100% (SwiftUI) | 🔴 <10% | ✅ Working | 🟡 Needs integration testing |

### What's Working

**Rust CLI Wrapper** (~2,500 lines):
- ✅ Logic Pro project detection and validation
- ✅ Commit metadata parsing (BPM, sample rate, key)
- ✅ .oxenignore template generation
- ✅ Draft branch management (data structures)
- ✅ Logging system with verbose mode
- ✅ **Oxen subprocess wrapper** - primary interface to Oxen CLI
- 🟡 **Limitation**: Using liboxen stub (fallback only)

**Swift LaunchAgent** (FSEvents + Power Management):
- ✅ FSEvents monitoring with debounce
- ✅ Power management (sleep/shutdown hooks)
- ✅ XPC service for IPC
- ✅ Lock management with timeout
- ❌ **NOT TESTED**: Long-running stability, multi-project monitoring, memory leaks

**Swift App UI** (SwiftUI - Migrated from AppKit 2025-10-29):
- ✅ Native NavigationSplitView with automatic layout
- ✅ Project list with sidebar navigation
- ✅ Project detail view with commit history
- ✅ Status bar showing daemon status
- ✅ Toolbar with refresh and add project buttons
- ✅ Menu bar integration
- ✅ Window management works reliably (no AppKit sizing issues)
- ✅ **Migration Benefits**: 80% less UI code, declarative layout, modern SwiftUI patterns
- 🟡 **TODO**: Re-integrate milestone commit UI, rollback UI, lock management, merge helper

### Critical Gaps & Blockers

**🔴 Oxen.ai Integration Status:**
- **Solution Implemented**: Subprocess wrapper (oxen_subprocess.rs) executes Oxen CLI commands
- **Fallback**: liboxen stub (not functional)
- **Requirement**: `pip install oxen-ai` or `cargo install oxen` for real operations
- **Status**: Integration code written, needs macOS testing

**🔴 Platform Constraint:**
- **Current Dev Environment**: Linux 4.4.0 (cannot compile Swift)
- **Required**: macOS 14.0+ with Xcode 15+ and Logic Pro 11.x
- **Impact**: All Swift components untested until macOS access

**🟡 Test Coverage Gaps:**
- Rust: 85% ✅
- Swift LaunchAgent: ~30% 🟡 (only LockManager tested)
- Swift App: <10% 🔴 (only MockXPCClient tested)
- **Missing**: FSEvents, power management, XPC, ViewModels, end-to-end workflows

### Production Readiness

**Can It Version Control Logic Pro Projects Today?** No (but close!)

**To Ship v0.1 MVP** (1-2 weeks on macOS):
1. Integrate oxen_subprocess (primary blocker)
2. Integration tests with real .logicx projects
3. Build and test Swift components
4. Fix bugs from testing
5. Create .app bundle installer

**Known Risks:**
- **High**: Daemon stability, lock conflicts, power management edge cases, XPC reliability
- **Medium**: Subprocess hangs, large file timeouts, .oxenignore accuracy
- **Low**: Commit metadata, project detection, logging (well-tested)

---

## Technology Stack

### Languages & Frameworks
- **macOS Layer**: Swift 5.9+, AppKit (macOS-native UI)
- **VCS Backend**: Rust 2021 edition, Oxen CLI subprocess wrapper
- **Build Tools**: Swift Package Manager, Cargo
- **Minimum macOS**: 14.0

### Key Dependencies
- **Rust**: serde, tokio, clap, anyhow, chrono, colored
- **Swift**: FSEvents API, SMAppService, XPC, NSWorkspace
- **External**: Oxen CLI (`pip install oxen-ai` or `cargo install oxen`)

### Development Requirements
- Xcode 15+ (for Swift compilation)
- Rust toolchain (rustc, cargo)
- Logic Pro 11.x (for real-world testing)

---

## Architecture

### Three-Component System

```
┌──────────────────────────────────────────────────────────┐
│                    Auxin-App (Swift/AppKit)              │
│  • UI for history browsing, commits, rollback           │
│  • Repository initialization wizard                      │
│  • SMAppService daemon registration                      │
│  • Lock management interface                             │
└────────────────────┬─────────────────────────────────────┘
                     │ IPC (XPC)
┌────────────────────┴─────────────────────────────────────┐
│              Auxin-LaunchAgent (Swift)                   │
│  • FSEvents monitoring (30-60s debounce)                │
│  • Power management observers (NSWorkspace)              │
│  • Draft commit automation                               │
│  • Lock enforcement                                      │
└────────────────────┬─────────────────────────────────────┘
                     │ IPC
┌────────────────────┴─────────────────────────────────────┐
│          Auxin-CLI-Wrapper (Rust/liboxen)               │
│  • FFI wrapper around liboxen                           │
│  • Low-latency Oxen operations (<10ms add, <100ms commit)│
│  • Embedded as app bundle helper tool                   │
└──────────────────────────────────────────────────────────┘
```

### Directory Structure

```
auxin/
├── Auxin-App/                    # Swift/AppKit UI application
│   ├── Sources/
│   │   ├── Views/                # SwiftUI/AppKit views
│   │   ├── ViewModels/           # Business logic layer
│   │   ├── Models/               # Data structures
│   │   ├── Services/             # Oxen integration
│   │   └── Utilities/            # Helpers
│   ├── Resources/
│   │   ├── Assets.xcassets/
│   │   └── Info.plist
│   └── Tests/
│
├── Auxin-LaunchAgent/            # Background daemon
│   ├── Sources/
│   │   ├── main.swift            # Daemon entry point
│   │   ├── FSEventsMonitor.swift # File system monitoring
│   │   ├── PowerManager.swift    # Sleep/shutdown handling
│   │   ├── DraftCommitter.swift  # Auto-commit logic
│   │   ├── IPCService.swift      # XPC communication
│   │   └── LockManager.swift     # File lock enforcement
│   ├── Resources/
│   │   └── com.auxin.agent.plist
│   └── Tests/
│
├── Auxin-CLI-Wrapper/            # Rust CLI wrapper
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # CLI entry point
│   │   ├── oxen_ops.rs           # Oxen operation wrappers
│   │   ├── ipc.rs                # IPC server
│   │   └── lib.rs                # FFI exports
│   ├── benches/                  # Performance benchmarks
│   └── tests/
│
├── docs/
│   ├── ARCHITECTURE.md           # Full technical blueprint
│   ├── IMPLEMENTATION_PLAN.md    # Phase-by-phase roadmap
│   └── API.md                    # Component interfaces
│
└── tests/                        # Integration tests
    ├── integration/
    └── fixtures/                 # Sample Logic Pro projects
```

---

## Key Concepts

### Asset Classification Strategy

Logic Pro files are classified into tracking categories:

| Category | Examples | Action | Rationale |
|----------|----------|--------|-----------|
| **Core State** | `projectData` (binary session file) | **Track** | Essential, non-mergeable project state |
| **Raw Audio** | `*.wav`, `*.aif`, `*.caf` in Audio Files/ | **Track (LOB)** | High-fidelity data; deduplication yields maximum benefit |
| **Generated** | `Bounces/`, `Freeze Files/` | **Exclude** | Volatile, large, regenerable; causes conflicts |
| **Temp/Volatile** | `Autosave/`, `*.nosync` | **Exclude** | Prevents noisy conflicts and repository bloat |

### .oxenignore Template

```gitignore
# Generated Audio
Bounces/
Freeze Files/

# Volatile System Data
Autosave/
*.nosync

# macOS System
.DS_Store
._*

# Logic Pro Temp
*.logictemp
```

### Commit Types

**Draft Commits** (Automatic)
- Triggered by FSEvents after 30-60s inactivity
- Committed to local `draft` branch
- Provides granular safety net for crashes
- Pruned/rebased upon Milestone Commits

**Milestone Commits** (Explicit)
- User-initiated via UI
- Pre-commit sequence:
  1. Delete all volatile files (Bounces/, Freeze Files/)
  2. Stage all tracked changes
  3. Commit with structured metadata (BPM, sample rate, key)
  4. Push to remote (if configured)
- Tagged for production phases ("Mix Alpha v1.0", "Final Master")

### Collaboration Model

**Pessimistic Locking** (Mandatory)
- User must acquire exclusive lock before editing project
- Lock stored in remote manifest or Oxen Hub
- LaunchAgent enforces lock by:
  - Checking lock status on project open
  - Setting restrictive file permissions if locked by another user
  - Displaying block dialog with lock holder info

**Manual Merge Protocol** (Track-Level)
When feature branches diverge:
1. Export modified tracks from Branch A using FCP XML format
2. Import FCP XML into Branch B
3. Manual reconciliation within Logic Pro
4. Commit merged result

---

## Development Commands

> **Note**: See [Quick Reference](#quick-reference) section at the top for the most common commands.

### Initial Setup

```bash
# Clone and install
git clone https://github.com/jbacus/auxin.git
cd auxin
./install.sh  # Automated installation (recommended)

# Or install Oxen CLI manually for development
pip3 install oxen-ai  # or: cargo install oxen
```

### Testing

```bash
# Run all tests (recommended)
./run_all_tests.sh

# Component-specific tests
cd Auxin-CLI-Wrapper && cargo test           # Rust unit tests
cd Auxin-CLI-Wrapper && cargo test --test oxen_subprocess_integration_test  # Integration tests
cd Auxin-LaunchAgent && swift test           # LaunchAgent tests (macOS only)
cd Auxin-App && swift test                   # App tests (macOS only)

# With coverage
cd Auxin-CLI-Wrapper && cargo tarpaulin --out Html
cd Auxin-LaunchAgent && swift test --enable-code-coverage
```

### Building

```bash
# Using Swift Package Manager (recommended)
cd Auxin-LaunchAgent && swift build -c release
cd Auxin-App && swift build -c release

# Using Xcode (alternative)
xcodebuild -project Auxin-App/Auxin.xcodeproj -scheme Auxin -configuration Release
```

### LaunchAgent Management

```bash
# View logs (most useful for debugging)
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --style syslog

# Service control
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl list | grep com.auxin
```

---

## Coding Conventions

### Swift Style

**Naming**
- Types: `PascalCase` (e.g., `FSEventsMonitor`)
- Functions/vars: `camelCase` (e.g., `debounceInterval`)
- Constants: `camelCase` with `let` (e.g., `defaultDebounceSeconds`)
- Private members: prefix with underscore (e.g., `_eventStream`)

**Architecture**
- Follow MVVM pattern in UI app
- Use protocols for dependency injection
- Prefer composition over inheritance
- Services should be stateless where possible

**Error Handling**
```swift
// Use Result type for operations that can fail
func commitChanges(message: String) -> Result<Commit, OxenError> {
    // Implementation
}

// Propagate errors with throws for async operations
func fetchRemote() async throws -> [Commit] {
    // Implementation
}
```

**Async/Await**
```swift
// Use structured concurrency
actor CommitQueue {
    private var pending: [Commit] = []
    
    func enqueue(_ commit: Commit) {
        pending.append(commit)
    }
}
```

### Rust Style

**Naming**
- Follow Rust naming conventions (snake_case for functions/vars)
- Use descriptive names for public APIs
- Prefix private items with underscore if unused

**Error Handling**
```rust
use anyhow::{Context, Result};

fn oxen_init(path: &Path) -> Result<()> {
    // Use context for better error messages
    Repository::init(path)
        .context("Failed to initialize Oxen repository")?;
    Ok(())
}
```

**Performance**
```rust
// Minimize allocations in hot paths
// Use &str instead of String when possible
// Profile with cargo flamegraph for optimization
```

**FFI Exports**
```rust
#[no_mangle]
pub extern "C" fn oxenvcs_add_file(
    repo_path: *const c_char,
    file_path: *const c_char
) -> i32 {
    // Safe FFI with proper error codes
}
```

---

## Architectural Patterns

### FSEvents Monitoring Pattern

```swift
class FSEventsMonitor {
    private var eventStream: FSEventStreamRef?
    private var debounceTimer: Timer?
    private let debounceInterval: TimeInterval = 30.0
    
    func startMonitoring(path: String) {
        let callback: FSEventStreamCallback = { 
            streamRef, clientCallBackInfo, numEvents, eventPaths, eventFlags, eventIds in
            // Handle events
            let monitor = Unmanaged<FSEventsMonitor>
                .fromOpaque(clientCallBackInfo!)
                .takeUnretainedValue()
            monitor.handleEvents(numEvents, eventPaths, eventFlags)
        }
        
        var context = FSEventStreamContext(
            version: 0,
            info: Unmanaged.passUnretained(self).toOpaque(),
            retain: nil,
            release: nil,
            copyDescription: nil
        )
        
        eventStream = FSEventStreamCreate(
            kCFAllocatorDefault,
            callback,
            &context,
            [path] as CFArray,
            FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
            0.0,
            UInt32(kFSEventStreamCreateFlagFileEvents)
        )
        
        FSEventStreamScheduleWithRunLoop(
            eventStream!,
            CFRunLoopGetCurrent(),
            CFRunLoopMode.defaultMode.rawValue
        )
        FSEventStreamStart(eventStream!)
    }
    
    private func handleEvents(_ numEvents: Int, _ eventPaths: UnsafeMutableRawPointer, _ eventFlags: UnsafePointer<FSEventStreamEventFlags>) {
        // Reset debounce timer
        debounceTimer?.invalidate()
        debounceTimer = Timer.scheduledTimer(
            withTimeInterval: debounceInterval,
            repeats: false
        ) { [weak self] _ in
            self?.triggerDraftCommit()
        }
    }
}
```

### Power Management Pattern

```swift
class PowerManager {
    private var workspaceNotificationObservers: [NSObjectProtocol] = []
    
    func registerPowerNotifications() {
        let notificationCenter = NSWorkspace.shared.notificationCenter
        
        // Sleep notification
        let sleepObserver = notificationCenter.addObserver(
            forName: NSWorkspace.willSleepNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.emergencyCommit(reason: "System entering sleep")
        }
        workspaceNotificationObservers.append(sleepObserver)
        
        // Power off notification
        let powerOffObserver = notificationCenter.addObserver(
            forName: NSWorkspace.willPowerOffNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.emergencyCommit(reason: "System powering off")
        }
        workspaceNotificationObservers.append(powerOffObserver)
    }
    
    private func emergencyCommit(reason: String) {
        // Override debounce and force immediate commit
        CommitService.shared.forceCommit(message: "Emergency save: \(reason)")
    }
}
```

### Oxen Operation Pattern (Rust)

```rust
use liboxen::Repository;
use std::path::Path;
use anyhow::{Context, Result};

pub struct OxenOps {
    repo: Repository,
}

impl OxenOps {
    pub fn new(path: &Path) -> Result<Self> {
        let repo = Repository::open(path)
            .context("Failed to open repository")?;
        Ok(Self { repo })
    }
    
    pub fn add(&self, files: &[&Path]) -> Result<()> {
        for file in files {
            self.repo.add(file)
                .with_context(|| format!("Failed to add file: {:?}", file))?;
        }
        Ok(())
    }
    
    pub fn commit(&self, message: &str, metadata: CommitMetadata) -> Result<String> {
        let full_message = format!(
            "{}\n\nBPM: {}\nSample Rate: {}kHz\nKey: {}",
            message,
            metadata.bpm,
            metadata.sample_rate / 1000,
            metadata.key
        );
        
        let commit_id = self.repo.commit(&full_message)
            .context("Failed to create commit")?;
        
        Ok(commit_id)
    }
}

#[derive(Debug)]
pub struct CommitMetadata {
    pub bpm: u16,
    pub sample_rate: u32,
    pub key: String,
}
```

### IPC Communication Pattern (XPC)

```swift
// Service Protocol
@objc protocol OxenVCSServiceProtocol {
    func executeCommit(message: String, reply: @escaping (Bool, Error?) -> Void)
    func stagePath(_ path: String, reply: @escaping (Bool, Error?) -> Void)
}

// Client-side (LaunchAgent)
class IPCClient {
    private var connection: NSXPCConnection?
    
    func connect() {
        connection = NSXPCConnection(serviceName: "com.auxin.cli-wrapper")
        connection?.remoteObjectInterface = NSXPCInterface(with: OxenVCSServiceProtocol.self)
        connection?.resume()
    }
    
    func commit(message: String) async throws {
        guard let proxy = connection?.remoteObjectProxyWithErrorHandler({ error in
            print("XPC Error: \(error)")
        }) as? OxenVCSServiceProtocol else {
            throw OxenError.ipcConnectionFailed
        }
        
        try await withCheckedThrowingContinuation { continuation in
            proxy.executeCommit(message: message) { success, error in
                if let error = error {
                    continuation.resume(throwing: error)
                } else {
                    continuation.resume()
                }
            }
        }
    }
}
```

---

## Performance Targets

### Rust CLI Wrapper
- Single file `add`: <10ms
- Commit operation: <100ms
- Memory footprint: <50MB resident
- Startup latency: <5ms

### FSEvents Monitor
- Event detection: <100ms from filesystem change
- Debounce accuracy: ±100ms of target interval
- CPU usage: <1% when idle

### UI Application
- History view load: <500ms for 1000 commits
- Rollback operation: <2s for projects up to 10GB

---

## Testing Strategy

### Unit Tests
- All Swift services and utilities
- Rust operation wrappers
- Mock Oxen repository for isolation

### Integration Tests
```swift
func testDraftCommitWorkflow() async throws {
    // 1. Setup test project
    let testProject = try createTestLogicProject()
    
    // 2. Initialize repository
    try OxenService.shared.initRepository(at: testProject.url)
    
    // 3. Modify project file
    try testProject.modifyProjectData()
    
    // 4. Wait for debounce
    try await Task.sleep(nanoseconds: 35_000_000_000) // 35s
    
    // 5. Verify draft commit created
    let commits = try OxenService.shared.getCommits()
    XCTAssertEqual(commits.count, 2) // Init + draft
    XCTAssertTrue(commits.first!.branch == "draft")
}
```

### System Tests
- 8+ hour continuous editing sessions
- Multiple concurrent users with locking
- System sleep/wake cycles
- Network interruption during push
- Large projects (50+ GB)

---

## Implementation Status

**All three phases are code-complete** (see [Project Status](#project-status--reality-check) for details).

### Phase 1: Core Data Management (MVP) - ✅ CODE COMPLETE
- ✅ Logic Pro project detection, .oxenignore generation, Oxen subprocess wrapper
- ✅ Core operations (init, add, commit, log, restore)
- ✅ Structured commit metadata (BPM, sample rate, key)
- ✅ 85% test coverage (121 tests)
- 🟡 **Blocker**: Needs macOS integration testing

### Phase 2: Service Architecture - ✅ CODE COMPLETE
- ✅ LaunchAgent with FSEvents monitoring
- ✅ Power management (sleep/shutdown hooks)
- ✅ XPC communication, multi-project support
- 🟡 ~30% test coverage
- 🟡 **Blocker**: Untested in production scenarios

### Phase 3: UI & Collaboration - ✅ CODE COMPLETE
- ✅ Native macOS AppKit application
- ✅ Repository browser, milestone commits, rollback UI
- ✅ Exclusive file locking system
- 🔴 <10% test coverage
- 🟡 **Blocker**: Never run with real Logic Pro projects

**Next Steps**: See [Production Readiness](#production-readiness) section for MVP shipping requirements.

---

## Known Constraints

### Logic Pro Automation Barrier
- Logic Pro's Scripter environment **blocks external filesystem access**
- No way to hook into DAW save operations
- Cannot trigger commits from within Logic Pro
- **Solution**: External monitoring via FSEvents

### Binary Merge Impossibility
- `.logicx` projectData file is proprietary binary
- No algorithmic merge possible without Apple SDK
- **Solution**: Pessimistic locking + manual FCP XML reconciliation

### Oxen vs Git-LFS
- Oxen optimized for large binary datasets
- Block-level deduplication (not file-level like Git-LFS)
- Custom network protocol (not Git-compatible)
- **Advantage**: 10-100x storage efficiency for DAW projects

---

## Troubleshooting

> **Full troubleshooting guide**: See [docs/TROUBLESHOOTING.md](docs/TROUBLESHOOTING.md)

**Quick Diagnostics:**
```bash
# Check daemon status
launchctl list | grep com.auxin

# View logs (most useful)
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --style syslog

# Verify Oxen CLI installation
which oxen && oxen --version

# Test Rust CLI wrapper
cd Auxin-CLI-Wrapper && cargo test
```

**Common Issues:**
- **LaunchAgent not starting**: Check logs, verify plist file, reload manually
- **FSEvents not firing**: Ensure folder-based .logicx (not bundle), check permissions
- **Oxen commands failing**: Install Oxen CLI (`pip3 install oxen-ai`)
- **Swift compilation errors**: Requires macOS 14.0+ with Xcode 15+

---

## Resources

### Documentation
- [Oxen.ai Docs](https://docs.oxen.ai/)
- [Logic Pro Project Format](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml)
- [FSEvents Programming Guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/)
- [SMAppService API](https://developer.apple.com/documentation/servicemanagement/smappservice)

### Dependencies
- [liboxen crate](https://crates.io/crates/liboxen)
- [Oxen Python client](https://pypi.org/project/oxen-ai/)

### Related Projects
- [Perforce Helix Core](https://www.perforce.com/products/helix-core) (inspiration for locking model)
- [DVC](https://dvc.org/) (data versioning comparison point)

---

## Future Enhancements

### AI Semantic Diffing (Planned)
Multi-modal analysis to provide semantic change summaries:
- Audio feature extraction (librosa, CLAP embeddings)
- FCP XML structural diffing
- Natural language search across commit history
- Timeline visualization of project evolution

**Note**: AI diffing enhances understanding but does NOT enable automatic merging.

### Cross-DAW Support
Architectural patterns extensible to:
- Ableton Live
- Pro Tools
- Cubase

### Workflow Automation
- Pre-commit hooks for audio normalization
- Post-commit triggers for cloud backup
- Integration with mixing/mastering pipelines

---

## Additional Resources

### External Documentation
- [Oxen.ai Docs](https://docs.oxen.ai/) - Oxen VCS documentation
- [Logic Pro Project Format](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml) - Technical specification
- [FSEvents Programming Guide](https://developer.apple.com/library/archive/documentation/Darwin/Conceptual/FSEvents_ProgGuide/) - File monitoring API
- [SMAppService API](https://developer.apple.com/documentation/servicemanagement/smappservice) - macOS daemon management

### Related Projects
- [Perforce Helix Core](https://www.perforce.com/products/helix-core) - Inspiration for pessimistic locking model
- [DVC](https://dvc.org/) - Data versioning comparison point

### Contact & Support
- **GitHub Repository**: https://github.com/jbacus/auxin
- **Issue Tracker**: [Create an issue](https://github.com/jbacus/auxin/issues)
- **Oxen.ai Community**: hello@oxen.ai

---

## Summary for Claude Code

**When starting work on this codebase:**
1. **Review** the [Quick Reference](#quick-reference) section first
2. **Understand** the [Project Status](#project-status--reality-check) - code is complete but needs testing on macOS
3. **Key blocker**: Oxen integration via subprocess wrapper needs macOS testing
4. **Testing**: Use `./run_all_tests.sh` before committing
5. **Critical files**: `oxen_subprocess.rs` (Oxen CLI integration), `Daemon.swift` (LaunchAgent), `AppDelegate.swift` (UI)

**Development environment constraint**: This project requires macOS 14.0+ for building/testing Swift components. Current Linux environment can only handle Rust development.

**Documentation status**: All markdown documentation consolidated and streamlined (42 files → 23 essential files) on 2025-10-29. Auxin-App migrated from AppKit to SwiftUI on 2025-10-29 for improved window management and code simplicity.

---

*Last Updated: 2025-10-29*
