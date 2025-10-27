# OxVCS Architecture Blueprint

## Table of Contents
1. [System Overview](#system-overview)
2. [Component Architecture](#component-architecture)
3. [Communication Stack](#communication-stack)
4. [Data Flow](#data-flow)
5. [File System Monitoring](#file-system-monitoring)
6. [Lock Management](#lock-management)
7. [Configuration Management](#configuration-management)
8. [Security Considerations](#security-considerations)
9. [Performance Optimization](#performance-optimization)
10. [Error Handling](#error-handling)

## System Overview

OxVCS is a comprehensive version control system for Logic Pro projects, consisting of three main components:

```
┌─────────────────────────────────────────────────────────────┐
│                     OxVCS System                             │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────────┐   ┌──────────────────┐   ┌──────────┐ │
│  │   GUI App       │   │  LaunchAgent     │   │ CLI      │ │
│  │   (Swift/AppKit)│◄─►│  (Swift/Daemon)  │◄─►│ (Rust)   │ │
│  └─────────────────┘   └──────────────────┘   └──────────┘ │
│         │                      │                     │       │
│         └──────────────────────┴─────────────────────┘       │
│                              │                                │
│                    ┌─────────▼─────────┐                     │
│                    │   Oxen VCS Core   │                     │
│                    │   (liboxen.a)     │                     │
│                    └───────────────────┘                     │
└─────────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: GUI, daemon, and CLI are independent modules
2. **MVVM Pattern**: Clean separation between view and business logic
3. **Asynchronous Communication**: XPC for inter-process communication
4. **Thread Safety**: Lock operations and file monitoring are thread-safe
5. **Fail-Safe**: Graceful degradation when components are unavailable

## Component Architecture

### 1. OxVCS-App (GUI Application)

**Technology**: Swift 5.9+, AppKit (macOS native UI)

**Structure**:
```
OxVCS-App/
├── Models/              # Data models
│   ├── Project.swift    # Project data structure
│   └── CommitInfo.swift # Commit metadata
├── ViewModels/          # MVVM business logic
│   ├── ProjectListViewModel.swift
│   └── ProjectDetailViewModel.swift
├── Views/               # UI components
│   ├── MainViewController.swift
│   ├── ProjectListView.swift
│   ├── ProjectDetailView.swift
│   ├── MilestoneCommitWindow.swift
│   ├── RollbackWindow.swift
│   ├── SettingsWindow.swift
│   ├── ProjectWizardWindow.swift
│   ├── MergeHelperWindow.swift
│   └── LockManagementView.swift
└── Services/            # External communication
    └── OxenDaemonXPCClient.swift
```

**Key Responsibilities**:
- User interaction and presentation
- Project browsing and status display
- Commit creation with metadata
- Rollback/restore interface
- Lock management UI
- Configuration management

**MVVM Architecture**:
```
┌─────────┐        ┌──────────────┐        ┌────────┐
│  View   │◄──────►│  ViewModel   │◄──────►│ Model  │
└─────────┘        └──────────────┘        └────────┘
     │                    │                      │
     │                    │                      │
     └────────────────────┴──────────────────────┘
                          │
                   ┌──────▼──────┐
                   │ XPC Client  │
                   └─────────────┘
```

### 2. OxVCS-LaunchAgent (Background Daemon)

**Technology**: Swift 5.9+, Foundation, FSEvents

**Structure**:
```
OxVCS-LaunchAgent/
├── OxVCSDaemon.swift       # Entry point
├── Daemon.swift            # Daemon lifecycle
├── FSEventsMonitor.swift   # File system monitoring
├── CommitOrchestrator.swift # Commit coordination
├── LockManager.swift       # File locking
├── PowerManagement.swift   # Sleep/shutdown handling
├── XPCService.swift        # IPC protocol
└── ServiceManager.swift    # Service coordination
```

**Key Responsibilities**:
- FSEvents-based file system monitoring
- Automatic commit orchestration
- Lock acquisition and management
- Power event handling
- XPC service endpoint
- Configuration storage

**Service Lifecycle**:
```
┌──────────────┐
│  launchd     │
└──────┬───────┘
       │ spawns
       ▼
┌──────────────┐
│   Daemon     │
└──────┬───────┘
       │ initializes
       ▼
┌──────────────┐     ┌──────────────┐     ┌──────────────┐
│ FSEvents     │     │ Lock Manager │     │ XPC Service  │
│ Monitor      │     │              │     │              │
└──────────────┘     └──────────────┘     └──────────────┘
```

### 3. OxVCS-CLI-Wrapper (Command-Line Tool)

**Technology**: Rust 1.70+

**Structure**:
```
OxVCS-CLI-Wrapper/
├── main.rs              # CLI entry point
├── lib.rs              # Library exports
├── oxen_ops.rs         # Oxen operations
├── logic_project.rs    # Logic Pro handling
├── commit_metadata.rs  # Metadata structures
├── draft_manager.rs    # Draft branch management
└── ignore_template.rs  # .oxenignore generation
```

**Key Responsibilities**:
- Project initialization
- Commit operations with metadata
- History retrieval
- Restore/rollback functionality
- Logic Pro project format handling
- Oxen VCS wrapper

## Communication Stack

### XPC (Cross-Process Communication)

**Mach Service Name**: `com.oxen.logic.daemon.xpc`

**Protocol Definition**:
```swift
@objc protocol OxenDaemonXPCProtocol {
    // Project Management
    func registerProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func unregisterProject(_ projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func getMonitoredProjects(withReply reply: @escaping ([String]) -> Void)

    // Commit Operations
    func commitProject(_ projectPath: String, message: String?, withReply reply: @escaping (String?, String?) -> Void)
    func getCommitHistory(for projectPath: String, limit: Int, withReply reply: @escaping ([[String: Any]]) -> Void)
    func restoreProject(_ projectPath: String, toCommit commitId: String, withReply reply: @escaping (Bool, String?) -> Void)

    // Monitoring Control
    func pauseMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void)
    func resumeMonitoring(for projectPath: String, withReply reply: @escaping (Bool) -> Void)

    // Lock Management
    func acquireLock(for projectPath: String, timeoutHours: Int, withReply reply: @escaping (Bool, String?) -> Void)
    func releaseLock(for projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func forceBreakLock(for projectPath: String, withReply reply: @escaping (Bool, String?) -> Void)
    func getLockInfo(for projectPath: String, withReply reply: @escaping ([String: Any]?) -> Void)

    // Configuration
    func getConfiguration(withReply reply: @escaping ([String: Any]) -> Void)
    func setDebounceTime(_ seconds: Int, withReply reply: @escaping (Bool) -> Void)
    func setLockTimeout(_ hours: Int, withReply reply: @escaping (Bool) -> Void)

    // Health Check
    func ping(withReply reply: @escaping (Bool) -> Void)
    func getStatus(withReply reply: @escaping ([String: Any]) -> Void)
}
```

**Communication Flow**:
```
GUI App                    Daemon
   │                          │
   │──registerProject()──────►│
   │                          │ validates
   │                          │ starts FSEvents
   │◄────reply(success)───────│
   │                          │
   │                          │ detects changes
   │                          │ debounce expires
   │                          │ performs commit
   │                          │
   │──getCommitHistory()─────►│
   │◄────reply(commits)───────│
```

## Data Flow

### Automatic Commit Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. User edits Logic Pro project                        │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 2. FSEvents detects file change                        │
│    - Triggers on .logicx package modifications         │
│    - Updates lastEventTime                             │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Debounce timer resets (default: 30s)               │
│    - Prevents commit on every keystroke                │
│    - Waits for editing session to complete             │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Debounce expires (no changes for 30s)              │
│    - CommitOrchestrator.performCommit() called         │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 5. CLI wrapper invoked                                 │
│    - oxenvcs-cli commit --project <path>               │
│    - Metadata extraction                               │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 6. Oxen VCS performs commit                            │
│    - Creates commit object                             │
│    - Updates branch pointer                            │
│    - Returns commit hash                               │
└─────────────────────────────────────────────────────────┘
```

### Manual Commit Flow (Milestone)

```
┌─────────────────────────────────────────────────────────┐
│ 1. User opens Milestone Commit window                  │
│    - Enters metadata (BPM, sample rate, etc.)          │
│    - Adds tags                                         │
│    - Optionally enables cleanup                        │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 2. GUI validates input                                 │
│    - Checks required fields                            │
│    - Validates numeric ranges                          │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 3. Pre-flight cleanup (optional)                       │
│    - Removes Bounces/ directory                        │
│    - Removes Freeze Files/ directory                   │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 4. XPC call to daemon                                  │
│    - commitProject(path, message)                      │
│    - Metadata passed as JSON                           │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 5. CLI wrapper with metadata                           │
│    - oxenvcs-cli commit --metadata <json>              │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 6. Commit stored with metadata tags                    │
│    - Searchable by BPM, key, etc.                      │
└─────────────────────────────────────────────────────────┘
```

## File System Monitoring

### FSEvents Integration

**Technology**: macOS FSEvents API (CoreServices framework)

**Configuration**:
```swift
private let debounceThreshold: TimeInterval = 30.0  // configurable
private let minCheckInterval: TimeInterval = 5.0
```

**Event Stream Setup**:
```swift
var context = FSEventStreamContext(
    version: 0,
    info: Unmanaged.passUnretained(self).toOpaque(),
    retain: nil,
    release: nil,
    copyDescription: nil
)

let stream = FSEventStreamCreate(
    nil,
    callback,
    &context,
    [watchedPath] as CFArray,
    FSEventStreamEventId(kFSEventStreamEventIdSinceNow),
    latency,
    FSEventStreamCreateFlags(kFSEventStreamCreateFlagFileEvents |
                             kFSEventStreamCreateFlagNoDefer)
)
```

**Event Handling**:
```
FSEvent Received
       │
       ▼
  Filter Events
  (ignore temp files)
       │
       ▼
  Update lastEventTime
       │
       ▼
  Reset Debounce Timer
       │
       ▼
  Wait for Debounce
       │
       ▼
  Trigger Commit
```

### Monitored File Patterns

**Included**:
- `*.logicx/` (Logic Pro project packages)
- `Alternatives/` (project alternatives)
- `Resources/` (audio files, samples)
- Project metadata files

**Excluded** (via `.oxenignore`):
```
# Temporary files
*.tmp
*.cache
*~

# macOS system files
.DS_Store
.AppleDouble
.LSOverride

# Lock files
.lock
*.lock

# Optional: Large temporary audio
Bounces/
Freeze Files/
```

## Lock Management

### Lock Data Structure

```swift
struct ProjectLock: Codable {
    let projectPath: String
    let lockedBy: String        // user@hostname
    let lockId: String          // UUID
    let acquiredAt: Date
    let expiresAt: Date

    var isExpired: Bool {
        return Date() > expiresAt
    }

    var remainingHours: Double {
        return expiresAt.timeIntervalSinceNow / 3600
    }
}
```

### Lock File Location

```
<project-path>/.oxen/lock.json
```

### Lock Acquisition Flow

```
┌─────────────────────────────────────────────────────────┐
│ 1. User requests lock via GUI                          │
│    - Specifies timeout (default: 24 hours)             │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 2. XPC call: acquireLock(projectPath, timeout)        │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 3. LockManager checks existing lock                    │
│    - Read .oxen/lock.json                              │
│    - Check if expired                                  │
└───────────────┬─────────────────────────────────────────┘
                ▼
        ┌───────┴───────┐
        │ Lock exists?  │
        └───┬───────┬───┘
            │       │
        NO  │       │ YES
            │       │
            │       ▼
            │   ┌─────────────┐
            │   │  Expired?   │
            │   └──┬──────┬───┘
            │      │      │
            │   NO │      │ YES
            │      │      │
            │      ▼      │
            │   ┌──────┐ │
            │   │REJECT│ │
            │   └──────┘ │
            │            │
            └────────────┘
                 │
                 ▼
┌─────────────────────────────────────────────────────────┐
│ 4. Create new lock                                     │
│    - Generate lock ID                                  │
│    - Set expiration time                               │
│    - Write lock.json                                   │
└───────────────┬─────────────────────────────────────────┘
                ▼
┌─────────────────────────────────────────────────────────┐
│ 5. Return success to GUI                               │
│    - Display lock status in UI                         │
└─────────────────────────────────────────────────────────┘
```

### Lock Release

**Normal Release**:
```
User → releaseLock(path) → LockManager
                             │
                             ▼
                    Verify ownership
                    (user@hostname)
                             │
                             ▼
                    Delete lock.json
                             │
                             ▼
                    Return success
```

**Force Break** (Admin):
```
Admin → forceBreakLock(path) → LockManager
                                  │
                                  ▼
                         Delete lock.json
                         (no ownership check)
                                  │
                                  ▼
                         Return success
```

### Lock Timeout

**Automatic Cleanup**:
- Expired locks are automatically ignored
- Cleanup happens on next lock acquisition attempt
- No background cleanup process (by design)

**Configurable Timeout**:
- Default: 24 hours
- Range: 1-168 hours (1 week)
- Stored in UserDefaults via XPC

## Configuration Management

### Configuration Storage

**Location**: `UserDefaults.standard`

**Keys**:
- `debounceTime`: Integer (5-300 seconds, default: 30)
- `lockTimeout`: Integer (1-168 hours, default: 24)

### Configuration Architecture

```
┌─────────────────┐
│ Settings Window │
└────────┬────────┘
         │ getConfiguration()
         ▼
┌─────────────────┐
│  XPC Client     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  XPC Service    │
│  (Daemon)       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ UserDefaults    │
│  - debounceTime │
│  - lockTimeout  │
└─────────────────┘
```

### Configuration Updates

**Set Debounce Time**:
```
User enters value → Validate (5-300) → XPC setDebounceTime()
                                              │
                                              ▼
                                    Save to UserDefaults
                                              │
                                              ▼
                          Apply to new FSEvents monitors
```

**Set Lock Timeout**:
```
User enters value → Validate (1-168) → XPC setLockTimeout()
                                              │
                                              ▼
                                    Save to UserDefaults
                                              │
                                              ▼
                          Apply to new lock acquisitions
```

## Security Considerations

### Lock Spoofing

**Current Implementation**:
- Lock owner identified by `user@hostname`
- Can be spoofed on local network
- No cryptographic verification

**Recommendations for Production**:
1. Use cryptographic signatures
2. Implement centralized lock server
3. Add TLS/mTLS for network communication
4. Use macOS Keychain for identity storage

### XPC Security

**Current Protection**:
- Mach service sandboxing
- Same-user restriction
- No authentication required

**Entitlements**:
```xml
<key>com.apple.security.files.user-selected.read-write</key>
<true/>
<key>com.apple.security.network.client</key>
<true/>
```

### File System Access

**Permissions Required**:
- Full Disk Access (for Logic Pro project folders)
- File system events monitoring

**Sandbox Considerations**:
- App must request user permission for project folders
- Bookmark URLs for persistent access

## Performance Optimization

### Debouncing Strategy

**Problem**: Frequent file changes during editing
**Solution**: 30-second debounce timer

**Impact**:
- Reduces commits from ~1000/session to ~5-10
- Prevents repository bloat
- Maintains usable commit history

### XPC Connection Pooling

**Current**: Single persistent connection
**Optimization**: Connection reuse with error recovery

```swift
connection.invalidationHandler = {
    print("Connection invalidated - reconnecting")
    self.setupConnection()
}
```

### FSEvents Latency

**Configuration**:
```swift
let latency: CFTimeInterval = 5.0  // 5 second aggregation
```

**Trade-off**:
- Lower latency = More CPU usage
- Higher latency = Delayed commit detection

### Memory Management

**Weak References**:
```swift
xpcClient.commitProject(path) { [weak self] success in
    self?.updateUI()
}
```

**Timer Cleanup**:
```swift
deinit {
    debounceTimer?.invalidate()
    refreshTimer?.invalidate()
}
```

## Error Handling

### XPC Communication Errors

**Error Types**:
1. Connection interrupted
2. Connection invalidated
3. Daemon not running
4. Method invocation timeout

**Recovery Strategy**:
```swift
guard let proxy = getProxy() else {
    completion(false)
    return
}

proxy.ping(withReply: { success in
    if !success {
        // Retry with exponential backoff
    }
})
```

### File System Errors

**Common Issues**:
- Permission denied
- File not found
- Disk full

**Handling**:
```swift
guard FileManager.default.fileExists(atPath: projectPath) else {
    reply(false, "Project not found at path")
    return
}

guard projectPath.hasSuffix(".logicx") else {
    reply(false, "Invalid Logic Pro project")
    return
}
```

### Lock Conflicts

**Scenario**: Two users try to acquire lock simultaneously

**Resolution**:
```swift
let lock = try FileManager.default.createFile(
    atPath: lockPath,
    contents: lockData,
    attributes: [.posixPermissions: 0o644]
)

// Atomic operation prevents race conditions
```

### Commit Failures

**Causes**:
- Oxen repository not initialized
- Network error (for remote repos)
- Disk space exhausted
- Corrupted repository

**Recovery**:
```
Commit Failed
     │
     ▼
  Retry (3x)
     │
     ▼
 Still Failed?
     │
     ▼
Show User Error
     │
     ▼
Pause Monitoring
```

## Future Enhancements

### 1. Remote Synchronization

**Proposed Architecture**:
```
Local Daemon → oxen push → Remote Server
                              │
                              ▼
                        Shared Repository
                              │
                              ▼
Remote Daemon ← oxen pull ← Remote Server
```

### 2. Real-Time Lock Notifications

**Current**: Polling (every 30 seconds)
**Proposed**: Push notifications via WebSocket

### 3. Visual Diff Viewer

**Challenge**: Binary project files
**Solution**: FCP XML comparison with visual highlighting

### 4. Automated Conflict Resolution

**Proposal**: ML-based merge suggestions
**Training Data**: Historical merge decisions

### 5. Multi-Window Support

**Current**: Single main window
**Proposed**: Separate windows per project

---

**Document Version**: 1.0
**Last Updated**: 2025-10-27
**Maintained By**: OxVCS Development Team
