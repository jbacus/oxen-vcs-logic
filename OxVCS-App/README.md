# OxVCS Main Application

Swift/AppKit UI application for user interaction with the Oxen-VCS system.

## Responsibilities

- Repository browser and history visualization
- Project initialization wizard
- Milestone commit interface
- Rollback/restore operations
- Settings and preferences
- LaunchAgent registration via SMAppService

## Structure

```
OxVCS-App/
├── OxVCS.xcodeproj/
├── Sources/
│   ├── Views/           # UI components
│   ├── ViewModels/      # Business logic
│   ├── Models/          # Data structures
│   ├── Services/        # Oxen integration layer
│   └── Utilities/       # Helpers
├── Resources/
│   ├── Assets.xcassets/
│   └── Info.plist
└── Tests/
```

## Build Requirements

- Xcode 15+
- macOS 14.0+ deployment target
- Swift 5.9+

## Development

```bash
open OxVCS.xcodeproj
# Build and run (⌘R)
```

## Implementation Status

See [IMPLEMENTATION_PLAN.md](../docs/IMPLEMENTATION_PLAN.md) Phase 3.1
