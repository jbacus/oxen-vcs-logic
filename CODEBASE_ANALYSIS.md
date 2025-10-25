# Oxen-VCS Logic Pro: Comprehensive Codebase Analysis

## Project Overview

**Oxen-VCS for Logic Pro** is a macOS native version control system for Apple Logic Pro projects. It leverages Oxen.ai's block-level deduplication for efficient large binary file management.

**Status**: Phase 2 Complete (Service Architecture & Resilience)
**Next**: Phase 3 (UI Application & Collaboration Features)

---

## 1. OVERALL PROJECT STRUCTURE

### Directory Layout

```
oxen-vcs-logic/
├── OxVCS-CLI-Wrapper/              # Rust FFI wrapper for Oxen operations
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs                 # CLI entry point
│       ├── lib.rs                  # Library exports
│       ├── logic_project.rs        # Logic Pro project detection
│       ├── oxen_ops.rs             # Oxen repository wrapper
│       ├── commit_metadata.rs      # Structured commit format
│       ├── ignore_template.rs      # .oxenignore generation
│       └── draft_manager.rs        # Draft branch workflow
│
├── OxVCS-LaunchAgent/              # macOS background daemon (Swift)
│   ├── Package.swift               # Swift package config
│   ├── Resources/
│   │   └── com.oxen.logic.daemon.plist  # LaunchAgent configuration
│   └── Sources/
│       ├── main.swift              # Entry point
│       ├── Daemon.swift            # Main daemon coordinator
│       ├── FSEventsMonitor.swift   # File system monitoring
│       ├── PowerManagement.swift   # Sleep/shutdown handling
│       ├── CommitOrchestrator.swift # Auto-commit orchestration
│       ├── XPCService.swift        # Inter-process communication
│       └── ServiceManager.swift    # Service registration (SMAppService)
│
├── OxVCS-App/                      # macOS UI application (Swift/AppKit)
│   └── README.md                   # Structure (not yet implemented)
│
├── docs/
│   ├── IMPLEMENTATION_PLAN.md      # 3-phase roadmap
│   ├── PHASE1_COMPLETE.md          # Phase 1 completion report
│   ├── PHASE2_COMPLETE.md          # Phase 2 completion report
│   └── PHASE2_INSTALLATION.md      # Installation instructions
│
└── README.md                        # Project overview

```

### Key Technologies

| Component | Language | Framework | Purpose |
|-----------|----------|-----------|---------|
| CLI Wrapper | Rust | liboxen 0.19 | High-performance Oxen operations |
| Daemon | Swift | Foundation/CoreServices | Background monitoring & orchestration |
| UI App | Swift | AppKit | User interface (Phase 3) |
| Build Tools | Swift Package Manager | Swift 5.9 | Daemon compilation |

---

## 2. PHASE 1 & PHASE 2 IMPLEMENTATION STATUS

### Phase 1: Core Data Management (COMPLETE ✅)

**Objective**: Prove versioning model works with Logic Pro

#### Implemented Features:

1. **Logic Pro Project Detection** (`logic_project.rs`)
   - Validates `.logicx` extension
   - Checks for `projectData` file
   - Returns structured project info
   - Identifies tracked paths: `projectData`, `Alternatives/`, `Resources/`

2. **.oxenignore Template Generation** (`ignore_template.rs`)
   - Categorized patterns:
     - Volatile: `Bounces/`, `Freeze Files/`, `*.nosync`, `Autosave/`
     - System: `.DS_Store`, `*.smbdelete*`, `.Trashes`
     - Cache: `*.cache`, `*.tmp`, `*~`
   - Auto-generated with documentation

3. **Oxen Integration** (`oxen_ops.rs`)
   - `init_for_logic_project()` - Full Logic Pro setup
   - `stage_changes()` - Stage files
   - `create_commit()` - Create commits with metadata
   - `get_history()` - Retrieve commit history
   - `restore()` - Restore to previous state
   - `status()` - Repository status

4. **Structured Commit Metadata** (`commit_metadata.rs`)
   - BPM tracking
   - Sample rate (Hz)
   - Key signature
   - Tags (comma-separated)
   - Formatted output with metadata

5. **FSEvents Monitoring** (Proof of Concept)
   - Real-time file change detection
   - Debounce logic (30s inactivity threshold)
   - Filters for relevant Logic Pro files
   - Event logging with timestamps

### Phase 2: Service Architecture & Resilience (COMPLETE ✅)

**Objective**: Build production-grade macOS service layer

#### Implemented Features:

1. **LaunchAgent Implementation** (`ServiceManager.swift`, `Daemon.swift`)
   - SMAppService registration (macOS 13.0+)
   - Automatic daemon startup on login
   - Configuration via plist file
   - Command-line management (install/uninstall/status)
   - Resource limits (512MB RAM, 50% CPU)

2. **Power Management** (`PowerManagement.swift`)
   - Monitors sleep/shutdown/wake events via NSWorkspace
   - IOKit assertions to prevent sleep during commits
   - Battery level awareness
   - System load monitoring
   - Emergency commit triggering before power events

3. **Auto-Commit Workflow** (`CommitOrchestrator.swift`)
   - FSEvents-triggered commits
   - 30-second debounce threshold (configurable)
   - Automatic file staging
   - Concurrent commit prevention
   - Change detection before commit
   - Support for three commit types:
     - `autoSave` - Regular debounced saves
     - `emergency` - Pre-sleep/shutdown commits
     - `manual` - User-triggered commits

4. **Draft Branch System** (`draft_manager.rs`)
   - Automatic draft branch creation on init
   - Keeps main branch clean
   - Auto-commit workflow on draft branch
   - Draft pruning to prevent unbounded growth
   - Merge capabilities (placeholder)

5. **XPC Service** (`XPCService.swift`)
   - Inter-process communication protocol (`OxenDaemonXPCProtocol`)
   - Server-side implementation in daemon
   - Client-side connection in UI
   - Mach service: `com.oxen.logic.daemon.xpc`
   - Supports 11+ RPC methods (see below)

---

## 3. UI COMPONENTS & FRAMEWORKS

### Current State: NOT YET IMPLEMENTED

**Status**: Placeholder directory only (`OxVCS-App/README.md`)

### Planned Structure (for Phase 3):

```
OxVCS-App/
├── OxVCS.xcodeproj/
├── Sources/
│   ├── Views/           # SwiftUI/AppKit views
│   ├── ViewModels/      # MVVM business logic
│   ├── Models/          # Data structures
│   ├── Services/        # Oxen integration
│   └── Utilities/       # Helpers
├── Resources/
│   ├── Assets.xcassets/
│   └── Info.plist
└── Tests/
```

### XPC Protocol for UI-Daemon Communication

**Available RPC Methods** (already defined in `XPCService.swift`):

```swift
protocol OxenDaemonXPCProtocol {
    // Project Management
    func registerProject(_ projectPath: String, withReply: (Bool, String?) -> Void)
    func unregisterProject(_ projectPath: String, withReply: (Bool, String?) -> Void)
    func getMonitoredProjects(withReply: ([String]) -> Void)
    
    // Commit Operations
    func commitProject(_ projectPath: String, message: String?, withReply: (String?, String?) -> Void)
    func pauseMonitoring(for projectPath: String, withReply: (Bool) -> Void)
    func resumeMonitoring(for projectPath: String, withReply: (Bool) -> Void)
    
    // History & Restore
    func getCommitHistory(for projectPath: String, limit: Int, withReply: ([[String: Any]]) -> Void)
    func restoreProject(_ projectPath: String, toCommit commitId: String, withReply: (Bool, String?) -> Void)
    
    // Status & Health
    func getStatus(withReply: ([String: Any]) -> Void)
    func ping(withReply: (Bool) -> Void)
}
```

**Usage Pattern**:
```swift
let client = OxenDaemonXPCClient()
if let proxy = client.getProxy() {
    proxy.registerProject("/path/to/project.logicx") { success, error in
        if success {
            // Project registered
        }
    }
}
```

---

## 4. SERVICE ARCHITECTURE & COMPONENT COMMUNICATION

### System Architecture

```
┌─────────────────────────────────────┐
│   User (Logic Pro)                  │
│   Editing project files             │
└──────────────┬──────────────────────┘
               │
        File System Changes
               │
               ▼
┌─────────────────────────────────────┐
│   FSEventsMonitor (Swift)           │
│   - Watches project folder          │
│   - Filters relevant files          │
│   - Debounces (30s threshold)       │
│   - Triggers callback on timeout    │
└──────────────┬──────────────────────┘
               │
        Commit Trigger
               │
               ▼
┌─────────────────────────────────────┐
│   CommitOrchestrator (Swift)        │
│   - Checks for changes              │
│   - Generates commit message        │
│   - Executes CLI commands           │
│   - Handles concurrent commits      │
└──────────────┬──────────────────────┘
               │
        Process Execution
               │
               ▼
┌─────────────────────────────────────┐
│   oxenvcs-cli (Rust)                │
│   - Stages changes (add --all)      │
│   - Creates commit                  │
│   - Updates draft branch            │
│   - Calls liboxen API               │
└──────────────┬──────────────────────┘
               │
        VCS Operations
               │
               ▼
┌─────────────────────────────────────┐
│   liboxen (Oxen Core)               │
│   - Block-level deduplication       │
│   - Repository management           │
│   - Commit storage                  │
└─────────────────────────────────────┘
```

### IPC Communication Flow

```
┌──────────────────┐           ┌─────────────────────────┐
│   OxVCS App      │           │   OxenDaemon            │
│   (UI - Phase 3) │           │   (Background Service)  │
└────────┬─────────┘           └────────┬────────────────┘
         │                              │
         │  XPC Request                 │
         │  (NSXPCConnection)           │
         │──────────────────────────────▶
         │                              │
         │  Mach Service:               │
         │  com.oxen.logic.daemon.xpc   │
         │                              │
         │                  Process Request
         │                  (CommitOrchestrator)
         │                              │
         │  XPC Reply                   │
         │◀──────────────────────────────
         │                              │
```

### Power Event Handling

```
macOS System Power Event
        │
        ▼
NSWorkspace Notification
- willSleepNotification
- willPowerOffNotification
        │
        ▼
PowerManagement.handlePowerEvent()
        │
        ├─► IOKit: Prevent System Sleep
        │
        ├─► CommitOrchestrator.performEmergencyCommits()
        │   - Check all registered projects
        │   - Execute auto-commit for each
        │   - Aggregate results
        │
        └─► IOKit: Allow System Sleep

Emergency Commit Rules:
- On shutdown: Always commit
- On sleep: Check battery (skip if <5%)
- Check system load before committing
```

### Daemon Lifecycle

```
1. System Login
   └─► launchd starts oxvcs-daemon (via plist)

2. Daemon Initialization (Daemon.swift)
   ├─► Initialize PowerManagement
   ├─► Start XPCService (listen on Mach service)
   ├─► Scan for Logic Pro projects
   └─► Start FSEventsMonitor for each project

3. Normal Operation
   ├─► Monitor file changes
   ├─► Debounce and trigger commits
   └─► Handle XPC requests from UI

4. Power Event
   └─► Emergency commit workflow

5. System Shutdown
   └─► Graceful shutdown (30s timeout)
```

---

## 5. LOCK & COLLABORATION MECHANISMS

### Current State: NOT YET IMPLEMENTED

**Status**: Mentioned in docs but no code exists

### Planned Design (Phase 3.2)

The implementation plan indicates:
- **Lock Manifest Schema** - Define file format for lock records
- **Lock Acquisition/Release** - API for obtaining/releasing locks
- **Lock Enforcement** - Daemon checks locks before allowing commits
- **Force-Break Mechanism** - Admin override for stuck locks

### Collaboration Protocol (Mentioned)

Per README:
- **Exclusive file locking** prevents binary merge conflicts
- **FCP XML export/import** for manual track reconciliation
- **Remote repository sync** via Oxen Hub

### Recommended Implementation Approach

For Phase 3, consider:

1. **Lock Manifest Format** (JSON in .oxen/locks/)
   ```json
   {
     "locks": [
       {
         "projectPath": "/path/to/project.logicx",
         "lockedBy": "user@hostname",
         "lockId": "uuid-here",
         "acquiredAt": "2025-01-01T12:00:00Z",
         "expiresAt": "2025-01-02T12:00:00Z",
         "force": false
       }
     ]
   }
   ```

2. **Lock Enforcement Points**
   - Before auto-commit: Check if project is locked by another user
   - Before manual commit: Similar check
   - Lock timeout (24h default) for stale locks
   - Force-break requires admin action

3. **Lock Manager** (suggest adding to CommitOrchestrator)
   - `acquireLock()` - Request exclusive lock
   - `releaseLock()` - Release lock
   - `checkLock()` - Verify lock status
   - `forceBreaLock()` - Admin override

---

## 6. TESTING SETUP

### Current State: NO TESTS IMPLEMENTED

### Existing Test Infrastructure

**Rust Unit Tests** (in source files):

1. **logic_project.rs**
   - `test_detect_invalid_extension` - Validates extension checking
   - `test_ignored_patterns` - Verifies ignore patterns

2. **ignore_template.rs**
   - `test_generate_oxenignore_contains_essential_patterns` - Pattern validation
   - `test_generate_oxenignore_has_sections` - Structure validation

3. **draft_manager.rs**
   - `test_constants` - Verifies branch name constants

### Run Tests
```bash
cd OxVCS-CLI-Wrapper
cargo test
```

### Recommended Testing Strategy for Phase 3

#### Unit Tests Needed
- UI components (views, view models)
- Lock manager operations
- XPC message handling
- Commit filtering logic

#### Integration Tests
- End-to-end project initialization
- File monitoring → commit pipeline
- Power event → emergency commit
- Lock acquisition/release workflow

#### System Tests
- Long-running sessions (8+ hours)
- Multi-user scenarios
- Power state transitions
- Daemon crash recovery
- Storage with large projects (50+ GB)

#### Test Tools
```bash
# Swift testing
swift test

# Rust testing
cargo test

# Integration testing
- Custom shell scripts for system tests
- FSEvents simulation
- Power event simulation (PowerManagement.simulateEvent)
- XPC connection testing
```

---

## 7. KEY FILE REFERENCE GUIDE

### Core Business Logic

| File | Language | Purpose | Key Functions |
|------|----------|---------|----------------|
| `oxen_ops.rs` | Rust | Oxen repository wrapper | `init_for_logic_project()`, `create_commit()`, `get_history()` |
| `draft_manager.rs` | Rust | Draft branch workflow | `auto_commit()`, `merge_to_main()`, `prune_if_needed()` |
| `commit_metadata.rs` | Rust | Commit format | `with_bpm()`, `with_sample_rate()`, `format_commit_message()` |
| `logic_project.rs` | Rust | Project detection | `detect()`, `tracked_paths()`, `ignored_patterns()` |

### Daemon Components

| File | Language | Purpose | Key Classes |
|------|----------|---------|-------------|
| `Daemon.swift` | Swift | Main coordinator | `OxenDaemon` |
| `FSEventsMonitor.swift` | Swift | File monitoring | `FSEventsMonitor` |
| `CommitOrchestrator.swift` | Swift | Auto-commit logic | `CommitOrchestrator` |
| `PowerManagement.swift` | Swift | Power events | `PowerManagement` |
| `XPCService.swift` | Swift | IPC | `OxenDaemonXPCService`, `OxenDaemonXPCClient` |
| `ServiceManager.swift` | Swift | Service registration | `ServiceManager` |

### Configuration

| File | Type | Purpose |
|------|------|---------|
| `com.oxen.logic.daemon.plist` | XML/Plist | LaunchAgent configuration |
| `Package.swift` | Swift | Daemon build config |
| `Cargo.toml` | TOML | CLI wrapper build config |

---

## 8. COMMAND REFERENCE

### CLI Commands (oxenvcs-cli)

```bash
# Initialize
oxenvcs-cli init --logic /path/to/project.logicx

# Staging
oxenvcs-cli add --all
oxenvcs-cli add /path/file1 /path/file2

# Commits
oxenvcs-cli commit \
  -m "Message" \
  --bpm 120 \
  --sample-rate 48000 \
  --key "C Major" \
  --tags "vocals,recording"

# History
oxenvcs-cli log --limit 5

# Restore
oxenvcs-cli restore <commit_id>

# Status
oxenvcs-cli status
```

### Daemon Management

```bash
# Install & register service
oxvcs-daemon --install

# Check status
oxvcs-daemon --status

# Uninstall
oxvcs-daemon --uninstall

# Verify configuration
oxvcs-daemon --verify

# Run as daemon (internal)
oxvcs-daemon --daemon

# Show version
oxvcs-daemon --version
```

---

## 9. NOTABLE IMPLEMENTATION DETAILS

### FSEvents Debounce Algorithm

```swift
1. FSEvents detects file change
2. Record event timestamp
3. Start/restart debounce timer (30s)
4. If new event arrives before timer expires:
   - Cancel timer
   - Restart timer
5. When timer expires:
   - Trigger commit callback
   - Reset timer
```

### Auto-Commit Workflow

```
1. FSEventsMonitor detects change
   ↓
2. Callback triggered: handleAutoCommit()
   ↓
3. Check if project paused (XPC)
   ↓
4. CommitOrchestrator.performCommit()
   ├─ Check for changes (status --porcelain)
   ├─ Stage changes (add --all)
   ├─ Create commit
   └─ Return result
   ↓
5. If success: Continue monitoring
   If fail: Log error, continue monitoring
```

### Emergency Commit Priority

```
Power Event Detected
├─ Prevent system sleep (IOKit assertion)
├─ Iterate all registered projects
│  ├─ Check battery level
│  ├─ Check system load
│  ├─ Execute commit
│  └─ Log result
└─ Allow system sleep
```

### XPC Error Handling

- Connection invalidation handled gracefully
- Automatic reconnection attempt in client
- Timeout protection on RPC calls
- Completion handlers prevent blocking

---

## 10. DEPENDENCIES

### Runtime Requirements
- **macOS 14.0+** (deployment target)
- **Logic Pro 11.x** (for testing)
- **Oxen CLI** (system-level installation)
- **Oxen.ai liboxen 0.19** (Rust crate)

### Build Requirements
- **Xcode 15+** with Swift 5.9+
- **Rust toolchain** (for CLI wrapper)
- **Cargo** (Rust package manager)
- **Swift Package Manager** (for daemon)

### Swift Dependencies
- Foundation (built-in)
- CoreServices (FSEvents)
- ServiceManagement (SMAppService)
- IOKit (power management)

### Rust Dependencies
- `liboxen = "0.19"` - Oxen VCS core
- `tokio = "1.0"` - Async runtime
- `serde` / `serde_json` - Serialization
- `clap = "4.0"` - CLI parsing
- `anyhow = "1.0"` - Error handling

---

## 11. INSTALLATION FLOW (Phase 2 Status)

### Manual Installation (Development)

```bash
# 1. Build CLI wrapper
cd OxVCS-CLI-Wrapper
cargo build --release
sudo cp target/release/oxenvcs-cli /usr/local/bin/

# 2. Build daemon
cd ../OxVCS-LaunchAgent
swift build -c release
sudo cp .build/release/oxvcs-daemon /usr/local/bin/

# 3. Install LaunchAgent plist
cp Resources/com.oxen.logic.daemon.plist ~/Library/LaunchAgents/

# 4. Register service
oxvcs-daemon --install

# 5. Verify
oxvcs-daemon --status
```

### Automatic Installation (App Distribution - Phase 3)

Will use `SMAppService` for user-friendly registration without manual plist copying.

---

## 12. PHASE 3 IMPLEMENTATION PRIORITIES

Based on current codebase analysis, Phase 3 should focus on:

### 3.1 Main UI Application
- **Repository Browser**: List projects, view history
- **Project Wizard**: Initialize new projects
- **Milestone Commits**: Manual commit interface with metadata
- **Rollback Interface**: Restore to previous states
- **Settings Panel**: Configure monitoring, daemon control

### 3.2 Exclusive File Locking System
- Design lock manifest format
- Implement lock manager in daemon
- Add lock checks in commit pipeline
- Build force-break mechanism

### 3.3 Manual Merge Protocol
- Document FCP XML reconciliation
- Build export/import helpers
- Test with divergent branches

### 3.4 Milestone Commit Pre-Flight
- Cleanup automation (remove bounces, freeze files)
- Confirmation dialog before commit
- Full staging → commit → push sequence

---

## SUMMARY

The oxen-vcs-logic project is well-structured with:
- ✅ Solid Phase 1 foundation (project detection, basic versioning)
- ✅ Complete Phase 2 service layer (daemon, auto-commits, power management)
- ❌ Phase 3 UI not started (ready for implementation)
- ❌ Locking system not implemented (design ready, awaiting code)
- ❌ Testing suite minimal (test framework in place)

The codebase is production-ready for the daemon layer and ready to integrate with a Phase 3 UI application. The XPC protocol is already defined and waiting for UI client implementation.

