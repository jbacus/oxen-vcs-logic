# Oxen-VCS for Logic Pro

A native macOS version control system for Apple Logic Pro projects, leveraging Oxen.ai for efficient large binary file management.

## Overview

Professional DAW projects contain massive binary assets that overwhelm traditional version control systems like Git. OxVCS provides:

- **Block-level deduplication** for efficient storage of large audio files
- **Automatic background tracking** via FSEvents monitoring
- **Collision avoidance** through exclusive file locking
- **Power-safe commits** that trigger before system sleep
- **Non-destructive rollback** to any previous project state

## Architecture

### Three-Component System

1. **OxVCS-App** (Swift/AppKit) - Main UI application for user interaction, history browsing, and milestone commits
2. **OxVCS-LaunchAgent** (Swift/FSEvents) - Persistent background daemon for file monitoring and draft tracking
3. **OxVCS-CLI-Wrapper** (Rust/liboxen) - Optimized command executor for Oxen operations

## Requirements

- macOS 14.0+
- Logic Pro 11.x
- Oxen.ai CLI
- Xcode 15+
- Rust toolchain (for CLI wrapper)

## Installation

### Automated Installation (Recommended)

```bash
# Clone the repository
git clone https://github.com/YOUR_USERNAME/oxen-vcs-logic.git
cd oxen-vcs-logic

# Run the automated installer
./install.sh
```

The installer will build all components, install binaries, and configure the daemon service.

See [Installation Guide](INSTALL.md) for detailed instructions and troubleshooting.

### Manual Installation

See [Installation Guide](INSTALL.md) for step-by-step manual installation instructions.

## Quick Start

After installation:

```bash
# Initialize a Logic Pro project
cd ~/Music/YourProject.logicx
oxenvcs-cli init --logic .

# Stage and commit
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial commit" --bpm 120 --sample-rate 48000
```

See [Quick Start Guide](docs/QUICKSTART.md) for detailed usage instructions.

## Testing

We maintain comprehensive test coverage across all components:

```bash
# Run all Rust tests
cd OxVCS-CLI-Wrapper && cargo test

# Run all Swift tests
cd OxVCS-LaunchAgent && swift test
cd OxVCS-App && swift test
```

**Documentation**:
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Comprehensive testing approach
- [Test Implementation Plan](docs/TEST_IMPLEMENTATION_PLAN.md) - Phased implementation roadmap
- See [CONTRIBUTING.md](CONTRIBUTING.md#testing) for detailed testing guidelines

**Coverage Goals**: 70-80% overall, with 90%+ coverage for critical paths (locks, commits, power management)

## Development Setup

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/oxen-vcs-logic.git
cd oxen-vcs-logic

# Install dependencies
pip install oxen-ai  # Optional

# Run automated installation
./install.sh
```

For manual build instructions, see [Installation Guide](INSTALL.md).

## Implementation Status

See [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for detailed roadmap.

- [x] **Phase 1: Core Data Management (MVP)** - ✅ COMPLETE ([Details](docs/PHASE1_COMPLETE.md))
  - Logic Pro project detection and validation
  - .oxenignore template generation
  - Oxen initialization wrapper
  - Core operations (init, add, commit, log, restore)
  - Structured commit metadata (BPM, sample rate, key signature)
  - FSEvents monitoring with debounce (proof of concept)
- [x] **Phase 2: Service Architecture & Resilience** - ✅ COMPLETE ([Details](docs/PHASE2_COMPLETE.md))
  - LaunchAgent integration with automatic startup
  - Power management integration (sleep/shutdown commits)
  - Auto-commit workflow with draft branches
  - XPC communication for UI integration
  - Multi-project monitoring
- [x] **Phase 3: UI Application & Collaboration** - ✅ COMPLETE ([Details](PHASE3_COMPLETE.md))
  - Native macOS AppKit UI application
  - Repository browser and project management
  - Milestone commit interface with rich metadata
  - Rollback/restore interface
  - Exclusive file locking system
  - Manual merge protocol documentation
  - Settings and configuration panel

## Key Features

### Automatic Draft Tracking
Background daemon monitors Logic Pro project folder and automatically commits working states to a local draft branch after detecting file inactivity.

### Milestone Commits
User-triggered commits that:
- Clean up volatile files (bounces, freeze files)
- Tag significant production milestones
- Include metadata (BPM, sample rate, key signature)
- Sync to remote Oxen repository

### Power-Safe Operation
System power observers force immediate draft commits before sleep/shutdown to prevent data loss.

### Collaboration Protocol
- Exclusive file locking prevents binary merge conflicts
- FCP XML export/import for manual track reconciliation
- Remote repository sync via Oxen Hub

## Project Structure

```
oxen-vcs-logic/
├── OxVCS-App/              # Swift/AppKit UI application
├── OxVCS-LaunchAgent/      # Background monitoring daemon
├── OxVCS-CLI-Wrapper/      # Rust FFI wrapper for Oxen
├── docs/                   # Architecture and API documentation
│   ├── ARCHITECTURE.md
│   └── IMPLEMENTATION_PLAN.md
└── tests/                  # Unit and integration tests
```

## Documentation

### User Guides
- [Installation Guide](INSTALL.md) - Complete installation instructions
- [Quick Start Guide](docs/QUICKSTART.md) - Get started in 5 minutes
- [Usage Guide](OxVCS-CLI-Wrapper/USAGE.md) - Complete CLI reference

### Technical Documentation
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Comprehensive testing approach
- [Test Implementation Plan](docs/TEST_IMPLEMENTATION_PLAN.md) - Phased test implementation
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md) - Development roadmap
- [Architecture Blueprint](docs/ARCHITECTURE.md) - Full technical specification (TBD)
- [API Reference](docs/API.md) - Component interfaces (TBD)

### Completion Reports
- [Phase 1 Completion Report](docs/PHASE1_COMPLETE.md) - MVP implementation details
- [Phase 2 Completion Report](docs/PHASE2_COMPLETE.md) - Service architecture details
- [Phase 3 Completion Report](PHASE3_COMPLETE.md) - UI & collaboration features

### Developer Resources
- [Contributing Guidelines](CONTRIBUTING.md) - Code style, testing, and PR process

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

This project has completed all three phases and is ready for production use. Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on:
- Code style and standards
- Testing requirements
- Pull request process
- Development workflow

## References

- [Oxen.ai Documentation](https://docs.oxen.ai/)
- [Logic Pro Project Format](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml)
- [FSEvents Framework](https://developer.apple.com/documentation/coreservices/file_system_events)
