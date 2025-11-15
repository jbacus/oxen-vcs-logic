# OxVCS Command Line Examples for Musicians

**For:** Music producers who prefer Terminal or need remote access
**Skill Level:** Beginner-friendly (no programming required!)
**Time to Learn:** 10 minutes

---

## Why Use the Command Line?

**You might prefer the CLI if you:**
- Work on remote servers or over SSH
- Like keyboard shortcuts more than clicking
- Want to script repetitive tasks
- Find Terminal faster than GUI
- Work on headless systems

**Don't worry!** These examples show exactly what to type and what you'll see.

---

## 🚀 Quick Start (5 Minutes)

### 1. Initialize Your Project

**Open Terminal** (Applications → Utilities → Terminal)

```bash
cd ~/Music/MyProject.logicx
oxenvcs-cli init --logic .
```

**What you'll see:**
```
⠹ Validating Logic Pro project structure...
✓ Logic Pro project repository initialized

✓ Repository created at: MyProject.logicx
ℹ Next steps:
  1. cd MyProject.logicx
  2. oxenvcs-cli add --all
  3. oxenvcs-cli commit -m "Initial commit"
```

**What just happened:** OxVCS checked your Logic Pro project and set up version control!

---

### 2. Create Your First Commit

```bash
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial project setup" --bpm 120 --sample-rate 48000
```

**What you'll see:**
```
⠹ Staging all changes...
✓ All changes staged

ℹ Next step: oxenvcs-cli commit -m "Your message"

⠹ Preparing commit...
⠹ Creating commit...
✓ Commit created: a1b2c3d

ℹ Commit Details:
  Message: Initial project setup
  BPM: 120
  Sample Rate: 48000 Hz
```

**What just happened:** You created a permanent snapshot with your project's tempo and sample rate!

---

### 3. Check What Changed

After working in Logic Pro for a while:

```bash
oxenvcs-cli status
```

**What you'll see:**
```
┌─ Repository Status ─────────────────────────────────────┐
│                                                          │
│  Changes: 0 staged, 3 modified, 1 untracked             │
│                                                          │
└──────────────────────────────────────────────────────────┘

◆ Modified files (3):
  M projectData
  M Alternatives/000/DisplayState.plist
  M Alternatives/000/RegionData.plist

? Untracked files (1):
  ? Resources/vocals.wav

ℹ Next step: oxenvcs-cli add --all
```

**What this means:**
- **Modified files:** You changed these in Logic Pro
- **Untracked files:** New files you added (like new audio recordings)
- **Next step:** The tool tells you what to do next!

---

### 4. Save Your Progress

```bash
oxenvcs-cli add --all
oxenvcs-cli commit -m "Recorded lead vocals" --bpm 120 --tags "vocals,tracking"
```

**What you'll see:**
```
⠹ Staging all changes...
✓ All changes staged

⠹ Creating commit...
✓ Commit created: d4e5f6g

ℹ Commit Details:
  Message: Recorded lead vocals
  BPM: 120
  Tags: vocals, tracking
```

**What just happened:** Your vocal recording is now permanently saved in version history!

---

## 📖 Common Workflows

### Morning: Check What You Did Yesterday

```bash
oxenvcs-cli log --limit 5
```

**What you'll see:**
```
┌─ Commit History ────────────────────────────────────────┐
│ Showing last 5 commit(s)                                 │
└──────────────────────────────────────────────────────────┘

● d4e5f6g - now
  │ Recorded lead vocals
  │ BPM: 120 | Tags: vocals, tracking
  │
● a1b2c3d - now
  │ Added drum arrangement
  │ BPM: 120 | Sample Rate: 48000 Hz
  │
● 7h8i9j0 - now
  │ Initial project setup
  │ BPM: 120 | Sample Rate: 48000 Hz

ℹ Showing 5 commit(s)
```

**Useful for:** Quick reminder of your progress

---

### Before Experimenting: Create a Checkpoint

You're about to try something risky (like completely rearranging your song).

```bash
# Save current state
oxenvcs-cli add --all
oxenvcs-cli commit -m "Pre-experiment checkpoint - current mix sounds good" --tags "checkpoint"

# Now experiment in Logic Pro!
```

**If experiment goes wrong:**
```bash
oxenvcs-cli log --tag checkpoint
```

**What you'll see:**
```
┌─ Commit History ────────────────────────────────────────┐
│ Filters: tag = checkpoint                                │
│ Found 1 of 23 commit(s)                                  │
└──────────────────────────────────────────────────────────┘

● k1l2m3n - now
  │ Pre-experiment checkpoint - current mix sounds good
  │ Tags: checkpoint
```

**Restore to that checkpoint:**
```bash
oxenvcs-cli restore k1l2m3n
```

**What you'll see:**
```
⠹ Restoring to commit k1l2m3n...
⠹ Checking out files...
✓ Restored to commit k1l2m3n

⚠ Your working directory has been updated to match this commit
ℹ To create a new commit from here, use:
  oxenvcs-cli add --all
  oxenvcs-cli commit -m "Your message"
```

**What just happened:** Your project is back to how it was before the experiment!

---

### Find That Perfect Mix

You remember making a great mix at 128 BPM but can't remember which version.

```bash
oxenvcs-cli log --bpm 128 --tag mixing
```

**What you'll see:**
```
┌─ Commit History ────────────────────────────────────────┐
│ Filters: BPM = 128, tag = mixing                        │
│ Found 3 of 23 commit(s)                                  │
└──────────────────────────────────────────────────────────┘

● o4p5q6r - now
  │ Final mix - ready for mastering
  │ BPM: 128 | Sample Rate: 48000 Hz | Tags: mixing, final
  │
● s7t8u9v - now
  │ Mix v2 - increased bass
  │ BPM: 128 | Tags: mixing, wip
  │
● w0x1y2z - now
  │ First mix attempt
  │ BPM: 128 | Tags: mixing, draft

ℹ Showing 3 commit(s)
```

**See details of one:**
```bash
oxenvcs-cli show o4p5q6r
```

**What you'll see:**
```
┌─ Commit Details ────────────────────────────────────────┐
│                                                          │
│  Commit: o4p5q6r7s8t9u0v1w2x3y4z5a6b7c8d9e0f1a2b3c4d5    │
│                                                          │
└──────────────────────────────────────────────────────────┘

Message:
  Final mix - ready for mastering

Metadata:
  BPM: 128
  Sample Rate: 48000 Hz
  Tags: mixing, final

ℹ Use 'oxenvcs-cli restore o4p5q6r' to restore to this commit
```

---

### See Exactly What Changed

Before committing, see what's different:

```bash
oxenvcs-cli diff
```

**What you'll see:**
```
┌─ Uncommitted Changes ───────────────────────────────────┐
│                                                          │
└──────────────────────────────────────────────────────────┘

◆ Modified files (2):
  ~ projectData (125648 bytes)
  ~ Alternatives/000/DisplayState.plist (4523 bytes)

◆ Added files (3):
  + Resources/vocals-lead.wav (3.2 MB)
  + Resources/vocals-harmony.wav (2.8 MB)
  + Resources/vocals-double.wav (2.5 MB)

ℹ Total changes: 2 modified, 3 added
```

**What this means:**
- You modified your Logic Pro project file
- You added 3 new vocal recordings (8.5 MB total)

---

## 👥 Working with a Team

### Check if Someone is Editing

```bash
oxenvcs-cli lock status
```

**If unlocked:**
```
┌─ Lock Status ───────────────────────────────────────────┐
│                                                          │
│  Status: ● Unlocked                                      │
│                                                          │
│  The project is available for editing                    │
│                                                          │
└──────────────────────────────────────────────────────────┘

ℹ Acquire lock with: oxenvcs-cli lock acquire
```

**If locked by someone else:**
```
┌─ Lock Status ───────────────────────────────────────────┐
│                                                          │
│  Status: ● Locked                                        │
│  Holder: jane@studio-mac.local                          │
│  Since: 2025-11-15 14:30:00                              │
│  Expires: 2025-11-15 18:30:00 (2h 15m remaining)         │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

**What this means:** Jane is working on the project. Wait for her to finish or contact her!

---

### Your Turn to Edit

When the project is unlocked:

```bash
oxenvcs-cli lock acquire --timeout 4
```

**What you'll see:**
```
⠹ Acquiring project lock...
✓ Lock acquired

┌─ Lock Acquired ─────────────────────────────────────────┐
│                                                          │
│  ✓ You now have exclusive editing rights                │
│                                                          │
│  Lock expires in: 4 hours                                │
│                                                          │
└──────────────────────────────────────────────────────────┘

ℹ You can now safely edit the project in Logic Pro
⚠ Remember to release the lock when done: oxenvcs-cli lock release
```

**What just happened:** You have exclusive access! Others can't edit until you release the lock.

**Now:**
1. Open Logic Pro
2. Make your changes
3. Save normally (⌘S)

---

### When You're Done Editing

```bash
# Save your work
oxenvcs-cli add --all
oxenvcs-cli commit -m "Added bass line" --bpm 120 --tags "bass,tracking"

# Release the lock
oxenvcs-cli lock release
```

**What you'll see:**
```
⠹ Releasing project lock...
✓ Lock released

✓ Lock released successfully
ℹ Other team members can now acquire the lock
```

**What just happened:** Your changes are saved and others can now edit!

---

## 🎯 Real Production Scenarios

### Scenario 1: Client Says "I Liked Yesterday's Mix Better"

**Problem:** You changed the mix today, client wants yesterday's version back.

**Solution:**
```bash
# Find yesterday's mix
oxenvcs-cli log --tag mix --limit 10

# Let's say yesterday was commit a1b2c3d
# View details to confirm
oxenvcs-cli show a1b2c3d

# Yep, that's it! Restore it
oxenvcs-cli restore a1b2c3d

# Now export for client
# (Open Logic Pro → File → Bounce → Project)
```

**Time saved:** 2 minutes vs. hours of trying to recreate it!

---

### Scenario 2: Logic Pro Crashed, Did I Lose Work?

**Problem:** Logic crashed before you could save.

**Solution:**
```bash
# Check if auto-save happened
oxenvcs-cli log --limit 3

# See the most recent commit
# If it's within the last minute, you're saved!
```

**What you'll see:**
```
┌─ Commit History ────────────────────────────────────────┐
│ Showing last 3 commit(s)                                 │
└──────────────────────────────────────────────────────────┘

● x9y8z7w - now
  │ Auto-save draft commit
  │
● a1b2c3d - now
  │ Vocal tracking complete
  │ BPM: 128 | Tags: vocals
```

**What this means:** The auto-save caught your work! Just open Logic Pro and continue.

---

### Scenario 3: Find All Your Mixing Sessions

**Problem:** You want to compare 3 different mix approaches you tried.

**Solution:**
```bash
# Find all mixing commits
oxenvcs-cli log --tag mixing

# Compare specific ones
oxenvcs-cli show m1x2i3n4   # Mix 1
oxenvcs-cli show m5x6i7n8   # Mix 2
oxenvcs-cli show m9x0i1n2   # Mix 3

# Restore each one to listen in Logic Pro
oxenvcs-cli restore m1x2i3n4
# (Listen in Logic Pro)
oxenvcs-cli restore m5x6i7n8
# (Listen in Logic Pro)
```

**Time saved:** Minutes vs. hours of trying to undo/redo changes!

---

### Scenario 4: Remote Collaboration

**Problem:** Your bandmate is in another city. You need to hand off the project.

**Solution (You):**
```bash
# Finish your work
oxenvcs-cli add --all
oxenvcs-cli commit -m "Finished drum tracking" --bpm 120 --tags "drums,done"

# Release lock
oxenvcs-cli lock release

# Tell bandmate on Slack: "Drums are done, lock is released!"
```

**Solution (Bandmate):**
```bash
# Check if available
oxenvcs-cli lock status

# Acquire lock
oxenvcs-cli lock acquire

# Pull your changes (future feature - push/pull)
# For now, use shared Dropbox/drive

# Work on bass
oxenvcs-cli commit -m "Added bass line" --tags "bass"

# Release when done
oxenvcs-cli lock release
```

---

## 🔍 Advanced Tips

### Combine Filters to Find Exact Version

```bash
# Find vocal tracking at 128 BPM in C Major
oxenvcs-cli log --bpm 128 --tag vocals --key "C Major"
```

**Perfect for:** Projects with many versions across different keys and tempos

---

### See File Sizes Before Committing

```bash
oxenvcs-cli diff
```

**Useful to know:**
- How much disk space this commit will use
- If you accidentally added huge files
- What actually changed

---

### Quick Status Check

Add this to your morning routine:

```bash
cd ~/Music/MyProject.logicx
oxenvcs-cli status
oxenvcs-cli log --limit 3
```

**Shows you:**
- Any uncommitted changes
- Your last 3 commits (what you did yesterday)

---

## 💡 Pro Tips

### 1. Descriptive Commit Messages

**Bad:**
```bash
oxenvcs-cli commit -m "changes"
```

**Good:**
```bash
oxenvcs-cli commit -m "Vocal tracking session 1 - 8 takes recorded" --tags "vocals,tracking"
```

**Why:** Future you will thank you when searching!

---

### 2. Use Tags Consistently

Pick a tagging system and stick to it:

```bash
--tags "tracking"       # Recording new parts
--tags "mixing"         # Mix sessions
--tags "editing"        # Editing audio/arrangement
--tags "final"          # Delivery versions
--tags "experiment"     # Trying new ideas
--tags "checkpoint"     # Before risky changes
```

---

### 3. Commit Before Big Changes

**Always do this before:**
- Completely rearranging your song
- Trying a new mix approach
- Deleting tracks
- Major tempo/key changes

```bash
oxenvcs-cli add --all
oxenvcs-cli commit -m "Before [risky thing]" --tags "checkpoint"
```

**Then:** If it goes wrong, restore in 5 seconds!

---

### 4. Check Status Often

Get in the habit:

```bash
# After every Logic Pro session
oxenvcs-cli status
oxenvcs-cli add --all
oxenvcs-cli commit -m "End of session" --tags "wip"
```

**Prevents:** "Wait, did I save that?"

---

## 🚨 Common Mistakes (And How to Fix Them)

### Mistake 1: Forgot to Commit

**Problem:** You made changes yesterday but forgot to commit.

**Fix:**
```bash
oxenvcs-cli status    # See what changed
oxenvcs-cli diff      # See details
oxenvcs-cli add --all
oxenvcs-cli commit -m "Yesterday's changes - [describe what you did]"
```

---

### Mistake 2: Committed Too Soon

**Problem:** You committed but then made more changes.

**Fix:**
```bash
# Just make another commit! It's cheap.
oxenvcs-cli add --all
oxenvcs-cli commit -m "Additional changes"
```

**Don't worry:** Commits are free! Make as many as you want.

---

### Mistake 3: Can't Remember Commit ID

**Problem:** You want to restore but forgot the commit ID.

**Fix:**
```bash
# Use filters to find it
oxenvcs-cli log --tag final           # Find final versions
oxenvcs-cli log --bpm 120             # Find by tempo
oxenvcs-cli log --key "A Minor"       # Find by key

# Or just browse recent history
oxenvcs-cli log --limit 20
```

---

### Mistake 4: Restored Wrong Version

**Problem:** You restored but it's not the one you wanted.

**Fix:**
```bash
# Find the right one
oxenvcs-cli log --limit 10

# Restore to the correct one
oxenvcs-cli restore [correct-id]
```

**Good news:** Restoring doesn't delete anything! You can restore back and forth.

---

## 📱 Quick Reference Card

**Print this and keep it by your keyboard:**

```
┌─ OxVCS Quick Commands ──────────────────────────────────┐
│                                                          │
│  oxenvcs-cli status              See what changed       │
│  oxenvcs-cli diff                See file details       │
│  oxenvcs-cli add --all           Stage changes          │
│  oxenvcs-cli commit -m "msg"     Save version           │
│  oxenvcs-cli log --limit 10      Recent history         │
│  oxenvcs-cli show <id>           View commit details    │
│  oxenvcs-cli restore <id>        Go back to version     │
│                                                          │
│  Team Commands:                                          │
│  oxenvcs-cli lock status         Check availability     │
│  oxenvcs-cli lock acquire        Start editing          │
│  oxenvcs-cli lock release        Finish editing         │
│                                                          │
│  Filters (combine any):                                  │
│  --bpm 120                       Find by tempo          │
│  --tag mixing                    Find by tag            │
│  --key "C Major"                 Find by key            │
│  --limit 10                      Limit results          │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

---

## 🎓 Next Steps

**You're ready if you can:**
- ✅ Check your project status
- ✅ Create a commit with a message
- ✅ View your commit history
- ✅ Restore to a previous version
- ✅ (Teams) Acquire and release locks

**Keep learning:**
- Try filtering logs by BPM and tags
- Practice restoring to old versions
- Set up team workflows with locks

**Get help:**
- Full command reference: `oxenvcs-cli --help`
- Specific command help: `oxenvcs-cli commit --help`
- Community: [GitHub Issues](https://github.com/jbacus/oxen-vcs-logic/issues)

---

**Remember:** Commits are cheap and fast. When in doubt, commit! Better to have too many snapshots than too few.

**Happy producing!** 🎵

---

*Last Updated: November 15, 2025*
*For GUI users: See [FOR_MUSICIANS.md](FOR_MUSICIANS.md)*
