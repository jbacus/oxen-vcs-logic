# Auxin Architecture

This document describes the system architecture and design decisions of Auxin.

---

## Overview

Auxin is a three-component macOS application for version control of creative projects.

### Components

1. **Auxin-App** (Swift/SwiftUI) - Native macOS UI application
2. **Auxin-LaunchAgent** (Swift) - Background daemon for file monitoring
3. **Auxin-CLI-Wrapper** (Rust) - Command-line interface and Oxen integration

---

## System Diagram

```
┌─────────────────────────────────────────────────────────┐
│                    Auxin System                          │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐      ┌──────────────┐      ┌────────┐ │
│  │  Auxin.app  │◄────►│LaunchAgent   │◄────►│  CLI   │ │
│  │ (SwiftUI)   │ XPC  │ (FSEvents)   │ Exec │ (Rust) │ │
│  └─────────────┘      └──────────────┘      └────────┘ │
│                              │                            │
│                    ┌─────────▼────────┐                  │
│                    │  Oxen CLI        │                  │
│                    │  (subprocess)    │                  │
│                    └──────────────────┘                  │
└─────────────────────────────────────────────────────────┘
```

---

## Communication Patterns

### GUI ↔ Daemon (XPC)

The app communicates with the daemon via XPC (inter-process communication):

```swift
// Protocol definition
@objc protocol AuxinServiceProtocol {
    func executeCommit(message: String, reply: @escaping (Bool, Error?) -> Void)
    func stagePath(_ path: String, reply: @escaping (Bool, Error?) -> Void)
}
```

### Daemon ↔ CLI (Subprocess)

The daemon invokes CLI operations via subprocess execution:

```rust
pub fn execute(args: &[&str], timeout_ms: u64) -> Result<Output> {
    let child = Command::new("oxen")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    // ... timeout handling
}
```

---

## Design Principles

### 1. Separation of Concerns

Each component has a single responsibility:
- **GUI**: Presentation and user interaction
- **Daemon**: Automation and event handling
- **CLI**: VCS operations and Oxen integration

### 2. Oxen-First

All version control operations go through the Oxen subprocess wrapper:
- Consistent error handling
- Timeout management
- Output caching
- Batch processing

### 3. Binary-Aware

Auxin understands that creative project files cannot be merged:
- Pessimistic locking prevents conflicts
- No attempt at algorithmic merge
- Manual reconciliation via FCP XML export

### 4. Application-Specific

Each supported application has custom:
- Project detection logic
- Commit metadata extraction
- .oxenignore patterns

### 5. Fail-Safe

The system is designed to prevent data loss:
- Emergency commits before sleep/shutdown
- Graceful degradation if components fail
- Comprehensive error handling

---

## Component Details

### Auxin-CLI-Wrapper

**Language**: Rust 2021
**Location**: `Auxin-CLI-Wrapper/`

Key modules:
- `oxen_subprocess.rs` - Core Oxen CLI integration
- `config.rs` - ProjectType enum and configuration
- `logic_project.rs` - Logic Pro detection
- `sketchup_project.rs` - SketchUp detection
- `commit_metadata.rs` - Structured metadata

Features:
- Timeout handling (30s default, 120s network)
- Output caching (1s TTL)
- Automatic batching (1000 files/batch)
- Error categorization (retryable detection)

### Auxin-LaunchAgent

**Language**: Swift 5.9+
**Location**: `Auxin-LaunchAgent/`

Key files:
- `Daemon.swift` - Main orchestration
- `FSEventsMonitor.swift` - File system monitoring
- `PowerManagement.swift` - Sleep/shutdown handling
- `LockManager.swift` - Lock enforcement

Features:
- FSEvents monitoring with 30-60s debounce
- Power management hooks
- XPC service for app communication
- Multi-project support

### Auxin-App

**Language**: Swift 5.9+ / SwiftUI
**Location**: `Auxin-App/`

Key files:
- `ContentView.swift` - Main NavigationSplitView
- `ProjectListContentView.swift` - Sidebar
- `ProjectDetailContentView.swift` - Detail view
- `SwiftUIStatusBar.swift` - Status overlay

Features:
- Native macOS UI
- Project browser
- Commit history
- Status bar

---

## Data Flow

### Draft Commit Flow

```
1. User saves in Logic Pro / SketchUp / Blender
2. FSEvents detects file change
3. Debounce timer starts (30-60s)
4. Timer expires → CommitOrchestrator triggered
5. CLI executes: oxen add + oxen commit
6. Draft commit created on local branch
```

### Milestone Commit Flow

```
1. User clicks "Create Milestone" in app
2. App sends XPC request to daemon
3. Daemon invokes CLI:
   a. Delete volatile files (bounces, freezes)
   b. Stage all changes
   c. Commit with metadata
   d. Push to remote (if configured)
4. Response sent back to app
5. UI updates to show new commit
```

### Lock Acquisition Flow

```
1. User requests lock in app
2. App sends XPC request to daemon
3. Daemon invokes CLI: auxin lock acquire
4. CLI creates lock file in repository
5. If using remote, CLI pushes lock manifest
6. Daemon confirms lock acquired
7. App displays lock status
```

---

## Storage Model

### Repository Structure

```
MyProject.logicx/
├── .oxen/                 # Oxen repository data
│   ├── objects/           # Content-addressed storage
│   ├── refs/              # Branch references
│   └── lock               # Lock file
├── .oxenignore            # Ignore patterns
├── projectData            # Logic Pro project file
├── Resources/             # Audio files
└── Alternatives/          # Project alternatives
```

### Block-Level Deduplication

Oxen stores content at the block level:
- Only changed blocks are stored
- Identical blocks across files share storage
- 10-100x more efficient than file-level (Git-LFS)

---

## Error Handling

### Rust (CLI)

```rust
use anyhow::{Context, Result};

fn oxen_operation() -> Result<()> {
    something()
        .context("Failed during operation")?;
    Ok(())
}
```

### Swift (Daemon/App)

```swift
enum OxenError: Error {
    case operationFailed(String)
    case timeout
    case lockConflict(holder: String)
}

func performOperation() throws {
    guard success else {
        throw OxenError.operationFailed("Description")
    }
}
```

---

## Threading Model

### CLI Wrapper
- Single-threaded synchronous operations
- Timeout handling via wait-timeout crate

### LaunchAgent
- Main thread for FSEvents
- Background queues for commit operations
- Actor model for lock management

### App
- Main thread for UI
- Background tasks for XPC calls
- Combine for reactive updates

---

## Configuration

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `AUXIN_TIMEOUT` | 30000 | Operation timeout (ms) |
| `AUXIN_NETWORK_TIMEOUT` | 120000 | Network timeout (ms) |
| `AUXIN_CACHE_TTL` | 1000 | Cache TTL (ms) |

### Project Configuration

Per-project configuration in `.oxen/config`:
- Debounce interval
- Lock timeout
- Remote URL

---

## Security Considerations

### File Permissions
- Lock files use filesystem atomicity
- Daemon requires Full Disk Access
- App is sandboxed (if distributed via App Store)

### Network Security
- HTTPS for Oxen Hub communication
- Token-based authentication
- Lock manifests signed

---

## Performance Targets

| Operation | Target | Current |
|-----------|--------|---------|
| File add (<10MB) | <10ms | ~5ms |
| Commit (1GB) | <10s | ~8s |
| Log (1000 commits) | <500ms | ~300ms |
| Lock acquire | <100ms | ~50ms |
| Daemon CPU idle | <1% | <1% |
| Daemon memory | <50MB | ~30MB |

---

## Future Architecture

### Planned Changes

1. **Network Resilience** (Phase 6)
   - Offline commit queue
   - Retry with exponential backoff
   - Partial push recovery

2. **Auxin Server** (Phase 7)
   - Rust backend (Axum)
   - React frontend
   - WebSocket notifications

3. **AI Diffing** (Phase 8)
   - Audio feature extraction
   - Semantic change summaries

---

*Last Updated: 2025-11-19*
