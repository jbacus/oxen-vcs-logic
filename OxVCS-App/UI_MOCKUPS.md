# OxVCS UI/UX Design Mockups

## Design Philosophy

**Target Users:** Music producers, audio engineers, Logic Pro users
**Context:** Professional DAW workflow - fast, efficient, non-intrusive
**Key Principles:**
- **Minimal Clicks**: Common actions should be 1-2 clicks maximum
- **Visual Hierarchy**: Most important info at a glance
- **Audio-First**: Metadata (BPM, key, sample rate) as first-class citizens
- **Pro Tools**: Dark theme, high information density, keyboard shortcuts

---

## 1. Main Window - Project Dashboard

### Layout Structure
```
┌─────────────────────────────────────────────────────────────────────────┐
│ OxVCS - Logic Pro Version Control                              ⚙ ℹ︎ ×   │
├─────────────────────────────────────────────────────────────────────────┤
│ ┌─ PROJECT BROWSER ──────┐ ┌─ PROJECT DETAIL ──────────────────────┐  │
│ │                         │ │                                        │  │
│ │  🎵 MyTrack.logicx   ● │ │  MyTrack.logicx                        │  │
│ │     47 commits         │ │  Last commit: 2 hours ago              │  │
│ │     Updated 5m ago     │ │  Status: Clean · No changes            │  │
│ │     🔒 You             │ │                                        │  │
│ │                        │ │  Quick Stats                           │  │
│ │  🎸 Demo.logicx        │ │  ┌────────┬────────┬───────┬─────────┐ │  │
│ │     103 commits        │ │  │  BPM   │  Rate  │  Key  │  Tags   │ │  │
│ │     Updated 1h ago     │ │  │  128   │ 48000  │ A min │ mixing  │ │  │
│ │                        │ │  └────────┴────────┴───────┴─────────┘ │  │
│ │  🎹 Podcast.logicx     │ │                                        │  │
│ │     28 commits         │ │  Recent Commits                        │  │
│ │     Updated 3d ago     │ │  ╔═════════════════════════════════════╗ │  │
│ │                        │ │  ║ a1b2c3 "Final mix tweaks" 2h ago  ║ │  │
│ │                        │ │  ║ BPM: 128 · 48kHz · A minor         ║ │  │
│ │  ┌───────────────────┐ │ │  ╟─────────────────────────────────────╢ │  │
│ │  │   + Add Project   │ │ │  ║ c4d5e6 "Added reverb" 5h ago      ║ │  │
│ │  └───────────────────┘ │ │  ║ BPM: 128 · 48kHz · A minor         ║ │  │
│ │                        │ │  ╟─────────────────────────────────────╢ │  │
│ │  [Refresh ⟲]          │ │  ║ e7f8g9 "Vocal edits" 1d ago       ║ │  │
│ │                        │ │  ║ BPM: 128 · 48kHz · A minor         ║ │  │
│ │                        │ │  ╚═════════════════════════════════════╝ │  │
│ │                        │ │                                        │  │
│ └────────────────────────┘ │  ┌──────────────────────────────────┐  │  │
│                            │  │  [💾 Commit] [⟲ Rollback] [🔒 Lock] │  │
│                            │  └──────────────────────────────────┘  │  │
│                            └────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────────────┘
```

### Key Features
- **Left Sidebar**: All monitored projects
  - Visual indicators: 🎵 active, 🔒 locked by you/others
  - Real-time status updates every 30s
  - One-click project switching

- **Right Panel**: Selected project details
  - At-a-glance metadata display
  - Recent commit feed with audio metadata
  - Large action buttons for common tasks

- **Color Coding**:
  - Green dot (●) = changes to commit
  - Orange 🔒 = locked
  - Gray = no changes

---

## 2. Commit Dialog - The Heart of the Workflow

### Full Dialog Mockup
```
┌────────────────────────────────────────────────────────────────────┐
│ Create Milestone Commit - MyTrack.logicx                      × │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Commit Message *                                                 │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ Final mix with improved vocal balance                        │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Audio Metadata                                                   │
│  ┌──────────────┬──────────────┬──────────────┬──────────────┐   │
│  │ BPM          │ Sample Rate  │ Key          │ Time Sig     │   │
│  │ ┌──────────┐ │ ┌──────────┐ │ ┌──────────┐ │ ┌──────────┐ │   │
│  │ │ 128   ▼ │ │ │ 48000 ▼ │ │ │ A minor▼ │ │ │ 4/4   ▼ │ │   │
│  │ └──────────┘ │ └──────────┘ │ └──────────┘ │ └──────────┘ │   │
│  └──────────────┴──────────────┴──────────────┴──────────────┘   │
│                                                                    │
│  Tags (comma-separated)                                           │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ mixing, final, vocals, reverb                                │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Changes to Commit                                                │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ ✓ projectData                                      2.1 MB    │ │
│  │ ✓ Alternatives/000/DisplayState.plist               45 KB    │ │
│  │ ✓ Resources/Vocals_Final.wav                      89.3 MB    │ │
│  │ ✓ Resources/Reverb_Send.wav                       12.7 MB    │ │
│  │                                                 Total: 104 MB │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ☑ Clean up temporary files (Bounces, Freeze Files)              │
│  ☐ Push to remote after commit                                   │
│                                                                    │
│              ┌──────────────┐  ┌──────────────┐                   │
│              │    Cancel    │  │  💾 Commit   │                   │
│              └──────────────┘  └──────────────┘                   │
│                                  ⌘↩                                │
└────────────────────────────────────────────────────────────────────┘
```

### Smart Features
1. **Auto-populated from last commit**
   - BPM, sample rate, key pre-filled
   - Suggests similar tags based on history

2. **Real-time validation**
   - Message required (shows warning if empty)
   - Validates BPM range (40-300)
   - Sample rate dropdown (44100, 48000, 96000)

3. **File list with sizes**
   - Visual feedback on what's being committed
   - Total size calculation
   - Deselect individual files if needed

4. **Keyboard shortcuts**
   - ⌘↩ to commit
   - ⌘W to cancel
   - Tab through fields efficiently

---

## 3. History Viewer - Beautiful Commit Log

### Timeline View
```
┌────────────────────────────────────────────────────────────────────┐
│ Commit History - MyTrack.logicx                          [✕ Close] │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Filter: [All ▼] [Tags: mixing ×] [Date: Last 30 days ▼] [Search]│
│                                                                    │
│  ╔══════════════════════════════════════════════════════════════╗ │
│  ║ ● TODAY                                                      ║ │
│  ╟──────────────────────────────────────────────────────────────╢ │
│  ║  a1b2c3  Final mix with improved vocal balance    2h ago    ║ │
│  ║          BPM: 128 · 48kHz · A minor · Tags: mixing, final   ║ │
│  ║          john@studio · 104 MB · 4 files changed             ║ │
│  ║          [View Changes] [Restore] [🔖 Tag]                  ║ │
│  ╟──────────────────────────────────────────────────────────────╢ │
│  ║  c4d5e6  Added reverb to vocals                    5h ago    ║ │
│  ║          BPM: 128 · 48kHz · A minor · Tags: mixing          ║ │
│  ║          john@studio · 12 MB · 2 files changed              ║ │
│  ║          [View Changes] [Restore] [🔖 Tag]                  ║ │
│  ╟──────────────────────────────────────────────────────────────╢ │
│  ║ ● YESTERDAY                                                  ║ │
│  ╟──────────────────────────────────────────────────────────────╢ │
│  ║  e7f8g9  Vocal editing pass                        1d ago    ║ │
│  ║          BPM: 128 · 48kHz · A minor · Tags: editing         ║ │
│  ║          john@studio · 89 MB · 3 files changed              ║ │
│  ║          [View Changes] [Restore] [🔖 Tag]                  ║ │
│  ╟──────────────────────────────────────────────────────────────╢ │
│  ║  h0i1j2  Added bass line                           1d ago    ║ │
│  ║          BPM: 128 · 48kHz · A minor · Tags: bass, tracking  ║ │
│  ║          john@studio · 156 MB · 7 files changed             ║ │
│  ║          [View Changes] [Restore] [🔖 Tag]                  ║ │
│  ╚══════════════════════════════════════════════════════════════╝ │
│                                                                    │
│  Showing 47 commits                         [Load More ↓]         │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Smart Features
1. **Grouped by date** - Easy temporal navigation
2. **Rich metadata display** - All audio info inline
3. **Quick actions** - Restore, tag, view changes without opening new dialogs
4. **Filtering** - By tags, date range, author
5. **Search** - Full text search across commit messages

---

## 4. Rollback Dialog - Safe Time Travel

### Confirmation Dialog
```
┌────────────────────────────────────────────────────────────────────┐
│ ⚠️  Confirm Rollback - MyTrack.logicx                         × │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  You are about to restore your project to:                        │
│                                                                    │
│  ╔══════════════════════════════════════════════════════════════╗ │
│  ║  Commit: e7f8g9                                              ║ │
│  ║  Message: "Vocal editing pass"                               ║ │
│  ║  Date: Yesterday at 3:42 PM                                  ║ │
│  ║  Author: john@studio                                         ║ │
│  ║  Metadata: BPM 128 · 48kHz · A minor                         ║ │
│  ║  Tags: editing, vocals                                       ║ │
│  ╚══════════════════════════════════════════════════════════════╝ │
│                                                                    │
│  This will:                                                        │
│  • Restore all project files to this state                        │
│  • Create a backup of current state (can undo)                    │
│  • Take approximately 30 seconds                                   │
│                                                                    │
│  Current uncommitted changes:                                      │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │ ⚠️  You have 3 uncommitted files:                            │ │
│  │   · projectData (modified 5 minutes ago)                     │ │
│  │   · Resources/Vocals.wav (modified 2 hours ago)              │ │
│  │   · Alternatives/000/DisplayState.plist                      │ │
│  │                                                               │ │
│  │ These changes will be LOST unless you commit them first.     │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  ☑ Close Logic Pro before restoring (recommended)                 │
│  ☐ Create backup tag for current state                            │
│                                                                    │
│     ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│     │   Cancel     │  │ Commit First │  │ ⟲ Restore    │         │
│     └──────────────┘  └──────────────┘  └──────────────┘         │
│        ⌘W                ⌘S                ⌘⇧R                     │
└────────────────────────────────────────────────────────────────────┘
```

### Safety Features
1. **Clear warnings** about uncommitted changes
2. **Three-button choice**:
   - Cancel (safe escape)
   - Commit First (smart workflow)
   - Restore anyway (with confirmation)
3. **Auto-backup** current state before restore
4. **Logic Pro integration** - reminds to close app first

---

## 5. Lock Management - Collaboration Made Easy

### Lock Status Panel
```
┌────────────────────────────────────────────────────────────────────┐
│ Lock Management - MyTrack.logicx                              × │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Current Status:  🔓 Available                                    │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  This project is not locked. You can work on it freely.     │ │
│  │  Acquire a lock to prevent others from making changes.      │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Lock Duration                                                     │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  ◉ 24 hours (default)                                        │ │
│  │  ○ 8 hours (work session)                                    │ │
│  │  ○ 72 hours (weekend project)                                │ │
│  │  ○ Custom: [___] hours                                       │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Lock History                                                      │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  sarah@mixing · Locked 3 days ago · Released 3 days ago      │ │
│  │  john@studio · Locked 5 days ago · Released 4 days ago       │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Team Notifications                                                │
│  ☑ Notify team when acquiring lock                                │
│  ☑ Notify when releasing lock                                     │
│                                                                    │
│                       ┌──────────────┐                             │
│                       │  🔒 Acquire  │                             │
│                       └──────────────┘                             │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### When Locked by Others
```
┌────────────────────────────────────────────────────────────────────┐
│ Lock Management - MyTrack.logicx                              × │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Current Status:  🔒 Locked by sarah@mixing                       │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────────┐ │
│  │  ⚠️  This project is currently locked                        │ │
│  │                                                               │ │
│  │  Locked by: sarah@mixing                                     │ │
│  │  Acquired: 2 hours ago                                       │ │
│  │  Expires:  in 22 hours                                       │ │
│  │                                                               │ │
│  │  You cannot make commits until the lock is released.         │ │
│  └──────────────────────────────────────────────────────────────┘ │
│                                                                    │
│  Actions                                                           │
│  ┌──────────────────┐  ┌──────────────────┐                      │
│  │ 📧 Request       │  │ ⚠️  Force Break   │                      │
│  │    Release       │  │    (Admin Only)   │                      │
│  └──────────────────┘  └──────────────────┘                      │
│                                                                    │
│  💡 Tip: You can still view history and create branches          │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Smart Features
1. **Visual status** - Green/Orange/Red indicators
2. **Preset durations** - Common workflows as radio buttons
3. **Lock history** - See who locked when
4. **Team notifications** - Optional Slack/email integration
5. **Force break** - Admin-only with confirmation

---

## 6. Status/Changes View - What's Modified

### Inline Status Panel
```
┌────────────────────────────────────────────────────────────────────┐
│ Changes - MyTrack.logicx                                      [×] │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  ┌─ STAGED (4 files) ─────────────────────────────────────┐      │
│  │  ✓ projectData                            2.1 MB  [Unstage]│  │
│  │  ✓ Alternatives/000/DisplayState.plist     45 KB  [Unstage]│  │
│  │  ✓ Resources/Vocals_Final.wav            89.3 MB  [Unstage]│  │
│  │  ✓ Resources/Reverb_Send.wav             12.7 MB  [Unstage]│  │
│  │                                     Total: 104.2 MB         │  │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                    │
│  ┌─ MODIFIED (2 files) ───────────────────────────────────┐      │
│  │  M Alternatives/001/ProjectData           1.8 MB  [Stage]  │  │
│  │  M Resources/Bass.wav                    45.2 MB  [Stage]  │  │
│  │                                      [Stage All →]          │  │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                    │
│  ┌─ UNTRACKED (1 file) ───────────────────────────────────┐      │
│  │  ? Resources/Guitar_Take2.wav            67.8 MB  [Stage]  │  │
│  │                                      [Stage All →]          │  │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                    │
│  ┌─ IGNORED ──────────────────────────────────────────────┐      │
│  │  5 files (Bounces, Freeze Files) - 234 MB             │  │
│  │                                      [Show Details ↓]       │  │
│  └──────────────────────────────────────────────────────────┘      │
│                                                                    │
│                       ┌──────────────┐                             │
│                       │ 💾 Commit...  │                             │
│                       └──────────────┘                             │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Smart Features
1. **Three sections** - Staged, Modified, Untracked
2. **Individual control** - Stage/unstage per file
3. **File sizes** - See what's taking space
4. **Ignored files** - Collapsible section for transparency
5. **Quick commit** - Goes straight to commit dialog

---

## 7. Project Initialization - First-Time Setup

### Welcome Wizard
```
┌────────────────────────────────────────────────────────────────────┐
│ Initialize Logic Pro Project                                  [×] │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│  Welcome to OxVCS Version Control!                                 │
│                                                                    │
│  Let's set up version control for your Logic Pro project.         │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  Step 1: Select your Logic Pro project                     │   │
│  │                                                              │   │
│  │  Project Path:                                               │   │
│  │  ┌────────────────────────────────────────────────────────┐ │   │
│  │  │ /Users/john/Music/Logic/MyTrack.logicx            [📁] │ │   │
│  │  └────────────────────────────────────────────────────────┘ │   │
│  │                                                              │   │
│  │  ✓ Valid Logic Pro project detected                         │   │
│  │  ✓ 234 MB of audio files found                              │   │
│  │  ✓ projectData and Alternatives present                     │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  Step 2: Configure initial settings                         │   │
│  │                                                              │   │
│  │  Project BPM: [128 ▼]    Sample Rate: [48000 ▼]            │   │
│  │  Key: [A minor ▼]         Time Sig: [4/4 ▼]                 │   │
│  │                                                              │   │
│  │  Initial commit message:                                     │   │
│  │  ┌────────────────────────────────────────────────────────┐ │   │
│  │  │ Initial project setup                                  │ │   │
│  │  └────────────────────────────────────────────────────────┘ │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                    │
│  ┌────────────────────────────────────────────────────────────┐   │
│  │  Step 3: What to track                                      │   │
│  │                                                              │   │
│  │  ✓ Track projectData (required)                             │   │
│  │  ✓ Track Alternatives (recommended)                         │   │
│  │  ✓ Track Resources (audio files)                            │   │
│  │  ☐ Track Bounces (usually not needed)                       │   │
│  │  ☐ Track Freeze Files (usually not needed)                  │   │
│  │                                                              │   │
│  │  💡 .oxenignore file will be created automatically          │   │
│  └────────────────────────────────────────────────────────────┘   │
│                                                                    │
│     ┌──────────────┐              ┌──────────────┐                │
│     │    Cancel    │              │ ✨ Initialize │                │
│     └──────────────┘              └──────────────┘                │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Smart Features
1. **Validation** - Live feedback on project validity
2. **Pre-configured** - Smart defaults for Logic Pro
3. **Educational** - Explains what each option does
4. **.oxenignore** - Automatically created with best practices

---

## Design System Summary

### Color Palette (Dark Theme)
- **Background**: `#1e1e1e` (VSCode dark)
- **Panel**: `#252526`
- **Border**: `#3e3e42`
- **Text Primary**: `#cccccc`
- **Text Secondary**: `#858585`
- **Accent (Action)**: `#007acc` (blue)
- **Success**: `#4ec9b0` (teal/green)
- **Warning**: `#ff9800` (orange)
- **Error**: `#f44747` (red)
- **Lock**: `#ff9800` (orange)

### Typography
- **Headers**: SF Pro Display, 18-20pt, Semibold
- **Body**: SF Pro Text, 13pt, Regular
- **Monospace**: SF Mono, 12pt (for commit hashes, file paths)
- **Metadata**: SF Pro Text, 11pt, Medium (BPM, key, etc.)

### Spacing
- **Margin**: 16px
- **Padding**: 12px (small), 16px (medium), 24px (large)
- **Gap**: 8px (tight), 12px (standard), 16px (loose)

### Interactive Elements
- **Buttons**:
  - Primary: Blue background, white text, 32px height
  - Secondary: Gray background, 28px height
  - Danger: Red background, requires confirmation
- **Inputs**:
  - Height: 24px (compact), 32px (standard)
  - Border radius: 4px
  - Focus: Blue outline

### Icons
- Use SF Symbols for macOS
- Size: 16px (inline), 24px (standalone), 32px (feature)
- Style: Regular weight, 2px stroke

---

## Navigation & Shortcuts

### Global Shortcuts
- `⌘N` - New project / Initialize
- `⌘R` - Refresh
- `⌘,` - Settings
- `⌘?` - Help

### Workflow Shortcuts
- `⌘S` / `⌘K` - Quick Commit
- `⌘L` - View Log/History
- `⌘⇧R` - Rollback
- `⌘⇧L` - Lock Management
- `⌘⇧S` - Status/Changes

### Within Dialogs
- `⌘↩` - Confirm (Commit, Restore, etc.)
- `⌘W` / `Esc` - Cancel/Close
- `Tab` / `⇧Tab` - Navigate fields
- `Space` - Toggle checkboxes

---

## Next Steps for Implementation

See `UI_IMPLEMENTATION_PLAN.md` for detailed development roadmap.
