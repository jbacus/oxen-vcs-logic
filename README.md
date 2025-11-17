# Oxen-VCS for Logic Pro

[![Test Suite](https://github.com/jbacus/auxin/actions/workflows/test.yml/badge.svg)](https://github.com/jbacus/auxin/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![macOS](https://img.shields.io/badge/macOS-14.0+-blue.svg)](https://www.apple.com/macos)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Swift](https://img.shields.io/badge/swift-5.9+-orange.svg)](https://swift.org)

A native macOS version control system for Apple Logic Pro projects, leveraging Oxen.ai for efficient large binary file management.

## Overview

Professional DAW projects contain massive binary assets that overwhelm traditional version control systems like Git. Auxin provides:

- **Block-level deduplication** for efficient storage of large audio files
- **Automatic background tracking** via FSEvents monitoring
- **Collision avoidance** through exclusive file locking
- **Power-safe commits** that trigger before system sleep
- **Non-destructive rollback** to any previous project state

## Recent Updates

### ✨ SwiftUI Migration (October 29, 2025)

Migrated Auxin.app from AppKit to SwiftUI for improved reliability:
- **Migrated**: Complete UI rewrite using SwiftUI instead of AppKit
- **Fixed**: Window sizing issues that plagued the AppKit implementation
- **Simplified**: 80% reduction in UI code complexity
- **Improved**: Native NavigationSplitView with automatic layout
- **Benefits**: Declarative UI, better window management, modern macOS features

### 📚 Documentation Consolidation (November 15, 2025)

Project documentation has been streamlined and organized:
- **Removed**: 23 outdated development plans, phase reports, and session summaries
- **Consolidated**: Created two audience-specific guides (FOR_MUSICIANS.md and FOR_DEVELOPERS.md)
- **Cleaned**: 42 markdown files reduced to 8 essential documents
- **Updated**: All references and dates refreshed

### 🚀 Advanced CLI Features - Week 3 Complete! (November 15, 2025)

Powerful new features for professional workflows:

**🔍 Semantic Diff Comparison**
- Compare commit metadata side-by-side (BPM, sample rate, key, tags)
- Colored terminal output showing changes
- Multiple output formats (colored, plain, JSON, compact)

**🔎 AI-Powered Search**
- Natural language queries: `bpm:120-140 key:minor tag:mixing`
- Filter by BPM range, sample rate, key signature, tags, message
- Relevance scoring and ranked results

**⚙️ Workflow Automation Hooks**
- Pre-commit hooks (validation, file size checks)
- Post-commit hooks (notifications, backups)
- 4 built-in templates ready to install
- Full scripting support (bash, python, ruby, etc.)

**🖥️ Interactive Console TUI**
- Full-screen terminal interface with 7 modes
- Real-time daemon monitoring and activity log
- Interactive commit dialog with metadata fields
- Browse history and restore commits with keyboard navigation
- All features accessible from one unified interface

**📊 Test Coverage**
- 349 total tests (274 unit + 49 integration + 26 doctests)
- All tests passing ✅

See [CHANGELOG.md](CHANGELOG.md) for complete project history.

## Architecture

### Three-Component System

1. **Auxin-App** (Swift/SwiftUI) - Native macOS UI application with modern declarative interface
2. **Auxin-LaunchAgent** (Swift/FSEvents) - Persistent background daemon for file monitoring and draft tracking
3. **Auxin-CLI-Wrapper** (Rust/liboxen) - Optimized command executor for Oxen operations

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
git clone https://github.com/YOUR_USERNAME/auxin.git
cd auxin

# Run the automated installer
./install.sh
```

The installer will build all components, install binaries, configure the daemon service, and install the GUI app to `/Applications/Auxin.app`.

See [Installation Guide](INSTALL.md) for detailed instructions and troubleshooting.

### Manual Installation

See [Installation Guide](INSTALL.md) for step-by-step manual installation instructions.

## Quick Start

### Option A: GUI Application (Point and Click)

After installation, launch the double-clickable app:

1. Open Finder → Applications → Auxin
2. Click "Add Project..." to initialize a Logic Pro project
3. The app will monitor changes and create automatic draft commits
4. Create milestone commits with rich metadata (BPM, sample rate, tags)
5. Browse history and rollback to any previous version

### Option B: Command Line (Fast and Powerful) ✨ NEW!

**Enhanced with beautiful visual feedback and progress indicators:**

```bash
# Initialize a Logic Pro project
cd ~/Music/YourProject.logicx
auxin init --logic .
# ⠹ Validating Logic Pro project structure...
# ✓ Logic Pro project repository initialized

# Check what changed
auxin status
# Shows: staged, modified, and untracked files with color coding

# Stage and commit with metadata
auxin add --all
auxin commit -m "Vocal tracking done" --bpm 120 --tags "vocals"
# ✓ Commit created: abc123f

# View history with filtering
auxin log --bpm 120 --tag vocals --limit 10
# Shows: filtered commits with metadata

# Restore to previous version
auxin restore abc123f
```

### Option C: Advanced CLI Features ✨ NEW!

**Compare commits semantically:**
```bash
# Compare two commits (shows BPM, key, sample rate changes)
auxin compare abc123f def456g
# Shows colored diff of metadata changes

# Compare in different formats
auxin compare abc123f def456g --format json
auxin compare abc123f def456g --format compact
```

**Search with natural language:**
```bash
# Find all commits with specific criteria
auxin search "bpm:120-140 key:minor tag:mixing"
auxin search "bpm:>128 tag:vocals,final"

# Get ranked results
auxin search "bpm:120-140 key:minor" --ranked
```

**Automate workflows with hooks:**
```bash
# Initialize hooks directory
auxin hooks init

# Install built-in hooks
auxin hooks install validate-metadata --type pre-commit
auxin hooks install backup --type post-commit

# List installed hooks
auxin hooks list
```

**Interactive console mode:**
```bash
# Launch full-screen TUI
auxin console

# Keyboard shortcuts:
# i - Create commit (with metadata fields)
# l - Browse commit history
# d - Compare commits side-by-side
# s - Search commits
# k - Manage hooks
# r - Refresh status
# ? - Help
```

**See:** [CLI Examples](docs/CLI_EXAMPLES.md) for real production scenarios and team workflows.

**Which should you use?** Both work equally well! Choose based on preference:
- **New to version control?** Start with GUI
- **Comfortable with Terminal?** CLI is faster
- **Working remotely?** CLI works over SSH

See [User Guide for Musicians](docs/FOR_MUSICIANS.md) for detailed usage instructions.

## Testing

We maintain comprehensive test coverage across all components:

```bash
# Run all Rust tests
cd Auxin-CLI-Wrapper && cargo test

# Run all Swift tests
cd Auxin-LaunchAgent && swift test
cd Auxin-App && swift test
```

**Documentation**:
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Comprehensive testing approach
- See [CONTRIBUTING.md](CONTRIBUTING.md#testing) for detailed testing guidelines

**Coverage Goals**: 70-80% overall, with 90%+ coverage for critical paths (locks, commits, power management)

## Development Setup

```bash
# Clone repository
git clone https://github.com/YOUR_USERNAME/auxin.git
cd auxin

# Install dependencies
pip install oxen-ai  # Optional

# Run automated installation
./install.sh
```

For manual build instructions, see [Installation Guide](INSTALL.md).

## Implementation Status

All three development phases are complete. See [Developer Guide](docs/FOR_DEVELOPERS.md) for detailed architecture and roadmap.

- [x] **Phase 1: Core Data Management (MVP)** - ✅ COMPLETE
  - Logic Pro project detection and validation
  - .oxenignore template generation
  - Oxen initialization wrapper
  - Core operations (init, add, commit, log, restore)
  - Structured commit metadata (BPM, sample rate, key signature)
  - FSEvents monitoring with debounce (proof of concept)
- [x] **Phase 2: Service Architecture & Resilience** - ✅ COMPLETE
  - LaunchAgent integration with automatic startup
  - Power management integration (sleep/shutdown commits)
  - Auto-commit workflow with draft branches
  - XPC communication for UI integration
  - Multi-project monitoring
- [x] **Phase 3: UI Application & Collaboration** - ✅ COMPLETE
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
auxin/
├── Auxin-App/              # Swift/SwiftUI UI application
├── Auxin-LaunchAgent/      # Background monitoring daemon
├── Auxin-CLI-Wrapper/      # Rust wrapper for Oxen CLI
├── docs/                   # User and developer documentation
│   ├── FOR_MUSICIANS.md    # User guide for music producers
│   ├── FOR_DEVELOPERS.md   # Technical guide for contributors
│   └── TESTING_STRATEGY.md # Testing approach and coverage
└── tests/                  # Unit and integration tests
```

## Documentation

### User Guides
- [For Musicians](docs/FOR_MUSICIANS.md) - Complete guide for music producers (non-technical, covers both GUI and CLI)
- [CLI Examples](docs/CLI_EXAMPLES.md) - Real-world command line examples with visual output
- [Installation Guide](INSTALL.md) - Complete installation instructions
- [App Bundle Guide](Auxin-App/APP_BUNDLE.md) - Double-clickable app creation
- [CLI Usage Guide](Auxin-CLI-Wrapper/USAGE.md) - Complete CLI reference

### Technical Documentation
- [For Developers](docs/FOR_DEVELOPERS.md) - Full technical specification, architecture, and API reference
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Comprehensive testing approach
- [Contributing Guidelines](CONTRIBUTING.md) - Code style, testing, and PR process
- [Week 1 Progress](WEEK1_PROGRESS.md) - CLI enhancement progress report

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
