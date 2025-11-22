# Auxin User Guide

<div align="center">
  <img src="assets/icon/icon-128.png" alt="Auxin Logo" width="80" height="80">
  <h2>Version Control for Creative Professionals</h2>
</div>

**For**: Music producers, 3D modelers, architects, and other creative professionals

This guide will help you install, configure, and use Auxin to version control your creative projects.

---

## Quick Start

**New to Auxin?** Start here in 3 steps:

### 1. Install Auxin

```bash
# Download and install (macOS only)
curl -fsSL https://raw.githubusercontent.com/jbacus/auxin/main/install.sh | bash

# Or clone and build from source
git clone https://github.com/jbacus/auxin.git
cd auxin && ./install.sh
```

**Requirements**: macOS 14.0+, Oxen CLI (`pip install oxen-ai`)

### 2. Initialize Your First Project

```bash
# Navigate to your project
cd ~/Music/MyBeat.logicx

# Initialize version control
auxin init

# Create your first commit
auxin commit -m "Initial version"
```

### 3. Make Changes and Track Versions

```bash
# Work on your project normally in Logic Pro, SketchUp, or Blender
# Auxin automatically tracks changes in the background

# View your version history
auxin log

# Restore a previous version
auxin restore <commit-id>
```

---

## Detailed Documentation

### Installation

👉 **[Full Installation Guide](INSTALL.md)**

Covers:
- System requirements
- Installing Oxen CLI dependency
- Building from source
- Troubleshooting installation issues

### Getting Started

👉 **[Getting Started Guide](docs/user/getting-started.md)**

Learn the basics:
- Initializing your first project
- Creating commits
- Viewing history
- Restoring previous versions
- Understanding Auxin's workflow

### Application-Specific Guides

Choose the guide for your creative tool:

#### Music Production (Logic Pro)

👉 **[For Musicians](docs/user/for-musicians.md)**

Complete guide covering:
- Setting up Logic Pro projects
- Auto-commit workflow
- Collaborating with band members
- Managing audio samples and stems
- Using pessimistic locks for shared projects

#### 3D Modeling (SketchUp & Blender)

👉 **[For Modelers](docs/user/for-modelers.md)**

Complete guide covering:
- Setting up SketchUp/Blender projects
- Managing large 3D assets
- Team collaboration workflows
- Texture and asset management
- Revision tracking

### Command Line Reference

👉 **[CLI Reference](docs/user/cli-reference.md)**

Complete command reference:
- `auxin init` - Initialize repository
- `auxin commit` - Create commits
- `auxin log` - View history
- `auxin restore` - Restore versions
- `auxin lock` - Acquire/release locks
- `auxin push/pull` - Sync with team
- And 20+ more commands

### Cloud Collaboration

👉 **[Cloud Sharing Guide](docs/user/cloud-sharing.md)**

Team workflow guide:
- Setting up Oxen Hub account
- Sharing projects with collaborators
- Managing file locks
- Resolving conflicts
- Remote collaboration best practices

### Troubleshooting

👉 **[Troubleshooting Guide](docs/user/troubleshooting.md)**

Common issues and solutions:
- Installation problems
- Lock conflicts
- Sync errors
- Performance issues
- Error code reference: [Error Codes](docs/user/error-codes.md)

---

## GUI vs CLI

Auxin offers both a graphical app and command-line interface. Both provide the same functionality - choose based on your preference:

| Preference | Recommendation |
|------------|----------------|
| New to version control | Start with **GUI app** (Auxin.app) |
| Comfortable with Terminal | **CLI** is faster for power users |
| Working remotely via SSH | **CLI** only option |
| Prefer keyboard shortcuts | **CLI** for scripting and automation |
| Prefer point-and-click | **GUI app** for visual workflow |

**GUI App Location**: `/Applications/Auxin.app` (after installation)

**CLI Location**: Available in your `$PATH` as `auxin`

---

## Supported Applications

| Application | Status | Guide |
|-------------|--------|-------|
| Logic Pro | ✅ Full support | [For Musicians](docs/user/for-musicians.md) |
| SketchUp | ✅ Full support | [For Modelers](docs/user/for-modelers.md) |
| Blender | 🚧 In progress | [For Modelers](docs/user/for-modelers.md) |
| Final Cut Pro | 📅 Planned | - |
| Others | 🔌 [Extensible](docs/developer/extensibility.md) | - |

---

## Core Concepts

### Version Control for Binary Files

Traditional version control (Git) doesn't work well with large binary files like:
- Logic Pro projects (.logicx bundles)
- SketchUp models (.skp files)
- Audio samples and stems
- Textures and 3D assets

**Auxin solves this** using:
- **Block-level deduplication** - Only stores changed blocks, not entire files
- **Pessimistic locking** - Prevents conflicts before they happen
- **Application-aware** - Understands project structure for each app
- **Power-safe** - Auto-commits before sleep/shutdown to prevent data loss

### Auto-Commit Workflow

Auxin's background daemon monitors your projects and automatically creates draft commits after periods of inactivity (default: 30 seconds).

**Benefits**:
- Never lose work
- Granular version history
- No manual "remember to commit"
- Can still create named commits manually

### Pessimistic Locking

Unlike Git (optimistic merging), Auxin uses **pessimistic locks**:

**Why?** Binary files cannot be automatically merged. A locked file prevents others from editing it, avoiding conflicts entirely.

**How it works**:
```bash
# Acquire lock before editing
auxin lock acquire

# Work on your project (others cannot edit)
# ...

# Release lock when done
auxin lock release
```

Locks auto-expire after 24 hours to prevent indefinite blocking.

---

## Workflow Examples

### Solo Workflow (Local Only)

```bash
# Initialize project
cd ~/Music/MyTrack.logicx
auxin init

# Work in Logic Pro
# Auto-commits happen in background every 30s

# Create named checkpoint
auxin commit -m "Finished drums section"

# Continue working...
auxin commit -m "Added vocals"

# View history
auxin log

# Restore earlier version
auxin restore <commit-id>
```

### Team Workflow (Cloud Sync)

```bash
# Team member A: Create and push project
cd ~/Music/SharedTrack.logicx
auxin init
auxin remote add origin https://hub.oxen.ai/team/SharedTrack
auxin commit -m "Initial version"
auxin push

# Team member B: Clone and work
auxin clone https://hub.oxen.ai/team/SharedTrack ~/Music/SharedTrack.logicx
cd ~/Music/SharedTrack.logicx

# Acquire lock before editing
auxin lock acquire

# Work in Logic Pro...
auxin commit -m "Added bass line"

# Release lock and push
auxin lock release
auxin push

# Team member A: Pull changes
cd ~/Music/SharedTrack.logicx
auxin pull
```

---

## Performance Tips

### Large Projects (1GB+)

Auxin is designed for large creative projects. Some tips for optimal performance:

1. **Use .auxinignore** to skip cache/temp files
2. **Commit regularly** to create incremental snapshots
3. **Use locks** to prevent concurrent edits
4. **Monitor disk space** - Auxin uses deduplication but still needs storage

### Network Sync

When working with remote collaborators:

1. **Push frequently** to share your work
2. **Pull before editing** to get latest changes
3. **Use locks** to coordinate who's editing
4. **Consider bandwidth** for large file transfers

---

## Getting Help

### Documentation

- 📖 **This guide** - Overview and quick reference
- 📖 **[Getting Started](docs/user/getting-started.md)** - Detailed tutorial
- 📖 **[CLI Reference](docs/user/cli-reference.md)** - All commands
- 📖 **[Troubleshooting](docs/user/troubleshooting.md)** - Common issues

### Support

- 🐛 **Issues**: [GitHub Issues](https://github.com/jbacus/auxin/issues)
- 💬 **Discussions**: [GitHub Discussions](https://github.com/jbacus/auxin/discussions)
- 📧 **Email**: support@auxin.dev (coming soon)

### Community

- Discord server (coming soon)
- User forum (coming soon)

---

## Next Steps

Ready to dive deeper?

1. **Users**: Start with [Getting Started](docs/user/getting-started.md)
2. **Developers**: See [Development Guide](DEVELOPMENT.md)
3. **DevOps**: See [Deployment Guide](DEPLOYMENT.md)

---

*Last Updated: 2025-11-22*
