# CLAUDE.md - Auxin System Prompt

**For**: AI assistants (Claude Code, Copilot, etc.)
**Purpose**: Provide essential context for code generation and project assistance
**Last Updated**: 2025-11-19

---

## Quick Reference

### Most Common Commands

```bash
# Run all tests
./run_all_tests.sh

# Build Rust CLI
cd Auxin-CLI-Wrapper && cargo build --release && cargo test

# Build Swift (macOS only)
cd Auxin-LaunchAgent && swift build
cd Auxin-App && swift build -c release && ./create-app-bundle.sh

# Lint and format
cd Auxin-CLI-Wrapper && cargo fmt && cargo clippy
```

### Critical Source Files

**Rust CLI** (`Auxin-CLI-Wrapper/src/`):
- `main.rs` - CLI entry point
- `oxen_subprocess.rs` - **CRITICAL**: Oxen integration with timeout/caching
- `config.rs` - ProjectType enum (Auto, LogicPro, SketchUp, Blender)
- `logic_project.rs` - Logic Pro detection
- `sketchup_project.rs` - SketchUp detection
- `commit_metadata.rs` - Structured metadata

**Swift LaunchAgent** (`Auxin-LaunchAgent/Sources/`):
- `Daemon.swift` - Main orchestration
- `FSEventsMonitor.swift` - File system monitoring
- `PowerManagement.swift` - Sleep/shutdown handling

**Swift App** (`Auxin-App/Sources/`):
- `ContentView.swift` - Main SwiftUI view
- `ProjectDetailContentView.swift` - Commit history

---

## Project Overview

**Auxin** is a macOS-native version control system for creative applications (Logic Pro, SketchUp, Blender).

### The Problem
- Creative projects have large binary files that cause Git bloat
- Binary project files cannot be merged
- Merge conflicts are catastrophic

### The Solution
- Block-level deduplication via Oxen.ai
- Pessimistic locking prevents conflicts
- Automatic draft commits with FSEvents
- Application-specific metadata

### Supported Applications
- **Logic Pro** (.logicx) - BPM, sample rate, key signature
- **SketchUp** (.skp) - Units, layers, components, groups
- **Blender** (.blend) - Scene metadata

---

## Architecture

```
┌─────────────┐      ┌──────────────┐      ┌────────┐
│  Auxin.app  │◄────►│LaunchAgent   │◄────►│  CLI   │
│ (SwiftUI)   │ XPC  │ (FSEvents)   │ Exec │ (Rust) │
└─────────────┘      └──────────────┘      └────────┘
                            │
                  ┌─────────▼────────┐
                  │  Oxen CLI        │
                  └──────────────────┘
```

### Components
1. **Auxin-App** - SwiftUI UI application
2. **Auxin-LaunchAgent** - Background daemon
3. **Auxin-CLI-Wrapper** - Rust CLI

---

## Project Status

**Last Updated**: 2025-11-19
**Test Suite**: 434 tests passing, 88% coverage

| Component | Status | Test Coverage |
|-----------|--------|---------------|
| Rust CLI | Production-ready | 88% |
| Swift LaunchAgent | Code complete | ~30% |
| Swift App | Code complete | <10% |

### Phase Completion

- Phase 1: Core CLI & Logic Pro - **100%**
- Phase 2: Background Services - **100%**
- Phase 3: GUI Application - **100%**
- Phase 4: Team Collaboration - **95%**
- Phase 5: 3D Modeling Support - **100%**
- Phase 6: Network Resilience - **0%** (planned)
- Phase 7: Auxin Server - **30%** (in progress)

---

## Technology Stack

### Languages
- **Rust 2021** - CLI wrapper
- **Swift 5.9+** - Daemon and app

### Key Dependencies

**Rust**: clap, tokio, serde, anyhow, chrono, colored, wait-timeout

**Swift**: SwiftUI, FSEvents, XPC, ServiceManagement

### Requirements
- macOS 14.0+
- Xcode 15+
- Rust stable
- Oxen CLI (`pip install oxen-ai`)

---

## Coding Conventions

### Rust Style

```rust
use anyhow::{Context, Result};

fn oxen_operation(path: &Path) -> Result<()> {
    Repository::open(path)
        .context("Failed to open repository")?;
    Ok(())
}
```

- Use `cargo fmt` and `cargo clippy`
- snake_case for functions/variables
- Document public APIs

### Swift Style

```swift
func commitChanges(message: String) -> Result<Commit, OxenError> {
    // Implementation
}
```

- Follow Swift API Design Guidelines
- camelCase for functions/variables
- Use protocols for dependency injection

### Commit Messages

```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

---

## Directory Structure

```
auxin/
├── Auxin-App/              # Swift/SwiftUI UI
│   ├── Sources/
│   │   ├── Views/
│   │   ├── ViewModels/
│   │   └── Services/
│   └── Tests/
│
├── Auxin-LaunchAgent/      # Background daemon
│   ├── Sources/
│   └── Tests/
│
├── Auxin-CLI-Wrapper/      # Rust CLI
│   ├── src/
│   └── tests/
│
├── docs/
│   ├── user/               # User documentation
│   ├── developer/          # Developer documentation
│   └── system/             # AI/system prompts
│
└── tests/                  # Integration tests
```

---

## Key Patterns

### Oxen Subprocess Pattern

```rust
pub fn execute(args: &[&str], timeout_ms: u64) -> Result<Output> {
    // Timeout handling, caching, error categorization
}
```

Configuration via environment:
- `AUXIN_TIMEOUT` - Default timeout (30000ms)
- `AUXIN_NETWORK_TIMEOUT` - Network timeout (120000ms)
- `AUXIN_CACHE_TTL` - Cache TTL (1000ms)

### FSEvents Pattern

```swift
class FSEventsMonitor {
    private var debounceTimer: Timer?
    private let debounceInterval: TimeInterval = 30.0

    func handleEvents() {
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
    func registerPowerNotifications() {
        let center = NSWorkspace.shared.notificationCenter
        center.addObserver(
            forName: NSWorkspace.willSleepNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.emergencyCommit(reason: "System sleep")
        }
    }
}
```

---

## Common Tasks

### Adding a New Application Type

1. Add variant to `ProjectType` in `config.rs`
2. Create `{app}_project.rs` for detection
3. Create `{app}_metadata.rs` for metadata
4. Add patterns to `ignore_template.rs`
5. Write tests

### Running Tests

```bash
# All tests
./run_all_tests.sh

# Rust with coverage
cd Auxin-CLI-Wrapper && cargo tarpaulin --out Html

# Specific test
cargo test test_name -- --nocapture
```

### Debugging

```bash
# Daemon logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h

# Daemon status
launchctl list | grep auxin

# Restart daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

---

## Known Constraints

### Platform
- **Current Dev Environment**: May be Linux (cannot compile Swift)
- **Required for Testing**: macOS 14.0+ with Xcode 15+

### Logic Pro
- Scripter environment blocks external filesystem access
- No way to hook into DAW save operations
- Solution: External monitoring via FSEvents

### Binary Files
- Project files are proprietary binary
- No algorithmic merge possible
- Solution: Pessimistic locking

---

## Documentation Links

### User Documentation
- [For Musicians](../user/for-musicians.md)
- [For Modelers](../user/for-modelers.md)
- [CLI Reference](../user/cli-reference.md)
- [Troubleshooting](../user/troubleshooting.md)

### Developer Documentation
- [Contributing](../developer/contributing.md)
- [Architecture](../developer/architecture.md)
- [Testing Strategy](../developer/testing.md)
- [Development Setup](../developer/development-setup.md)

### Project Management
- [Roadmap](../../ROADMAP.md)
- [Feature Status](../../FEATURE_STATUS.md)
- [Changelog](../../CHANGELOG.md)

---

## AI Assistant Guidelines

### When Working on This Codebase

1. **Review** the Quick Reference section first
2. **Check** Project Status for completion state
3. **Use** `./run_all_tests.sh` before committing
4. **Follow** coding conventions for the language

### Critical Files to Know

| Purpose | File |
|---------|------|
| Oxen integration | `oxen_subprocess.rs` |
| Project types | `config.rs` |
| Daemon main | `Daemon.swift` |
| Main UI | `ContentView.swift` |

### Testing Requirements

- All new features must include tests
- Run `cargo fmt` and `cargo clippy` for Rust
- Maintain 70-80% coverage overall

---

## Summary

**Auxin** is version control for creative applications:
- Rust CLI (production-ready, 88% coverage)
- Swift daemon (needs macOS testing)
- SwiftUI app (needs integration testing)

Key concepts:
- Block-level deduplication via Oxen
- Pessimistic locking (no merges)
- Automatic draft commits
- Application-specific metadata

Use this system prompt for quick context. Refer to linked documentation for detailed information.

---

*This is a condensed system prompt. For full details, see the linked documentation.*
