# SketchUp Workflow Examples

This document provides real-world usage examples for versioning SketchUp projects with Auxin.

## Table of Contents

- [Quick Start](#quick-start)
- [Solo Designer Workflow](#solo-designer-workflow)
- [Team Collaboration Workflow](#team-collaboration-workflow)
- [Project Milestones](#project-milestones)
- [Restoring Previous Versions](#restoring-previous-versions)
- [Best Practices](#best-practices)

---

## Quick Start

### Initialize a New Project

```bash
# Auto-detect SketchUp project
cd "/Users/designer/Projects/My House"
auxin init "House Model.skp"

# Or explicitly specify SketchUp type
auxin init --type sketchup "House Model.skp"

# Initialize with current directory
cd "/Users/designer/Projects/House Model.skp"  # If it's a folder-based project
auxin init .
```

### Make Your First Commit

```bash
# Simple commit
auxin commit -m "Initial model with basic geometry"

# Commit with metadata
auxin commit -m "Completed floor plan" \
  --units Feet \
  --layers 5 \
  --components 25 \
  --tags "floor-plan,draft"
```

### View History

```bash
# See all commits
auxin log

# Limit to last 5
auxin log --limit 5
```

---

## Solo Designer Workflow

### Scenario: Architectural House Design

A solo architect working on a residential design project.

```bash
# Day 1: Initialize project
cd "/Users/architect/Projects"
auxin init --type sketchup "Johnson Residence.skp"

# Work in SketchUp, save your changes
# Auxin automatically creates draft commits in the background

# Create milestone: Floor plan complete
auxin commit -m "Floor plan layout finalized" \
  --units Feet \
  --layers 8 \
  --components 45 \
  --tags "milestone,floor-plan"

# Day 2: Continue modeling
# (Draft commits happen automatically)

# Milestone: Exterior walls complete
auxin commit -m "Exterior walls and windows added" \
  --units Feet \
  --layers 12 \
  --components 120 \
  --tags "milestone,exterior"

# Day 3: Interior design
auxin commit -m "Kitchen and bathrooms modeled" \
  --units Feet \
  --layers 15 \
  --components 200 \
  --tags "milestone,interior"

# Week later: Final presentation model
auxin commit -m "Presentation model with materials and lighting" \
  --units Feet \
  --layers 20 \
  --components 350 \
  --file-size 52428800 \
  --tags "presentation,milestone,final"

# View your progress
auxin log
```

### Output:

```
commit a1b2c3d4 - 2025-11-18 10:30:00
  Presentation model with materials and lighting

  Units: Feet
  Layers: 20
  Components: 350
  File Size: 50.00 MB
  Tags: presentation, milestone, final

commit e5f6g7h8 - 2025-11-15 14:20:00
  Kitchen and bathrooms modeled

  Units: Feet
  Layers: 15
  Components: 200
  Tags: milestone, interior

...
```

---

## Team Collaboration Workflow

### Scenario: Commercial Building Project

Team of 3 designers working on a commercial office building.

#### Designer A: Initialize and Set Up Locks

```bash
# Initialize the project
auxin init --type sketchup "Office Building.skp"

# Push to remote for team access
auxin remote add origin https://hub.oxen.ai/team/office-building
auxin push

# Acquire lock before editing
auxin lock acquire
# Lock acquired by alice@workstation (expires in 4 hours)

# Model the structural framework
# Save in SketchUp

# Commit and release lock
auxin commit -m "Structural framework complete" \
  --units Feet \
  --layers 10 \
  --components 500 \
  --tags "structure,milestone"

auxin lock release
auxin push
```

#### Designer B: Pull and Continue

```bash
# Clone the project
auxin clone https://hub.oxen.ai/team/office-building

# Check lock status
auxin lock status
# Project is not locked

# Acquire lock
auxin lock acquire
# Lock acquired by bob@laptop (expires in 4 hours)

# Add HVAC and electrical systems
# Save in SketchUp

# Commit changes
auxin commit -m "HVAC and electrical systems added" \
  --units Feet \
  --layers 15 \
  --components 750 \
  --tags "mep,milestone"

auxin lock release
auxin push
```

#### Designer C: Conflict Prevention

```bash
# Pull latest changes
auxin pull

# Try to edit without lock (daemon prevents this)
# Open SketchUp...
# Auxin daemon: "Project is locked by bob@laptop (expires in 2 hours)"

# Check who has the lock
auxin lock status
# Locked by: bob@laptop
# Acquired: 2025-11-18 14:00:00
# Expires: 2025-11-18 18:00:00

# Wait for Bob to finish, then acquire lock
auxin lock acquire
# Lock acquired by charlie@desktop (expires in 4 hours)

# Add interior partitions and doors
auxin commit -m "Interior partitions and doors" \
  --units Feet \
  --layers 20 \
  --components 1200 \
  --tags "interior,milestone"

auxin lock release
auxin push
```

---

## Project Milestones

### Tracking Design Phases

Use tags and metadata to track major project milestones:

```bash
# Conceptual Design Phase
auxin commit -m "Concept sketch model" \
  --units Meters \
  --layers 3 \
  --components 20 \
  --tags "concept,draft"

# Schematic Design Phase
auxin commit -m "Schematic design for client review" \
  --units Meters \
  --layers 8 \
  --components 150 \
  --tags "schematic,client-review,milestone"

# Design Development Phase
auxin commit -m "Design development with details" \
  --units Millimeters \
  --layers 15 \
  --components 400 \
  --tags "design-development,milestone"

# Construction Documents Phase
auxin commit -m "Construction documentation model" \
  --units Millimeters \
  --layers 25 \
  --components 800 \
  --tags "construction-docs,milestone,final"

# Search for specific milestones
auxin log --tag milestone
auxin log --tag client-review
```

---

## Restoring Previous Versions

### Scenario: Revert to Earlier Design

Client wants to see an earlier design option:

```bash
# View history
auxin log

# Restore to specific commit (client-review version)
auxin restore e5f6g7h8

# SketchUp now shows the earlier version
# Work with it, show client

# Return to latest version
auxin restore main
```

### Scenario: Recover from Mistake

Accidentally deleted important components:

```bash
# View recent commits
auxin log --limit 5

# Restore to commit before deletion
auxin restore a1b2c3d4

# Verify everything is back
# If good, commit as current version
auxin commit -m "Recovered components from earlier version"
```

---

## Best Practices

### 1. **Commit Frequently with Meaningful Messages**

**Good:**
```bash
auxin commit -m "Added roof structure with trusses" \
  --layers 12 \
  --components 250 \
  --tags "roof,structure"
```

**Bad:**
```bash
auxin commit -m "Updated stuff"
```

### 2. **Use Metadata Consistently**

Always include:
- **Units** - Helps track if model units change
- **Layers** - Shows organizational complexity
- **Components** - Tracks model detail level
- **Tags** - For easy searching and filtering

```bash
# Template for milestones
auxin commit -m "Descriptive message" \
  --units <Feet|Meters|Inches|Millimeters> \
  --layers <number> \
  --components <number> \
  --tags "milestone,<phase>"
```

### 3. **Tag Your Work Phases**

Use consistent tags for project phases:
- `concept` - Initial concept models
- `schematic` - Schematic design
- `design-development` - Detailed design
- `construction-docs` - Construction documentation
- `client-review` - Versions shown to clients
- `milestone` - Important checkpoints
- `presentation` - Presentation-ready models

### 4. **Lock Management for Teams**

- **Always acquire lock before editing**
- **Release lock when done for the day**
- **Use reasonable timeouts** (2-4 hours for short sessions, 8 hours for full day)
- **Never force-break locks** unless emergency

```bash
# Start of work session
auxin lock acquire --timeout 4

# End of work session
auxin lock release
```

### 5. **File Size Tracking**

Track file size to monitor model bloat:

```bash
# Check current file size
ls -lh "My Model.skp"

# Include in commit
auxin commit -m "Added landscaping" \
  --units Feet \
  --file-size 73400320  # 70 MB
  --tags "landscape"

# If file size growing too fast, investigate:
# - Purge unused components: Window > Model Info > Statistics > Purge Unused
# - Optimize textures: Reduce image sizes
# - Remove hidden geometry
```

### 6. **Organize Asset Directories**

Keep assets organized:

```
My Project.skp/
├── My Project.skp         # Main model file
├── textures/              # Custom textures (tracked)
│   ├── brick_red.jpg
│   └── wood_floor.jpg
├── components/            # External components (tracked)
│   ├── furniture/
│   └── fixtures/
├── materials/             # Material libraries (tracked)
├── exports/               # NOT tracked (.oxenignore)
└── renders/               # NOT tracked (.oxenignore)
```

### 7. **Backup Workflow**

Combine Auxin with periodic backups:

```bash
# Daily: Let Auxin handle version control automatically

# Weekly: Create explicit milestone commits
auxin commit -m "Weekly checkpoint - $(date +%Y-%m-%d)" \
  --units Feet \
  --layers $(count_layers) \
  --components $(count_components) \
  --tags "weekly-backup"

# Push to remote for team backup
auxin push
```

---

## Summary

- **Initialize** with `auxin init --type sketchup "Model.skp"`
- **Commit milestones** with meaningful messages and metadata
- **Use locks** for team collaboration
- **Tag consistently** for easy searching
- **Track file size** to monitor bloat
- **Restore easily** when needed

For more information, see:
- [SKETCHUP_CONFIGURATION.md](SKETCHUP_CONFIGURATION.md) - Complete SketchUp guide
- [USER_GUIDE.md](USER_GUIDE.md) - General Auxin usage
