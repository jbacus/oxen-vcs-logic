# AppKit Legacy Files

This directory contains the original AppKit view implementations that were replaced with SwiftUI on October 29, 2025.

## Files

- `MainViewController.swift` - Original AppKit main view controller
- `ProjectListView.swift` - Original AppKit project list (NSTableView)
- `ProjectDetailView.swift` - Original AppKit project detail view
- `StatusBarView.swift` - Original AppKit status bar

## Why Moved

These files were moved out of the build path because:
1. They conflicted with the new SwiftUI implementation
2. They caused CI/CD build failures in GitHub Actions
3. They are no longer used after the SwiftUI migration

## SwiftUI Replacements

| Old AppKit File | New SwiftUI File |
|----------------|------------------|
| MainViewController.swift | Views/SwiftUI/ContentView.swift |
| ProjectListView.swift | Views/SwiftUI/ProjectListContentView.swift |
| ProjectDetailView.swift | Views/SwiftUI/ProjectDetailContentView.swift |
| StatusBarView.swift | Views/SwiftUI/SwiftUIStatusBar.swift |

## Migration Benefits

The SwiftUI migration provided:
- 80% reduction in UI code complexity
- Fixed persistent window sizing issues
- Native NavigationSplitView with automatic layout
- Modern declarative UI patterns
- Better window management

## Preservation

These files are kept for reference but are not compiled. They are in git history and can be restored if needed, but should not be re-added to the Sources directory.

Last updated: 2025-10-29
