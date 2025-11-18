# Auxin Version Control for Blender

A Blender addon that integrates Auxin version control directly into Blender's interface, providing Git-like version control specifically optimized for 3D projects.

## Features

- **Native Blender UI**: Version control panels in the 3D View sidebar
- **Scene Metadata**: Automatically captures object counts, materials, render settings, and animation data with each commit
- **Commit History**: Browse and restore previous versions of your project
- **Lock Management**: Acquire exclusive locks for team collaboration
- **Remote Operations**: Push and pull to/from Oxen Hub
- **Auto-staging**: Option to automatically stage all changes before commit

## Requirements

- **Blender**: 3.0 or later
- **Auxin CLI**: Must be installed and available in your system PATH
- **Oxen CLI**: Required by Auxin for repository operations

## Installation

### Method 1: Install from ZIP (Recommended)

1. Download or create a ZIP file of the `blender-auxin-plugin` folder
2. In Blender, go to **Edit > Preferences > Add-ons**
3. Click **Install...**
4. Select the ZIP file
5. Enable the addon by checking the checkbox next to "Auxin Version Control"

### Method 2: Manual Installation

1. Copy the `blender-auxin-plugin` folder to your Blender addons directory:
   - **Windows**: `%APPDATA%\Blender Foundation\Blender\<version>\scripts\addons\`
   - **macOS**: `~/Library/Application Support/Blender/<version>/scripts/addons/`
   - **Linux**: `~/.config/blender/<version>/scripts/addons/`

2. Rename the folder to `auxin_version_control` (optional, for cleaner naming)

3. In Blender, go to **Edit > Preferences > Add-ons**

4. Search for "Auxin" and enable the addon

### Install Auxin CLI

The addon requires the Auxin CLI to be installed:

```bash
# Clone the repository
git clone https://github.com/jbacus/auxin.git
cd auxin

# Build and install
cd Auxin-CLI-Wrapper
cargo build --release

# Add to PATH (adjust for your system)
export PATH="$PATH:$(pwd)/target/release"

# Or use the installer
cd ..
./install.sh
```

Also install the Oxen CLI:

```bash
pip3 install oxen-ai
# or
cargo install oxen
```

## Usage

### Accessing the Panel

1. Open the 3D View sidebar by pressing `N`
2. Click on the "Auxin" tab
3. The version control panels will appear

### Initializing a Repository

1. Save your Blender file (`.blend`)
2. In the Auxin panel, click **Initialize**
3. This creates an Auxin/Oxen repository in your project directory

### Making Commits

1. Make changes to your scene
2. Save your file (`Ctrl+S`)
3. Enter a commit message in the "Commit Message" field
4. Optionally add tags (comma-separated)
5. Click **Commit**

The commit will automatically include scene metadata:
- Object counts (meshes, lights, cameras)
- Material count
- Render engine and resolution
- Animation frame range and FPS
- File size
- Blender version

### Viewing History

1. Expand the **History** panel
2. Click **Refresh** to load commits
3. Click on a commit to select it
4. Click **Restore Selected** to restore that version

### Lock Management (Team Collaboration)

1. Expand the **Lock Management** panel
2. Click **Acquire Lock** before editing
3. Click **Release Lock** when done

Locks prevent multiple team members from editing simultaneously, avoiding merge conflicts with binary `.blend` files.

### Remote Operations

1. Expand the **Remote** panel
2. Use **Push** to upload commits to Oxen Hub
3. Use **Pull** to download commits from Oxen Hub

Note: You must be authenticated with Oxen Hub (`auxin auth login`).

## Panel Overview

### Main Panel
- Shows current file and repository status
- Quick access to stage all changes
- Status refresh button

### Commit Panel
- Commit message input
- Tags input
- Metadata inclusion toggle
- Auto-stage toggle
- Preview of scene metadata

### History Panel
- List of recent commits
- Select and restore previous versions

### Remote Panel
- Push/pull operations

### Lock Management Panel
- Current lock status
- Acquire/release lock controls

### Tools Panel
- Open terminal in project directory

## Addon Preferences

Access via **Edit > Preferences > Add-ons > Auxin Version Control**

- **Auxin CLI Path**: Custom path to auxin executable (leave empty for system PATH)
- **Auto Check Status**: Automatically check repository status when opening files

## Keyboard Shortcuts

The addon doesn't register any default keyboard shortcuts, but you can add them via **Edit > Preferences > Keymap**.

## Troubleshooting

### "auxin CLI not found"

Make sure the Auxin CLI is installed and in your system PATH:

```bash
# Check if auxin is available
auxin --version
```

### "Not an Oxen repository"

Initialize the repository first:
1. Save your Blender file
2. Click **Initialize** in the Auxin panel

### Commit fails with timeout

Large files may take longer to process. Try:
1. Ensure you're not including unnecessary files
2. Check your `.oxenignore` file for proper exclusions

### Lock operations fail

Ensure:
1. You're connected to the auxin-server for lock management
2. Check server status with `auxin server status`

## Scene Metadata

Each commit automatically captures:

| Field | Description |
|-------|-------------|
| Scenes | Number of scenes in the file |
| Active Scene | Name of the active scene |
| Objects | Total number of objects |
| Meshes | Number of mesh objects |
| Lights | Number of light objects |
| Cameras | Number of camera objects |
| Materials | Number of materials |
| Render Engine | Current render engine (Cycles, Eevee, etc.) |
| Resolution | Render resolution |
| Frame Range | Animation start and end frames |
| FPS | Frames per second |
| Blender Version | Version that created the file |
| File Size | Size of the .blend file |

## File Structure

```
blender-auxin-plugin/
├── __init__.py          # Main addon code
├── README.md            # This file
└── LICENSE              # MIT License
```

## Contributing

Contributions are welcome! Please see the main Auxin repository for contribution guidelines.

## License

MIT License - see the main Auxin project for details.

## Support

- **Issues**: https://github.com/jbacus/auxin/issues
- **Documentation**: https://github.com/jbacus/auxin

## Changelog

### Version 1.0.0
- Initial release
- Core version control operations (init, add, commit, restore)
- Scene metadata extraction
- Commit history browser
- Lock management
- Remote push/pull
- Addon preferences
