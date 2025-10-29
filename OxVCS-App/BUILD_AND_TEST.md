# OxVCS App - Build and Test Guide

**Platform:** macOS 14.0+ (Sonoma or later)
**Requirements:** Xcode 15+, Swift 5.9+, Oxen CLI

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start](#quick-start)
3. [Detailed Build Instructions](#detailed-build-instructions)
4. [Running the App](#running-the-app)
5. [Testing Checklist](#testing-checklist)
6. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### System Requirements

```bash
# Check macOS version (must be 14.0+)
sw_vers

# Expected output:
# ProductName:    macOS
# ProductVersion: 14.x.x or higher
```

### Install Xcode Command Line Tools

```bash
xcode-select --install

# Verify installation
xcode-select -p
# Should output: /Applications/Xcode.app/Contents/Developer
```

### Install Oxen CLI

```bash
# Option 1: via pip (recommended)
pip3 install oxen-ai

# Option 2: via cargo
# cargo install oxen

# Verify installation
oxen --version
# Should output version number
```

### Build LaunchAgent Daemon

The app requires the OxVCS-LaunchAgent to be running for full functionality.

```bash
cd ../OxVCS-LaunchAgent
swift build -c release

# Install the daemon
./install_daemon.sh

# Verify daemon is running
launchctl list | grep com.oxenvcs
```

---

## Quick Start

### Automated Build (Recommended)

```bash
cd /path/to/oxen-vcs-logic

# Run the complete installation script
./install.sh

# This will:
# 1. Build Rust CLI wrapper
# 2. Build LaunchAgent daemon
# 3. Build GUI app
# 4. Install everything to system locations
# 5. Launch the app
```

### Manual Build (Development)

```bash
cd OxVCS-App

# Build debug version
swift build

# Build release version
swift build -c release

# Run directly
.build/debug/OxVCS
```

---

## Detailed Build Instructions

### Step 1: Clean Build

```bash
cd OxVCS-App

# Remove any previous build artifacts
rm -rf .build

# Clean Swift package cache (if needed)
swift package clean
```

### Step 2: Resolve Dependencies

```bash
# Resolve Swift package dependencies
swift package resolve

# This should complete quickly as we have no external dependencies
```

### Step 3: Build

```bash
# Debug build (faster, includes debug symbols)
swift build

# Release build (optimized, recommended for testing)
swift build -c release

# Verbose output (for debugging build issues)
swift build -c release -v
```

### Step 4: Verify Build

```bash
# Check that binary was created
ls -lh .build/release/OxVCS

# Expected output:
# -rwxr-xr-x  1 user  staff   XXX KB  Date  OxVCS
```

---

## Running the App

### From Command Line

```bash
# Run debug build
.build/debug/OxVCS

# Run release build
.build/release/OxVCS
```

### From Xcode

```bash
# Open package in Xcode
open Package.swift

# Or use:
xed .

# In Xcode:
# 1. Select "OxVCS" scheme (top left)
# 2. Press ⌘R to build and run
# 3. Check console for any errors
```

### As App Bundle

```bash
# Create app bundle using install script
cd ..
./install.sh

# Launch installed app
open /Applications/OxVCS.app

# Or from Finder: Applications → OxVCS
```

---

## Testing Checklist

### Pre-Testing Setup

```bash
# 1. Ensure daemon is running
launchctl list | grep com.oxenvcs
# Should show: com.oxenvcs.agent with a PID

# 2. Create a test Logic Pro project
# Open Logic Pro, create a new project, save as:
# ~/Music/Test-Project.logicx

# 3. Verify Oxen CLI works
oxen --version
oxen init --help
```

### Phase 1: App Launch Tests

- [ ] **Test 1.1:** App launches without crashing
  ```bash
  .build/release/OxVCS
  ```
  **Expected:** Main window appears

- [ ] **Test 1.2:** Window layout is correct
  - Split view with project list (left) and placeholder (right)
  - Toolbar with "Add Project" and "Refresh" buttons
  - Status bar at bottom showing daemon status

- [ ] **Test 1.3:** Daemon status updates
  - Status bar should show "Daemon: Running" in green
  - If red, check: `launchctl list | grep com.oxenvcs`

- [ ] **Test 1.4:** Menu bar is functional
  - Click each menu to verify it opens
  - Check keyboard shortcuts are displayed

### Phase 2: Project Management Tests

- [ ] **Test 2.1:** Add new project
  1. Click toolbar "Add Project" or press ⌘N
  2. Project Wizard window opens
  3. Click "Browse..." button
  4. Navigate to ~/Music/Test-Project.logicx
  5. Select the .logicx file
  6. Status shows "Ready to initialize" (green)
  7. Click "Initialize"
  8. Progress spinner appears
  9. Success message displays
  10. Window closes automatically

- [ ] **Test 2.2:** Project appears in list
  - Project "Test-Project" visible in left pane
  - Shows full path below name
  - Shows "0 commits • No commits yet" status
  - No lock icon visible

- [ ] **Test 2.3:** Select project
  - Click on project in list
  - Right pane changes from placeholder to detail view
  - Project name and path displayed at top
  - Three buttons visible: Milestone Commit, Rollback, Lock Management
  - Empty commit history table

### Phase 3: Commit Tests

- [ ] **Test 3.1:** Create milestone commit
  1. Click "Milestone Commit" button or press ⌘K
  2. Window opens with form fields
  3. Enter commit message: "Test initial commit"
  4. Enter BPM: 120
  5. Enter Sample Rate: 48000
  6. Enter Key: "C Major"
  7. Enter Tags: "test, initial"
  8. Cleanup checkbox is checked (default)
  9. Click "Commit" or press Enter
  10. Progress spinner shows
  11. Success dialog appears
  12. Window closes

- [ ] **Test 3.2:** Commit appears in history
  - Main window refreshes
  - Commit table shows 1 entry
  - Commit hash (7 chars), message, date, author visible
  - Status shows "1 commits • Last: Just now"

- [ ] **Test 3.3:** Multiple commits
  1. Make a change to the Logic project (open, edit, save)
  2. Wait 30-60 seconds for auto-commit (draft)
  3. Refresh project list (⌘R)
  4. Create another milestone commit
  5. Both commits visible in history

### Phase 4: Rollback Tests

- [ ] **Test 4.1:** Open rollback window
  1. Click "Rollback" button
  2. Window opens showing commit history
  3. Warning message visible at top
  4. All commits listed in table

- [ ] **Test 4.2:** Rollback to previous commit
  1. Select first commit (oldest)
  2. Click "Rollback" button
  3. Confirmation dialog shows:
     - Commit hash and message
     - Date and author
     - Warning about irreversibility
  4. Click "Rollback"
  5. Progress spinner shows
  6. Success message appears
  7. Window closes

- [ ] **Test 4.3:** Verify rollback worked
  - Project files reverted to old state
  - New commit created: "Rollback to <hash>"
  - History shows rollback commit

### Phase 5: Lock Management Tests

- [ ] **Test 5.1:** Check lock status
  1. Click "Lock Management" button or press ⌘L
  2. Window opens
  3. Status shows "🔓 Not Locked" (green)
  4. Lock info shows "No active lock"
  5. "Acquire Lock" enabled
  6. "Release Lock" and "Force Break Lock" disabled

- [ ] **Test 5.2:** Acquire lock
  1. Click "Acquire Lock"
  2. Dialog prompts for timeout hours
  3. Enter: 24
  4. Click "Acquire"
  5. Status updates to "🔒 Locked" (orange)
  6. Lock info shows:
     - Locked by: your-username@hostname
     - Acquired: current timestamp
     - Expires: 24 hours from now

- [ ] **Test 5.3:** Release lock
  1. Click "Release Lock"
  2. Confirmation dialog appears
  3. Click "Release"
  4. Status back to "🔓 Not Locked"
  5. Lock info cleared

- [ ] **Test 5.4:** Force break lock
  1. Acquire lock again
  2. Click "Force Break Lock"
  3. **CRITICAL** warning dialog appears
  4. Click "Force Break"
  5. Lock removed

### Phase 6: Settings Tests

- [ ] **Test 6.1:** Open settings
  1. Press ⌘, or menu: OxVCS → Preferences
  2. Settings window opens
  3. Daemon status shows "Daemon is running" (green)

- [ ] **Test 6.2:** Change debounce time
  1. Current value loads (default: 30)
  2. Change to: 60
  3. Click "Save Configuration"
  4. Success message appears
  5. Close and reopen settings
  6. Value persists: 60

- [ ] **Test 6.3:** Change lock timeout
  1. Current value loads (default: 24)
  2. Change to: 48
  3. Click "Save Configuration"
  4. Success message appears

- [ ] **Test 6.4:** Input validation
  1. Try debounce: 300 → saves
  2. Try debounce: 301 → error "must be between 5 and 300"
  3. Try lock timeout: 169 → error "must be between 1 and 168"

### Phase 7: Merge Helper Tests

- [ ] **Test 7.1:** Open merge helper
  1. Select a project
  2. Menu: View → Merge Helper
  3. Window opens with instructions
  4. All 5 steps visible with action buttons

- [ ] **Test 7.2:** Open project in Logic
  1. Click "Open Project in Logic Pro"
  2. Logic Pro launches with the project

- [ ] **Test 7.3:** Open diff tool
  1. Click "Open Diff Tool"
  2. File picker appears for first XML
  3. After selection, second file picker appears
  4. Diff tool (opendiff) launches with both files

### Phase 8: Keyboard Shortcut Tests

- [ ] ⌘N - Opens Project Wizard
- [ ] ⌘K - Creates Milestone Commit (with project selected)
- [ ] ⌘R - Refreshes project list
- [ ] ⌘L - Opens Lock Management (with project selected)
- [ ] ⌘, - Opens Settings
- [ ] ⌘W - Closes current window
- [ ] ⌘Q - Quits app

### Phase 9: UI Polish Tests

- [ ] Window resizing works smoothly
- [ ] Split view divider is draggable
- [ ] Minimum window size enforced (800x600)
- [ ] Status bar updates every 5 seconds
- [ ] SF Symbols display correctly on buttons
- [ ] Colors are appropriate (green=success, red=error, orange=warning)
- [ ] Text truncates properly for long paths
- [ ] Table views scroll smoothly
- [ ] Progress indicators animate

### Phase 10: Error Handling Tests

- [ ] **Test 10.1:** Daemon not running
  1. Stop daemon: `launchctl unload ~/Library/LaunchAgents/com.oxenvcs.agent.plist`
  2. Launch app
  3. Status bar shows "Daemon: Not Running" (red)
  4. Try to add project → error dialog

- [ ] **Test 10.2:** Invalid project path
  1. Try to initialize a non-.logicx file
  2. Error: "Selected file must be a Logic Pro project"

- [ ] **Test 10.3:** Missing required fields
  1. Open Milestone Commit window
  2. Leave message empty
  3. Click Commit
  4. Error: "Please enter a commit message"

### Phase 11: Memory and Performance Tests

- [ ] App starts quickly (<2 seconds)
- [ ] No memory leaks (check Activity Monitor after 10 minutes)
- [ ] CPU usage low when idle (<5%)
- [ ] Status bar updates don't cause lag
- [ ] Large project lists (10+ projects) render smoothly
- [ ] Large commit histories (100+ commits) scroll smoothly

---

## Running Unit Tests

```bash
cd OxVCS-App

# Run all tests
swift test

# Run with verbose output
swift test -v

# Run specific test target
swift test --filter OxVCS-AppTests

# Generate code coverage (requires Xcode)
swift test --enable-code-coverage

# View coverage report
xcodebuild test -scheme OxVCS -enableCodeCoverage YES
```

---

## Troubleshooting

### Build Failures

**Issue:** "Command line tools are not installed"
```bash
xcode-select --install
```

**Issue:** "Swift version X.X is not supported"
```bash
# Update Xcode to latest version
# Or check Package.swift swift-tools-version
```

**Issue:** "Build failed with unresolved symbols"
```bash
# Clean and rebuild
rm -rf .build
swift package clean
swift build
```

### Runtime Failures

**Issue:** App crashes on launch
```bash
# Run with debug output
.build/debug/OxVCS

# Check Console.app for crash logs
# Filter by "OxVCS"
```

**Issue:** "Daemon: Not Running"
```bash
# Check daemon status
launchctl list | grep com.oxenvcs

# Manually start daemon
cd ../OxVCS-LaunchAgent
.build/release/OxVCS-LaunchAgent &

# Or install as service
./install_daemon.sh
```

**Issue:** "Failed to initialize project"
```bash
# Check Oxen CLI is installed
which oxen
oxen --version

# Check oxenvcs-cli is installed
which oxenvcs-cli
/usr/local/bin/oxenvcs-cli --version

# If missing, build CLI wrapper
cd ../OxVCS-CLI-Wrapper
cargo build --release
sudo cp target/release/oxenvcs-cli /usr/local/bin/
```

**Issue:** XPC connection failures
```bash
# Check daemon is running
ps aux | grep OxVCS-LaunchAgent

# Check XPC service name matches
# In code: "com.oxen.logic.daemon.xpc"
# Should match daemon's registered service name
```

### UI Issues

**Issue:** Window doesn't appear
```bash
# Check if window is off-screen
# Press ⌘M to minimize, then click dock icon
# Or delete preferences:
defaults delete com.oxen.logic.OxVCS
```

**Issue:** Toolbar icons missing
- Requires macOS 14.0+ for SF Symbols
- Update system or use text-only buttons

**Issue:** Status bar not updating
- Check daemon is running and responding
- Try manual refresh (⌘R)

---

## Performance Benchmarks

**Expected Performance:**

| Operation | Expected Time | Acceptable Time |
|-----------|---------------|-----------------|
| App launch | <2 seconds | <5 seconds |
| Project init | <5 seconds | <15 seconds |
| Commit (small) | <2 seconds | <10 seconds |
| Commit (large, 1GB) | <30 seconds | <2 minutes |
| Rollback | <5 seconds | <30 seconds |
| Lock acquire | <1 second | <3 seconds |
| History load (100 commits) | <1 second | <3 seconds |

---

## Continuous Integration

### GitHub Actions (Example)

```yaml
name: Build and Test OxVCS App

on: [push, pull_request]

jobs:
  test:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v3

      - name: Install dependencies
        run: |
          pip3 install oxen-ai

      - name: Build app
        run: |
          cd OxVCS-App
          swift build -c release

      - name: Run tests
        run: |
          cd OxVCS-App
          swift test
```

---

## Reporting Bugs

When reporting issues, include:

1. **System Info:**
   ```bash
   sw_vers
   xcode-select -p
   swift --version
   ```

2. **Build Info:**
   ```bash
   cd OxVCS-App
   swift build 2>&1 | tee build.log
   ```

3. **Runtime Logs:**
   ```bash
   # App logs
   .build/debug/OxVCS 2>&1 | tee run.log

   # Daemon logs
   log show --predicate 'process == "OxVCS-LaunchAgent"' --last 1h > daemon.log
   ```

4. **Steps to Reproduce**
5. **Expected vs Actual Behavior**
6. **Screenshots** (if applicable)

Submit to: https://github.com/jbacus/oxen-vcs-logic/issues

---

## Next Steps After Testing

1. **Create App Bundle**
   ```bash
   cd ..
   ./create_app_bundle.sh
   ```

2. **Notarize for Distribution** (Apple Developer Account required)
   ```bash
   # Sign the app
   codesign --deep --force --verify --verbose --sign "Developer ID Application: YOUR_NAME" /Applications/OxVCS.app

   # Submit for notarization
   xcrun notarytool submit /Applications/OxVCS.app --apple-id YOUR_EMAIL --password APP_SPECIFIC_PASSWORD --team-id YOUR_TEAM_ID
   ```

3. **Create DMG Installer**
   ```bash
   # Use create-dmg or similar tool
   create-dmg --volname "OxVCS Installer" --window-size 600 400 --icon OxVCS.app 200 200 --app-drop-link 400 200 OxVCS-Installer.dmg /Applications/OxVCS.app
   ```

---

**Happy Testing! 🚀**

For more information, see:
- [UI_IMPLEMENTATION_SUMMARY.md](UI_IMPLEMENTATION_SUMMARY.md)
- [../docs/USER_GUIDE.md](../docs/USER_GUIDE.md)
- [../CONTRIBUTING.md](../CONTRIBUTING.md)
