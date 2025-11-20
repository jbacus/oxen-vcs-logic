# Auxin: Version Control for Music Producers

**Think of it like "Track Changes" for Logic Pro**

Last Updated: November 2025

---

## What is This?

**Auxin** is like having an unlimited "undo" button for your entire Logic Pro project. It automatically saves snapshots of your work so you can go back to any previous version - even from weeks ago.

### The Problem It Solves

Have you ever:
- Spent hours on a mix, then wished you could hear yesterday's version?
- Accidentally deleted tracks and couldn't recover them?
- Wanted to try a radical arrangement change but were afraid to lose your current work?
- Had Logic crash and lost recent changes?
- Worked with bandmates and wondered "who changed the drums?"

**Auxin solves all of these problems.**

---

## How It Works (The Simple Version)

Think of it like this:

**Traditional Saving:**
- You hit Save (⌘S) in Logic Pro
- Your project is saved over the old version
- The old version is **gone forever**

**With Auxin:**
- You hit Save (⌘S) in Logic Pro
- Auxin automatically creates a "snapshot" of your entire project
- **Every previous version is still available**
- You can jump back to any snapshot, anytime

It's like having a time machine for your music.

---

## Why Do You Need This?

### For Solo Producers

**Experiment Fearlessly**
- Try that crazy effect chain - you can always undo it
- Completely rearrange your song - if you don't like it, go back
- Test different mix approaches side-by-side

**Never Lose Work**
- Logic crashed? Your last save is automatically backed up
- Accidentally deleted important tracks? Restore from 10 minutes ago
- Changed your mind after "finalizing"? Roll back instantly

**Track Your Progress**
- See how your song evolved over time
- Compare your first rough mix to your final master
- Learn from your mixing decisions

### For Teams & Collaborators

**Clear Ownership**
- Only one person can edit the project at a time (no conflicts!)
- See who made what changes and when
- Coordinate editing sessions without stepping on each other's toes

**Safe Handoffs**
- Producer finishes tracking → commits changes → hands off to mixer
- Mixer works on their version → sends back to producer
- No confusion about "which version is the latest?"

**Client Revisions Made Easy**
- "Can I hear mix v2 again?" → Restore that snapshot instantly
- Track all client feedback as separate versions
- Never lose a version the client might want back

---

## Real-World Workflow Example

Let's say you're producing a song. Here's how Auxin works in practice:

### Morning Session
```
9:00 AM  - Open Logic Pro, start working on drums
9:15 AM  - Save your work (⌘S)
9:16 AM  - Auxin automatically snapshots: "draft_09-15-2025_09-16"
9:45 AM  - Try adding a new snare layer
9:46 AM  - Save
9:47 AM  - Auxin snapshots: "draft_09-15-2025_09-47"
10:00 AM - Hate the snare. Click "Restore" → pick 9:16 version → snare is gone!
```

### Creating Milestones
```
2:00 PM  - Drums sound perfect!
2:01 PM  - Create "Milestone Commit" in Auxin app:
           Message: "Drum tracking complete"
           BPM: 128
           Tags: tracking, drums
2:02 PM  - This version is now permanently marked as important
```

### Collaboration
```
Next Day - Send project to vocalist
        - They acquire "lock" in Auxin (so you can't edit at same time)
        - They record vocals, create milestone: "Vocal tracking done"
        - They release lock
        - You pull their changes → vocals are now in your project
```

---

## What Gets Saved?

### ✅ Tracked (Saved in Every Snapshot)

- **Your Logic Pro project file** (the .logicx folder)
- **Audio recordings** you made in Logic
- **MIDI data** and arrangements
- **Plugin settings** and automation
- **All tracks, busses, and routing**

### ❌ Not Tracked (Excluded to Save Space)

- **Bounces** - You can always re-export these
- **Freeze files** - These are temporary anyway
- **System temp files** - MacOS junk like .DS_Store

This means your version history stays manageable even for huge projects.

---

## Daily Workflow

### Typical Day in the Studio

**1. Morning - Start Working**
```
Open Logic Pro
Make changes, hit Save (⌘S)
Work normally - Auxin watches in the background
```

**2. During Work - Automatic Safety Net**
```
Every 30-60 seconds after you stop editing:
→ Auxin creates a "draft snapshot"
→ You don't need to do anything
→ Keep working!
```

**3. Important Moments - Create Milestones**
```
Just finished tracking guitar?
→ Open Auxin app
→ Click "Create Milestone"
→ Add note: "Lead guitar tracking complete"
→ Add BPM, key signature, any tags
→ Done! This version is marked as important
```

**4. When You Mess Up - Easy Recovery**
```
"Oh no, I deleted the wrong track!"
→ Open Auxin app
→ Browse recent snapshots
→ Click the one from 10 minutes ago
→ Click "Restore"
→ 10 seconds later, your project is back!
```

---

## Understanding "Drafts" vs "Milestones"

### Draft Snapshots (Automatic)

**What:** Automatic saves that happen in the background
**When:** Every 30-60 seconds after you stop editing
**Why:** Safety net for crashes and mistakes
**How Long:** Keeps last ~100 drafts, then prunes old ones

**Think of them like:** Auto-save in a word processor

### Milestone Commits (Manual)

**What:** Important versions YOU mark as significant
**When:** You decide! After tracking, before major changes, at end of session
**Why:** Mark important progress points
**How Long:** Forever! These never get deleted

**Think of them like:** Saving a document with a descriptive filename

---

## Installation & Setup

### What You Need
- Mac computer running macOS 14.0 or newer
- Logic Pro 11.x (the version with folder-based projects)
- About 10 minutes to install

### Installation Steps

**Option 1: Easy Automatic Install**
```bash
# Open Terminal (Applications → Utilities → Terminal)
# Copy and paste these commands:

git clone https://github.com/jbacus/auxin.git
cd auxin
./install.sh
```

The installer does everything for you!

**Option 2: Step-by-Step Instructions**

See the [Installation Guide](../INSTALL.md) for detailed manual installation.

### First-Time Setup

**1. Launch the App**
- Open Applications → Auxin.app
- You'll see a menu bar icon (looks like a folder with a clock)

**2. Add Your First Project**
- Click "Add Project..."
- Navigate to your Logic Pro project (the .logicx folder)
- Click "Initialize"
- Wait 10-30 seconds while it sets up

**3. Start Working!**
- Open your project in Logic Pro
- Work normally - hit Save (⌘S) when you want
- Auxin handles everything else automatically

---

## Using Auxin from Terminal (Alternative to GUI)

**Note:** The GUI app (above) and command line tool do the exact same thing. Pick whichever you prefer!

### Why Use Terminal?

**You might prefer the command line if you:**
- Like keyboard shortcuts more than clicking
- Work on remote servers (SSH access)
- Want to script repetitive tasks
- Are comfortable with Terminal already

**Quick Start Example:**

```bash
# Initialize your project (one time only)
cd ~/Music/MyProject.logicx
auxin init --logic .

# Daily workflow after working in Logic Pro
auxin status              # See what changed
auxin add --all           # Stage your changes
auxin commit -m "Finished vocals" --bpm 120 --tags "vocals"

# View history and restore
auxin log --limit 10      # See recent versions
auxin restore abc123f     # Go back to a version
```

**What You'll See:**

The CLI has beautiful visual output with colors and progress indicators:

```
┌─ Repository Status ─────────────────────────────────────┐
│                                                          │
│  Changes: 2 staged, 3 modified, 1 untracked             │
│                                                          │
└──────────────────────────────────────────────────────────┘

● Staged files (2):
  + projectData
  + Resources/vocals.wav

ℹ Next step: auxin commit -m "Your message"
```

**Full CLI Guide:** See [CLI Reference](cli-reference.md) for detailed examples, real production scenarios, and team workflows.

**GUI vs CLI - Which Should I Use?**

| Feature | GUI App | Terminal (CLI) |
|---------|---------|----------------|
| Ease of Use | ✅ Point and click | 🟡 Type commands |
| Speed | 🟡 Click through menus | ✅ Instant (keyboard) |
| Remote Access | ❌ Not possible | ✅ Works over SSH |
| Visual Feedback | ✅ Windows and dialogs | ✅ Beautiful text output |
| Automation | ❌ Manual only | ✅ Can script |
| Learning Curve | ✅ Easy (5 minutes) | 🟡 Medium (10 minutes) |

**Recommendation:**
- **New to version control?** Start with the GUI app
- **Comfortable with Terminal?** CLI is faster for daily use
- **Working remotely?** CLI is your only option
- **Not sure?** Try the GUI first, switch to CLI later if you want

Both methods work equally well and use the same underlying system!

---

## Common Questions

### "Will this slow down Logic Pro?"

**No!** Auxin runs in the background and doesn't affect Logic's performance at all. It waits until you **stop** editing before creating snapshots.

### "How much disk space does this use?"

**Way less than you'd think!** Auxin only stores the parts of files that changed, not entire copies.

**Example:**
- 5 GB Logic project
- 50 snapshots over 3 months
- Total storage: ~6.5 GB (only 1.5 GB extra!)

Compare this to manually duplicating your project 50 times (250 GB!).

### "What if Logic Pro crashes?"

**You're protected!** Your last snapshot (usually 30-60 seconds old) is safe. When you reopen Logic:
1. Logic may offer its own crash recovery
2. Check Auxin for the latest snapshot
3. If needed, restore from there
4. You'll lose at most ~1 minute of work

### "Can I use this with iCloud Drive / Dropbox?"

**Not recommended.** Cloud services and version control don't play well together.

**Better approach:**
- Store projects **locally** on your Mac
- Use Auxin for version history
- Use Time Machine for backups
- Optionally push to Oxen Hub for cloud storage

### "Do I need to know programming or 'Git'?"

**Absolutely not!** Auxin was designed for musicians, not programmers. The app has a simple point-and-click interface. The only time you might use Terminal is for installation.

### "What's the difference between this and Splice?"

**Splice:**
- Cloud-based (requires internet)
- Subscription service ($$$)
- Supports multiple DAWs
- Automatic cloud backup

**Auxin:**
- Works offline (local-first)
- Free and open-source
- Logic Pro only (for now)
- You control your data
- More efficient storage

**You can use both!** Splice for cloud backup, Auxin for local version control.

---

## Working with a Team

### The "Lock" System

**The Rule:** Only one person can edit at a time.

**Why?** Logic Pro projects can't be automatically merged. If two people edit simultaneously, someone's work gets lost.

**How It Works:**

1. **Acquire Lock**
   - Open Auxin app → "Acquire Lock"
   - You now have exclusive editing rights
   - Others see "Locked by [your name]"

2. **Edit Project**
   - Make your changes in Logic Pro
   - Save normally
   - Take your time (but be considerate!)

3. **Create Milestone & Release**
   - When done, create milestone commit
   - Click "Release Lock"
   - Now others can acquire it

**Lock Timeout:** Automatically expires after 4 hours (configurable)

### Coordinating with Bandmates

**Best Practices:**
```
Morning Standup:
"I need the lock for 2 hours to record bass"
"Cool, I'll work on my own branch for drums"
"I'll need it after lunch for mixing"
```

**Use Slack/Discord:**
- Announce when you're grabbing the lock
- Estimate how long you need
- Let team know when you're done
- Coordinate before long sessions

---

## Tips for Success

### 1. Commit Often (Milestones)
**Do this:**
- After tracking session: "Guitars recorded - 12 takes"
- Before risky changes: "Pre-arrangement experiment"
- End of day: "Mix v3 - increased bass"
- Client deliverables: "Master v1.0 - delivered to client"

**Not this:**
- Only when project is "done"
- Every 5 minutes (drafts handle that)

### 2. Write Descriptive Messages
**Good:**
- "Vocal tracking complete - 8 takes recorded"
- "Mix v2: reduced reverb on vox, boosted bass"
- "Rearranged chorus per client feedback"

**Bad:**
- "Update"
- "Changes"
- "asdf"

### 3. Use Tags and Metadata
```
Message: Final mix - ready for mastering
BPM: 128
Key: A Minor
Tags: mix, final, v3, client-approved
```

Later, you can search:
- "Show me all mixes at 128 BPM"
- "Find versions tagged 'client-approved'"
- "Show all changes in A Minor"

### 4. Create Snapshots Before Experiments
```
Current state: Mix sounds good
↓
Create milestone: "Mix v2 - before trying parallel compression"
↓
Experiment with compression
↓
Don't like it? Restore to "Mix v2"
Like it? Create "Mix v3 - parallel compression works!"
```

---

## Troubleshooting

### "Automatic snapshots stopped working"

**Fix:**
1. Open Activity Monitor (Applications → Utilities)
2. Search for "Auxin"
3. If not running, open Auxin.app to restart it

### "I can't initialize my Logic Pro project"

**Common causes:**
- Project is in "package" format (not folder)
- Project hasn't been saved yet in Logic

**Fix:**
1. Open project in Logic Pro
2. File → Save As → Change Format to "Folder"
3. Save
4. Try initializing again in Auxin

### "Lock is stuck / held by someone who's gone home"

**Fix:**
1. Try contacting them first!
2. If no response, use "Force Break Lock" (admin feature)
3. Warning: Their unsaved work will be lost

For more help, see [Troubleshooting](troubleshooting.md).

---

## Next Steps

### Getting Started
1. ✅ Install Auxin (see Installation above)
2. ✅ Initialize your first project
3. ✅ Work normally - let drafts accumulate
4. ✅ Create your first milestone commit
5. ✅ Practice restoring to a previous version

### Learning More
- **Full User Guide:** [Getting Started](getting-started.md)
- **CLI Reference:** [CLI Reference](cli-reference.md)
- **Troubleshooting:** [Troubleshooting](troubleshooting.md)

### Getting Help
- **Questions:** Open an issue on [GitHub](https://github.com/jbacus/auxin/issues)
- **Discord:** Join our community (coming soon)
- **Email:** support@oxen-vcs.com

---

## Summary: Why You'll Love This

**Peace of Mind**
- Never lose work to crashes, mistakes, or bad decisions
- Always have a safety net
- Sleep better knowing your music is protected

**Creative Freedom**
- Experiment without fear
- Try radical ideas
- Explore different directions
- Compare multiple mix approaches

**Professional Workflow**
- Track project evolution
- Collaborate without conflicts
- Deliver client revisions with confidence
- Maintain clean version history

**It's Free!**
- Open source, no subscription
- Community-driven development
- You own your data

---

**Ready to never lose your work again?**

[Install Auxin](../../INSTALL.md) · [Read Full Guide](getting-started.md) · [Get Help](troubleshooting.md)

---

*Made with ❤️ by musicians, for musicians*
