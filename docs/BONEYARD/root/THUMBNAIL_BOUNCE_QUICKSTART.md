# Thumbnail & Bounce Quickstart Guide

## What's New?

Every Logic Pro milestone commit now automatically includes:
- 📸 **Visual Screenshot** - Extracted from your Logic project
- 🎵 **Audio Bounce** - Optional stereo mix attachment

## Quick Examples

### Create a Milestone with Thumbnail (Automatic!)

```bash
# Just add Logic metadata - thumbnail extracts automatically
auxin commit -m "Verse 2 complete" --bpm 128 --key "A Minor"
```

Output:
```
✓ Commit created: abc123def
✓ Thumbnail extracted
  Thumbnail: jpg
  BPM: 128
  Key: A Minor
```

### Add a Bounce to Your Milestone

```bash
# First, export your mix from Logic Pro:
# File → Bounce → Project or Section...

# Then attach it:
auxin commit -m "Mix v1" --bpm 120 --bounce ~/Desktop/my-mix.wav
```

Output:
```
✓ Commit created: def456abc
✓ Thumbnail extracted
  Thumbnail: jpg
✓ Bounce added
  Bounce: my-mix.wav (3:45.00, 45.2 MB)
  BPM: 120
```

## What You'll See in the UI

### Before This Feature:
```
📋 Commit: "Verse 2 complete"
   2025-11-22 14:30
   🎵 120 BPM | 🎹 A Minor
```

### After This Feature:
```
[Logic Window] 📋 Commit: "Verse 2 complete"    🔊
Screenshot         2025-11-22 14:30             Play
Preview            🎵 120 BPM | 🎹 A Minor      Bounce
```

## File Storage

Everything is stored locally in your project:

```
YourProject.logicx/
└── .auxin/
    ├── thumbnails/
    │   ├── abc123.jpg   ← Your Logic window screenshot
    │   └── abc123.json  ← Metadata
    └── bounces/
        ├── def456.wav   ← Your audio bounce
        └── def456.json  ← Metadata
```

## UI Features

### Commit History View

1. **Thumbnails**: Each commit shows an 80×60px preview of your Logic arrangement
2. **Placeholders**: Commits without thumbnails show a Logic icon
3. **Bounce Playback**: Click the 🔊 icon to hear the bounce
4. **Visual Timeline**: Scroll through your project's evolution visually

### Playing Bounces

- Click the waveform icon: 🎵
- Icon fills while playing: 🎵 (filled)
- Plays in background - keep working
- Auto-stops when finished

## When Thumbnails Are Created

✅ **Logic Pro saves WindowImage.jpg when:**
- You save the project (⌘S)
- Auto-save triggers
- You close the project

❌ **No thumbnail if:**
- Project never saved (unsaved new project)
- Logic is still open and hasn't saved yet
- Working in an older Logic version

**Solution:** Just save your Logic project before committing!

## Best Practices

### For Solo Musicians

```bash
# Work session end
1. Save Logic project (⌘S)
2. Export bounce: File → Bounce → Project
3. Commit with bounce:
   auxin commit -m "Session end" --bpm 120 --bounce latest-mix.wav
```

### For Team Collaboration

```bash
# Before handing off
1. Save and close Logic
2. Export reference mix
3. Create detailed commit:
   auxin commit -m "Ready for vocals - ref mix attached" \
     --bpm 120 --key "G Major" \
     --bounce reference-mix.wav \
     --tags "ready-for-vocals,reference"

4. Push to team:
   auxin push
```

Team member receives:
- Visual of your arrangement
- Audio of how it sounds
- All metadata (BPM, key, etc.)

## Workflow Integration

### Traditional Git Workflow
```
Save → Stage → Commit
```

### Auxin Workflow (New!)
```
Save Logic → Auto-thumbnail → Commit → Optionally attach bounce
     ↓                ↓           ↓              ↓
WindowImage.jpg  Extracted   Milestone    Reference audio
  created        to .auxin    created      for review
```

## Tips & Tricks

### Automatic Workflow
Let Auxin handle everything:
```bash
# This one command:
auxin commit -m "Chorus done" --bpm 128 --bounce mix.wav

# Does all this:
# 1. Creates commit
# 2. Extracts Logic screenshot
# 3. Attaches audio bounce
# 4. Saves all metadata
```

### Browse History Visually
```bash
# Open the UI
open Auxin.app

# Navigate to your project
# Click on commit history
# See thumbnails + play bounces!
```

### Compare Versions
Click through commits to:
- See how arrangement evolved (thumbnails)
- Hear how mix progressed (bounces)
- Review metadata changes (BPM, key, tags)

## Supported Formats

### Thumbnails
- ✅ JPEG (`.jpg`, `.jpeg`)
- ✅ PNG (`.png`)
- ✅ GIF (`.gif`)

### Bounces
- ✅ WAV (`.wav`) - Recommended
- ✅ AIFF (`.aiff`)
- ✅ MP3 (`.mp3`)
- ✅ FLAC (`.flac`)
- ✅ M4A (`.m4a`)

## Troubleshooting

### "Could not extract thumbnail"
**Why:** Logic hasn't saved the project yet
**Fix:** Save your Logic project (⌘S), then commit again

### Thumbnail shows old version
**Why:** Logic's WindowImage.jpg is cached
**Fix:** Save Logic project, wait 1 second, then commit

### Bounce won't play
**Why:** File path incorrect or afplay not available
**Fix:** Verify bounce file exists: `ls .auxin/bounces/`

### No thumbnail for old commits
**Why:** Feature only works for new commits
**Note:** This is expected - old commits show placeholder icons

## What's Under the Hood?

**Rust Backend:**
- `ThumbnailManager` - Extracts and manages screenshots
- `BounceManager` - Handles audio files (already existed)
- Automatic integration in commit workflow

**Swift UI:**
- Enhanced `CommitRowView` with thumbnail display
- Audio playback using macOS `afplay`
- Lazy loading for performance

**Testing:**
- 8 new integration tests
- 499 total tests passing
- 88% code coverage

## Learn More

- Full documentation: `THUMBNAIL_BOUNCE_FEATURE_SUMMARY.md`
- Auxin docs: `docs/user/for-musicians.md`
- Report issues: `https://github.com/jbacus/auxin/issues`

---

**🎉 Happy creating with visual and audio snapshots!**
