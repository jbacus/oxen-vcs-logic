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

## Quick Start

```bash
# Build the CLI tool
cd OxVCS-CLI-Wrapper
cargo build --release

# Initialize a Logic Pro project
cd ~/Music/YourProject.logicx
oxenvcs-cli init --logic .

# Stage and commit
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial commit" --bpm 120 --sample-rate 48000
```

See [Quick Start Guide](docs/QUICKSTART.md) for detailed instructions.

## Development Setup

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/oxen-vcs-logic.git
cd oxen-vcs-logic

# Install Oxen CLI
pip install oxen-ai

# Build CLI wrapper (requires Rust)
cd OxVCS-CLI-Wrapper
cargo build --release

# Build FSEvents monitor (requires Swift/Xcode)
cd ../OxVCS-LaunchAgent
swift build -c release
```

## Implementation Status

See [IMPLEMENTATION_PLAN.md](docs/IMPLEMENTATION_PLAN.md) for detailed roadmap.

- [x] **Phase 1: Core Data Management (MVP)** - ✅ COMPLETE ([Details](docs/PHASE1_COMPLETE.md))
  - Logic Pro project detection and validation
  - .oxenignore template generation
  - Oxen initialization wrapper
  - Core operations (init, add, commit, log, restore)
  - Structured commit metadata (BPM, sample rate, key signature)
  - FSEvents monitoring with debounce (proof of concept)
- [ ] Phase 2: Service Architecture & Resilience
- [ ] Phase 3: UI Application & Collaboration

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

- [Quick Start Guide](docs/QUICKSTART.md) - Get started in 5 minutes
- [Usage Guide](OxVCS-CLI-Wrapper/USAGE.md) - Complete CLI reference
- [Phase 1 Completion Report](docs/PHASE1_COMPLETE.md) - Technical implementation details
- [Implementation Plan](docs/IMPLEMENTATION_PLAN.md) - Development roadmap
- [Architecture Blueprint](docs/ARCHITECTURE.md) - Full technical specification (TBD)
- [API Reference](docs/API.md) - Component interfaces (TBD)

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

This project is in active development. Contributions welcome after Phase 1 completion.

## References

- [Oxen.ai Documentation](https://docs.oxen.ai/)
- [Logic Pro Project Format](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml)
- [FSEvents Framework](https://developer.apple.com/documentation/coreservices/file_system_events)
