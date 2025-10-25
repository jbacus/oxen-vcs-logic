# Phase 3 Implementation Quick Reference

## What's Already Built (Ready to Use)

### Backend Service (100% Complete)
- [x] LaunchAgent daemon running in background
- [x] File system monitoring (FSEvents)
- [x] Auto-commit workflow with 30s debounce
- [x] Power management (emergency commits)
- [x] Draft branch system
- [x] **XPC service** for UI communication

### CLI Tool (100% Complete)
- [x] Project detection and initialization
- [x] Commit operations with metadata
- [x] History retrieval
- [x] Restore functionality

### Available Interfaces
**XPC Protocol** (ready to call from UI):
- `registerProject(path)` - Start monitoring
- `unregisterProject(path)` - Stop monitoring
- `getMonitoredProjects()` - List tracked projects
- `commitProject(path, message)` - Manual commit
- `pauseMonitoring(path)` - Pause auto-commits
- `resumeMonitoring(path)` - Resume auto-commits
- `getCommitHistory(path, limit)` - Get history
- `restoreProject(path, commitId)` - Rollback
- `getStatus()` - Daemon status
- `ping()` - Health check

---

## What Needs to Be Built (Phase 3)

### 3.1: Main UI Application (AppKit/SwiftUI)
**Files to create**: `OxVCS-App/Sources/...`

Essential Views:
1. **ProjectListView** - Shows monitored projects
2. **ProjectDetailsView** - History, timeline
3. **MilestoneCommitView** - Manual commit with metadata (BPM, sample rate, tags)
4. **RollbackView** - Select and restore commit
5. **SettingsView** - Daemon control, preferences

**ViewModels needed**:
- `ProjectListViewModel` - Load from XPC
- `HistoryViewModel` - Fetch commits
- `CommitViewModel` - Handle manual commits

**Data Models**:
- `Project` - Path, name, status
- `CommitInfo` - ID, message, timestamp, metadata
- `DaemonStatus` - Running, project count, etc.

### 3.2: Exclusive File Locking System
**Files to create**: 
- `OxVCS-LaunchAgent/Sources/LockManager.swift` (daemon)
- `OxVCS-App/Sources/Models/Lock.swift` (UI models)

**Lock Manager** should implement:
```swift
class LockManager {
    func acquireLock(for projectPath: String) -> Result<LockInfo>
    func releaseLock(lockId: String) -> Result<Void>
    func checkLock(for projectPath: String) -> LockStatus
    func forceBreakLock(lockId: String, by admin: String) -> Result<Void>
    func cleanupStaleLocks() // Timeout-based cleanup
}
```

**Lock Enforcement Points**:
- In `CommitOrchestrator.performCommit()` - check lock before commit
- In `XPCService.commitProject()` - check before manual commit

### 3.3: Manual Merge Protocol
**Files to create**:
- Merge helper documentation
- FCP XML export/import utilities

**Note**: Full merge requires liboxen merge support (placeholder in draft_manager.rs)

### 3.4: Milestone Commit Pre-Flight
**Implement in**: `CommitOrchestrator` + `MilestoneCommitView`

**Pre-flight checks**:
1. Check for lock
2. Verify not already committing
3. Optionally clean volatile files:
   - Remove `Bounces/`
   - Remove `Freeze Files/`
   - Remove `Media.localized/`
4. Collect metadata (BPM, sample rate, etc.)
5. Confirm before committing
6. Execute: stage → commit → report

---

## Architecture Integration Points

### UI ↔ Daemon Communication
```
OxVCS App (SwiftUI/AppKit)
    ↓ (XPC calls)
OxenDaemonXPCClient
    ↓ (Mach service)
OxenDaemonXPCService (Daemon)
    ↓ (Process execution)
oxenvcs-cli (Rust)
    ↓ (Library calls)
liboxen (Oxen VCS)
```

### Key Integration Points
1. **XPC Client Setup** (in UI app)
   ```swift
   let client = OxenDaemonXPCClient()
   guard let proxy = client.getProxy() else { return }
   proxy.getMonitoredProjects { projects in
       // Update UI
   }
   ```

2. **Error Handling** - Handle daemon disconnection gracefully
3. **Polling** - May need to poll status periodically (no push notifications)
4. **Permissions** - App may need sandbox exceptions for file access

---

## Testing Strategy for Phase 3

### Unit Tests (Swift)
- View model logic
- Lock manager operations
- XPC proxy initialization

### Integration Tests
- Full project registration flow
- Commit submission and monitoring
- Lock lifecycle (acquire → use → release)
- Daemon disconnection recovery

### System Tests
- Multi-user locking scenarios
- Power event during manual commit
- Lock timeout cleanup
- UI responsiveness during long operations

---

## File Paths You'll Need

**Daemon XPC Protocol** (already defined):
- `/home/user/oxen-vcs-logic/OxVCS-LaunchAgent/Sources/XPCService.swift`

**Commit Orchestrator** (add lock checks here):
- `/home/user/oxen-vcs-logic/OxVCS-LaunchAgent/Sources/CommitOrchestrator.swift`

**LaunchAgent Plist** (daemon config):
- `/home/user/oxen-vcs-logic/OxVCS-LaunchAgent/Resources/com.oxen.logic.daemon.plist`

**CLI Operations** (available through XPC):
- `/home/user/oxen-vcs-logic/OxVCS-CLI-Wrapper/src/oxen_ops.rs`

---

## Quick Implementation Checklist

- [ ] Create OxVCS.xcodeproj structure
- [ ] Implement OxenDaemonXPCClient in app
- [ ] Build ProjectListView with monitored projects
- [ ] Build ProjectDetailsView with history
- [ ] Build MilestoneCommitView with metadata
- [ ] Build RollbackView
- [ ] Create LockManager in daemon
- [ ] Integrate lock checks in CommitOrchestrator
- [ ] Build pre-flight checks for milestone commits
- [ ] Add SettingsView for daemon management
- [ ] Implement error handling for XPC failures
- [ ] Add unit tests for views/view models
- [ ] Add integration tests
- [ ] Test with real Logic Pro projects

---

## Success Criteria for Phase 3

✅ UI app can list all monitored Logic Pro projects
✅ User can view commit history for each project
✅ User can create milestone commits with metadata
✅ User can rollback to previous commits
✅ File locking prevents concurrent edits
✅ Emergency commits still happen on power events
✅ App gracefully handles daemon crashes
✅ All features work with multiple projects
✅ Settings allow daemon control from UI

---

## Known Limitations to Address

1. **Merge support** - liboxen may not have full merge; manual FCP XML merge documented
2. **Remote sync** - Not implemented; local-only for now
3. **XPC blocking** - Some calls may block UI; use async/await
4. **Permission model** - May need sandbox exceptions for file access

