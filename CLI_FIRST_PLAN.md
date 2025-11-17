# CLI-First Release Plan

**Date:** November 15, 2025
**Strategy:** Ship CLI-only first release with interactive console for monitoring and control
**Timeline:** 2-3 weeks
**Status:** Week 3 Complete ✅ (Advanced features implemented and tested)

---

## 🎯 Implementation Status

- ✅ **Week 1 Complete** - Visual feedback, log filtering, lock management CLI
- ✅ **Week 2 Complete** - Interactive console TUI with daemon monitoring
- ✅ **Week 3 Complete** - Advanced features (compare, search, hooks) + comprehensive testing
- **Total:** 349 tests passing (274 unit + 49 integration + 26 doctests)
- **Production Code:** 6,500 lines
- **Ready for:** User testing with real Logic Pro projects

---

## Executive Summary

**Goal:** Make Auxin fully functional via CLI with rich visual feedback, interactive features, and real-time monitoring.

**Why CLI-First:**
- ✅ Plays to our strengths (85% test coverage, working backend)
- ✅ Avoids GUI complexity and SwiftUI testing gaps
- ✅ Gets real user feedback faster
- ✅ Technical early adopters can dogfood immediately
- ✅ Daemon testing happens naturally during daily use

**Target Users for v0.1:**
- Music producers comfortable with Terminal
- Power users who prefer CLI workflows
- Early adopters willing to provide feedback
- Teams who need reliable version control NOW

---

## Current CLI Capabilities

### ✅ What Works Today

**Core Commands:**
```bash
auxin init --logic <path>          # Initialize Logic Pro project
auxin add --all                     # Stage changes
auxin commit -m "msg" --bpm 120     # Commit with metadata
auxin log --limit 10                # View history
auxin restore <commit_id>           # Restore to version
auxin status                        # Show working directory status
auxin metadata-diff <a> <b>         # Compare project versions
```

**Features:**
- ✅ Logic Pro project detection and validation
- ✅ .oxenignore generation
- ✅ Structured commit metadata (BPM, sample rate, key, tags)
- ✅ Short hash support for restore
- ✅ Colored output
- ✅ Verbose mode
- ✅ 335 tests passing (85% coverage)

### 🔴 What's Missing

**Critical Gaps:**
1. **No daemon integration** - Can't monitor/control the background service
2. **No interactive mode** - One-shot commands only
3. **No progress indicators** - Long operations feel frozen
4. **No real-time status** - Can't see what daemon is doing
5. **No watch mode** - Can't monitor changes live
6. **No branch visualization** - Can't see draft vs milestone branches
7. **No lock management** - Can't acquire/release locks from CLI
8. **No diff preview** - Can't see what changed before commit

**User Experience Gaps:**
- No visual feedback during long operations (init, commit, restore)
- No indication if daemon is running
- No way to see automatic draft commits happening
- No easy way to compare current state vs last commit
- No interactive history browsing

---

## CLI Enhancement Plan

### Phase 1: Visual Feedback & Progress (3 days)

**Goal:** Make existing commands feel responsive and informative

#### 1.1 Progress Indicators
Add spinners and progress bars for long-running operations:

```rust
// Using indicatif crate
use indicatif::{ProgressBar, ProgressStyle};

// During init
let pb = ProgressBar::new_spinner();
pb.set_message("Initializing Oxen repository...");
pb.enable_steady_tick(Duration::from_millis(120));
// ... do work ...
pb.finish_with_message("✓ Repository initialized");

// During commit
let pb = ProgressBar::new(100);
pb.set_style(ProgressStyle::default_bar()
    .template("{msg} [{bar:40}] {pos}/{len}")
    .progress_chars("=>-"));
pb.set_message("Committing changes");
// ... do work ...
pb.finish_with_message("✓ Commit created");
```

**Commands to enhance:**
- `init --logic` - Show validation steps, repository creation
- `add --all` - Show files being staged
- `commit` - Show staging, oxen commit, draft management
- `restore` - Show checkout progress
- `metadata-diff` - Show parsing progress for large projects

#### 1.2 Rich Status Output
Enhance `status` command with visual formatting:

```bash
$ auxin status

┌─ Repository Status ─────────────────────────────────────┐
│ Branch: draft                                            │
│ Last commit: 3 minutes ago                               │
│ Daemon: ● Running (monitoring enabled)                  │
└──────────────────────────────────────────────────────────┘

● Staged (2 files)
  + projectData
  + Resources/vocals.wav

◆ Modified (1 file)
  M Alternatives/000/DisplayState.plist

? Untracked (1 file)
  ? Resources/new-bass.wav

Next: auxin commit -m "Your message"
```

#### 1.3 Enhanced Log Output
Make commit history more visual:

```bash
$ auxin log --limit 5

┌─ Commit History ────────────────────────────────────────┐

● abc123f (draft) - 3 minutes ago
  │ Auto-save draft commit
  │
  └─ projectData, Alternatives/000/*

● def456a (main) - 2 hours ago
  │ Vocal tracking complete
  │ BPM: 128 | Key: C Major | Tags: tracking, vocals
  │ Author: john@example.com
  │
  └─ 8 files changed

● ghi789b (main) - 1 day ago
  │ Initial mix checkpoint
  │ BPM: 128 | Sample Rate: 48000 Hz
  │
  └─ 12 files changed
```

**Dependencies:**
```toml
[dependencies]
indicatif = "0.17"      # Progress bars
console = "0.15"        # Terminal utilities
dialoguer = "0.11"      # Interactive prompts
```

---

### Phase 2: Interactive Console (5 days)

**Goal:** Create a TUI (Text User Interface) for real-time monitoring and control

#### 2.1 Console Architecture

```bash
$ auxin console
```

**Interface Design:**

```
┏━ Auxin Console ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃                                                             ┃
┃ Project: ~/Music/MyTrack.logicx                           ┃
┃ Daemon:  ● Running (PID: 12345)                           ┃
┃ Branch:  draft                                             ┃
┃ Changes: 3 modified, 1 untracked                          ┃
┃                                                             ┃
┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛

┌─ Live Activity ──────────────────────────────────────────┐
│                                                           │
│ [14:32:15] File changed: projectData                     │
│ [14:32:15] File changed: Alternatives/000/...            │
│ [14:32:16] Debounce timer started (30s)                  │
│ [14:32:46] Creating draft commit...                      │
│ [14:32:48] ✓ Draft commit created: abc123f               │
│                                                           │
│                                                           │
│                                                           │
└───────────────────────────────────────────────────────────┘

┌─ Recent Commits ─────────────────────────────────────────┐
│                                                           │
│ abc123f  3 seconds ago   Auto-save draft               │
│ def456a  2 hours ago     Vocal tracking complete       │
│ ghi789b  1 day ago       Initial mix checkpoint        │
│                                                           │
└───────────────────────────────────────────────────────────┘

 Commands: [c]ommit  [r]estore  [s]tatus  [l]ock  [q]uit
```

#### 2.2 TUI Framework

**Use `ratatui` (modern TUI framework):**

```toml
[dependencies]
ratatui = "0.25"        # TUI framework
crossterm = "0.27"      # Terminal control
tokio = { version = "1", features = ["full"] }
```

**Implementation Structure:**

```rust
// src/console/mod.rs
pub struct Console {
    daemon_client: DaemonClient,
    project_path: PathBuf,
    activity_log: Vec<LogEntry>,
    commits: Vec<CommitInfo>,
    status: RepositoryStatus,
}

impl Console {
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        let mut terminal = setup_terminal()?;

        // Main event loop
        loop {
            // Render UI
            terminal.draw(|f| self.render(f))?;

            // Handle events (keyboard, daemon updates)
            if let Some(event) = self.poll_events().await? {
                match event {
                    Event::Quit => break,
                    Event::Commit => self.create_milestone_commit().await?,
                    Event::Restore => self.interactive_restore().await?,
                    Event::DaemonUpdate(update) => self.handle_daemon_event(update),
                    _ => {}
                }
            }
        }

        restore_terminal(terminal)?;
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        // Header
        let header = self.render_header();
        frame.render_widget(header, chunks[0]);

        // Live activity log
        let activity = self.render_activity_log();
        frame.render_widget(activity, chunks[1]);

        // Recent commits
        let commits = self.render_commits();
        frame.render_widget(commits, chunks[2]);

        // Footer with commands
        let footer = self.render_footer();
        frame.render_widget(footer, chunks[3]);
    }
}
```

#### 2.3 Daemon Integration

**Add XPC/IPC client to Rust CLI:**

```rust
// src/daemon_client.rs
pub struct DaemonClient {
    connection: UnixStream,  // or XPC bridge
}

impl DaemonClient {
    pub async fn get_status(&self) -> Result<DaemonStatus> {
        // Call daemon XPC endpoint
    }

    pub async fn subscribe_to_events(&self) -> Result<EventStream> {
        // Stream real-time events from daemon
    }

    pub async fn trigger_commit(&self, message: String) -> Result<CommitId> {
        // Tell daemon to create commit
    }
}

pub struct DaemonStatus {
    pub running: bool,
    pub monitored_projects: Vec<PathBuf>,
    pub debounce_active: bool,
    pub time_until_commit: Option<Duration>,
}
```

#### 2.4 Interactive Features

**In-console commit creation:**
```
Press 'c' → Commit dialog appears

┌─ Create Milestone Commit ──────────────────────────┐
│                                                     │
│ Message: [Vocal tracking complete________]         │
│                                                     │
│ BPM: [128___]  Sample Rate: [48000__]              │
│                                                     │
│ Key: [C Major▼]                                    │
│      └─ C Major, A Minor, D Minor, ...             │
│                                                     │
│ Tags: [tracking, vocals___________]                │
│                                                     │
│         [Cancel]  [Create Commit]                  │
│                                                     │
└─────────────────────────────────────────────────────┘
```

**Interactive restore:**
```
Press 'r' → History browser appears

┌─ Restore to Previous Version ──────────────────────┐
│                                                     │
│ > abc123f  3 min ago   Auto-save draft            │
│   def456a  2 hrs ago   Vocal tracking complete    │
│   ghi789b  1 day ago   Initial mix checkpoint     │
│   jkl012c  2 days ago  Drum arrangement done      │
│                                                     │
│ ↑↓: Navigate  Enter: Preview  R: Restore  Esc: Cancel │
└─────────────────────────────────────────────────────┘
```

---

### Phase 3: Daemon Control & Monitoring (4 days)

**Goal:** Full daemon lifecycle management from CLI

#### 3.1 New Commands

```bash
# Check daemon status
auxin daemon status
> Daemon Status:
> Running: ✓ (PID: 12345)
> Monitored projects: 2
>   - ~/Music/Track1.logicx (active, 3 changes pending)
>   - ~/Music/Track2.logicx (idle)
> Next auto-commit: 23 seconds

# Start daemon
auxin daemon start

# Stop daemon
auxin daemon stop

# Restart daemon
auxin daemon restart

# View daemon logs (tail -f style)
auxin daemon logs --follow

# Add project to monitoring
auxin daemon monitor ~/Music/NewTrack.logicx

# Remove project from monitoring
auxin daemon unmonitor ~/Music/OldTrack.logicx

# Pause monitoring (useful during long edit sessions)
auxin daemon pause
auxin daemon resume

# Force immediate commit (override debounce)
auxin daemon commit-now
```

#### 3.2 Watch Mode

```bash
# Watch mode: continuous status updates
auxin watch

Output:
Every 2s: auxin status

┌─ Repository Status (Auto-refreshing) ───────────────┐
│ Last update: 14:32:48                                │
│                                                       │
│ ● Staged (0 files)                                  │
│                                                       │
│ ◆ Modified (3 files)                                │
│   M projectData                                      │
│   M Alternatives/000/DisplayState.plist              │
│   M Alternatives/000/RegionData.plist                │
│                                                       │
│ Daemon: Debounce active (18s until commit)          │
│                                                       │
│ Press Ctrl+C to exit                                 │
└───────────────────────────────────────────────────────┘
```

#### 3.3 Lock Management

```bash
# Acquire lock (for team workflows)
auxin lock acquire --timeout 4h
> ✓ Lock acquired
> Lock expires: 2025-11-15 18:30:00
> To release early: auxin lock release

# Check lock status
auxin lock status
> Lock Status:
> Locked by: john@macbook.local
> Acquired: 2025-11-15 14:30:00
> Expires: 2025-11-15 18:30:00 (3h 45m remaining)

# Release lock
auxin lock release
> ✓ Lock released

# Force break lock (admin only)
auxin lock break --force
> ⚠ Warning: This will break the lock held by jane@macbook.local
> Are you sure? [y/N]: y
> ✓ Lock forcibly broken
```

---

### Phase 4: Advanced Features (3 days)

#### 4.1 Diff Visualization

**Enhance diff to show file-level changes:**

```bash
auxin diff

Output:
┌─ Changes Since Last Commit ─────────────────────────┐
│                                                       │
│ Modified (3 files):                                  │
│                                                       │
│ ● projectData                                        │
│   ~ Binary file changed (125 KB → 127 KB)           │
│                                                       │
│ ● Alternatives/000/DisplayState.plist                │
│   + 15 lines added                                   │
│   - 3 lines removed                                  │
│                                                       │
│ ● Resources/vocals.wav                               │
│   ~ New file (3.2 MB)                                │
│                                                       │
│ Total: +3.2 MB                                       │
│                                                       │
└───────────────────────────────────────────────────────┘

Use --verbose to see detailed metadata diff
```

#### 4.2 Branch Management

```bash
# List branches
auxin branch list
> Branches:
> * draft   (12 commits ahead of main)
>   main    (latest: abc123f "Vocal tracking complete")

# Create milestone from draft
auxin branch merge-to-main -m "Week 1 progress"
> ✓ Merged draft branch to main
> ✓ Created commit: def456a
> ✓ Draft branch reset

# View draft commits
auxin branch show-drafts
> Draft Commits (12):
> abc123f  3 min ago   Auto-save
> def456a  5 min ago   Auto-save
> ghi789b  10 min ago  Auto-save
> ...
```

#### 4.3 Export/Import

```bash
# Export project at specific commit
auxin export <commit_id> --output ~/Desktop/MyTrack_v2.logicx

# Create archive (for sharing)
auxin archive --commit <commit_id> --output track_archive.tar.gz
> ✓ Archived commit abc123f to track_archive.tar.gz
> Size: 450 MB (compressed from 1.2 GB)
```

#### 4.4 Statistics

```bash
auxin stats

Output:
┌─ Repository Statistics ──────────────────────────────┐
│                                                       │
│ Total commits: 47                                    │
│ Milestone commits: 12                                │
│ Draft commits: 35                                    │
│                                                       │
│ Repository size: 1.8 GB                              │
│ Deduplicated size: 2.1 GB (savings: 86%)             │
│                                                       │
│ Most common tags:                                    │
│   1. mixing (8 commits)                              │
│   2. tracking (6 commits)                            │
│   3. arrangement (4 commits)                         │
│                                                       │
│ Tempo changes:                                       │
│   120 BPM → 128 BPM (commit def456a)                │
│                                                       │
│ First commit: 2025-10-01                             │
│ Last commit: 3 minutes ago                           │
│                                                       │
└───────────────────────────────────────────────────────┘
```

---

## Implementation Roadmap

### Week 1: Visual Feedback & Core Enhancements

**Days 1-2: Progress Indicators**
- [ ] Add `indicatif`, `console`, `dialoguer` dependencies
- [ ] Implement progress bars for `init`, `commit`, `restore`
- [ ] Add spinners for network operations
- [ ] Enhance `status` output with boxes and colors
- [ ] Test with real Logic Pro project (1-10 GB)

**Days 3-4: Enhanced Log & Diff**
- [ ] Redesign `log` output with visual timeline
- [ ] Add `diff` command showing file-level changes
- [ ] Implement commit filtering (`--since`, `--tag`, `--bpm`)
- [ ] Add `show <commit_id>` command for detailed commit view

**Day 5: Lock Management**
- [ ] Implement `lock acquire/release/status` commands
- [ ] Add lock timeout configuration
- [ ] Test lock acquisition/release flow
- [ ] Add lock conflict detection

### Week 2: Interactive Console

**Days 1-2: TUI Framework Setup**
- [ ] Add `ratatui`, `crossterm` dependencies
- [ ] Create console module structure
- [ ] Implement basic rendering (header, footer, activity log)
- [ ] Add keyboard event handling

**Days 3-4: Daemon Integration**
- [ ] Create daemon client module (Unix socket or XPC bridge)
- [ ] Implement `daemon status/start/stop` commands
- [ ] Add real-time event streaming from daemon
- [ ] Test daemon lifecycle management

**Day 5: Interactive Features**
- [ ] Add interactive commit dialog in console
- [ ] Add interactive restore browser
- [ ] Implement real-time activity log updates
- [ ] Polish UI and test usability

### Week 3: Advanced Features & Testing ✅ COMPLETE

**Days 1-2: Semantic Diff & Search**
- [x] Implement `compare` command with metadata diff visualization
- [x] Add multiple output formats (colored, plain, JSON, compact)
- [x] Implement `search` command with natural language queries
- [x] Add BPM range filtering, key signature matching, tag logic
- [x] Implement relevance scoring and ranking

**Days 3-4: Hooks & Interactive Console**
- [x] Implement `hooks` command (init, install, list, delete, run)
- [x] Add 4 built-in templates (validate-metadata, check-file-sizes, notify, backup)
- [x] Complete TUI implementation with 7 modes
- [x] Add Compare, Search, Hooks modes to interactive console
- [x] Implement keyboard navigation and state management

**Day 5: Testing & Documentation**
- [x] Add 29 comprehensive unit tests for TUI integration (274 total unit tests)
- [x] Update README.md with Week 3 features
- [x] Update FOR_DEVELOPERS.md with test counts and architecture
- [x] Add 4 new scenarios to CLI_EXAMPLES.md
- [x] Update CHANGELOG.md with Week 3 completion

**Bug Fixes (November 15):**
- [x] Fixed "HeadNotFound" error in `init --logic` command
- [x] Added automatic initial commit before draft branch creation
- [x] Updated success messages for better UX

---

## Success Criteria

### Functional Requirements
- [ ] All core operations have visual feedback
- [ ] Console mode provides real-time monitoring
- [ ] Daemon can be controlled entirely from CLI
- [ ] Lock management works for team workflows
- [ ] Progress indicators for all long operations
- [ ] Interactive commit and restore workflows

### User Experience
- [ ] No operation feels "frozen" or unresponsive
- [ ] Clear indication of what's happening at all times
- [ ] Errors are actionable with suggested fixes
- [ ] Commands feel fast and snappy (<200ms perceived latency)
- [ ] Console mode is intuitive and discoverable

### Testing
- [ ] All new commands have unit tests
- [ ] Integration tests with real Oxen CLI
- [ ] Daemon lifecycle tested (start/stop/restart)
- [ ] 8+ hour console session without crashes
- [ ] Lock acquisition tested with 2+ users

---

## Documentation Updates

### Update FOR_MUSICIANS.md
**New section: "Using Auxin from the Command Line"**

```markdown
### Daily Workflow (Terminal)

**Morning - Check Status**
```bash
cd ~/Music/MyProject.logicx
auxin status
```

**During Work - Watch Changes**
```bash
# Open in one terminal window
auxin console
# See real-time updates as you edit in Logic Pro
```

**End of Session - Create Milestone**
```bash
auxin add --all
auxin commit -m "Finished vocal tracking" --bpm 128 --tags "vocals,tracking"
```

**Need to Roll Back?**
```bash
# Browse history interactively
auxin console
# Press 'r' to restore to previous version
```
```

### Create CLI_QUICK_START.md

```markdown
# Auxin CLI Quick Start

## 5-Minute Setup

1. **Install Auxin**
   ```bash
   git clone https://github.com/jbacus/auxin.git
   cd auxin
   ./install.sh
   ```

2. **Initialize Your First Project**
   ```bash
   cd ~/Music/YourProject.logicx
   auxin init --logic .
   ```

3. **Start the Console**
   ```bash
   auxin console
   ```

   You'll see real-time updates as you work in Logic Pro!

4. **Create Your First Milestone**
   - Work in Logic Pro, hit Save
   - Wait 30 seconds for auto-save draft
   - Press 'c' in console to create milestone
   - Fill in commit message and metadata
   - Done!

## Essential Commands

```bash
auxin console          # Interactive monitoring & control
auxin status           # Quick status check
auxin log --limit 10   # Recent history
auxin watch            # Auto-refreshing status
```
```

---

## Dependencies to Add

```toml
[dependencies]
# Existing
clap = { version = "4", features = ["derive"] }
colored = "2"
anyhow = "1"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = "0.4"

# New for Phase 1-4
indicatif = "0.17"          # Progress bars and spinners
console = "0.15"            # Terminal utilities
dialoguer = "0.11"          # Interactive prompts
ratatui = "0.25"            # TUI framework
crossterm = "0.27"          # Terminal control
tui-logger = "0.10"         # Logging in TUI
unicode-width = "0.1"       # Text layout
textwrap = "0.16"           # Text wrapping
humantime = "2"             # Human-readable durations
```

---

## Marketing Position for v0.1

**Tagline:** "Logic Pro version control that doesn't get in your way"

**Positioning:**
- For power users who live in Terminal
- No GUI complexity - just works
- Real-time monitoring in your workflow
- Perfect for remote collaboration (SSH-friendly)

**Launch Plan:**
1. Blog post: "Why we shipped CLI-first"
2. Video demo: Console mode in action
3. Reddit post: /r/LogicPro, /r/audioengineering
4. Gather feedback for v0.2 (GUI)

---

## Next Action

**Immediate:** Start Week 1, Day 1 - Add progress indicators to existing commands.

Would you like me to begin implementation?
