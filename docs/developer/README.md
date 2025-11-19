# Auxin Developer Documentation

**For**: Contributors, maintainers, and developers extending Auxin

Welcome to the Auxin developer documentation. This guide will help you understand the codebase, contribute effectively, and extend Auxin for new applications.

---

## Quick Start for Contributors

```bash
# Clone repository
git clone https://github.com/jbacus/auxin.git
cd auxin

# Build all components
./install.sh

# Run tests
./run_all_tests.sh

# Or test individual components
cd Auxin-CLI-Wrapper && cargo test
cd Auxin-LaunchAgent && swift test
cd Auxin-App && swift test
```

---

## Documentation Guide

### Getting Started
- [Contributing Guide](contributing.md) - Code style, PR process, commit conventions
- [Development Setup](development-setup.md) - Environment configuration
- [Architecture Overview](architecture.md) - System design and component interaction

### Technical Reference
- [Testing Strategy](testing.md) - Testing approach, coverage targets
- [Extensibility Guide](extensibility.md) - Adding support for new applications
- [Roadmap](../../ROADMAP.md) - Project vision and phase timeline

### Component Documentation
Each component has its own README with specific details:
- [CLI Wrapper](../../Auxin-CLI-Wrapper/README.md) - Rust CLI documentation
- [LaunchAgent](../../Auxin-LaunchAgent/README.md) - Background daemon
- [App](../../Auxin-App/README.md) - SwiftUI application

---

## Architecture Overview

### Three-Component System

```
┌─────────────────────────────────────────────────────────┐
│                    Auxin Ecosystem                       │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐      ┌──────────────┐      ┌────────┐ │
│  │  Auxin.app  │◄────►│LaunchAgent   │◄────►│  CLI   │ │
│  │ (SwiftUI)   │ XPC  │ (FSEvents)   │ Exec │ (Rust) │ │
│  └─────────────┘      └──────────────┘      └────────┘ │
│         │                     │                    │     │
│         └─────────────────────┴────────────────────┘     │
│                              │                            │
│                    ┌─────────▼────────┐                  │
│                    │  Oxen CLI        │                  │
│                    │  (subprocess)    │                  │
│                    └──────────────────┘                  │
└─────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: GUI → Daemon → CLI
2. **Oxen-first**: All VCS operations through Oxen subprocess
3. **Binary-aware**: Never attempt algorithmic merge of binary files
4. **Pessimistic locking**: Prevent conflicts rather than resolve them
5. **Application-specific**: Metadata and ignore patterns per app type
6. **Power-safe**: Emergency commits before sleep/shutdown

---

## Technology Stack

### Languages
- **Swift 5.9+** - GUI app, background daemon
- **Rust 2021** - CLI wrapper

### Key Dependencies

**Swift**:
- SwiftUI - Declarative UI
- FSEvents - File system monitoring
- XPC - Inter-process communication
- ServiceManagement - LaunchAgent registration

**Rust**:
- clap 4.x - CLI parsing
- tokio - Async runtime
- serde - Serialization
- anyhow - Error handling

### Requirements
- macOS 14.0+
- Xcode 15+
- Rust stable toolchain
- Oxen CLI (`pip install oxen-ai`)

---

## Project Structure

```
auxin/
├── Auxin-App/              # Swift/SwiftUI UI application
│   ├── Sources/
│   │   ├── Views/          # SwiftUI views
│   │   ├── ViewModels/     # Business logic
│   │   └── Services/       # Oxen integration
│   └── Tests/
│
├── Auxin-LaunchAgent/      # Background monitoring daemon
│   ├── Sources/
│   │   ├── Daemon.swift    # Main orchestration
│   │   ├── FSEventsMonitor.swift
│   │   └── PowerManagement.swift
│   └── Tests/
│
├── Auxin-CLI-Wrapper/      # Rust CLI wrapper
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── oxen_subprocess.rs  # Core Oxen integration
│   │   └── config.rs       # ProjectType enum
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

## Key Files by Component

### CLI Wrapper (Rust)
| File | Purpose |
|------|---------|
| `oxen_subprocess.rs` | **Critical**: Oxen CLI integration with timeout, caching |
| `config.rs` | ProjectType enum (Auto, LogicPro, SketchUp, Blender) |
| `logic_project.rs` | Logic Pro detection and validation |
| `sketchup_project.rs` | SketchUp detection and validation |
| `commit_metadata.rs` | Structured commit metadata |

### LaunchAgent (Swift)
| File | Purpose |
|------|---------|
| `Daemon.swift` | Main daemon orchestration |
| `FSEventsMonitor.swift` | File system change monitoring |
| `PowerManagement.swift` | Sleep/shutdown event handling |
| `LockManager.swift` | File lock enforcement |

### App (Swift)
| File | Purpose |
|------|---------|
| `ContentView.swift` | Main NavigationSplitView |
| `ProjectListContentView.swift` | Project sidebar |
| `ProjectDetailContentView.swift` | Commit history display |

---

## Common Development Tasks

### Adding a New Application Type

See [Extensibility Guide](extensibility.md) for detailed instructions.

Summary:
1. Add variant to `ProjectType` enum in `config.rs`
2. Create `{app}_project.rs` for detection
3. Create `{app}_metadata.rs` for commit metadata
4. Add ignore patterns to `ignore_template.rs`
5. Write tests

### Running Tests

```bash
# All tests
./run_all_tests.sh

# Rust with coverage
cd Auxin-CLI-Wrapper
cargo tarpaulin --out Html

# Swift with coverage
cd Auxin-LaunchAgent
swift test --enable-code-coverage
```

### Debugging the Daemon

```bash
# View logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h

# Manual run with logging
cd Auxin-LaunchAgent && swift run

# Check status
launchctl list | grep auxin
```

---

## Code Style

### Swift
- Follow Swift API Design Guidelines
- Use meaningful variable names
- Maximum line length: 120 characters

### Rust
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Document public APIs

### Commits
Use conventional commit format:
```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

---

## Quality Standards

### Coverage Targets
- **Overall**: 70-80%
- **Critical paths** (locks, commits, power management): 90%+

### Performance Requirements
- Commit 1GB project: <10 seconds
- Lock acquisition: <100ms
- Daemon CPU usage: <5% average

### CI/CD
All tests run on:
- Push to `main`, `develop`, `claude/*`
- All pull requests

---

## Getting Help

- **Issues**: [GitHub Issues](https://github.com/jbacus/auxin/issues)
- **Architecture questions**: See [architecture.md](architecture.md)
- **API reference**: See component READMEs

---

*Last Updated: 2025-11-19*
