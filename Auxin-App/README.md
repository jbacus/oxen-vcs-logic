# Auxin Main Application

Native macOS AppKit application providing a complete user interface for version control of Logic Pro projects. Offers project management, commit history visualization, rollback capabilities, and collaboration features through an intuitive graphical interface.

## Overview

The Auxin App is the primary user-facing component of the Oxen VCS system for Logic Pro. It communicates with the background daemon via XPC to provide:
- Visual project management and monitoring
- Rich commit creation with metadata
- Interactive commit history browsing
- One-click rollback/restore operations
- File locking for team collaboration
- Merge helper tools for conflict resolution
- Settings and daemon configuration

## Key Features

### Repository Browser
- **Project List View**: All monitored Logic Pro projects at a glance
- **Real-time Status**: Live updates of commit counts and lock status
- **Auto-refresh**: Periodic updates every 30 seconds
- **Visual Indicators**: Icons and colors for locks, changes, and errors
- **Quick Actions**: Context menu for common operations
- **Search & Filter**: Find projects quickly

### Project Initialization Wizard
- **File Browser**: Native file picker for project selection
- **Validation**: Ensures selected path is valid .logicx project
- **One-Click Setup**: Initialize Oxen repository and start monitoring
- **Progress Feedback**: Visual progress during initialization
- **Error Handling**: Clear messages for common issues
- **Defaults Management**: Remember last used settings

### Milestone Commit Interface
- **Rich Metadata Input**: BPM, sample rate, key, time signature
- **Tag Management**: Add custom searchable tags
- **Pre-flight Cleanup**: Optional removal of temporary files
  - Bounces/
  - Freeze Files/
  - Media caches
- **Commit Message**: Multi-line text with character count
- **Preview Changes**: See what will be committed
- **Validation**: Ensures required fields are filled
- **Progress Indicator**: Visual feedback during commit

### Commit History & Rollback
- **Timeline View**: Visual representation of commit history
- **Table View**: Detailed list with hash, message, date, author
- **Metadata Display**: View BPM, sample rate, tags for each commit
- **Search & Filter**: Find specific commits by message or metadata
- **One-Click Restore**: Select and rollback to any previous commit
- **Safety Confirmation**: Warning dialog before destructive operations
- **Diff Preview**: See what changes between commits (future)

### File Locking System
- **Lock Acquisition**: Request exclusive access with timeout
- **Lock Status**: Visual indicators in project list
- **Lock Management View**: See who has locks and when they expire
- **Release Lock**: Voluntary unlock when work complete
- **Force Break**: Admin override with confirmation (use with caution)
- **Team Awareness**: Prevent simultaneous edits

### Merge Helper
- **Step-by-Step Workflow**: Guided process for manual merges
- **FCP XML Export**: Convert Logic Pro projects to diffable format
- **Branch Checkout**: Quick branch switching
- **Diff Tool Launch**: Open external diff tool (optional)
- **Documentation Link**: Access detailed merge protocol
- **Merge Commit**: Create merge commit when reconciliation complete

### Settings Panel
- **Daemon Status**: Monitor daemon health and uptime
- **Monitoring Control**: Pause/resume per project
- **Lock Configuration**: Default timeout settings (future)
- **Auto-refresh Interval**: Customize update frequency
- **Log Viewing**: Access daemon logs
- **Version Info**: Display Auxin version and build

## Architecture

### Component Structure

```
Auxin-App/
├── Package.swift                       # Swift Package Manager config
├── Sources/
│   ├── main.swift                      # Application entry point
│   ├── AppDelegate.swift               # App lifecycle & menu bar
│   │
│   ├── Models/
│   │   ├── Project.swift               # Project data model
│   │   ├── CommitInfo.swift            # Commit representation
│   │   ├── CommitMetadata.swift        # Structured metadata
│   │   └── LockInfo.swift              # Lock state model
│   │
│   ├── Services/
│   │   └── OxenDaemonXPCClient.swift   # XPC communication layer
│   │
│   ├── ViewModels/
│   │   ├── ProjectListViewModel.swift  # Project list logic
│   │   ├── ProjectDetailViewModel.swift # Single project logic
│   │   ├── CommitViewModel.swift       # Commit creation logic
│   │   └── SettingsViewModel.swift     # Settings logic
│   │
│   └── Views/
│       ├── MainViewController.swift    # Main window controller
│       ├── ProjectListView.swift       # Project list UI
│       ├── ProjectDetailView.swift     # Project detail UI
│       ├── MilestoneCommitWindow.swift # Commit creation window
│       ├── RollbackWindow.swift        # Rollback interface
│       ├── SettingsWindow.swift        # Settings panel
│       ├── ProjectWizardWindow.swift   # Initialization wizard
│       ├── MergeHelperWindow.swift     # Merge assistance
│       └── LockManagementView.swift    # Lock operations UI
│
├── Resources/
│   ├── Assets.xcassets/                # Icons and images
│   └── Info.plist                      # App configuration
│
└── Tests/
    ├── Auxin-AppTests.swift            # Unit tests
    └── TestUtils/
        └── MockXPCClient.swift         # Mock daemon for testing
```

### MVVM Architecture

```
User Interaction (View)
        ↓
    ViewModel (Business Logic)
        ↓
    Model (Data)
        ↓
OxenDaemonXPCClient (IPC)
        ↓
    XPC Mach Service
        ↓
Auxin-LaunchAgent Daemon
        ↓
    auxin
        ↓
    liboxen
```

### Data Flow Example: Milestone Commit

```
1. User fills out commit form (MilestoneCommitWindow)
        ↓
2. User clicks "Commit" button
        ↓
3. CommitViewModel validates inputs
        ↓
4. CommitViewModel calls OxenDaemonXPCClient.commitProject()
        ↓
5. XPC call to daemon: createMilestoneCommit(message, metadata)
        ↓
6. Daemon's CommitOrchestrator processes request
        ↓
7. Daemon calls auxin commit via Process
        ↓
8. Commit created in Oxen repository
        ↓
9. XPC response with commit hash or error
        ↓
10. CommitViewModel updates UI with result
        ↓
11. ProjectListViewModel refreshes to show new commit
```

## Installation

### Prerequisites

- macOS 14.0+ (Sonoma or later)
- Xcode 15+ (for building)
- Swift 5.9+
- Auxin-LaunchAgent daemon installed
- auxin in PATH

### Building from Source

```bash
cd Auxin-App

# Build via Swift Package Manager
swift build -c release

# Or open in Xcode
open Package.swift
# Then Product → Build (⌘B)

# Binary location: .build/release/Auxin
```

### Running the Application

```bash
# Run directly
.build/release/Auxin

# Or via Swift
swift run Auxin

# Or from Xcode (⌘R)
```

### Creating App Bundle (Future)

```bash
# Package as .app bundle
# (Requires additional configuration)
xcodebuild -scheme Auxin -configuration Release

# Result: build/Release/Auxin.app
```

## Usage

### Getting Started

1. **Launch Auxin App**
   - Double-click Auxin.app or run from Terminal
   - The app will check if the daemon is running

2. **Initialize Your First Project**
   - Click "Add Project..." in the menu or toolbar
   - Browse to your Logic Pro project (.logicx folder)
   - Click "Initialize Repository"
   - Wait for initialization to complete

3. **Monitor Project Status**
   - Your project appears in the project list
   - Auto-commits happen in the background (every 30s of inactivity)
   - Status updates every 30 seconds

4. **Create Milestone Commit**
   - Select your project
   - Click "Create Milestone Commit"
   - Fill in metadata (BPM, sample rate, key, tags)
   - Write descriptive commit message
   - Optionally enable pre-flight cleanup
   - Click "Commit"

5. **View Commit History**
   - Select your project
   - Click "View History"
   - Browse timeline of all commits
   - See metadata for each commit

6. **Rollback to Previous State**
   - In commit history, select desired commit
   - Click "Restore to This Commit"
   - Confirm in safety dialog
   - Logic Pro project files are restored

### Working with Locks (Team Collaboration)

```
# Acquire lock before starting work
1. Select project
2. Click "Acquire Lock"
3. Specify timeout (default 24 hours)
4. Work on project (only you can commit)

# Release lock when done
5. Click "Release Lock"
6. Lock removed, others can now edit
```

### Using the Merge Helper

```
# When you need to merge divergent branches
1. Open View → Merge Helper
2. Follow step-by-step workflow:
   a. Export current version to FCP XML
   b. Checkout other branch
   c. Export other version to FCP XML
   d. Use diff tool to compare
   e. Manually reconcile in Logic Pro
   f. Import reconciled XML
   g. Create merge commit
```

## XPC Communication

The app communicates with the daemon via XPC. Example usage:

```swift
import Foundation

// Get shared client instance
let client = OxenDaemonXPCClient.shared

// Example 1: Register a project
client.registerProject("/Users/me/Music/MyProject.logicx") { success, error in
    if success {
        print("✓ Project registered")
    } else {
        print("✗ Failed: \(error ?? "unknown")")
    }
}

// Example 2: Get monitored projects
client.getMonitoredProjects { projects in
    for project in projects {
        print("Monitoring: \(project)")
    }
}

// Example 3: Create milestone commit
let metadata = CommitMetadata(
    bpm: 128,
    sampleRate: 48000,
    keySignature: "A Minor",
    tags: ["mixing", "final"]
)

client.createMilestoneCommit(
    "/Users/me/Music/MyProject.logicx",
    message: "Final mix complete",
    metadata: metadata
) { commitHash, error in
    if let hash = commitHash {
        print("✓ Commit created: \(hash)")
    }
}

// Example 4: Acquire lock
client.acquireLock(
    for: "/Users/me/Music/MyProject.logicx",
    timeoutHours: 24
) { success, error in
    if success {
        print("✓ Lock acquired")
    } else {
        print("✗ Lock failed: \(error ?? "unknown")")
    }
}
```

## Configuration

### User Preferences

Stored in: `~/Library/Preferences/com.oxen.oxvcs.plist`

Settings include:
- Auto-refresh interval (default: 30s)
- Last project directory
- Window positions and sizes
- Lock timeout preferences
- Pre-flight cleanup defaults

### Daemon Connection

XPC Mach service name: `com.auxin.daemon.xpc`

The app automatically attempts to connect on launch. If the daemon is not running, the app will display a connection error and offer to start it.

## Testing

### Running Unit Tests

```bash
# Run all tests
swift test

# Run with verbose output
swift test --verbose

# Run specific test
swift test --filter ProjectListViewModelTests

# Run with code coverage
swift test --enable-code-coverage

# Generate coverage report
xcrun llvm-cov show \
  .build/debug/Auxin-AppPackageTests.xctest/Contents/MacOS/Auxin-AppPackageTests \
  -instr-profile=.build/debug/codecov/default.profdata \
  -format=html \
  -output-dir=coverage-report

open coverage-report/index.html
```

### Test Coverage

Current coverage: **50-70%** overall
- ViewModels: 80%+ (comprehensive tests with mock XPC)
- Models: 90%+ (data structures well-tested)
- Views: 30-40% (basic smoke tests)
- XPC Client: 70%+ (mocked responses)

See [TESTING_STRATEGY.md](../docs/TESTING_STRATEGY.md) for comprehensive testing approach.

### Manual Testing

```bash
# Test 1: Complete workflow
1. Launch app
2. Add new project
3. Make changes in Logic Pro
4. Wait for auto-commit
5. Create milestone commit
6. View history
7. Rollback to previous commit
8. Verify Logic Pro project state

# Test 2: Lock workflow
1. Acquire lock
2. Verify lock indicator in UI
3. Make commit (should succeed)
4. Release lock
5. Verify lock removed

# Test 3: UI responsiveness
1. Monitor 5 projects
2. Make changes to all
3. Verify UI remains responsive
4. Check CPU/memory usage
```

## Troubleshooting

### "Cannot connect to daemon"

```bash
# Check if daemon is running
ps aux | grep auxin-daemon

# Start daemon manually
auxin-daemon --daemon

# Check daemon logs
tail -f /tmp/com.auxin.daemon.stdout
```

### "Project initialization failed"

```bash
# Verify path is valid Logic Pro project
ls /path/to/project.logicx | grep projectData

# Check permissions
ls -la /path/to/project.logicx

# Ensure auxin is in PATH
which auxin
```

### UI not updating

```bash
# Check XPC connection
# Restart daemon:
launchctl stop com.auxin.daemon
launchctl start com.auxin.daemon

# Restart app
killall Auxin
open Auxin.app
```

### High memory usage

```bash
# Reduce auto-refresh frequency
# Settings → Auto-refresh interval → 60s

# Limit number of monitored projects
# Only add projects you're actively using

# Check for memory leaks
# Use Instruments.app: Product → Profile → Leaks
```

## Development

### Building for Development

```bash
# Quick build
swift build

# Build with optimizations
swift build -c release

# Clean build
swift package clean
swift build

# Run without building
swift run
```

### Debugging

```bash
# Run with lldb
lldb .build/debug/Auxin
(lldb) run

# Or from Xcode
# Set breakpoints and run (⌘R)

# View console logs
# Xcode: View → Debug Area → Show Debug Area (⌘⇧Y)
```

### Code Style

```bash
# Format code (if using SwiftFormat)
swiftformat Sources/

# Lint code (if using SwiftLint)
swiftlint

# Xcode: Editor → Format → Format File (⌃I)
```

### Adding New Features

1. Add model if needed (Sources/Models/)
2. Create ViewModel with business logic (Sources/ViewModels/)
3. Create View with UI (Sources/Views/)
4. Wire up XPC calls if daemon interaction needed
5. Add unit tests for ViewModels
6. Add UI smoke tests for Views
7. Update documentation

## Dependencies

### System Frameworks
- **AppKit**: Native macOS UI
- **Foundation**: Core Swift functionality
- **SwiftUI**: Modern UI components (if used)
- **Combine**: Reactive programming (if used)

### Internal Dependencies
- **OxenDaemonXPCClient**: XPC communication layer
- **Auxin-LaunchAgent**: Background daemon (runtime dependency)
- **auxin**: CLI tool (runtime dependency)

No external Swift Package Manager dependencies currently required.

## Related Documentation

- [Merge Protocol](../docs/MERGE_PROTOCOL.md) - Manual merge workflow
- [Testing Strategy](../docs/TESTING_STRATEGY.md) - Comprehensive testing
- [Implementation Plan](../docs/IMPLEMENTATION_PLAN.md) - Development roadmap (all phases complete)

## Screenshots

### Main Window
```
┌─────────────────────────────────────────────────┐
│ Auxin                                  ⚙ ➕ ⟲   │
├─────────────────────────────────────────────────┤
│ Project                 | Commits | Last Update │
├─────────────────────────────────────────────────┤
│ 🔒 MyTrack.logicx       │   47   │ 2m ago      │
│    SongProject.logicx   │   103  │ 1h ago      │
│    DemoSession.logicx   │   28   │ 3d ago      │
└─────────────────────────────────────────────────┘
```

### Commit Window
```
┌─────────────────────────────────────────────────┐
│ Create Milestone Commit                         │
├─────────────────────────────────────────────────┤
│ Message: [Final mix - ready for mastering___]  │
│                                                 │
│ BPM: [128]    Sample Rate: [48000] Hz          │
│ Key: [A Minor]  Time Sig: [4/4]                │
│                                                 │
│ Tags: [mixing, final, session-5]               │
│                                                 │
│ ☑ Pre-flight cleanup (remove temp files)       │
│                                                 │
│            [Cancel]  [Commit]                   │
└─────────────────────────────────────────────────┘
```

## Performance

### UI Responsiveness
- App launch: <1 second
- Project list refresh: <200ms
- Commit window open: <100ms
- XPC call roundtrip: <10ms

### Resource Usage
- Memory: 50-100MB resident
- CPU (idle): <1%
- CPU (refreshing): 2-5%

## Known Limitations

1. **No Automatic Merge**: Binary files require manual FCP XML workflow
2. **Single Window**: No multi-window support yet
3. **Basic Diff**: No visual diff viewer for project changes
4. **Lock Polling**: Lock status checked on refresh, not real-time push
5. **No Dark Mode**: UI uses system appearance but not optimized

## Future Enhancements

- [ ] Real-time lock notifications via NSXPCConnection delegate
- [ ] Visual diff viewer for project metadata
- [ ] Timeline visualization of commits
- [ ] Conflict detection UI
- [ ] Integration with Slack/Teams for lock notifications
- [ ] Multi-window support for managing multiple projects
- [ ] Preferences panel for all settings
- [ ] Automated FCP XML diff tool
- [ ] Dark mode optimizations
- [ ] Localization (internationalization)

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code style guidelines (Swift/AppKit)
- Testing requirements
- Pull request process
- UI/UX guidelines

## License

MIT License - See [LICENSE](../LICENSE) for details.
