# CLAUDE.md - Oxen-VCS for Logic Pro

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

## Project Status & Reality Check

**Last Updated**: 2025-10-28

### What's Actually Working

#### ✅ Rust CLI Wrapper (~85% complete)
- **Code**: ~2,500 lines of well-structured Rust
- **Tests**: 121 comprehensive unit tests (85% coverage)
- **Status**: Production-ready code structure
- **Limitation**: Using liboxen **stub** - not connected to real Oxen.ai yet

**Key Components:**
- ✅ Logic Pro project detection and validation
- ✅ Commit metadata parsing and formatting
- ✅ .oxenignore template generation
- ✅ Draft branch management (data structures)
- ✅ Logging system with verbose mode
- ✅ **NEW**: Oxen subprocess wrapper (via CLI commands)

#### ✅ LaunchAgent Daemon (Swift)
- **Code**: Fully implemented with FSEvents, power management, XPC
- **Tests**: ~30% coverage (only LockManager tested)
- **Status**: Code complete, needs testing
- **Limitation**: Untested in production scenarios

**Key Components:**
- ✅ FSEvents monitoring with debounce
- ✅ Power management (sleep/shutdown hooks)
- ✅ XPC service for IPC
- ✅ Lock management with timeout
- ❌ **NOT TESTED**: Long-running stability
- ❌ **NOT TESTED**: Multi-project monitoring
- ❌ **NOT TESTED**: Memory leaks under load

#### ✅ UI Application (Swift/AppKit)
- **Code**: Full MVVM implementation with all views
- **Tests**: <5% coverage (only MockXPCClient)
- **Status**: Code complete, needs testing
- **Limitation**: Never run with real Logic Pro projects

**Key Components:**
- ✅ Project browser and initialization wizard
- ✅ Milestone commit interface with metadata
- ✅ Rollback/restore UI
- ✅ Lock management views
- ✅ Merge helper window
- ❌ **NOT TESTED**: With actual .logicx files
- ❌ **NOT TESTED**: XPC communication
- ❌ **NOT TESTED**: User workflows

### Critical Gaps

#### 🔴 Oxen.ai Integration (BLOCKER)
**Problem**: The liboxen Rust crate doesn't exist on crates.io

**Current State:**
- Using a **stub implementation** that does nothing
- All "Oxen operations" are fake
- Cannot actually version control any files

**Solutions Available:**
1. **Subprocess Wrapper** (✅ IMPLEMENTED 2025-10-28)
   - Execute `oxen` CLI commands via subprocess
   - Parse stdout/stderr for results
   - Requires: `pip install oxen-ai` or `cargo install oxen`
   - Status: Code written, needs integration testing

2. **HTTP API** (Future)
   - Use Oxen Hub REST API
   - Requires network for all operations
   - Not suitable for local-only workflows

3. **Wait for liboxen** (Unknown timeline)
   - Official Rust bindings from Oxen.ai team
   - Unknown if/when this will be available

**Recommendation**: Use subprocess wrapper immediately for MVP testing

#### 🔴 Platform Mismatch (DEVELOPMENT BLOCKER)
**Problem**: Project requires macOS, but development environment is Linux

**Impact:**
- ✅ Can write code (Rust + Swift)
- ✅ Can write tests
- ❌ **CANNOT compile** Swift components
- ❌ **CANNOT run** any tests
- ❌ **CANNOT test** with Logic Pro
- ❌ **CANNOT build** .app bundle

**Required:**
- macOS 14.0+ with Xcode 15+
- Swift 5.9+ compiler
- Logic Pro 11.x for real-world testing

#### 🟡 Swift Testing Gap (QUALITY RISK)
**Current Coverage:**
- LaunchAgent: ~30% (only LockManager)
- App: <5% (only MockXPCClient)
- Total Swift: <10% coverage

**Missing Tests:**
- FSEventsMonitor behavior
- Power management triggers
- XPC communication reliability
- CommitOrchestrator logic
- All ViewModels
- UI integration flows

**Impact**: High risk of runtime failures in production

#### 🟡 Integration Testing (VALIDATION GAP)
**What's NOT Tested:**
- End-to-end commit workflows
- Real .logicx project handling
- Long-running daemon stability
- Multi-project monitoring
- Lock contention scenarios
- Power management edge cases
- System sleep/wake cycles

**Impact**: Unknown behavior in real-world usage

### What "Phase Complete" Actually Means

The README claims all three phases are complete. Here's the reality:

| Phase | Code | Tests | Integration | Production Ready? |
|-------|------|-------|-------------|-------------------|
| **Phase 1: MVP** | ✅ 100% | ✅ 85% (Rust only) | ❌ 0% | 🟡 With subprocess wrapper |
| **Phase 2: Service** | ✅ 100% | 🟡 30% | ❌ 0% | ❌ Untested |
| **Phase 3: UI & Collab** | ✅ 100% | 🔴 <10% | ❌ 0% | ❌ Untested |

**Translation:**
- ✅ **"Complete"** means: Code is written and compiles
- ❌ **"Complete"** does NOT mean: Tested or validated
- ❌ **"Complete"** does NOT mean: Connected to real Oxen
- ❌ **"Complete"** does NOT mean: Runs on macOS

### Honest Production Readiness Assessment

#### Can It Version Control Logic Pro Projects Today?
**Answer**: No (but close!)

**Why Not:**
1. Need to integrate subprocess wrapper (1-2 days)
2. Need macOS to compile and test (hardware req)
3. Need integration tests with real .logicx files (2-3 days)
4. Need Swift test coverage (1-2 weeks)

#### What Would It Take to Ship v0.1 MVP?
**Minimum Requirements** (1-2 weeks on macOS):
1. ✅ Integrate oxen_subprocess into CLI wrapper
2. ✅ Write integration tests for common workflows
3. ✅ Test with 3-5 real Logic Pro projects
4. ✅ Fix bugs discovered during testing
5. ✅ Create .app bundle installer
6. ✅ Write user documentation

**Nice to Have** (additional 2-3 weeks):
- Swift unit tests (70%+ coverage)
- Continuous monitoring (8+ hours)
- Multi-user lock testing
- Performance optimization
- Error recovery testing

#### What Could Go Wrong in Production?
**High Risk:**
- Daemon crashes and stops monitoring
- Lock conflicts cause data races
- Power management triggers miss commits
- Memory leaks on long-running daemon
- XPC connection drops unexpectedly

**Medium Risk:**
- Oxen CLI subprocess hangs
- Large files cause timeouts
- .oxenignore patterns miss files
- FCP XML export loses data
- UI freezes on large operations

**Low Risk (Well-Tested):**
- Commit metadata parsing
- Project detection
- .oxenignore generation
- Logger functionality

### Development Environment Constraints

**Current Environment**: Linux 4.4.0 (CI/Container)
**Capabilities:**
- ✅ Write Rust code
- ✅ Write Swift code (syntax only)
- ✅ Write unit tests
- ✅ Document architecture
- ❌ Compile Swift
- ❌ Run tests
- ❌ Test with Logic Pro
- ❌ Build .app bundle

**Required for Testing**: macOS 14.0+
**Required for Production**: macOS 14.0+ with Logic Pro 11.x

### Next Steps to Reality

#### Immediate (Can Do on Linux)
1. ✅ **DONE**: Write comprehensive Rust unit tests
2. ✅ **DONE**: Implement oxen subprocess wrapper
3. ✅ **DONE**: Document reality check
4. 🔄 **IN PROGRESS**: Update all documentation

#### Short-term (Requires macOS)
1. Integrate oxen_subprocess into main CLI
2. Write integration tests
3. Test with real .logicx projects
4. Build and test Swift components
5. Create .app bundle

#### Medium-term (Production Readiness)
1. Expand Swift test coverage to 70%+
2. 8-hour continuous monitoring test
3. Multi-user collaboration testing
4. Performance optimization
5. Beta user testing

---

## Architecture

### Three-Component System

```
┌──────────────────────────────────────────────────────────┐
│                    OxVCS-App (Swift/AppKit)              │
│  • UI for history browsing, commits, rollback           │
│  • Repository initialization wizard                      │
│  • SMAppService daemon registration                      │
│  • Lock management interface                             │
└────────────────────┬─────────────────────────────────────┘
                     │ IPC (XPC)
┌────────────────────┴─────────────────────────────────────┐
│              OxVCS-LaunchAgent (Swift)                   │
│  • FSEvents monitoring (30-60s debounce)                │
│  • Power management observers (NSWorkspace)              │
│  • Draft commit automation                               │
│  • Lock enforcement                                      │
└────────────────────┬─────────────────────────────────────┘
                     │ IPC
┌────────────────────┴─────────────────────────────────────┐
│          OxVCS-CLI-Wrapper (Rust/liboxen)               │
│  • FFI wrapper around liboxen                           │
│  • Low-latency Oxen operations (<10ms add, <100ms commit)│
│  • Embedded as app bundle helper tool                   │
└──────────────────────────────────────────────────────────┘
```

### Directory Structure

```
oxen-vcs-logic/
├── OxVCS-App/                    # Swift/AppKit UI application
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
├── OxVCS-LaunchAgent/            # Background daemon
│   ├── Sources/
│   │   ├── main.swift            # Daemon entry point
│   │   ├── FSEventsMonitor.swift # File system monitoring
│   │   ├── PowerManager.swift    # Sleep/shutdown handling
│   │   ├── DraftCommitter.swift  # Auto-commit logic
│   │   ├── IPCService.swift      # XPC communication
│   │   └── LockManager.swift     # File lock enforcement
│   ├── Resources/
│   │   └── com.oxenvcs.agent.plist
│   └── Tests/
│
├── OxVCS-CLI-Wrapper/            # Rust CLI wrapper
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

## Technology Stack

### macOS Layer
- **Language**: Swift 5.9+
- **UI Framework**: AppKit (macOS-native)
- **File Monitoring**: FSEvents API
- **Service Management**: SMAppService (macOS 13+)
- **IPC**: XPC (Inter-Process Communication)
- **Power Events**: NSWorkspace notifications

### VCS Backend
- **Engine**: Oxen.ai (via liboxen Rust crate)
- **Storage**: Block-level deduplicated object store
- **Features**: Merkle trees, smart network protocols, compression
- **Remote**: Oxen Hub or self-hosted instances

### CLI Wrapper
- **Language**: Rust 2021 edition
- **Core Dependency**: `liboxen = "0.19"`
- **IPC**: XPC bindings or Darwin notifications
- **Build**: Embedded as helper tool in app bundle

### Development Tools
- **IDE**: Xcode 15+
- **Build System**: Swift Package Manager + Cargo
- **Testing**: XCTest + Criterion (Rust benchmarks)
- **Minimum macOS**: 14.0

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

### Initial Setup

```bash
# Clone repository
git clone https://github.com/jbacus/oxen-vcs-logic.git
cd oxen-vcs-logic

# Install Oxen CLI (for testing)
pip install oxen-ai

# Build Rust CLI wrapper
cd OxVCS-CLI-Wrapper
cargo build --release
cd ..

# Open main app in Xcode
open OxVCS-App/OxVCS.xcodeproj
```

### Build & Test

**Swift Components**
```bash
# Build main app (Xcode)
xcodebuild -project OxVCS-App/OxVCS.xcodeproj \
           -scheme OxVCS \
           -configuration Release

# Run tests
xcodebuild test -project OxVCS-App/OxVCS.xcodeproj \
                -scheme OxVCS

# Build LaunchAgent
xcodebuild -project OxVCS-LaunchAgent/OxVCS-LaunchAgent.xcodeproj \
           -scheme OxVCS-LaunchAgent \
           -configuration Release
```

**Rust CLI Wrapper**
```bash
cd OxVCS-CLI-Wrapper

# Build
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench

# Check performance
cargo run --release -- --help
```

### LaunchAgent Management

```bash
# Load agent for testing
launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist

# Unload agent
launchctl unload ~/Library/LaunchAgents/com.oxenvcs.agent.plist

# View logs
log show --predicate 'process == "OxVCS-LaunchAgent"' --last 1h
```

### Linting & Formatting

```bash
# Swift (using SwiftLint if configured)
swiftlint lint
swiftlint autocorrect

# Rust
cargo fmt
cargo clippy -- -D warnings
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
        connection = NSXPCConnection(serviceName: "com.oxenvcs.cli-wrapper")
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

## Implementation Phases

### Phase 1: MVP (8-10 weeks)
**Goal**: Functional single-user rollback system

- [ ] Folder structure enforcement
- [ ] .oxenignore integration
- [ ] Basic Oxen command wrapper
- [ ] Simple FSEvents watcher
- [ ] Minimal UI (commit history, rollback)
- [ ] Manual commit workflow

**Deliverable**: Single user can version projects and rollback reliably.

### Phase 2: Production Hardening (6-8 weeks)
**Goal**: Robust, system-integrated daemon

- [ ] SMAppService LaunchAgent
- [ ] Advanced debouncing
- [ ] Power management hooks
- [ ] Rust CLI wrapper optimization
- [ ] Enhanced UI with branches
- [ ] Draft pruning
- [ ] Error recovery and logging

**Deliverable**: Resilient background service for production environments.

### Phase 3: Collaboration (6-8 weeks)
**Goal**: Multi-user workflows

- [ ] Exclusive file locking system
- [ ] Lock manifest management
- [ ] FCP XML merge protocol
- [ ] Remote synchronization
- [ ] Conflict resolution UI
- [ ] Team permissions

**Deliverable**: Full collaborative VCS for production teams.

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

### LaunchAgent Not Starting
```bash
# Check registration status
launchctl list | grep com.oxenvcs

# View detailed error logs
log show --predicate 'process == "OxVCS-LaunchAgent"' \
         --style syslog \
         --last 1h

# Manually load for debugging
launchctl load -w ~/Library/LaunchAgents/com.oxenvcs.agent.plist
```

### FSEvents Not Triggering
```swift
// Verify path is correctly monitored
let path = "/path/to/Logic/Project.logicx"
let url = URL(fileURLWithPath: path)

// Check for .logicx bundle vs folder structure
if url.pathExtension == "logicx" {
    print("Error: Must use folder-based project, not bundle")
}
```

### Rust FFI Crashes
```bash
# Enable debug symbols
cargo build --release --features debug-symbols

# Use lldb for debugging
lldb target/release/oxenvcs-cli
(lldb) run
```

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

## Contact & Support

**GitHub**: https://github.com/jbacus/oxen-vcs-logic  
**Issues**: [Create an issue](https://github.com/jbacus/oxen-vcs-logic/issues)  
**Oxen.ai Community**: hello@oxen.ai

---

*Last Updated: 2025-10-28*
