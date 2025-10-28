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
