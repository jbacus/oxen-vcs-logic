# CLAUDE.md

**Auxin System Prompt for AI Assistants**

---

## Quick Start

For AI assistants working with this codebase, see the condensed system prompt:

**[docs/system/CLAUDE.md](docs/system/CLAUDE.md)**

This contains:
- Quick reference commands
- Critical source files
- Architecture overview
- Coding conventions
- Common tasks

---

## Documentation Structure

### For Users
Location: `docs/user/`
- [Getting Started](docs/user/getting-started.md)
- [For Musicians](docs/user/for-musicians.md)
- [For Modelers](docs/user/for-modelers.md)
- [CLI Reference](docs/user/cli-reference.md)
- [Troubleshooting](docs/user/troubleshooting.md)

### For Developers
Location: `docs/developer/`
- [Development Setup](docs/developer/development-setup.md)
- [Architecture](docs/developer/architecture.md)
- [Contributing](docs/developer/contributing.md)
- [Testing](docs/developer/testing.md)

### For AI Assistants
Location: `docs/system/`
- [System Prompt](docs/system/CLAUDE.md) - Condensed context
- [Future Features](docs/system/future-features.md) - Planned features

### Project Overview
- [README](README.md) - Project overview
- [Roadmap](ROADMAP.md) - Phase timeline
- [Feature Status](FEATURE_STATUS.md) - Completion status
- [Changelog](CHANGELOG.md) - Version history

---

## Essential Context

### What is Auxin?
A macOS-native version control system for creative applications (Logic Pro, SketchUp, Blender).

### Why does it exist?
Creative projects have large binary files that cause Git to fail with bloat and unresolvable merge conflicts.

### How does it work?
- Block-level deduplication via Oxen.ai
- Pessimistic locking prevents conflicts
- Automatic draft commits
- Application-specific metadata

### Current Status
- **Rust CLI**: Production-ready (88% test coverage)
- **Swift components**: Code complete, need macOS testing
- **Phase 6-7**: In progress (Network Resilience, Server)

---

## Quick Commands

```bash
# Run all tests
./run_all_tests.sh

# Build Rust CLI
cd Auxin-CLI-Wrapper && cargo build --release

# Build Swift (macOS only)
cd Auxin-LaunchAgent && swift build
cd Auxin-App && swift build -c release
```

---

## Need More Detail?

See the full system prompt: **[docs/system/CLAUDE.md](docs/system/CLAUDE.md)**

---

*Last Updated: 2025-11-19*
