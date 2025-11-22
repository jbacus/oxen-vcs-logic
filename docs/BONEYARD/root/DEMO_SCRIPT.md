# Auxin Demo Script - "Pete & Louis: Collaborating Across Continents"

**Duration**: 10 minutes
**Format**: Live CLI demo with narrative
**Audience**: Musicians, producers, potential users
**Goal**: Show how Auxin enables conflict-free remote collaboration

---

## Setup (Do Before Demo)

### Pre-Demo Checklist (30 min before)
- [ ] macOS machine with Auxin CLI installed
- [ ] Oxen CLI installed: `pip install oxen-ai`
- [ ] Terminal with good font size (24pt minimum for projector)
- [ ] Demo Logic Pro project ready (1-2GB, in `~/Music/DemoProject.logicx`)
- [ ] Clean terminal history: `clear && history -c`
- [ ] Test internet connection
- [ ] Open demo script on second screen
- [ ] Start screen recording (backup)
- [ ] Have pre-recorded video ready (fallback)

### Terminal Setup
```bash
# Set up nice terminal appearance
export PS1="\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[0m\]\$ "

# Navigate to demo directory
cd ~/Music
```

---

## Demo Script

### PART 1: The Problem (2 minutes)

**[SLIDE: Photo of Pete and Louis]**

**Presenter**:
> "Meet Pete and Louis. They're music production students who just started a company together. Pete lives in Colorado, Louis lives in London. They're 4,500 miles apart, working across a 7-hour time difference on Logic Pro sessions that can be 5-10 gigabytes.

> They tried Git. Here's what happened..."

**[SWITCH TO TERMINAL]**

**Presenter**:
> "Let me show you Git with a typical Logic Pro project:"

```bash
# Show the Git problem
cd GitDemo
git add MyProject.logicx/
# [wait 10+ seconds while Git struggles]
```

**Presenter** (while waiting):
> "Notice how slow this is? Git wasn't designed for multi-gigabyte binary files. And it gets worse..."

```bash
git commit -m "Added vocals"
# Repository size: 2.3 GB

# Make a small change
# Commit again
git commit -m "Tweaked EQ"
# Repository size: 4.6 GB  😱
```

**Presenter**:
> "One small change, and Git stores the entire file again. After 20 versions, their repo is 46 gigabytes.

> But the real disaster is when they both edit at the same time..."

```bash
git merge origin/main
# Auto-merging MyProject.logicx/projectData
# CONFLICT (content): Merge conflict in MyProject.logicx/projectData
```

**Presenter**:
> "This is a binary file. There's no way to automatically merge it. One of them just lost hours of work.

> This is why professional studios pay $900+ per user per year for Perforce. But Pete and Louis can't afford that.

> So they built something better. Let me show you Auxin."

---

### PART 2: Meet Auxin (1 minute)

**[SLIDE: Auxin logo + tagline]**

**Presenter**:
> "Auxin is version control designed specifically for creative applications. It combines:
> - Git's power (full version history)
> - Perforce's conflict prevention (pessimistic locking)
> - Creative-specific features (BPM, key signature, searchable metadata)
> - All free and open source, works completely offline.

> Let me show you Pete and Louis's actual workflow."

---

### PART 3: Pete's Morning Session (2 minutes)

**[SWITCH TO TERMINAL]**

**Presenter**:
> "It's Monday morning in Colorado. Pete opens his Logic Pro project and initializes Auxin."

```bash
cd ~/Music
auxin init DemoProject.logicx --type logicpro
```

**Expected Output**:
```
✓ Detected Logic Pro project
✓ Initialized Oxen repository
✓ Created .oxenignore (excluding bounces, freeze files)
✓ Project ready for version control

Repository: /Users/pete/Music/DemoProject.logicx
Type: Logic Pro
Next: auxin add --all && auxin commit -m "Initial version"
```

**Presenter**:
> "Notice it auto-detected Logic Pro and set up intelligent ignore patterns. No configuration needed.

> Pete's been working on guitar tracks. Let's see what changed:"

```bash
auxin status
```

**Expected Output**:
```
┌─ Repository Status ─────────────────────────────────────┐
│                                                          │
│  Branch: main                                            │
│  Changes: 3 modified, 1 untracked                        │
│                                                          │
└──────────────────────────────────────────────────────────┘

● Modified files (3):
  M projectData                    (2.1 MB)
  M Resources/Guitar_Take1.wav     (45.3 MB)
  M Resources/Guitar_Take2.wav     (47.8 MB)

● Untracked files (1):
  ? Resources/Guitar_Take3.wav     (46.1 MB)

ℹ Next step: auxin add --all
```

**Presenter**:
> "Beautiful colored output shows exactly what changed. Now Pete commits his work with metadata:"

```bash
auxin add --all
auxin commit -m "Guitar tracking complete - 12 takes recorded" \
  --bpm 128 \
  --key "A minor" \
  --tags "tracking,guitars,v1"
```

**Expected Output**:
```
Adding files... ━━━━━━━━━━━━━━━━━━━━━━━ 100% (139.2 MB)
✓ Added 4 files

Committing changes...
✓ Commit successful

Commit: a1b2c3d4
Author: Pete <pete@example.com>
Date: 2025-11-21 09:30:00 MST

Message: Guitar tracking complete - 12 takes recorded

Metadata:
  BPM: 128
  Key: A minor
  Tags: tracking, guitars, v1

Files: 4 changed, 139.2 MB
```

**Presenter**:
> "Notice the metadata - BPM, key signature, tags. This makes commits searchable later.

> Now Pete needs to hand off to Louis, but he doesn't want Louis editing at the same time. So he acquires a lock:"

```bash
auxin lock acquire
```

**Expected Output**:
```
✓ Lock acquired successfully

Lock holder: pete@colorado
Acquired: 2025-11-21 09:30:00 MST
Expires: 2025-11-21 13:30:00 MST (4 hours)

⚠ Remember to release the lock when done:
  auxin lock release
```

**Presenter**:
> "The lock prevents conflicts. Louis literally cannot edit while Pete has the lock.

> Pete finishes his work and releases:"

```bash
auxin lock release
auxin push
```

**Expected Output**:
```
✓ Lock released

Pushing to remote...
Uploading: Guitar_Take1.wav ━━━━━━━━━━━━━━━ 100% (45.3 MB)
Uploading: Guitar_Take2.wav ━━━━━━━━━━━━━━━ 100% (47.8 MB)
Uploading: Guitar_Take3.wav ━━━━━━━━━━━━━━━ 100% (46.1 MB)

✓ Pushed 1 commit to remote
✓ All locks released
```

**Presenter**:
> "Fast, reliable push. Now Louis can take over."

---

### PART 4: Louis's Evening Session (2 minutes)

**Presenter**:
> "It's Monday evening in London - 7 hours ahead. Louis sees Pete's message on Slack: 'Guitar tracks are done, project is all yours!'

> Louis pulls Pete's changes:"

```bash
# Simulate switching users (or use second terminal)
export USER=louis
cd ~/Music/DemoProject.logicx

auxin pull
```

**Expected Output**:
```
Fetching from remote...
✓ Fetched 1 new commit

Downloading: Guitar_Take1.wav ━━━━━━━━━━━━ 100% (45.3 MB)
Downloading: Guitar_Take2.wav ━━━━━━━━━━━━ 100% (47.8 MB)
Downloading: Guitar_Take3.wav ━━━━━━━━━━━━ 100% (46.1 MB)

✓ Updated 4 files
✓ Current: a1b2c3d4 (main)

Latest commit:
  Guitar tracking complete - 12 takes recorded
  BPM: 128, Key: A minor
  Tags: tracking, guitars, v1
```

**Presenter**:
> "Instant conflict-free update. Louis now has all of Pete's guitar tracks.

> Before editing, Louis checks the lock status:"

```bash
auxin lock status
```

**Expected Output**:
```
✓ Project is not locked

Available for editing.
To acquire lock: auxin lock acquire
```

**Presenter**:
> "Perfect. Pete released the lock. Louis acquires it for his session:"

```bash
auxin lock acquire
```

**Expected Output**:
```
✓ Lock acquired successfully

Lock holder: louis@london
Acquired: 2025-11-21 18:30:00 GMT
Expires: 2025-11-21 22:30:00 GMT (4 hours)
```

**Presenter**:
> "Now Louis is protected. Even if Pete wakes up early and tries to edit, Auxin will prevent it.

> Louis adds synth pads and commits:"

```bash
# Simulate work in Logic Pro
# (in real demo, you could actually edit a file)

auxin commit -m "Synth pads added - analog warmth" \
  --bpm 128 \
  --key "A minor" \
  --tags "production,synths,v1"
```

**Expected Output**:
```
✓ Added 2 files
✓ Commit successful

Commit: e5f6g7h8
Author: Louis <louis@example.com>
Date: 2025-11-21 18:45:00 GMT

Message: Synth pads added - analog warmth

Metadata:
  BPM: 128
  Key: A minor
  Tags: production, synths, v1

Files: 2 changed, 67.4 MB
```

**Presenter**:
> "Louis releases the lock and pushes:"

```bash
auxin lock release
auxin push
```

**Presenter**:
> "Perfect handoff. No conflicts, no confusion. Let's see their progress."

---

### PART 5: Project History (1 minute)

**Presenter**:
> "Let's look at their complete history with metadata:"

```bash
auxin log --limit 5
```

**Expected Output**:
```
commit e5f6g7h8 - 2025-11-21 18:45:00 GMT
  Synth pads added - analog warmth

  Author: Louis <louis@example.com>
  BPM: 128
  Key: A minor
  Tags: production, synths, v1

commit a1b2c3d4 - 2025-11-21 09:30:00 MST
  Guitar tracking complete - 12 takes recorded

  Author: Pete <pete@example.com>
  BPM: 128
  Key: A minor
  Tags: tracking, guitars, v1

commit b2c3d4e5 - 2025-11-20 15:20:00 MST
  Initial structure and drums

  Author: Pete <pete@example.com>
  BPM: 128
  Key: A minor
  Tags: tracking, drums, initial
```

**Presenter**:
> "Beautiful history with metadata. Now watch this - natural language search:"

```bash
auxin search "bpm:128 key:minor tag:tracking"
```

**Expected Output**:
```
Found 2 commits matching query:

[92% match] a1b2c3d4 - Guitar tracking complete
  BPM: 128 ✓
  Key: A minor ✓
  Tags: tracking ✓, guitars, v1
  Date: 2025-11-21 09:30:00

[88% match] b2c3d4e5 - Initial structure and drums
  BPM: 128 ✓
  Key: A minor ✓
  Tags: tracking ✓, drums, initial
  Date: 2025-11-20 15:20:00

To restore: auxin restore <commit-hash>
```

**Presenter**:
> "Powerful search across all their history. No other VCS can do this for creative projects."

---

### PART 6: Time Travel (1 minute)

**Presenter**:
> "Let's say their client wants to hear yesterday's mix before the synths. Easy:"

```bash
auxin restore a1b2c3d4
```

**Expected Output**:
```
Restoring commit a1b2c3d4...

Downloading files... ━━━━━━━━━━━━━━━━━━━━━ 100%

✓ Restored to: Guitar tracking complete - 12 takes recorded
✓ Files updated: 4

⚠ You are now at commit a1b2c3d4 (detached HEAD)
  To return to latest: auxin restore main
```

**Presenter**:
> "Instant time travel. The Logic Pro project now looks exactly like it did yesterday. Open Logic, and all the synths are gone.

> Client decides they like the new version better:"

```bash
auxin restore main
```

**Expected Output**:
```
✓ Restored to latest commit (e5f6g7h8)
✓ Back on main branch
```

**Presenter**:
> "Back to the latest version. Non-destructive, instant, reliable."

---

### PART 7: The Wow Factor - Storage Efficiency (30 seconds)

**Presenter**:
> "Here's something amazing. Let me show you the storage efficiency:"

```bash
# Show project size
du -sh DemoProject.logicx
# 2.4 GB

# Show Oxen repository size (all versions)
du -sh DemoProject.logicx/.oxen
# 2.6 GB
```

**Presenter**:
> "Original project: 2.4 GB. All 20 versions with full history: 2.6 GB.

> With Git, this would be 48 gigabytes. With manual backups, 50+ gigabytes.

> Auxin uses block-level deduplication - only changed blocks are stored. 10 to 100x more efficient."

---

### PART 8: Wrap-Up (1 minute)

**[SLIDE: Competitive comparison]**

**Presenter**:
> "Let's recap what we just saw:

> **Git**: Can't handle large binaries, creates conflicts, bloats repository
> **Auxin**: Designed for large binaries, prevents conflicts, efficient storage

> **Perforce**: Does this well, costs $900+ per user per year
> **Auxin**: Does this well, completely free and open source

> **Splice**: Cloud-based, requires internet, $240/year subscription
> **Auxin**: Works offline, you own your data, free forever

> **Manual backups**: Wastes storage, hard to organize, no collaboration
> **Auxin**: Efficient storage, searchable history, conflict-free teams

**[SLIDE: Key features]**

**Features**:
- ✅ Block-level deduplication (10-100x storage efficiency)
- ✅ Pessimistic locking (binary conflicts impossible)
- ✅ Application metadata (BPM, key, layers - searchable)
- ✅ Works completely offline (local-first)
- ✅ Free and open source (MIT license)
- ✅ Supports Logic Pro, SketchUp, Blender (more coming)

**[SLIDE: Get started]**

**Presenter**:
> "Ready to try Auxin?

**Installation**:
```bash
# Homebrew (coming soon)
brew tap jbacus/auxin
brew install auxin

# Or download from GitHub
https://github.com/jbacus/auxin/releases
```

**Quick Start**:
```bash
auxin init YourProject.logicx
auxin add --all
auxin commit -m "Your first commit"
```

**Learn More**:
- GitHub: github.com/jbacus/auxin
- Docs: github.com/jbacus/auxin/docs
- Discord: [coming soon]

**Questions?"

---

## Presenter Notes

### Key Talking Points

**Problem Statement** (emphasize pain):
- Git wasn't designed for multi-GB binary files
- Binary merge conflicts are unresolvable (data loss)
- Creative teams need better tools
- Existing solutions are expensive or cloud-only

**Solution Positioning**:
- "Git for large binaries, done right"
- "Perforce workflow at indie price (free)"
- "Open-source alternative to Splice"

**Competitive Advantages** (repeat these):
1. **Storage efficiency** - 10-100x better than Git
2. **Conflict prevention** - Locks, not merges
3. **Application aware** - Searchable metadata
4. **Local-first** - Works offline, you own data
5. **Free forever** - Open source, no subscription

### Timing Breakdown

| Section | Duration | Purpose |
|---------|----------|---------|
| Problem (Git fails) | 2 min | Establish pain point |
| Meet Auxin | 1 min | Position solution |
| Pete's session | 2 min | Show solo workflow |
| Louis's session | 2 min | Show collaboration |
| Project history | 1 min | Show metadata power |
| Time travel | 1 min | Show restore |
| Storage efficiency | 30 sec | Show technical advantage |
| Wrap-up | 1 min | Call to action |
| **Total** | **10 min** | |

### Demo Flow Tips

**Pacing**:
- Speak slowly and clearly (projector lag)
- Pause after each command (let output sink in)
- Don't rush the "Expected Output" - let people read

**Engagement**:
- Make eye contact during presenter sections
- Show enthusiasm (this solves a real problem!)
- Use hand gestures to emphasize points

**Technical**:
- Have water nearby (talking for 10 min)
- Test terminal font size before presenting
- Keep demo script visible on second screen

**Handling Delays**:
- If command takes >5 seconds, explain what's happening
- "Notice how it's processing 140 MB of audio files..."
- "The progress bar shows chunked upload..."

---

## Fallback Options

### If Live Demo Fails

**Option 1: Pre-recorded Video** (safest)
- Have 10-minute screencast ready
- Narrate over it live
- Still feels authentic

**Option 2: Screenshot Walkthrough**
- Slide deck with terminal screenshots
- Walk through step-by-step
- "Here's what you would see..."

**Option 3: Partial Demo**
- Show only working parts (e.g., local workflow)
- Use slides for remote collaboration
- "I'll show you this part recorded..."

### If Specific Commands Fail

**If `auxin init` fails**:
- Use pre-initialized repo
- "I've already initialized this project, so let's see the status..."

**If `auxin push/pull` fails**:
- Show pre-recorded video of remote sync
- "Due to network conditions, here's what that looks like..."

**If output is garbled**:
- Take screenshot beforehand
- "The output looks like this..." [show slide]

### If Time Runs Over

**Cut these sections**:
1. Storage efficiency demo (nice-to-have)
2. Search demo (can mention verbally)
3. Detailed wrap-up (go straight to CTA)

**Minimum viable demo** (7 minutes):
- Problem (1 min)
- Pete's session (2 min)
- Louis's session (2 min)
- Time travel (1 min)
- CTA (1 min)

---

## Audience-Specific Variations

### For Musicians (Default)
- Use Pete & Louis story (relatable)
- Emphasize Logic Pro, audio files
- Focus on "never lose your work again"

### For 3D Modelers
- Change to SketchUp/Blender project
- Pete & Louis become architects
- Emphasize large model files, layers, components

### For Technical Audience
- Spend more time on architecture
- Show test coverage: `cargo test`
- Explain Oxen backend, block deduplication
- Dive deeper into locking algorithm

### For Business/Investors
- Emphasize market size (millions of Logic/SketchUp users)
- Show competitive comparison table
- Mention business model (free core + paid hosting)
- Present growth metrics if available

---

## Pre-Demo Setup Script

Copy-paste this to set up your demo environment:

```bash
#!/bin/bash
# Demo setup script - run 30 min before presenting

echo "Setting up Auxin demo environment..."

# 1. Clean environment
cd ~/Music
rm -rf DemoProject.logicx GitDemo 2>/dev/null
clear
history -c

# 2. Create demo project (or copy real Logic project)
# Option A: Use real Logic Pro project
cp -r ~/Music/RealProject.logicx ~/Music/DemoProject.logicx

# Option B: Create mock project structure for demo
mkdir -p DemoProject.logicx/Resources
dd if=/dev/urandom of=DemoProject.logicx/projectData bs=1m count=2
dd if=/dev/urandom of=DemoProject.logicx/Resources/Guitar_Take1.wav bs=1m count=45
dd if=/dev/urandom of=DemoProject.logicx/Resources/Guitar_Take2.wav bs=1m count=47

# 3. Set up Git comparison
mkdir GitDemo
cd GitDemo
git init
cp -r ../DemoProject.logicx .
cd ..

# 4. Configure terminal
export PS1="\[\e[32m\]\u@\h\[\e[0m\]:\[\e[34m\]\w\[\e[0m\]\$ "

# 5. Verify Auxin works
auxin --version
oxen --version

# 6. Start screen recording
echo "✓ Environment ready!"
echo "⚠ Remember to start screen recording"
echo "⚠ Open demo script on second screen"
echo ""
echo "Press Enter to begin demo practice..."
read
```

---

## Post-Demo Follow-Up

### Immediate (During Q&A)
- Collect email addresses for beta access
- Share GitHub repo link
- Answer specific technical questions
- Demo advanced features if time permits

### Within 24 Hours
- Email recording to attendees
- Share slide deck
- Post to social media with highlights
- Follow up with interested users

### Within 1 Week
- Write blog post about demo response
- Create tutorial video based on feedback
- Update documentation with common questions
- Plan next demo based on learnings

---

## Success Metrics

### Demo Quality
- [ ] Completed in <11 minutes
- [ ] Zero command failures
- [ ] Audience engaged (questions, nods)
- [ ] Clear call-to-action delivered

### Audience Response
- [ ] 5+ questions asked
- [ ] 10+ GitHub stars within 24 hours
- [ ] 3+ email signups for beta
- [ ] Social media shares

### Follow-Through
- [ ] Recording shared within 24 hours
- [ ] All questions answered
- [ ] Follow-up content created
- [ ] Next demo scheduled

---

**Demo Checklist**: Print this and bring to presentation

```
PRE-DEMO (30 min before):
□ Terminal font size 24pt+
□ Demo script open on second screen
□ Screen recording started
□ Internet connection tested
□ Demo project ready in ~/Music/DemoProject.logicx
□ Backup video ready to play
□ Water bottle
□ Clicker batteries checked

DURING DEMO:
□ Speak slowly
□ Pause after commands
□ Make eye contact
□ Show enthusiasm
□ Handle questions gracefully

POST-DEMO:
□ Share GitHub link
□ Collect emails
□ Answer questions
□ Thank attendees
```

---

*Good luck with your demo! Remember: Pete and Louis's story is relatable, the CLI is solid, and you have backup options. You've got this!* 🚀
