# SketchUp Configuration for Auxin

This document describes how to configure and use Auxin for version controlling SketchUp projects.

## Overview

Auxin now supports SketchUp projects (.skp files) in addition to Logic Pro projects. The same powerful version control features - pessimistic locking, automatic draft tracking, power-safe commits, and block-level deduplication - work seamlessly with SketchUp's binary 3D model files.

## The Problem with SketchUp & Traditional VCS

SketchUp projects face similar challenges to Logic Pro when using Git:

- **Binary file format**: `.skp` files are proprietary binary (non-mergeable)
- **Large file sizes**: Complex models can be 50MB+ and grow quickly
- **Backup file bloat**: SketchUp creates `.skb` backup files that bloat repositories
- **Component libraries**: External component files and textures need tracking
- **Non-destructive workflow**: Models evolve through iterations, not algorithmic merges

Traditional Git/Git-LFS fails because:
- Git-LFS stores entire files on modification → massive bloat
- Binary project files cannot be algorithmically merged
- Merge conflicts are unresolvable without data loss
- No understanding of 3D modeling workflows

## The Solution: Auxin for SketchUp

Auxin leverages Oxen.ai's block-level deduplication and implements:
- **Pessimistic locking** to prevent binary merge conflicts
- **Intelligent asset classification** with `.oxenignore` strategies
- **Automatic draft tracking** via FSEvents monitoring
- **Power-safe commits** triggered before system sleep
- **Structured metadata** (units, layer count, component count)

## Quick Start

### 1. Initialize a Repository

```bash
# Auto-detect project type (recommended)
auxin init /path/to/MyModel.skp

# Or explicitly specify SketchUp
auxin init /path/to/MyModel.skp --type sketchup
```

This will:
- Initialize an Oxen repository in the project directory
- Create a `.oxenignore` file with SketchUp-specific patterns
- Stage the `.skp` file and any asset directories (textures/, components/)

### 2. Create Your First Commit

```bash
cd /path/to/project/
auxin commit -m "Initial model - base geometry"
```

With metadata (optional):

```bash
auxin commit -m "Architectural model v1" \
  --units "Inches" \
  --layers 10 \
  --components 150
```

### 3. Normal Workflow

The daemon automatically creates draft commits as you work:

```bash
# Open your .skp file in SketchUp
# Make changes, save
# Auxin automatically commits after 30-60s of inactivity
```

Create milestone commits for important versions:

```bash
auxin commit -m "Presentation ready - added materials and lighting"
```

## Configuration

### Project-Level Config

Create `.auxin/config.toml` in your project directory:

```toml
[project]
project_type = "sketchup"

[defaults]
verbose = false

[lock]
timeout_hours = 4

[ui]
progress = true
emoji = true
```

### User-Level Config

Create `~/.auxin/config.toml` to set defaults for all projects:

```toml
[project]
project_type = "auto"  # Auto-detect based on file extensions

[ui]
emoji = false  # Disable emoji if you prefer plain text
```

### Environment Variables

Override config via environment variables:

```bash
export AUXIN_PROJECT_TYPE=sketchup
export AUXIN_VERBOSE=true
auxin status
```

## .oxenignore Patterns for SketchUp

The auto-generated `.oxenignore` file excludes common SketchUp temp/generated files:

```gitignore
# Backup and Temporary Files
*.skb               # SketchUp auto-backup files
*~.skp              # Temporary backup copies
*.tmp               # Temporary processing files
*.swp               # Swap files
.sketchup_session   # Session data

# Generated Output
exports/            # User-exported files (DWG, OBJ, etc.)
renders/            # Rendered images and animations
output/             # General output directory

# Cache and Thumbnails
.thumbnails/        # Preview image cache
cache/              # General cache directory

# System Files
.DS_Store           # macOS Finder metadata
Thumbs.db           # Windows thumbnail cache
desktop.ini         # Windows folder settings
*.smbdelete*        # SMB network deletion markers
```

### Why Exclude These Files?

**Backup Files (*.skb):**
- SketchUp automatically creates backup files with .skb extension
- These are copies of previous saves, redundant with version control
- Can bloat repository size quickly

**Exports & Renders:**
- Large files (images, videos, 3D exports) that bloat repository
- Easily regenerable from the source .skp file
- Users intentionally export when needed

**.thumbnails/ Cache:**
- Machine-generated preview images
- Differs per machine/view settings
- Creates noisy, meaningless commits

## Structured Metadata

SketchUp commits support structured metadata for rich project history:

```bash
auxin commit -m "Furniture layout complete" \
  --units "Feet" \
  --layers 15 \
  --components 234 \
  --groups 45 \
  --tag milestone \
  --tag client-review
```

Metadata is embedded in commit messages:

```
Furniture layout complete

Units: Feet
Layers: 15
Components: 234
Groups: 45
Tags: milestone, client-review
```

### Available Metadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `--units` | String | Model units (Inches, Feet, Meters, Millimeters, etc.) |
| `--layers` | Integer | Number of layers/tags in the model |
| `--components` | Integer | Number of component instances |
| `--groups` | Integer | Number of groups in the model |
| `--tag` | String | Tag for categorization (can specify multiple times) |

## Asset Directory Tracking

Auxin automatically detects and tracks common SketchUp asset directories:

- **textures/** - Custom texture image files
- **materials/** - Material definition files
- **components/** - External component `.skp` files

If these directories exist in your project folder, they'll be included in version control automatically.

## Collaboration & Locking

### Acquiring a Lock

Before editing, acquire exclusive lock:

```bash
auxin lock acquire
# Opens SketchUp file for editing
```

### Releasing a Lock

After committing changes:

```bash
auxin lock release
```

### Checking Lock Status

```bash
auxin lock status
```

Sample output:

```
Lock Status: LOCKED
Holder: john@example.com
Acquired: 2025-01-15 10:30:00
Expires: 2025-01-15 14:30:00 (in 3h 45m)
```

## Comparison: SketchUp vs Logic Pro

| Feature | SketchUp | Logic Pro |
|---------|----------|-----------|
| **File Format** | `.skp` (binary 3D model) | `.logicx` (folder structure) |
| **Backup Files** | `*.skb` | `Autosave/` |
| **Generated Output** | `exports/`, `renders/` | `Bounces/`, `Freeze Files/` |
| **Asset Directories** | `textures/`, `components/` | `Resources/`, `Audio Files/` |
| **Metadata** | Units, layers, components | BPM, sample rate, key signature |
| **Project Detection** | Single `.skp` file | `.logicx` directory |

## File Size Recommendations

Oxen.ai's block-level deduplication is highly efficient for SketchUp files:

- **Small models** (<10MB): Full history with minimal storage
- **Medium models** (10-50MB): ~10-20% storage per version (typical)
- **Large models** (50-200MB): ~15-30% storage per version
- **Huge models** (200MB+): Consider splitting into component files

### Storage Savings Example

Traditional Git-LFS:
```
Version 1: 50 MB
Version 2: 50 MB (total: 100 MB)
Version 3: 50 MB (total: 150 MB)
```

Auxin with Oxen.ai:
```
Version 1: 50 MB
Version 2: +8 MB (total: 58 MB)  ← only changed blocks
Version 3: +6 MB (total: 64 MB)  ← only changed blocks
```

**Savings: ~57% less storage for 3 versions!**

## Workflow Best Practices

### 1. Regular Milestone Commits

Create commits at logical stopping points:

```bash
# After major geometry changes
auxin commit -m "Base geometry complete"

# After adding details
auxin commit -m "Added furniture and fixtures"

# Before client review
auxin commit -m "Presentation ready" --tag client-review
```

### 2. Use Descriptive Messages

Bad:
```bash
auxin commit -m "updates"
```

Good:
```bash
auxin commit -m "Revised kitchen layout per client feedback" \
  --tag revision --tag kitchen
```

### 3. Tag Important Versions

```bash
# Presentations
auxin commit -m "Client presentation v1" --tag presentation --tag v1

# Construction documents
auxin commit -m "Construction docs - Phase 1" --tag construction-docs

# Milestones
auxin commit -m "Design development complete" --tag milestone
```

### 4. Clean Up Before Committing

Delete unnecessary files before milestone commits:

```bash
# Remove old renders
rm -rf renders/

# Remove test exports
rm -rf exports/test/

# Commit clean project
auxin commit -m "Final presentation model"
```

## Troubleshooting

### "Path is not a SketchUp file (.skp)"

**Cause:** You're trying to initialize a non-.skp file.

**Solution:** Make sure you're pointing to a `.skp` file:

```bash
auxin init /path/to/MyModel.skp
```

### "No .skp file found in project"

**Cause:** Auto-detection failed to find a `.skp` file.

**Solution:** Explicitly specify the file:

```bash
auxin init /path/to/project/MyModel.skp --type sketchup
```

### ".skb files are being tracked"

**Cause:** `.oxenignore` wasn't generated or is misconfigured.

**Solution:** Regenerate `.oxenignore`:

```bash
cd /path/to/project/
rm .oxenignore
auxin init . --type sketchup --force
```

### "Lock acquisition failed"

**Cause:** Another user has the lock, or network issues.

**Solution:** Check lock status and wait for release:

```bash
auxin lock status
auxin lock wait  # Wait for lock to be released
```

## Advanced Configuration

### Custom Ignore Patterns

Edit `.oxenignore` to add project-specific patterns:

```gitignore
# ... (auto-generated patterns above)

# Custom Ignore Patterns
# Project-specific excludes
old_versions/
reference_photos/
*.pdf
```

### Multiple Models in One Repository

Track multiple `.skp` files:

```bash
# Initialize repository at directory level
cd /path/to/project/
auxin init .

# Add specific models
auxin add Model_A.skp
auxin add Model_B.skp
auxin add variants/Model_C.skp

# Commit
auxin commit -m "Multi-model project setup"
```

### Component Library Workflow

Track a library of reusable components:

```bash
# Structure
components/
├── furniture/
│   ├── chair_modern.skp
│   ├── table_dining.skp
├── fixtures/
│   ├── light_pendant.skp
└── ...

# Initialize
cd components/
auxin init .

# Commit component updates
auxin commit -m "Added new modern furniture collection"
```

## Integration with Build Tools

### Export Automation

Use Auxin hooks to trigger exports after commit:

```bash
# .auxin/hooks/post-commit
#!/bin/bash
# Auto-export to DWG for contractors
sketchup -export DWG ./exports/latest.dwg Model.skp
```

### Render Pipeline

```bash
# .auxin/hooks/pre-push
#!/bin/bash
# Generate renders before pushing to remote
render_script.sh --input Model.skp --output renders/
```

## FAQ

### Can I use Auxin with SketchUp Free (web version)?

Auxin is designed for SketchUp Pro desktop application. SketchUp Free uses cloud storage and has different file management, so Auxin may not work as expected.

### Does Auxin track SketchUp LayOut files?

Not currently. Auxin is optimized for `.skp` files. However, you can manually add `.layout` files using `auxin add`.

### Can I version control 3D Warehouse components?

Yes! Any `.skp` file can be tracked, including downloaded components. Consider creating a separate repository for your component library.

### What about SketchUp plugins/extensions?

Plugin code (Ruby `.rb` files) can be tracked with Auxin. However, this is better suited for traditional Git since they're text files.

## Next Steps

- Read [ARCHITECTURE.md](ARCHITECTURE.md) for technical details
- See [USER_GUIDE.md](USER_GUIDE.md) for general Auxin usage
- Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md) for common issues

## Support

- **GitHub Issues**: [Create an issue](https://github.com/jbacus/auxin/issues)
- **Oxen.ai Community**: hello@oxen.ai

---

*Last Updated: 2025-11-18*
