# OxVCS LaunchAgent

Persistent background daemon for file system monitoring and automatic draft commits.

## Responsibilities

- FSEvents-based monitoring of Logic Pro project folders
- Debounced change detection (30-60s inactivity threshold)
- Automatic draft commits on save
- Power management observers (sleep/shutdown)
- Emergency commits before system suspension
- IPC listener for commands from main app
- File lock enforcement

## Structure

```
OxVCS-LaunchAgent/
├── Sources/
│   ├── main.swift              # Daemon entry point
│   ├── FSEventsMonitor.swift   # File system monitoring
│   ├── PowerManager.swift      # Sleep/shutdown handling
│   ├── DraftCommitter.swift    # Auto-commit logic
│   ├── IPCService.swift        # Communication with UI app
│   └── LockManager.swift       # File locking enforcement
├── Resources/
│   └── com.oxenvcs.agent.plist # LaunchAgent configuration
└── Tests/
```

## Build Requirements

- Xcode 15+
- FSEvents framework
- Foundation/AppKit for power notifications

## Installation

The LaunchAgent is registered by the main UI app using SMAppService.
Manual registration for development:

```bash
launchctl load ~/Library/LaunchAgents/com.oxenvcs.agent.plist
```

## Implementation Status

See [IMPLEMENTATION_PLAN.md](../docs/IMPLEMENTATION_PLAN.md) Phase 2.1-2.4
