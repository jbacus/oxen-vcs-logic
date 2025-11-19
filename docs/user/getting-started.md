# Getting Started with Auxin

**Time to complete**: 5 minutes

This guide gets you from zero to your first commit in 5 minutes.

---

## Prerequisites

- macOS 14.0 or newer
- One of: Logic Pro 11.x, SketchUp, or Blender
- A project you want to version control

---

## Step 1: Install Auxin

**Option A: Automated (Recommended)**

```bash
# Open Terminal and run:
git clone https://github.com/jbacus/auxin.git
cd auxin
./install.sh
```

**Option B: Manual Installation**

See [Installation Guide](../../INSTALL.md) for detailed steps.

---

## Step 2: Initialize Your Project

### Using the GUI App

1. Open **Auxin.app** from Applications
2. Click **"Add Project..."**
3. Navigate to your project file (.logicx, .skp, or .blend)
4. Click **"Initialize"**
5. Wait 10-30 seconds

### Using the Command Line

```bash
# Navigate to your project
cd ~/Music/MyProject.logicx

# Initialize (auto-detects project type)
auxin init .
```

**What happens:**
- Auxin creates a `.oxen` folder inside your project
- An optimized `.oxenignore` file is generated
- Your project is ready for version control

---

## Step 3: Create Your First Commit

### Using the GUI App

1. Make some changes in your creative app
2. Save your work (Cmd+S)
3. Open Auxin.app
4. Click **"Create Milestone"**
5. Enter a message: "Initial project setup"
6. Add metadata (BPM, sample rate, etc.)
7. Click **"Commit"**

### Using the Command Line

```bash
# Stage all files
auxin add --all

# Commit with metadata
auxin commit -m "Initial project setup" --bpm 120 --sample-rate 48000
```

**What you'll see:**
```
Staging all changes...
All changes staged

Creating commit...
Commit created: a1b2c3d

Commit Details:
  Message: Initial project setup
  BPM: 120
  Sample Rate: 48000 Hz
```

---

## Step 4: Work Normally

After initialization, just work normally:

1. **Make changes** in Logic Pro, SketchUp, or Blender
2. **Save your work** (Cmd+S)
3. **Auxin automatically** creates draft snapshots in the background (every 30-60 seconds)
4. **Create milestones** when you reach important points

---

## Step 5: View History and Restore

### View Your Commits

```bash
auxin log --limit 5
```

Output:
```
Commit History
Showing last 5 commit(s)

a1b2c3d - now
  Initial project setup
  BPM: 120 | Sample Rate: 48000 Hz
```

### Restore a Previous Version

```bash
# View history to find commit ID
auxin log --limit 10

# Restore to that version
auxin restore a1b2c3d
```

---

## What's Next?

### For Music Producers
Read [For Musicians](for-musicians.md) for:
- Daily workflow tips
- Working with bandmates
- Best practices for commits

### For 3D Modelers
Read [For Modelers](for-modelers.md) for:
- SketchUp and Blender workflows
- Design phase milestones
- Team collaboration

### For Power Users
Read [CLI Reference](cli-reference.md) for:
- All commands and options
- Advanced filtering and search
- Workflow automation

---

## Quick Reference

| Action | GUI | CLI |
|--------|-----|-----|
| Initialize | Add Project → Initialize | `auxin init .` |
| Check status | Status bar | `auxin status` |
| Stage files | Automatic | `auxin add --all` |
| Create milestone | Create Milestone button | `auxin commit -m "msg"` |
| View history | History panel | `auxin log` |
| Restore | Right-click → Restore | `auxin restore <id>` |

---

## Troubleshooting

**Project won't initialize?**
- Ensure it's saved as a folder (not packaged)
- Check permissions on the project folder

**Daemon not running?**
```bash
launchctl list | grep auxin
```

**Need more help?**
- See [Troubleshooting Guide](troubleshooting.md)
- [Report an issue](https://github.com/jbacus/auxin/issues)

---

*Last Updated: 2025-11-19*
