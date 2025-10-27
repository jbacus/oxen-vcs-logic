# OxVCS API Reference

## Table of Contents
1. [XPC Protocol API](#xpc-protocol-api)
2. [CLI Wrapper API](#cli-wrapper-api)
3. [GUI Application API](#gui-application-api)
4. [Data Models](#data-models)
5. [Error Codes](#error-codes)
6. [Usage Examples](#usage-examples)

## XPC Protocol API

The XPC protocol (`OxenDaemonXPCProtocol`) provides inter-process communication between the GUI application and the background daemon.

**Mach Service**: `com.oxen.logic.daemon.xpc`

### Project Management

#### registerProject

Registers a Logic Pro project for automatic monitoring.

```swift
func registerProject(
    _ projectPath: String,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Absolute path to `.logicx` project
- `reply`: Completion handler with success status and optional error message

**Returns**:
- `(true, nil)` on success
- `(false, "error message")` on failure

**Example**:
```swift
OxenDaemonXPCClient.shared.registerProject(path: "/path/to/project.logicx") { success in
    if success {
        print("Project registered successfully")
    }
}
```

**Validation**:
- Path must exist
- Path must end with `.logicx`
- Must be a valid Logic Pro project package

---

#### unregisterProject

Removes a project from monitoring.

```swift
func unregisterProject(
    _ projectPath: String,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project to unregister
- `reply`: Completion handler

**Example**:
```swift
OxenDaemonXPCClient.shared.unregisterProject(path: projectPath) { success, error in
    if let error = error {
        print("Error: \(error)")
    }
}
```

---

#### getMonitoredProjects

Retrieves list of all monitored projects.

```swift
func getMonitoredProjects(
    withReply reply: @escaping ([String]) -> Void
)
```

**Parameters**:
- `reply`: Completion handler with array of project paths

**Example**:
```swift
OxenDaemonXPCClient.shared.getMonitoredProjects { projects in
    print("Monitoring \(projects.count) projects")
}
```

---

### Commit Operations

#### commitProject

Triggers a manual commit for a project.

```swift
func commitProject(
    _ projectPath: String,
    message: String?,
    withReply reply: @escaping (String?, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `message`: Optional commit message
- `reply`: Completion handler with commit ID or error

**Returns**:
- `(commitHash, nil)` on success
- `(nil, "error message")` on failure

**Example**:
```swift
OxenDaemonXPCClient.shared.commitProject(
    path: projectPath,
    message: "Milestone: Final mix",
    metadata: ["bpm": 120, "key": "Am"]
) { success in
    if success {
        print("Commit created")
    }
}
```

---

#### getCommitHistory

Retrieves commit history for a project.

```swift
func getCommitHistory(
    for projectPath: String,
    limit: Int,
    withReply reply: @escaping ([[String: Any]]) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `limit`: Maximum number of commits to return
- `reply`: Array of commit dictionaries

**Commit Dictionary Structure**:
```swift
[
    "id": String,           // Commit hash
    "message": String,      // Commit message
    "timestamp": Date,      // Commit date
    "author": String,       // Author name
    "bpm": Int?,           // Optional: BPM
    "sampleRate": Int?,    // Optional: Sample rate
    "key": String?,        // Optional: Key signature
    "tags": [String]?      // Optional: Tags
]
```

**Example**:
```swift
OxenDaemonXPCClient.shared.getCommitHistory(path: projectPath, limit: 10) { commits in
    for commit in commits {
        print("Commit: \(commit["id"]) - \(commit["message"])")
    }
}
```

---

#### restoreProject

Restores a project to a specific commit.

```swift
func restoreProject(
    _ projectPath: String,
    toCommit commitId: String,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `commitId`: Commit hash to restore
- `reply`: Completion handler

**Example**:
```swift
OxenDaemonXPCClient.shared.restoreProject(
    path: projectPath,
    commitHash: "a1b2c3d4"
) { success in
    if success {
        print("Project restored")
    }
}
```

**Warning**: This operation is destructive. Current changes will be lost.

---

### Monitoring Control

#### pauseMonitoring

Pauses automatic commits for a project.

```swift
func pauseMonitoring(
    for projectPath: String,
    withReply reply: @escaping (Bool) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `reply`: Success status

**Example**:
```swift
OxenDaemonXPCClient.shared.pauseMonitoring(for: projectPath) { success in
    if success {
        print("Monitoring paused")
    }
}
```

---

#### resumeMonitoring

Resumes automatic commits for a project.

```swift
func resumeMonitoring(
    for projectPath: String,
    withReply reply: @escaping (Bool) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `reply`: Success status

---

### Lock Management

#### acquireLock

Acquires an exclusive lock for a project.

```swift
func acquireLock(
    for projectPath: String,
    timeoutHours: Int,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `timeoutHours`: Lock timeout in hours (1-168)
- `reply`: Success status and error message

**Example**:
```swift
OxenDaemonXPCClient.shared.acquireLock(for: projectPath, timeoutHours: 24) { success, error in
    if success {
        print("Lock acquired")
    } else {
        print("Failed: \(error ?? "unknown error")")
    }
}
```

**Error Messages**:
- "Project is already locked by user@hostname"
- "Failed to acquire lock"

---

#### releaseLock

Releases a lock for a project.

```swift
func releaseLock(
    for projectPath: String,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `reply`: Success status and error message

**Example**:
```swift
OxenDaemonXPCClient.shared.releaseLock(for: projectPath) { success, error in
    if success {
        print("Lock released")
    }
}
```

**Requirements**:
- Caller must own the lock (same user@hostname)

---

#### forceBreakLock

Force-breaks a lock (admin operation).

```swift
func forceBreakLock(
    for projectPath: String,
    withReply reply: @escaping (Bool, String?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `reply`: Success status and error message

**Warning**: Use with caution. No ownership verification.

---

#### getLockInfo

Retrieves lock information for a project.

```swift
func getLockInfo(
    for projectPath: String,
    withReply reply: @escaping ([String: Any]?) -> Void
)
```

**Parameters**:
- `projectPath`: Path to project
- `reply`: Lock info dictionary or nil if not locked

**Lock Info Structure**:
```swift
[
    "projectPath": String,
    "lockedBy": String,        // user@hostname
    "lockId": String,          // UUID
    "acquiredAt": String,      // ISO8601 date
    "expiresAt": String,       // ISO8601 date
    "isExpired": Bool,
    "remainingHours": Double,
    "isLocked": Bool
]
```

**Example**:
```swift
OxenDaemonXPCClient.shared.getLockInfo(for: projectPath) { lockInfo in
    if let info = lockInfo {
        print("Locked by: \(info["lockedBy"])")
        print("Expires in: \(info["remainingHours"]) hours")
    } else {
        print("Project is not locked")
    }
}
```

---

### Configuration Management

#### getConfiguration

Retrieves current daemon configuration.

```swift
func getConfiguration(
    withReply reply: @escaping ([String: Any]) -> Void
)
```

**Parameters**:
- `reply`: Configuration dictionary

**Configuration Structure**:
```swift
[
    "debounceTime": Int,   // Seconds (5-300)
    "lockTimeout": Int     // Hours (1-168)
]
```

**Example**:
```swift
OxenDaemonXPCClient.shared.getConfiguration { config in
    let debounce = config["debounceTime"] as? Int ?? 30
    print("Debounce time: \(debounce)s")
}
```

---

#### setDebounceTime

Sets the debounce time for auto-commits.

```swift
func setDebounceTime(
    _ seconds: Int,
    withReply reply: @escaping (Bool) -> Void
)
```

**Parameters**:
- `seconds`: Debounce time (5-300 seconds)
- `reply`: Success status

**Example**:
```swift
OxenDaemonXPCClient.shared.setDebounceTime(60) { success in
    if success {
        print("Debounce time updated to 60s")
    }
}
```

**Note**: Applies to newly registered projects only.

---

#### setLockTimeout

Sets the default lock timeout.

```swift
func setLockTimeout(
    _ hours: Int,
    withReply reply: @escaping (Bool) -> Void
)
```

**Parameters**:
- `hours`: Lock timeout (1-168 hours)
- `reply`: Success status

**Example**:
```swift
OxenDaemonXPCClient.shared.setLockTimeout(48) { success in
    if success {
        print("Lock timeout updated to 48 hours")
    }
}
```

**Note**: Applies to newly acquired locks only.

---

### System Health

#### ping

Tests daemon connectivity.

```swift
func ping(
    withReply reply: @escaping (Bool) -> Void
)
```

**Parameters**:
- `reply`: Health status (true = daemon running)

**Example**:
```swift
OxenDaemonXPCClient.shared.ping { isRunning in
    if isRunning {
        print("Daemon is healthy")
    }
}
```

---

#### getStatus

Retrieves detailed daemon status.

```swift
func getStatus(
    withReply reply: @escaping ([String: Any]) -> Void
)
```

**Parameters**:
- `reply`: Status dictionary

**Status Structure**:
```swift
[
    "isRunning": Bool,
    "projectCount": Int,
    "pausedCount": Int,
    "version": String,
    "uptime": Double          // Seconds
]
```

**Example**:
```swift
OxenDaemonXPCClient.shared.getStatus { status in
    print("Monitoring \(status["projectCount"]) projects")
    print("Uptime: \(status["uptime"]) seconds")
}
```

---

## CLI Wrapper API

The Rust-based CLI tool (`oxenvcs-cli`) provides command-line access to Oxen operations.

### Commands

#### init

Initializes a new Oxen repository for a Logic Pro project.

```bash
oxenvcs-cli init --project <PROJECT_PATH>
```

**Arguments**:
- `--project`: Path to .logicx project (required)

**Example**:
```bash
oxenvcs-cli init --project /Users/producer/Music/MySong.logicx
```

**Operations**:
1. Validates Logic Pro project structure
2. Initializes Oxen repository
3. Creates `.oxenignore` with recommended exclusions
4. Commits initial state

---

#### commit

Creates a commit with optional metadata.

```bash
oxenvcs-cli commit --project <PATH> [OPTIONS]
```

**Arguments**:
- `--project`: Path to project (required)
- `--message`: Commit message (optional)
- `--bpm`: Beats per minute (optional)
- `--sample-rate`: Sample rate in Hz (optional)
- `--key`: Key signature (optional)
- `--time-signature`: Time signature (optional)
- `--tags`: Comma-separated tags (optional)

**Example**:
```bash
oxenvcs-cli commit \
  --project /Users/producer/Music/MySong.logicx \
  --message "Final mix" \
  --bpm 120 \
  --sample-rate 48000 \
  --key "Am" \
  --time-signature "4/4" \
  --tags "mix,final,release"
```

**Metadata Storage**:
Metadata is stored as Oxen tags and searchable via `oxen log`.

---

#### history

Retrieves commit history.

```bash
oxenvcs-cli history --project <PATH> [--limit <NUM>]
```

**Arguments**:
- `--project`: Path to project (required)
- `--limit`: Maximum commits to return (default: 10)

**Example**:
```bash
oxenvcs-cli history --project /Users/producer/Music/MySong.logicx --limit 5
```

**Output**:
```
Commit: a1b2c3d4e5f6
Date: 2025-10-27 14:30:00
Message: Final mix
BPM: 120
Key: Am
Tags: mix, final, release

Commit: b2c3d4e5f6a7
Date: 2025-10-26 10:15:00
Message: Added vocals
...
```

---

#### restore

Restores project to a specific commit.

```bash
oxenvcs-cli restore --project <PATH> --commit <HASH>
```

**Arguments**:
- `--project`: Path to project (required)
- `--commit`: Commit hash to restore (required)

**Example**:
```bash
oxenvcs-cli restore \
  --project /Users/producer/Music/MySong.logicx \
  --commit a1b2c3d4e5f6
```

**Warning**: Destructive operation. Current changes will be lost.

---

#### status

Shows current project status.

```bash
oxenvcs-cli status --project <PATH>
```

**Arguments**:
- `--project`: Path to project (required)

**Example**:
```bash
oxenvcs-cli status --project /Users/producer/Music/MySong.logicx
```

**Output**:
```
Project: MySong.logicx
Branch: main
Status: Modified
Uncommitted changes: 5 files
Last commit: a1b2c3d4e5f6 (2 hours ago)
```

---

## GUI Application API

### ViewModels

#### ProjectListViewModel

Manages the list of monitored projects.

**Properties**:
```swift
@Published var projects: [Project] = []
@Published var isLoading: Bool = false
@Published var errorMessage: String?
@Published var daemonStatus: DaemonStatus?
```

**Methods**:
```swift
// Load all projects from daemon
func loadProjects()

// Add a new project to monitoring
func addProject(path: String, completion: @escaping (Bool) -> Void)

// Check daemon health status
func checkDaemonStatus()
```

**Example**:
```swift
let viewModel = ProjectListViewModel()
viewModel.loadProjects()

// Observe changes
viewModel.$projects.sink { projects in
    print("Loaded \(projects.count) projects")
}
```

---

#### ProjectDetailViewModel

Manages details for a single project.

**Properties**:
```swift
@Published var commitHistory: [CommitInfo] = []
@Published var isLocked: Bool = false
@Published var lockedBy: String?
```

**Methods**:
```swift
func loadCommitHistory(limit: Int = 10)
func createCommit(message: String, metadata: CommitMetadata)
func restoreToCommit(_ commitHash: String)
func acquireLock(timeoutHours: Int)
func releaseLock()
```

---

### Windows

#### MilestoneCommitWindow

Window for creating milestone commits with metadata.

**Methods**:
```swift
func show()
func performCommit()
```

**Metadata Fields**:
- BPM (integer, optional)
- Sample Rate (integer, optional)
- Key Signature (string, optional)
- Time Signature (string, optional)
- Tags (array of strings, optional)
- Enable Cleanup (boolean)

---

#### SettingsWindow

Window for daemon configuration.

**Methods**:
```swift
func show()
func loadConfiguration()
func saveConfiguration()
func updateDaemonStatus()
```

**Configuration Fields**:
- Debounce Time (5-300 seconds)
- Lock Timeout (1-168 hours)

---

## Data Models

### Project

Represents a monitored Logic Pro project.

```swift
struct Project: Identifiable {
    let id: UUID
    let path: String
    let name: String
    var isMonitored: Bool
    var lastCommit: Date?
    var commitCount: Int
    var isLocked: Bool
    var lockedBy: String?
}
```

---

### CommitInfo

Represents a commit in the history.

```swift
struct CommitInfo: Identifiable {
    let id: String              // Commit hash
    let message: String
    let timestamp: Date
    let author: String
    var bpm: Int?
    var sampleRate: Int?
    var key: String?
    var timeSignature: String?
    var tags: [String]?
}
```

---

### DaemonStatus

Represents daemon health status.

```swift
struct DaemonStatus {
    let isRunning: Bool
    let monitoredProjectCount: Int
    let lastActivity: Date
}
```

---

### ProjectLock

Represents a project lock.

```swift
struct ProjectLock: Codable {
    let projectPath: String
    let lockedBy: String        // user@hostname
    let lockId: String          // UUID
    let acquiredAt: Date
    let expiresAt: Date

    var isExpired: Bool
    var remainingHours: Double
}
```

---

## Error Codes

### XPC Errors

| Code | Description | Recovery |
|------|-------------|----------|
| `XPC_CONNECTION_INVALID` | Connection to daemon lost | Retry connection |
| `XPC_METHOD_TIMEOUT` | Method call timed out | Retry with backoff |
| `XPC_DAEMON_NOT_RUNNING` | Daemon is not running | Check launchd status |

---

### CLI Errors

| Exit Code | Description |
|-----------|-------------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Project not found |
| 3 | Not an Oxen repository |
| 4 | Commit failed |
| 5 | Restore failed |

---

### Lock Errors

| Error | Description |
|-------|-------------|
| `LOCK_ALREADY_HELD` | Project is locked by another user |
| `LOCK_NOT_OWNED` | Cannot release lock (not owner) |
| `LOCK_EXPIRED` | Lock has expired |
| `LOCK_INVALID` | Lock file is corrupted |

---

## Usage Examples

### Complete Workflow Example

```swift
// 1. Initialize ViewModel
let viewModel = ProjectListViewModel()

// 2. Add a project
viewModel.addProject(path: "/path/to/project.logicx") { success in
    if success {
        print("Project added")
    }
}

// 3. Load projects
viewModel.loadProjects()

// 4. Acquire lock
OxenDaemonXPCClient.shared.acquireLock(for: projectPath, timeoutHours: 24) { success, error in
    if success {
        print("Lock acquired - safe to edit")
    }
}

// 5. Create milestone commit
let metadata = CommitMetadata(
    bpm: 120,
    sampleRate: 48000,
    key: "Am",
    timeSignature: "4/4",
    tags: ["final", "mix"]
)

OxenDaemonXPCClient.shared.commitProject(
    path: projectPath,
    message: "Final mix",
    metadata: metadata
) { success in
    if success {
        print("Milestone commit created")
    }
}

// 6. Release lock
OxenDaemonXPCClient.shared.releaseLock(for: projectPath) { success, _ in
    if success {
        print("Lock released")
    }
}
```

---

### CLI Integration Example

```bash
#!/bin/bash

# Initialize project
oxenvcs-cli init --project MySong.logicx

# Create commits with metadata
oxenvcs-cli commit \
  --project MySong.logicx \
  --message "Initial arrangement" \
  --bpm 120 \
  --tags "demo,draft"

# Check status
oxenvcs-cli status --project MySong.logicx

# View history
oxenvcs-cli history --project MySong.logicx --limit 10

# Restore to previous version
oxenvcs-cli restore \
  --project MySong.logicx \
  --commit a1b2c3d4e5f6
```

---

### Configuration Management Example

```swift
// Load current configuration
OxenDaemonXPCClient.shared.getConfiguration { config in
    let currentDebounce = config["debounceTime"] as? Int ?? 30
    print("Current debounce: \(currentDebounce)s")
}

// Update debounce time
OxenDaemonXPCClient.shared.setDebounceTime(60) { success in
    if success {
        print("Debounce time updated")
    }
}

// Update lock timeout
OxenDaemonXPCClient.shared.setLockTimeout(48) { success in
    if success {
        print("Lock timeout updated")
    }
}
```

---

### Lock Management Example

```swift
// Check lock status
OxenDaemonXPCClient.shared.getLockInfo(for: projectPath) { lockInfo in
    if let info = lockInfo {
        let isLocked = info["isLocked"] as? Bool ?? false
        let lockedBy = info["lockedBy"] as? String ?? "unknown"
        let remainingHours = info["remainingHours"] as? Double ?? 0

        if isLocked {
            print("Locked by \(lockedBy) for \(remainingHours) more hours")
        }
    } else {
        print("Project is available")
    }
}

// Acquire lock with custom timeout
OxenDaemonXPCClient.shared.acquireLock(for: projectPath, timeoutHours: 48) { success, error in
    if success {
        print("Lock acquired for 48 hours")
    } else {
        print("Failed: \(error ?? "unknown error")")
    }
}

// Force break lock (admin only)
OxenDaemonXPCClient.shared.forceBreakLock(for: projectPath) { success, _ in
    if success {
        print("Lock forcefully broken")
    }
}
```

---

## API Versioning

**Current Version**: 1.0

**Compatibility Promise**:
- Major version changes indicate breaking API changes
- Minor version changes are backward compatible
- Patch versions are bug fixes only

**Deprecation Policy**:
- Deprecated methods will be marked with `@available` attributes
- Minimum 2 minor versions before removal
- Migration guides provided for breaking changes

---

**Document Version**: 1.0
**Last Updated**: 2025-10-27
**Maintained By**: OxVCS Development Team
