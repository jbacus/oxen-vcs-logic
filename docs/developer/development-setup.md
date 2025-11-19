# Development Setup

This guide helps you set up your development environment for contributing to Auxin.

---

## Prerequisites

### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| macOS | 14.0+ | Development platform |
| Xcode | 15+ | Swift compilation |
| Rust | stable | CLI wrapper |
| Oxen CLI | latest | VCS backend |

### Optional Software

| Software | Purpose |
|----------|---------|
| Logic Pro 11.x | Testing Logic Pro support |
| SketchUp | Testing SketchUp support |
| Blender | Testing Blender support |

---

## Initial Setup

### 1. Clone Repository

```bash
git clone https://github.com/jbacus/auxin.git
cd auxin
```

### 2. Install Rust Toolchain

```bash
# Install rustup if not present
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### 3. Install Oxen CLI

```bash
# Option A: pip (recommended)
pip3 install oxen-ai

# Option B: cargo
cargo install oxen

# Verify
oxen --version
```

### 4. Install Xcode Command Line Tools

```bash
xcode-select --install
```

---

## Building Components

### All Components (Automated)

```bash
./install.sh
```

### Individual Components

```bash
# Rust CLI Wrapper
cd Auxin-CLI-Wrapper
cargo build --release

# Swift LaunchAgent
cd Auxin-LaunchAgent
swift build -c release

# Swift App (requires app bundle)
cd Auxin-App
swift build -c release
./create-app-bundle.sh
```

---

## Running Tests

### Full Test Suite

```bash
./run_all_tests.sh
```

### Component Tests

```bash
# Rust tests
cd Auxin-CLI-Wrapper && cargo test

# Rust with verbose output
cd Auxin-CLI-Wrapper && cargo test -- --nocapture

# Swift LaunchAgent tests
cd Auxin-LaunchAgent && swift test

# Swift App tests
cd Auxin-App && swift test
```

### Coverage Reports

```bash
# Rust coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cd Auxin-CLI-Wrapper && cargo tarpaulin --out Html
open tarpaulin-report.html

# Swift coverage
cd Auxin-LaunchAgent
swift test --enable-code-coverage
```

---

## Development Workflow

### 1. Create Feature Branch

```bash
git checkout -b feature/your-feature
```

### 2. Make Changes

Follow the [code style guidelines](contributing.md#code-style).

### 3. Run Tests

```bash
# Format code
cd Auxin-CLI-Wrapper && cargo fmt

# Run linter
cd Auxin-CLI-Wrapper && cargo clippy

# Run tests
cargo test
```

### 4. Commit with Conventional Format

```bash
git commit -m "feat(cli): add support for new project type"
```

### 5. Push and Create PR

```bash
git push -u origin feature/your-feature
```

---

## Debugging

### CLI Wrapper

```bash
# Run with verbose output
cd Auxin-CLI-Wrapper
RUST_LOG=debug cargo run -- status

# Debug specific test
cargo test test_name -- --nocapture
```

### LaunchAgent

```bash
# View system logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h

# Run manually
cd Auxin-LaunchAgent && swift run

# Check daemon status
launchctl list | grep auxin

# Restart daemon
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist
```

### App

```bash
# Run app
cd Auxin-App
swift run

# Or use app bundle
open Auxin.app
```

---

## Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `AUXIN_TIMEOUT` | 30000 | Default operation timeout (ms) |
| `AUXIN_NETWORK_TIMEOUT` | 120000 | Network operation timeout (ms) |
| `AUXIN_CACHE_TTL` | 1000 | Cache TTL for status operations (ms) |
| `RUST_LOG` | - | Logging level (debug, info, warn, error) |

---

## IDE Setup

### VS Code (Recommended for Rust)

Install extensions:
- rust-analyzer
- CodeLLDB (debugging)
- Better TOML

### Xcode (Required for Swift)

Open Swift packages:
```bash
open Auxin-LaunchAgent/Package.swift
open Auxin-App/Package.swift
```

---

## Common Issues

### "oxen: command not found"

```bash
# Check if installed
which oxen

# Add to PATH if needed
export PATH="$HOME/.local/bin:$PATH"
```

### Swift build fails

```bash
# Clean build folder
swift package clean
swift build
```

### Rust build fails

```bash
# Update dependencies
cargo update

# Clean and rebuild
cargo clean
cargo build
```

### Tests fail with permission errors

```bash
# Check Full Disk Access for Terminal
System Preferences → Security & Privacy → Privacy → Full Disk Access
```

---

## Next Steps

- Read [Contributing Guide](contributing.md) for code style and PR process
- Review [Architecture](architecture.md) for system design
- See [Testing Strategy](testing.md) for testing approach

---

*Last Updated: 2025-11-19*
