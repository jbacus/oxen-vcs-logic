# AUXIN Project Rename - Complete Implementation Plan

**Status**: PLANNING
**Risk Level**: 🔴 **CRITICAL** - Incomplete execution will break the entire project
**Estimated Total Changes**: 900+ occurrences across 80+ files

---

## Executive Summary

This document provides a **complete, systematic plan** to rename the project from "auxin" / "Auxin" to "Auxin". This is a critical operation that requires 100% completeness - any missed references will cause:
- Broken builds
- Failed tests
- Broken installation
- User confusion
- Non-functional CLI commands

**Scope**: 900+ occurrences across Rust code, Swift code, documentation, tests, scripts, and configuration.

---

## Naming Convention Specification

### Old → New Mapping

| Pattern | Old Name | New Name | Occurrences | Usage Context |
|---------|----------|----------|-------------|---------------|
| **Repository** | `auxin` | `auxin` | ~15 | GitHub URLs, clone commands |
| **Binary/CLI** | `auxin` | `auxin` | ~400 | Binary name, command invocations |
| **Crate/Package** | `auxin` | `auxin` | ~5 | Cargo.toml, Package.swift |
| **Short form** | `oxenvcs` | `auxin` | ~40 | Code comments, docs |
| **Capitalized** | `Auxin` | `Auxin` | ~150 | Headers, class names, UI text |
| **Uppercase** | `OXENVCS` | `AUXIN` | ~40 | Environment variables |
| **Config dir** | `.auxin` | `.auxin` | ~35 | File system paths |
| **Service ID (CLI)** | `com.auxin.agent` | `com.auxin.agent` | ~5 | Rust daemon client |
| **Service ID (LaunchAgent)** | `com.auxin.daemon` | `com.auxin.daemon` | ~25 | macOS LaunchAgent & XPC |
| **App Bundle ID** | `com.auxin.app` | `com.auxin.app` | ~5 | macOS App Info.plist |
| **Directories** | `Auxin-*` | `Auxin-*` | ~10 | Directory structure |
| **GitHub Repo** | `jbacus/auxin` | `jbacus/auxin` | ~30 | Repository URLs |

### Detailed Naming Decisions

**Binary Name**: `auxin` → `auxin`
- Rationale: Simpler, cleaner, matches modern CLI conventions (git, cargo, npm)
- Impact: All command examples must change from `auxin` to `auxin`

**Crate Name**: `auxin` → `auxin`
- Impact: Cargo.toml package name, lib name
- Impact: Homebrew formula class name

**Directory Structure**:
```
OLD:                           NEW:
auxin/               auxin/
├── Auxin-CLI-Wrapper/        ├── Auxin-CLI-Wrapper/
├── Auxin-LaunchAgent/        ├── Auxin-LaunchAgent/
└── Auxin-App/                └── Auxin-App/
```

**Config Directory**: `~/.auxin/` → `~/.auxin/`
- Impact: User config path, queue directory, credentials storage
- Migration: Users will need to copy their config

**Environment Variables**:
```
AUXIN_VERBOSE      → AUXIN_VERBOSE
AUXIN_COLOR        → AUXIN_COLOR
AUXIN_LOCK_TIMEOUT → AUXIN_LOCK_TIMEOUT
AUXIN_MAX_RETRIES  → AUXIN_MAX_RETRIES
AUXIN_QUEUE_DIR    → AUXIN_QUEUE_DIR
AUXIN_AUTO_SYNC    → AUXIN_AUTO_SYNC
AUXIN_LOG          → AUXIN_LOG
AUXIN_DRAFT_BRANCH → AUXIN_DRAFT_BRANCH
AUXIN_MAX_DRAFT_COMMITS → AUXIN_MAX_DRAFT_COMMITS
```

**macOS Service Identifiers** (CRITICAL - Multiple IDs):
```
Rust Daemon Client (src/daemon_client.rs):
  com.auxin.agent → com.auxin.agent

Swift LaunchAgent (Auxin-LaunchAgent/Resources/*.plist):
  com.auxin.daemon → com.auxin.daemon
  com.auxin.daemon.xpc → com.auxin.daemon.xpc

Swift App (Auxin-App/Resources/Info.plist):
  com.auxin.app → com.auxin.app
```
- Impact: Users must unload old LaunchAgent before installing new one
- Impact: XPC service communication will fail if IDs don't match
- Impact: macOS app signing/notarization requires new bundle ID
- Migration: Critical for daemon/app communication

**GitHub Repository**: `jbacus/auxin` → `jbacus/auxin`
- Impact: All clone URLs, documentation links, badges
- Impact: GitHub Actions workflows (may auto-update)
- Impact: Homebrew tap URL (if published)
- Impact: Issue/PR links in documentation
- **Action Required**: Rename repository on GitHub **after** code changes complete

**Google Cloud Platform**: NO CHANGES REQUIRED
- Status: This codebase has NO GCP infrastructure
- No Cloud Run, Cloud Storage, Cloud Build, or service accounts
- Only references: `hub.oxen.ai` (Oxen.ai's cloud service - not our infrastructure)

**Completion Files**:
```
_auxin        → _auxin
auxin.bash    → auxin.bash
auxin.fish    → auxin.fish
auxin.ps1     → auxin.ps1
```

**Homebrew Formula**:
```
Class: OxenvcsLi    → Auxin
File: auxin.rb → auxin.rb
```

---

## Implementation Strategy

### Phase 1: Preparation & Validation (CRITICAL - DO FIRST)

**Step 1.1**: Create clean Git branch
```bash
git checkout -b rename-to-auxin
git push -u origin rename-to-auxin
```

**Step 1.2**: Backup current state
```bash
cd ..
cp -r auxin auxin-backup
```

**Step 1.3**: Verify all tests pass before rename
```bash
cd Auxin-CLI-Wrapper
cargo test --lib  # Should show 331 passed
```

**Step 1.4**: Document current state
- Record current test count: 331 unit tests
- Record current build success state
- Save list of all environment variables in use

---

### Phase 2: Core Build Files (DO THIS FIRST - HIGHEST RISK)

These files control the build system. Change these first to catch compilation errors early.

**Priority Order**:

**2.1 - Rust Cargo.toml** (Auxin-CLI-Wrapper/Cargo.toml)
```toml
OLD:
[package]
name = "auxin"
...
[[bin]]
name = "auxin"

NEW:
[package]
name = "auxin"
...
[[bin]]
name = "auxin"
```
- Lines to change: 2, 40
- Risk: High - breaks all subsequent builds if wrong

**2.2 - Rust lib.rs** (src/lib.rs)
- No package name changes needed (uses crate name automatically)
- Verify: Check for any hardcoded "oxenvcs" strings

**2.3 - Test after Cargo.toml change**:
```bash
cargo build --release
# Binary should now be: target/release/auxin
cargo test --lib
# Should still pass 331 tests
```

**2.4 - Swift Package.swift files** (if they exist)
- Search for `name:` fields
- Change: `Auxin-LaunchAgent` → `Auxin-LaunchAgent`
- Change: `Auxin-App` → `Auxin-App`

---

### Phase 3: Source Code - Rust (HIGH IMPACT)

Change in this exact order to maintain compilation:

**3.1 - Constants in daemon_client.rs** (src/daemon_client.rs)
```rust
OLD: const LAUNCH_AGENT_LABEL: &str = "com.auxin.agent";
NEW: const LAUNCH_AGENT_LABEL: &str = "com.auxin.agent";
```
- Lines: 11, 230, 245

**3.2 - Config paths in config.rs** (src/config.rs)
```rust
OLD: dirs::home_dir().map(|home| home.join(".auxin").join("config.toml"))
NEW: dirs::home_dir().map(|home| home.join(".auxin").join("config.toml"))
```
- Lines: 259, 266
- Also update all documentation comments (lines 1-34)

**3.3 - Queue directory in offline_queue.rs** (src/offline_queue.rs)
```rust
OLD: const DEFAULT_QUEUE_DIR: &str = ".auxin/queue";
NEW: const DEFAULT_QUEUE_DIR: &str = ".auxin/queue";
```
- Line: 48

**3.4 - Credentials path in auth.rs** (src/auth.rs)
```rust
OLD: home.join(".auxin").join("credentials.json")
NEW: home.join(".auxin").join("credentials.json")
```
- Lines: 108, 112, 119, 444, 457

**3.5 - Environment variables in config.rs** (src/config.rs:283-314)
```rust
OLD:
std::env::var("AUXIN_VERBOSE")
std::env::var("AUXIN_COLOR")
std::env::var("AUXIN_LOCK_TIMEOUT")
std::env::var("AUXIN_MAX_RETRIES")
std::env::var("AUXIN_QUEUE_DIR")

NEW:
std::env::var("AUXIN_VERBOSE")
std::env::var("AUXIN_COLOR")
std::env::var("AUXIN_LOCK_TIMEOUT")
std::env::var("AUXIN_MAX_RETRIES")
std::env::var("AUXIN_QUEUE_DIR")
```

**3.6 - Help text and error messages in main.rs** (src/main.rs)
- Update CLI description (line 8): "Oxen.ai CLI wrapper" → "Auxin - Version control for Logic Pro"
- Update all command examples showing `auxin` → `auxin`
- Lines: ~127 occurrences throughout file

**3.7 - Module documentation** (All src/*.rs files)
Update documentation examples in:
- src/oxen_subprocess.rs (lines 16, 58, 108)
- src/commit_metadata.rs (lines 27, 87, 118, etc. - 11 occurrences)
- src/logic_project.rs (lines 19, 74, 280, 309, 346)
- src/ignore_template.rs (lines 54, 63)
- src/hooks.rs (lines 1, 26, 27, 297, 299)
- src/search.rs (lines 19, 20)
- src/collaboration.rs (line 15)
- src/auth.rs (line 16)
- src/offline_queue.rs (line 18)
- src/network_resilience.rs (line 13)
- src/remote_lock.rs (line 48)
- src/draft_manager.rs (line 33)
- src/lock_integration.rs (lines 60, 74, 171, 191, 209)
- src/console/mod.rs (lines 1151, 1490, 1723)

**Test after each file**:
```bash
cargo build
cargo test --lib
```

---

### Phase 3A: Source Code - Swift (macOS Service IDs - CRITICAL)

**CRITICAL**: Swift files reference macOS service identifiers that enable IPC communication between the CLI, LaunchAgent, and App. These MUST match exactly.

**3A.1 - LaunchAgent plist** (Auxin-LaunchAgent/Resources/com.auxin.daemon.plist)
```xml
OLD:
<key>Label</key>
<string>com.auxin.daemon</string>
<key>MachServices</key>
<dict>
    <key>com.auxin.daemon.xpc</key>
    <true/>
</dict>

NEW:
<key>Label</key>
<string>com.auxin.daemon</string>
<key>MachServices</key>
<dict>
    <key>com.auxin.daemon.xpc</key>
    <true/>
</dict>
```
- **Rename file**: `com.auxin.daemon.plist` → `com.auxin.daemon.plist`
- Impact: LaunchAgent registration path changes to `~/Library/LaunchAgents/com.auxin.daemon.plist`

**3A.2 - XPCService.swift** (Auxin-LaunchAgent/Sources/XPCService.swift)
- Update service name constant: `com.auxin.daemon.xpc` → `com.auxin.daemon.xpc`
- Search for all references to the service identifier

**3A.3 - OxenDaemonXPCClient.swift** (Auxin-App/Sources/Services/OxenDaemonXPCClient.swift)
- Update XPC connection: `com.auxin.daemon.xpc` → `com.auxin.daemon.xpc`
- This is how the App communicates with the LaunchAgent

**3A.4 - ServiceManager.swift** (Auxin-LaunchAgent/Sources/ServiceManager.swift)
- Update service label: `com.auxin.daemon` → `com.auxin.daemon`
- Update plist path reference

**3A.5 - CommitOrchestrator.swift** (Auxin-LaunchAgent/Sources/CommitOrchestrator.swift)
- Check for any hardcoded service references

**3A.6 - FSEventsMonitor.swift** (Auxin-LaunchAgent/Sources/FSEventsMonitor.swift)
- Check for service ID references

**3A.7 - LockManager.swift** (Auxin-LaunchAgent/Sources/LockManager.swift)
- Check for service ID references

**3A.8 - App Info.plist** (Auxin-App/Resources/Info.plist)
```xml
OLD:
<key>CFBundleIdentifier</key>
<string>com.auxin.app</string>

NEW:
<key>CFBundleIdentifier</key>
<string>com.auxin.app</string>
```

**3A.9 - Built App plist** (Auxin-App/Auxin.app/Contents/Info.plist)
- Same change as above (this is the generated bundle)

**Test after Swift changes**:
```bash
# Can only test on macOS with Xcode
cd Auxin-LaunchAgent
swift build
swift test

cd ../Auxin-App
swift build
```

**CRITICAL VALIDATION**: After these changes, the following MUST match:
- LaunchAgent plist Label == LaunchAgent ServiceManager reference
- LaunchAgent XPC service name == App XPCClient connection name
- Any mismatch will cause complete IPC failure

---

### Phase 4: Configuration Files (CRITICAL PATHS)

**4.1 - config.toml.example**
- Update all `AUXIN_*` env var references → `AUXIN_*`
- Update config paths: `.auxin` → `.auxin`
- Update command examples: `auxin` → `auxin`
- Lines: 33, 50, 74, 75, 80, 81, 112-116, 125, 128, 160

**4.2 - .claude/settings.local.json**
- Update all 50+ occurrences of `auxin` → `auxin`
- This affects test configurations

**4.3 - Homebrew formula** (formula/auxin.rb → formula/auxin.rb)
- Rename file: `auxin.rb` → `auxin.rb`
- Change class name: `OxenvcsLi` → `Auxin`
- Update description
- Update homepage URL: `github.com/jbacus/auxin` → `github.com/jbacus/auxin`
- Update url field
- Update all installation references
- Lines: Entire file (~95 lines)

---

### Phase 5: Installation & Distribution Files

**5.1 - install.sh**
- Update binary references: `auxin` → `auxin`
- Update config directory: `.auxin` → `.auxin`
- Update completion file names
- Lines: 95, 105, 106, 110, 122, 123, 126, 127, 130, 234, 235, 294

**5.2 - Shell completion files**
- Regenerate with new binary name:
```bash
./target/release/auxin completions bash > completions/auxin.bash
./target/release/auxin completions zsh > completions/_auxin
./target/release/auxin completions fish > completions/auxin.fish
./target/release/auxin completions powershell > completions/auxin.ps1
```
- Delete old files:
```bash
rm completions/auxin.bash
rm completions/_auxin
rm completions/auxin.fish
rm completions/auxin.ps1
```

**5.3 - Test scripts** (test-scripts/)
- Update all script references to `auxin`
- test_network_resilience.sh: Update 20+ occurrences

---

### Phase 6: Documentation (EXTENSIVE)

**Priority order** (most visible to users first):

**6.1 - README.md**
- Update title: "Auxin" → "Auxin"
- Update badge URLs
- Update clone command
- Update all usage examples
- Lines: ~25+ occurrences

**6.2 - CLAUDE.md** (Critical for AI development)
- Update project overview
- Update directory structure diagram
- Update all command examples (~50+ occurrences)
- Update service identifiers
- Lines: 48-156 and throughout

**6.3 - INSTALL.md**
- Update title and all references
- Update clone URL
- Update directory paths
- Lines: 1, 3, 11, 12, 91, 120, 124, 153, 294, 308, 381

**6.4 - CLI_HELP.md**
- Complete rewrite of command examples
- Lines: 1-722 (200+ occurrences)

**6.5 - CLI_POLISH_COMPLETE.md**
- Update all feature documentation
- Update env var references
- Lines: Throughout (~50+ occurrences)

**6.6 - USAGE.md**
- Update all usage examples
- Update directory structure
- Lines: Throughout

**6.7 - CONTRIBUTING.md**
- Update directory references
- Lines: 71, 88, 102, 124, 125, 129, 133, 160, 165, 193

**6.8 - CHANGELOG.md**
- Add rename entry for current version
- Update historical references
- Lines: 73, 91-100 (30+ occurrences)

**6.9 - Other documentation**
- completions/README.md
- formula/README.md
- resources/hooks_readme.md
- TEST_COVERAGE_REPORT.md
- SESSION_SUMMARY.md
- test-scripts/user-guide-scenarios/QUICK_REFERENCE.md

---

### Phase 7: Tests (CRITICAL FOR VALIDATION)

**7.1 - Update test assertions**
Search for test code checking for "auxin" in output:
```rust
// Example patterns to find and fix:
assert!(output.contains("auxin"));  → assert!(output.contains("auxin"));
assert_eq!(cmd, "auxin");          → assert_eq!(cmd, "auxin");
```

**7.2 - Update integration tests**
- tests/example_test.rs
- tests/oxen_subprocess_integration_test.rs
- tests/restore_integration_test.rs

**7.3 - Update test fixtures**
Any fixtures that reference the old names

**7.4 - Run full test suite**:
```bash
cargo test --lib
cargo test
./run_all_tests.sh  # If exists
```

---

### Phase 8: Directory Structure Rename (DO LAST)

**ONLY after all code changes are complete and tested**:

```bash
# Move to parent directory
cd "/Users/johnbacus/My Projects/Unit3"

# Rename main directory
mv auxin auxin

# Rename subdirectories
cd auxin
mv Auxin-CLI-Wrapper Auxin-CLI-Wrapper
mv Auxin-LaunchAgent Auxin-LaunchAgent
mv Auxin-App Auxin-App

# Update any internal references to old directory names
```

**Update references after directory rename**:
- CLAUDE.md directory structure diagram
- CONTRIBUTING.md build paths
- README.md directory references

---

### Phase 9: Verification & Testing (COMPREHENSIVE)

**9.1 - Build verification**:
```bash
cd Auxin-CLI-Wrapper
cargo clean
cargo build --release
# Should create: target/release/auxin (not auxin)
```

**9.2 - Binary verification**:
```bash
./target/release/auxin --version
# Should output: auxin 0.1.0

./target/release/auxin --help
# Should show "Auxin" not "Auxin" or "auxin"
```

**9.3 - Test suite verification**:
```bash
cargo test --lib
# Should pass 331 tests

cargo test
# Should pass all tests including integration tests
```

**9.4 - Installation test**:
```bash
./install.sh
# Should install to /usr/local/bin/auxin

which auxin
# Should show: /usr/local/bin/auxin

auxin --version
# Should work correctly
```

**9.5 - Config test**:
```bash
# Should create ~/.auxin/ (not ~/.auxin/)
mkdir -p ~/.auxin
cp config.toml.example ~/.auxin/config.toml
auxin --help  # Should load config from ~/.auxin/
```

**9.6 - Completion test**:
```bash
auxin completions bash
# Should generate valid bash completion

auxin completions zsh
# Should generate valid zsh completion
```

**9.7 - Search for any remaining old names**:
```bash
# From repository root
grep -r "auxin" . --exclude-dir=target --exclude-dir=.git
grep -r "Auxin" . --exclude-dir=target --exclude-dir=.git
grep -r "OXENVCS" . --exclude-dir=target --exclude-dir=.git
grep -r ".auxin" . --exclude-dir=target --exclude-dir=.git
grep -r "com.auxin" . --exclude-dir=target --exclude-dir=.git
grep -r "auxin" . --exclude-dir=target --exclude-dir=.git

# These should return ZERO results (except this plan document)
```

---

### Phase 10: Migration Guide for Users

**10.1 - Create MIGRATION.md**:
Document for existing users explaining:
- Why the rename
- What changed
- How to migrate their config
- How to uninstall old version
- How to install new version

**10.2 - Migration steps for users**:
```bash
# 1. Backup old config
cp -r ~/.auxin ~/.auxin-backup

# 2. Uninstall old version
sudo rm /usr/local/bin/auxin
rm completions/auxin.*
rm completions/_auxin

# 3. Unload old LaunchAgent (if installed)
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist
rm ~/Library/LaunchAgents/com.auxin.agent.plist

# 4. Install new version
./install.sh

# 5. Copy config
cp -r ~/.auxin ~/.auxin

# 6. Update environment variables in shell config
# Change AUXIN_* → AUXIN_* in ~/.bashrc, ~/.zshrc, etc.
```

---

### Phase 11: GitHub Repository Rename (DO LAST - AFTER ALL CODE CHANGES)

**CRITICAL**: Only rename the GitHub repository **AFTER** all code changes are committed and tested. This prevents broken clone URLs during the transition.

**11.1 - Pre-Rename Checklist**:
- ✅ All code changes committed to `rename-to-auxin` branch
- ✅ All tests passing (331 unit tests)
- ✅ Installation script tested
- ✅ Documentation updated with new URLs (but still showing old repo)
- ✅ MIGRATION.md created for users

**11.2 - GitHub Repository Rename Procedure**:

1. **On GitHub web interface**:
   - Go to https://github.com/jbacus/auxin/settings
   - Scroll to "Repository name"
   - Change `auxin` → `auxin`
   - Click "Rename"
   - GitHub will automatically redirect old URLs to new repository

2. **Update local git remote**:
   ```bash
   cd auxin  # (after directory rename)
   git remote set-url origin https://github.com/jbacus/auxin.git
   git remote -v  # Verify new URL
   ```

3. **Merge rename branch to main**:
   ```bash
   git checkout main
   git merge rename-to-auxin
   git push origin main
   ```

4. **Create GitHub release**:
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0 - Project renamed to Auxin"
   git push origin v0.2.0
   ```

5. **Update GitHub repository description**:
   - Old: "Version control system for Logic Pro projects using Oxen.ai"
   - New: "Auxin - Version control for Logic Pro projects, powered by Oxen.ai"

6. **Update repository topics/tags**:
   - Add: `auxin`, `logic-pro`, `version-control`, `music-production`
   - Remove: `oxenvcs` (if present)

**11.3 - Post-Rename Actions**:

1. **Update GitHub Actions badges in README.md**:
   - Old: `https://github.com/jbacus/auxin/actions/workflows/test.yml`
   - New: `https://github.com/jbacus/auxin/actions/workflows/test.yml`
   - Note: Workflows themselves auto-update, only badge URLs need manual change

2. **Update Homebrew tap** (if exists):
   - Create new repo: `homebrew-auxin` (instead of `homebrew-oxenvcs`)
   - Update formula with new repository URL
   - Archive old `homebrew-oxenvcs` repo with deprecation notice

3. **Update package registry URLs**:
   - Homebrew formula: Update download URL to new GitHub repo
   - Any other package managers

4. **Verify GitHub redirect**:
   ```bash
   # Test that old URLs redirect properly
   curl -I https://github.com/jbacus/auxin
   # Should show 301 redirect to https://github.com/jbacus/auxin
   ```

**11.4 - Final Verification**:
```bash
# Clone from new URL
cd /tmp
git clone https://github.com/jbacus/auxin.git
cd auxin

# Verify build
cd Auxin-CLI-Wrapper
cargo build --release
./target/release/auxin --version  # Should show: auxin 0.2.0

# Verify installation
./install.sh
auxin --help  # Should work
```

**11.5 - Deprecation Notice** (Optional):
Add a deprecation notice to the old repository description via GitHub's redirect page:
- "This repository has been renamed to 'auxin'. You are being automatically redirected."

**Important Notes**:
- GitHub's automatic redirect is **permanent** - old URLs will always work
- Git clones using old URL will be automatically redirected
- Existing clones can update with: `git remote set-url origin <new-url>`
- No risk of breaking existing workflows (GitHub handles redirects)

---

## Execution Checklist

Use this checklist to track progress:

- [ ] **Phase 1**: Create branch, backup, verify tests (331 passing)
- [ ] **Phase 2**: Update Cargo.toml, verify build
- [ ] **Phase 3**: Update all Rust source code files
  - [ ] daemon_client.rs (service ID)
  - [ ] config.rs (paths + env vars)
  - [ ] offline_queue.rs (queue dir)
  - [ ] auth.rs (credentials path)
  - [ ] main.rs (help text + examples)
  - [ ] All module documentation
- [ ] **Phase 3A**: Update Swift source code (macOS service IDs)
  - [ ] LaunchAgent plist (rename file + update IDs)
  - [ ] XPCService.swift
  - [ ] OxenDaemonXPCClient.swift
  - [ ] ServiceManager.swift
  - [ ] App Info.plist (CFBundleIdentifier)
  - [ ] Verify all service IDs match
- [ ] **Phase 4**: Update configuration files
  - [ ] config.toml.example
  - [ ] .claude/settings.local.json
  - [ ] Homebrew formula (rename + update)
- [ ] **Phase 5**: Update installation files
  - [ ] install.sh
  - [ ] Regenerate completions
  - [ ] Test scripts
- [ ] **Phase 6**: Update documentation (13 files)
  - [ ] README.md
  - [ ] CLAUDE.md
  - [ ] INSTALL.md
  - [ ] CLI_HELP.md
  - [ ] CLI_POLISH_COMPLETE.md
  - [ ] USAGE.md
  - [ ] CONTRIBUTING.md
  - [ ] CHANGELOG.md (add rename entry)
  - [ ] Other docs
- [ ] **Phase 7**: Update tests
  - [ ] Test assertions
  - [ ] Integration tests
  - [ ] Test fixtures
  - [ ] Verify all tests pass
- [ ] **Phase 8**: Rename directories (DO LAST)
  - [ ] auxin → auxin
  - [ ] Auxin-* → Auxin-*
  - [ ] Update directory references
- [ ] **Phase 9**: Comprehensive verification
  - [ ] Build succeeds
  - [ ] Binary named correctly
  - [ ] 331 tests pass
  - [ ] Installation works
  - [ ] Config loads from ~/.auxin/
  - [ ] Completions generate
  - [ ] grep searches return zero old names
- [ ] **Phase 10**: Create migration guide
  - [ ] MIGRATION.md
  - [ ] User instructions
- [ ] **Phase 11**: GitHub repository rename (DO LAST)
  - [ ] Rename on GitHub web interface
  - [ ] Update local git remote
  - [ ] Merge to main and tag release
  - [ ] Update repository description & topics
  - [ ] Update README badges
  - [ ] Update Homebrew tap (if exists)
  - [ ] Verify redirect works

---

## Risk Mitigation

### Critical Risks

**Risk 1**: Incomplete rename leaves broken references
- **Mitigation**: Systematic grep verification after completion
- **Validation**: Run all searches and ensure zero results

**Risk 2**: Tests fail after rename
- **Mitigation**: Run tests after each phase
- **Validation**: 331 unit tests must all pass

**Risk 3**: Broken installation
- **Mitigation**: Test install.sh on clean environment
- **Validation**: Binary installs to correct path with correct name

**Risk 4**: Users lose their config/data
- **Mitigation**: Migration guide with backup instructions
- **Validation**: Document ~/.auxin → ~/.auxin migration

### Recovery Plan

If rename causes catastrophic issues:

```bash
# 1. Return to parent directory
cd "/Users/johnbacus/My Projects/Unit3"

# 2. Delete broken rename attempt
rm -rf auxin

# 3. Restore from backup
cp -r auxin-backup auxin

# 4. Checkout original branch
cd auxin
git checkout main
```

---

## Success Criteria

The rename is **ONLY** complete when ALL of the following are true:

1. ✅ All 331 unit tests pass
2. ✅ Binary is named `auxin` (not `auxin`)
3. ✅ `cargo build --release` succeeds
4. ✅ `./install.sh` creates `/usr/local/bin/auxin`
5. ✅ `auxin --version` outputs "auxin 0.1.0"
6. ✅ `auxin --help` shows "Auxin" branding
7. ✅ Config loads from `~/.auxin/config.toml`
8. ✅ Environment variables work with `AUXIN_*` prefix
9. ✅ grep searches for old names return ZERO results
10. ✅ All documentation shows "Auxin" consistently
11. ✅ Completions generate correctly with new names
12. ✅ macOS service IDs updated and matching:
    - LaunchAgent plist uses `com.auxin.daemon`
    - XPC service uses `com.auxin.daemon.xpc`
    - App bundle ID is `com.auxin.app`
13. ✅ GitHub repository renamed to `auxin`
14. ✅ Old GitHub URLs redirect correctly
15. ✅ MIGRATION.md exists with user instructions

---

## Timeline Estimate

| Phase | Estimated Time | Risk Level |
|-------|----------------|------------|
| Phase 1: Preparation | 15 min | Low |
| Phase 2: Build files | 30 min | 🔴 Critical |
| Phase 3: Rust code | 2-3 hours | 🔴 Critical |
| Phase 3A: Swift code (service IDs) | 1-2 hours | 🔴 Critical |
| Phase 4: Config files | 1 hour | High |
| Phase 5: Installation | 1 hour | High |
| Phase 6: Documentation | 3-4 hours | Medium |
| Phase 7: Tests | 1-2 hours | 🔴 Critical |
| Phase 8: Directory rename | 30 min | Medium |
| Phase 9: Verification | 1-2 hours | 🔴 Critical |
| Phase 10: Migration guide | 1 hour | Low |
| Phase 11: GitHub rename | 30 min | Low |
| **TOTAL** | **14-19 hours** | **🔴 CRITICAL** |

---

## Notes

- **DO NOT** execute this rename in pieces. Either commit to doing it completely or don't start.
- **DO NOT** commit partial changes. Complete entire rename before committing.
- **TEST** after every phase to catch issues early.
- **BACKUP** before starting - this is not reversible once committed to Git.
- **COMMUNICATE** with users before releasing - provide migration guide.

---

*This plan was generated on 2025-11-17 based on comprehensive codebase analysis.*
*Total patterns identified: 900+ occurrences across 80+ files.*
*Includes: Rust code, Swift code, documentation, tests, GitHub repository, macOS service IDs.*
*NO Google Cloud Platform changes required (no GCP infrastructure in this project).*
