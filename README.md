# Auxin - Version Control for Creative Applications

[![Test Suite](https://github.com/jbacus/auxin/actions/workflows/test.yml/badge.svg)](https://github.com/jbacus/auxin/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![macOS](https://img.shields.io/badge/macOS-14.0+-blue.svg)](https://www.apple.com/macos)
[![Rust](https://img.shields.io/badge/rust-stable-orange.svg)](https://www.rust-lang.org)
[![Swift](https://img.shields.io/badge/swift-5.9+-orange.svg)](https://swift.org)

A native macOS version control system for creative applications, leveraging Oxen.ai for efficient large binary file management.

## Supported Applications

- **Logic Pro** (.logicx) - Audio production with BPM, sample rate, key signature metadata
- **SketchUp** (.skp) - 3D modeling with units, layers, components, groups metadata
- **Blender** (.blend) - 3D modeling and animation support

## Overview

Professional creative projects contain massive binary assets that overwhelm traditional version control systems like Git. Auxin provides:

- **Block-level deduplication** for efficient storage of large audio files
- **Automatic background tracking** via FSEvents monitoring
- **Collision avoidance** through exclusive file locking
- **Power-safe commits** that trigger before system sleep
- **Non-destructive rollback** to any previous project state

## Recent Updates

### 🎨 3D Modeling Support (November 18, 2025)

Expanded support for 3D modeling applications:
- **SketchUp** (.skp) - Full support with metadata tracking (units, layers, components, groups)
- **Blender** (.blend) - Project detection and ignore patterns
- **Auto-detection** - Project type automatically detected from file extension
- **Unified CLI** - Same commands work across all supported applications

### ✨ SwiftUI Migration (October 29, 2025)

Migrated Auxin.app from AppKit to SwiftUI for improved reliability:
- **Migrated**: Complete UI rewrite using SwiftUI instead of AppKit
- **Fixed**: Window sizing issues that plagued the AppKit implementation
- **Simplified**: 80% reduction in UI code complexity
- **Improved**: Native NavigationSplitView with automatic layout
- **Benefits**: Declarative UI, better window management, modern macOS features

### 📚 Documentation Restructure (November 18, 2025)

Project documentation has been streamlined and organized:
- **Created**: ROADMAP.md (project vision and progress) and FEATURE_STATUS.md (component assessment)
- **Removed**: 17 outdated session logs, progress reports, and completed phase documents
- **Consolidated**: 63 files reduced to 46 essential documents
- **Organized**: Documentation categorized into user-facing, developer-facing, and system prompt groups

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
- 348 unit tests passing with 88% coverage
- 40+ integration tests
- All tests passing ✅

See [CHANGELOG.md](CHANGELOG.md) for complete project history.

## Architecture

### Three-Component System

1. **Auxin-App** (Swift/SwiftUI) - Native macOS UI application with modern declarative interface
2. **Auxin-LaunchAgent** (Swift/FSEvents) - Persistent background daemon for file monitoring and draft tracking
3. **Auxin-CLI-Wrapper** (Rust/liboxen) - Optimized command executor for Oxen operations

## Requirements

- macOS 14.0+
- Oxen.ai CLI
- Xcode 15+
- Rust toolchain (for CLI wrapper)
- Application software as needed (Logic Pro, SketchUp, Blender)

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
# Initialize projects (auto-detects type from extension)
auxin init ~/Music/YourProject.logicx        # Logic Pro
auxin init ~/Models/Building.skp              # SketchUp
auxin init ~/Projects/Scene.blend             # Blender

# Or explicitly specify type
auxin init --type logicpro ~/Music/Project.logicx
auxin init --type sketchup ~/Models/Model.skp
auxin init --type blender ~/Projects/Scene.blend

# Check what changed
auxin status
# Shows: staged, modified, and untracked files with color coding

# Stage and commit with application-specific metadata
auxin add --all

# Logic Pro commit
auxin commit -m "Vocal tracking done" --bpm 120 --tags "vocals"

# SketchUp commit
auxin commit -m "Added roof structure" --units Feet --layers 15 --components 200

# View history with filtering
auxin log --limit 10
# Shows: commits with metadata

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

The first five phases are complete with Phases 6 and 7 in progress. See [Roadmap](ROADMAP.md) for detailed progress and [Feature Status](FEATURE_STATUS.md) for component assessment.

- [x] **Phase 1: Core CLI & Logic Pro** - ✅ COMPLETE (100%)
  - Logic Pro project detection, validation, and metadata
  - .oxenignore generation and Oxen subprocess wrapper
  - Core operations (init, add, commit, log, restore, status, diff, show)
  - 331 unit tests, 88% coverage
- [x] **Phase 2: Background Services** - ✅ COMPLETE (100%)
  - LaunchAgent with FSEvents monitoring and debounce
  - Power management (sleep/shutdown safety commits)
  - XPC communication and multi-project support
- [x] **Phase 3: GUI Application** - ✅ COMPLETE (100%)
  - Native macOS SwiftUI application
  - Repository browser, commit history, status bar
  - Needs macOS integration testing
- [x] **Phase 4: Team Collaboration** - ✅ COMPLETE (95%)
  - Authentication, distributed locking, activity feeds
  - Team discovery and commit comments
  - Gap: Network resilience
- [x] **Phase 5: 3D Modeling Support** - ✅ COMPLETE (100%)
  - SketchUp and Blender project support
  - Application-specific metadata and .oxenignore patterns
- [ ] **Phase 6: Network Resilience** - 🔄 IN PROGRESS
  - Offline mode, retry logic, partial push recovery
- [ ] **Phase 7: Auxin Server** - 🔄 IN PROGRESS (30%)
  - Self-hosted collaboration server with web interface

## Key Features

### Automatic Draft Tracking
Background daemon monitors project folders (Logic Pro, SketchUp, Blender) and automatically commits working states to a local draft branch after detecting file inactivity.

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

### Project Overview
- [Roadmap](ROADMAP.md) - Project vision, phases, and progress
- [Feature Status](FEATURE_STATUS.md) - Component completeness and test coverage
- [Changelog](CHANGELOG.md) - Version history

### User Guides
- [For Musicians](docs/FOR_MUSICIANS.md) - Complete guide for music producers (non-technical, covers both GUI and CLI)
- [CLI Examples](docs/CLI_EXAMPLES.md) - Real-world command line examples with visual output
- [SketchUp Examples](docs/SKETCHUP_EXAMPLES.md) - SketchUp workflow examples and best practices
- [Cloud Sharing Guide](docs/CLOUD_SHARING_GUIDE.md) - Team collaboration guide
- [Installation Guide](INSTALL.md) - Complete installation instructions
- [Troubleshooting](docs/TROUBLESHOOTING.md) - Common issues and solutions

### Technical Documentation
- [For Developers](docs/FOR_DEVELOPERS.md) - Full technical specification, architecture, and API reference
- [Extensibility Guide](docs/EXTENSIBILITY.md) - Adding support for new applications
- [Testing Strategy](docs/TESTING_STRATEGY.md) - Comprehensive testing approach
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
