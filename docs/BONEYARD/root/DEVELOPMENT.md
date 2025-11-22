# Auxin Development Guide

<div align="center">
  <img src="assets/icon/icon-128.png" alt="Auxin Logo" width="80" height="80">
  <h2>Contributing to Auxin</h2>
</div>

**For**: Contributors, maintainers, and developers extending Auxin

This guide will help you set up your development environment, understand the codebase, contribute effectively, and extend Auxin for new applications.

---

## Quick Start for Contributors

```bash
# 1. Clone repository
git clone https://github.com/jbacus/auxin.git
cd auxin

# 2. Install dependencies
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Oxen CLI
pip install oxen-ai

# 3. Build all components
./install.sh

# 4. Run tests
./run_all_tests.sh

# Or test individual components
cd Auxin-CLI-Wrapper && cargo test --all-features
cd Auxin-LaunchAgent && swift test
cd Auxin-App && swift test
```

**Prerequisites**:
- macOS 14.0+ (for Swift components)
- Xcode 15+ (for Swift/SwiftUI development)
- Rust stable toolchain (for CLI wrapper)
- Oxen CLI (`pip install oxen-ai`)

---

## Documentation Index

### Essential Reading

1. **[Contributing Guide](docs/developer/contributing.md)** ⭐ START HERE
   - Code style and conventions
   - Pull request process
   - Commit message format
   - Review guidelines

2. **[Development Setup](docs/developer/development-setup.md)**
   - Detailed environment configuration
   - IDE setup (Xcode, VS Code, CLion)
   - Debugging tips
   - Build troubleshooting

3. **[Architecture Overview](docs/developer/architecture.md)**
   - System design and components
   - Data flow and communication
   - Design principles
   - Technology stack

### Technical Reference

4. **[Testing Strategy](docs/developer/testing.md)**
   - Unit tests, integration tests, E2E tests
   - Coverage targets (70-80% overall)
   - Running tests and generating reports
   - CI/CD integration

5. **[Configuration System](docs/developer/configuration.md)**
   - TOML-based unified configuration
   - Environment variables
   - Project-specific settings
   - Docker configuration

6. **[API Reference](docs/developer/api-reference.md)**
   - REST API endpoints
   - WebSocket protocol
   - Authentication
   - Rate limiting

7. **[Extensibility Guide](docs/developer/extensibility.md)** 🔌
   - Adding support for new applications
   - Creating project type detectors
   - Implementing metadata extractors
   - Custom ignore patterns

### Project Planning

8. **[Roadmap](ROADMAP.md)** - Project vision and phase timeline
9. **[Feature Status](FEATURE_STATUS.md)** - Current implementation status
10. **[Next Steps](NEXT_STEPS.md)** - Current sprint and immediate tasks
11. **[Changelog](CHANGELOG.md)** - Version history and release notes

### CI/CD

12. **[CI/CD Guide](.github/CI_CD_GUIDE.md)**
    - GitHub Actions workflows
    - Automated testing
    - Security scanning
    - Code coverage tracking
    - Performance benchmarks
    - Dependency updates (Dependabot)

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

1. **Separation of Concerns**: GUI → Daemon → CLI → Oxen
2. **Oxen-first**: All VCS operations through Oxen subprocess (never direct)
3. **Binary-aware**: Never attempt algorithmic merge of binary files
4. **Pessimistic locking**: Prevent conflicts rather than resolve them
5. **Application-specific**: Custom metadata and ignore patterns per app type
6. **Power-safe**: Emergency commits before sleep/shutdown to prevent data loss

---

## Technology Stack

### Languages

- **Swift 5.9+** - GUI app, background daemon, macOS integration
- **Rust 2021** - CLI wrapper, performance-critical operations
- **TypeScript/React** - Server web dashboard (auxin-server frontend)
- **Python** - Build scripts, utilities

### Key Dependencies

**Swift**:
- SwiftUI - Declarative UI framework
- FSEvents - File system change monitoring
- XPC - Inter-process communication
- ServiceManagement - LaunchAgent registration
- IOKit - Power management

**Rust**:
- clap 4.x - CLI parsing and argument handling
- tokio - Async runtime
- serde - Serialization/deserialization
- anyhow - Error handling
- tempfile - Temporary directory management

**Server (Rust)**:
- actix-web - Web framework
- tokio - Async runtime
- serde_json - JSON handling
- bcrypt - Password hashing
- jsonwebtoken - JWT authentication

---

## Project Structure

```
auxin/
├── Auxin-App/              # Swift/SwiftUI UI application
│   ├── Sources/
│   │   ├── Views/          # SwiftUI views
│   │   ├── ViewModels/     # Business logic
│   │   ├── Services/       # XPC, Oxen integration
│   │   └── Models/         # Data models
│   ├── Tests/              # Unit and UI tests
│   └── README.md
│
├── Auxin-LaunchAgent/      # Background monitoring daemon
│   ├── Sources/
│   │   ├── Daemon.swift              # Main orchestration
│   │   ├── FSEventsMonitor.swift     # File change detection
│   │   ├── PowerManagement.swift     # Sleep/shutdown handling
│   │   ├── LockManager.swift         # File locking
│   │   ├── NetworkMonitor.swift      # Network status
│   │   └── XPCService.swift          # IPC protocol
│   ├── Tests/              # Comprehensive test suite (216 tests)
│   └── README.md
│
├── Auxin-CLI-Wrapper/      # Rust CLI wrapper
│   ├── src/
│   │   ├── main.rs                   # CLI entry point
│   │   ├── oxen_subprocess.rs        # Core Oxen integration
│   │   ├── config.rs                 # ProjectType enum
│   │   ├── logic_project.rs          # Logic Pro detection
│   │   ├── sketchup_project.rs       # SketchUp detection
│   │   ├── commit_metadata.rs        # Structured metadata
│   │   └── ignore_template.rs        # .auxinignore patterns
│   ├── tests/              # Integration tests (88% coverage)
│   └── README.md
│
├── auxin-server/           # Collaboration server (Rust/Actix)
│   ├── src/
│   │   ├── main.rs         # Server entry point
│   │   ├── api/            # REST API endpoints
│   │   ├── auth.rs         # Authentication
│   │   └── websocket.rs    # Real-time updates
│   ├── frontend/           # React dashboard
│   ├── tests/              # API integration tests
│   └── docs/
│       ├── api/            # OpenAPI spec
│       └── deployment/     # Production guides
│
├── auxin-config/           # Unified configuration crate
│   ├── src/
│   │   └── lib.rs          # TOML-based config
│   └── README.md
│
├── docs/
│   ├── user/               # User documentation
│   ├── developer/          # Developer documentation ⭐
│   └── system/             # AI/system prompts
│
├── .github/
│   ├── workflows/          # CI/CD pipelines
│   └── CI_CD_GUIDE.md      # Workflow documentation
│
└── tests/                  # End-to-end test scripts
    ├── user-guide-scenarios/
    └── E2E-TEST-README.md
```

---

## Key Files by Component

### CLI Wrapper (Rust)

| File | Purpose | Criticality |
|------|---------|-------------|
| `oxen_subprocess.rs` | Oxen CLI integration with timeout, caching | ⚠️ CRITICAL |
| `config.rs` | ProjectType enum (Auto, LogicPro, SketchUp, Blender) | High |
| `logic_project.rs` | Logic Pro detection and validation | High |
| `sketchup_project.rs` | SketchUp detection and validation | Medium |
| `commit_metadata.rs` | Structured commit metadata (BPM, key, etc.) | Medium |
| `ignore_template.rs` | Default .auxinignore patterns | Medium |

### LaunchAgent (Swift)

| File | Purpose | Criticality |
|------|---------|-------------|
| `Daemon.swift` | Main daemon orchestration | ⚠️ CRITICAL |
| `FSEventsMonitor.swift` | File system change monitoring | ⚠️ CRITICAL |
| `PowerManagement.swift` | Sleep/shutdown event handling | ⚠️ CRITICAL |
| `LockManager.swift` | File lock enforcement | High |
| `XPCService.swift` | IPC protocol definition | High |
| `NetworkMonitor.swift` | Network connectivity detection | Medium |

### App (Swift)

| File | Purpose | Criticality |
|------|---------|-------------|
| `ContentView.swift` | Main NavigationSplitView | High |
| `ProjectListContentView.swift` | Project sidebar | Medium |
| `ProjectDetailContentView.swift` | Commit history display | Medium |
| `OxenService.swift` | Oxen CLI integration | High |

### Server (Rust)

| File | Purpose | Criticality |
|------|---------|-------------|
| `api/repo_ops.rs` | Repository operations API | High |
| `api/project_ops.rs` | Project management API | High |
| `auth.rs` | JWT authentication | ⚠️ CRITICAL |
| `websocket.rs` | Real-time status updates | Medium |

---

## Common Development Tasks

### Adding a New Application Type

See **[Extensibility Guide](docs/developer/extensibility.md)** for detailed step-by-step instructions.

**Summary**:

1. **Add ProjectType variant** in `Auxin-CLI-Wrapper/src/config.rs`:
   ```rust
   pub enum ProjectType {
       Auto,
       LogicPro,
       SketchUp,
       Blender,
       YourApp,  // Add here
   }
   ```

2. **Create detection module** `src/{yourapp}_project.rs`:
   ```rust
   pub fn is_yourapp_project(path: &Path) -> bool {
       // Detection logic
   }
   ```

3. **Create metadata module** `src/{yourapp}_metadata.rs`:
   ```rust
   pub fn extract_yourapp_metadata(path: &Path) -> Result<Metadata> {
       // Extract app-specific metadata
   }
   ```

4. **Add ignore patterns** in `src/ignore_template.rs`:
   ```rust
   ProjectType::YourApp => vec![
       "*.cache",
       "**/temp/**",
   ],
   ```

5. **Write tests** in `tests/{yourapp}_tests.rs`

6. **Update documentation** in `docs/user/for-{yourapp}-users.md`

### Running Tests

```bash
# All tests across all components
./run_all_tests.sh

# Rust CLI with coverage
cd Auxin-CLI-Wrapper
cargo test --all-features
cargo tarpaulin --out Html --output-dir coverage/

# Swift LaunchAgent
cd Auxin-LaunchAgent
swift test --enable-code-coverage
xcrun llvm-cov show .build/debug/Auxin-LaunchAgentPackageTests.xctest/Contents/MacOS/Auxin-LaunchAgentPackageTests

# Swift App
cd Auxin-App
swift test --enable-code-coverage

# Server (Rust)
cd auxin-server
cargo test --all-features

# E2E integration tests
cd tests/user-guide-scenarios
./run_all_scenarios.sh
```

### Debugging the Daemon

```bash
# View daemon logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --info

# Manual run with logging
cd Auxin-LaunchAgent
swift run

# Check LaunchAgent status
launchctl list | grep com.auxin.daemon
launchctl print gui/$(id -u)/com.auxin.daemon

# Restart daemon
launchctl kickstart -k gui/$(id -u)/com.auxin.daemon
```

### Debugging the CLI

```bash
# Run with verbose output
auxin --verbose commit -m "Test"

# Use Rust debugging
cd Auxin-CLI-Wrapper
RUST_BACKTRACE=1 cargo run -- commit -m "Test"

# Use lldb for breakpoints
lldb target/debug/auxin
```

### Building for Release

```bash
# Build all components optimized
./install.sh --release

# Or build individually
cd Auxin-CLI-Wrapper && cargo build --release
cd Auxin-LaunchAgent && swift build -c release
cd Auxin-App && swift build -c release

# Create app bundle
cd Auxin-App && ./create-app-bundle.sh

# Sign for distribution (requires Apple Developer ID)
codesign --sign "Developer ID Application: Your Name" Auxin.app
```

---

## Code Style and Conventions

### Swift

- Follow [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)
- Use meaningful, self-documenting variable names
- Maximum line length: 120 characters
- Use `// MARK: -` for section organization
- Prefer `guard` for early returns
- Use `async/await` for asynchronous code

**Example**:
```swift
// MARK: - Public Methods

/// Starts monitoring the specified path for file system events
/// - Parameter path: The directory path to monitor
/// - Throws: MonitorError if path is invalid or stream cannot be created
public func start(watchingPath path: String) async throws {
    guard !path.isEmpty else {
        throw MonitorError.invalidPath
    }
    // ...
}
```

### Rust

- Use `cargo fmt` before committing (enforced by CI)
- Run `cargo clippy -- -D warnings` and address all warnings
- Document all public APIs with rustdoc comments
- Prefer explicit error types over `anyhow` for libraries
- Use `?` operator for error propagation
- Maximum line length: 100 characters

**Example**:
```rust
/// Detects if the given path is a Logic Pro project
///
/// # Arguments
/// * `path` - Path to check
///
/// # Returns
/// * `true` if path ends with .logicx and contains projectData file
pub fn is_logic_pro_project(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "logicx")
        .unwrap_or(false)
}
```

### Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat` - New feature
- `fix` - Bug fix
- `docs` - Documentation changes
- `refactor` - Code refactoring
- `test` - Adding or updating tests
- `chore` - Build process, tooling

**Example**:
```
feat(cli): Add support for Blender projects

Implemented Blender project detection and metadata extraction.

- Added is_blender_project() detection
- Created BlenderMetadata struct
- Added default .auxinignore patterns for Blender
- Updated ProjectType enum

Closes #123
```

---

## Quality Standards

### Code Coverage Targets

- **Overall**: 70-80%
- **Critical paths** (locks, commits, power management): 90%+
- **New features**: Must maintain or improve coverage

**Current Coverage**:
- CLI Wrapper: 88% ✅
- LaunchAgent: ~75% (216 tests passing)
- Server: ~70%

### Performance Requirements

- **Commit 1GB project**: <10 seconds
- **Lock acquisition**: <100ms
- **Daemon CPU usage**: <5% average
- **Memory usage**: <50MB average

### CI/CD Requirements

All tests run automatically on:
- Push to `main`, `develop`, `claude/*` branches
- All pull requests
- Scheduled nightly builds

**Required Checks** (must pass before merge):
- ✅ All unit tests passing
- ✅ All integration tests passing
- ✅ Code coverage ≥70%
- ✅ No compiler warnings
- ✅ Clippy lints passing
- ✅ Swift tests passing (macOS runner)
- ✅ Security scan (CodeQL)

---

## Review Process

### Pull Request Checklist

Before submitting a PR:

- [ ] Code follows style guidelines (Swift/Rust)
- [ ] All tests pass locally
- [ ] New tests added for new features
- [ ] Documentation updated (if needed)
- [ ] Commit messages follow conventions
- [ ] No compiler warnings or clippy lints
- [ ] PR description explains changes clearly

### Review Guidelines

When reviewing PRs:

1. **Functionality**: Does it work as intended?
2. **Testing**: Adequate test coverage?
3. **Design**: Fits the architecture?
4. **Performance**: No obvious bottlenecks?
5. **Security**: No vulnerabilities introduced?
6. **Documentation**: Clear for future maintainers?

---

## Getting Help

### Documentation

- 📖 **[Architecture](docs/developer/architecture.md)** - System design
- 📖 **[Testing](docs/developer/testing.md)** - Testing strategy
- 📖 **[Extensibility](docs/developer/extensibility.md)** - Adding new apps

### Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/jbacus/auxin/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/jbacus/auxin/discussions)
- 💡 **Ideas**: Open a feature request issue

### Community

- Discord server (coming soon)
- Developer forum (coming soon)

---

## Next Steps

**New contributors**:
1. Read [Contributing Guide](docs/developer/contributing.md)
2. Set up your environment: [Development Setup](docs/developer/development-setup.md)
3. Find a "good first issue" on GitHub
4. Join Discord to ask questions

**Experienced contributors**:
1. Review [Architecture](docs/developer/architecture.md)
2. Check [Roadmap](ROADMAP.md) for upcoming features
3. See [Next Steps](NEXT_STEPS.md) for current priorities

**Deploying/Operating Auxin Server**:
1. See [Deployment Guide](DEPLOYMENT.md)

---

*Last Updated: 2025-11-22*
