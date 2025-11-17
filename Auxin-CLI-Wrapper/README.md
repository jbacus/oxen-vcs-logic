# Auxin CLI Wrapper

High-performance Rust CLI and library for Oxen VCS operations on Logic Pro projects. This component provides the core VCS functionality for the Auxin system with minimal latency and low resource usage.

## Overview

The CLI Wrapper is a Rust-based tool that interfaces directly with `liboxen` (Oxen's core library) to provide:
- Fast VCS operations optimized for large binary files
- Logic Pro project-specific functionality
- Draft branch management for automatic commits
- Structured commit metadata (BPM, sample rate, key signature, tags)
- Low-overhead execution suitable for daemon use

## Architecture

### Components

```
Auxin-CLI-Wrapper/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point & argument parsing
│   ├── lib.rs               # Library exports for FFI
│   ├── logic_project.rs     # Logic Pro project detection & validation
│   ├── oxen_ops.rs          # Core Oxen operations wrapper
│   ├── commit_metadata.rs   # Structured metadata handling
│   └── draft_manager.rs     # Draft branch management
└── tests/
    ├── common/
    │   └── mod.rs           # Test utilities and fixtures
    └── integration/         # Integration tests
```

### Design Principles

- **Direct liboxen Integration**: No subprocess overhead, direct API calls
- **Async/Await**: Non-blocking operations for daemon usage
- **Type Safety**: Strong typing for all operations and metadata
- **Error Handling**: Comprehensive Result types with descriptive errors
- **Zero-Copy**: Efficient handling of large binary files

## Features

### Logic Pro Project Detection

Automatically validates Logic Pro project structure:
- Detects `.logicx` folder projects
- Validates required directories (`Alternatives/`, `Resources/`)
- Checks for `projectData` file
- Extracts project metadata (BPM, sample rate, key signature)

### Core VCS Operations

All standard Oxen operations with Logic Pro optimizations:
- `init` - Initialize repository with Logic Pro templates
- `add` - Stage files with intelligent filtering
- `commit` - Create commits with structured metadata
- `log` - View history with metadata display
- `status` - Check working tree state
- `restore` - Rollback to previous commits
- `branch` - Manage branches including draft branch

### Draft Branch Management

Automatic draft branch workflow:
- Creates `draft` branch on initialization
- Auto-commits to draft branch during work
- Keeps `main` branch clean for milestones
- Configurable pruning (max commits threshold)
- Statistics tracking for commits

### Structured Commit Metadata

Rich metadata support beyond standard commit messages:
```json
{
  "message": "Completed bass line recording",
  "bpm": 128,
  "sampleRate": 48000,
  "keySignature": "A Minor",
  "timeSignature": "4/4",
  "tags": ["recording", "bass", "session-1"]
}
```

## Installation

### Prerequisites

- Rust 1.70+ (install from https://rustup.rs)
- Oxen.ai CLI or liboxen library
- macOS 14.0+ (for full feature support)

### Building

```bash
cd Auxin-CLI-Wrapper

# Development build
cargo build

# Optimized release build
cargo build --release

# The binary will be at: target/release/auxin
```

### Installing to PATH

```bash
# Option 1: Copy to /usr/local/bin
sudo cp target/release/auxin /usr/local/bin/

# Option 2: Add to PATH in ~/.zshrc or ~/.bash_profile
export PATH="/path/to/Auxin-CLI-Wrapper/target/release:$PATH"

# Option 3: Use cargo install (from crate)
cargo install auxin
```

## Usage

### Basic Commands

```bash
# Initialize a Logic Pro project
auxin init --logic /path/to/Project.logicx

# Stage all changes
auxin add --all

# Create a milestone commit with metadata
auxin commit \
  -m "Finished mixing drums" \
  --bpm 140 \
  --sample-rate 48000 \
  --key "E Minor" \
  --tags "mixing,drums,session-3"

# View commit history
auxin log --limit 10

# Check status
auxin status

# Restore to previous commit
auxin restore <commit-hash>

# List branches
auxin branch

# Switch branches
auxin checkout main
```

### Advanced Usage

```bash
# Initialize with custom draft branch name
auxin init --logic . --draft-branch my-drafts

# Commit with full metadata
auxin commit \
  -m "Complex commit message" \
  --bpm 128 \
  --sample-rate 96000 \
  --key "C# Major" \
  --time-signature "7/8" \
  --tags "experimental,progressive"

# View detailed log with metadata
auxin log --with-metadata

# Status with porcelain output (machine-readable)
auxin status --porcelain

# Force restore (discard uncommitted changes)
auxin restore --force <commit-hash>
```

### Draft Branch Workflow

```bash
# After init, you're automatically on the draft branch
auxin branch
# * draft
#   main

# Work and commit to draft
auxin add --all
auxin commit -m "WIP: working on arrangement"

# Switch to main for milestone commits
auxin checkout main
auxin merge draft
auxin commit -m "Version 1.0 - Ready for mixing" --tags "milestone"

# Return to draft for continued work
auxin checkout draft
```

## Configuration

### .oxenignore

The CLI automatically creates a `.oxenignore` file optimized for Logic Pro:

```gitignore
# Logic Pro temporary files
Bounces/
Freeze Files/
Autosave/
*.autosave

# macOS system files
.DS_Store
.Spotlight-V100
.Trashes

# Plugin caches
*.nks
PlugInData/
```

### Environment Variables

```bash
# Enable debug logging
export AUXIN_LOG=debug

# Custom draft branch name
export AUXIN_DRAFT_BRANCH=my-drafts

# Maximum draft commits before pruning
export AUXIN_MAX_DRAFT_COMMITS=100
```

## Performance

### Benchmarks

Expected performance on a typical Logic Pro project (~500MB):

| Operation | Time | Notes |
|-----------|------|-------|
| `init` | 100-200ms | Includes template creation |
| `add --all` | 50-100ms | 100 files |
| `commit` | 100-200ms | With metadata |
| `status` | 20-50ms | `--porcelain` mode |
| `log` | 10-30ms | Last 10 commits |
| `restore` | 200-500ms | Depends on project size |

Memory footprint: **30-50MB** resident

### Optimization Tips

```bash
# Use --porcelain for machine parsing (faster)
auxin status --porcelain

# Limit log output
auxin log --limit 10

# Use specific paths for add (faster than --all)
auxin add Resources/Audio_01.wav
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_is_logic_project

# Run integration tests only
cargo test --test '*'

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage
```

### Test Coverage

Current coverage: **70-80%** overall
- Logic project detection: 90%+
- Core operations: 80%+
- Draft management: 75%+
- Metadata handling: 85%+

See [TESTING_STRATEGY.md](../docs/TESTING_STRATEGY.md) for details.

## Integration with Other Components

### Used by LaunchAgent Daemon

The daemon calls the CLI for automated operations:

```swift
// Swift code in Auxin-LaunchAgent
let process = Process()
process.executableURL = URL(fileURLWithPath: "/usr/local/bin/auxin")
process.arguments = ["commit", "-m", "Auto-draft", projectPath]
try process.run()
process.waitUntilExit()
```

### Used by UI Application

The UI app calls the CLI via the daemon's XPC service:

```swift
// Swift code in Auxin-App
OxenDaemonXPCClient.shared.commitProject(
    projectPath,
    message: "Milestone commit",
    metadata: CommitMetadata(bpm: 120, sampleRate: 48000)
) { commitHash, error in
    // Handle result
}
```

## Development

### Building for Development

```bash
# Build and run
cargo run -- init --logic test-project.logicx

# Build with debug symbols
cargo build

# Watch mode (requires cargo-watch)
cargo watch -x build
```

### Code Style

```bash
# Format code
cargo fmt

# Lint code
cargo clippy

# Check without building
cargo check
```

### Adding New Features

1. Add function to appropriate module (`oxen_ops.rs`, `draft_manager.rs`, etc.)
2. Add CLI argument parsing in `main.rs`
3. Add tests in `tests/`
4. Update documentation
5. Run `cargo fmt` and `cargo clippy`

## Troubleshooting

### "Repository not found"

Make sure you've initialized the repository:
```bash
auxin init --logic .
```

### "Path is not a Logic Pro project"

Verify you're in a `.logicx` directory with:
```bash
ls -la | grep -E "(Alternatives|projectData)"
```

### Performance Issues

Enable debug logging to diagnose:
```bash
AUXIN_LOG=debug auxin status
```

### Linking Errors (Build)

Ensure liboxen is properly installed:
```bash
# If using system liboxen
export LIBRARY_PATH=/path/to/liboxen

# Or use bundled liboxen
cargo build --features bundled-liboxen
```

## API Documentation

Generate and view API docs:
```bash
cargo doc --open
```

## Dependencies

Major dependencies:
- `liboxen` 0.19+ - Core Oxen VCS library
- `clap` 4.x - Command-line argument parsing
- `serde` 1.x - Serialization/deserialization
- `tokio` 1.x - Async runtime
- `anyhow` - Error handling
- `serde_json` - JSON metadata handling

See `Cargo.toml` for complete dependency list.

## Related Documentation

- [USAGE.md](USAGE.md) - Complete CLI reference
- [TESTING_STRATEGY.md](../docs/TESTING_STRATEGY.md) - Testing approach
- [IMPLEMENTATION_PLAN.md](../docs/IMPLEMENTATION_PLAN.md) - Development roadmap (all phases complete)

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code style guidelines
- Testing requirements
- Pull request process

## License

MIT License - See [LICENSE](../LICENSE) for details.
