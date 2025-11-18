# Auxin SketchUp Extension

A SketchUp extension that provides a native front-end to the Auxin version control system, enabling version control for SketchUp projects directly within the SketchUp interface.

## Features

- **Repository Management**: Initialize Auxin repositories for SketchUp projects
- **Commit History**: View and browse commit history with full metadata
- **Quick Commits**: Create commits with automatic model metadata extraction
- **Version Restore**: Restore models to any previous commit
- **Model Metadata**: Automatic extraction and tracking of:
  - Model units (Inches, Feet, Meters, etc.)
  - Layer/tag count
  - Component definitions
  - Groups
  - Faces and edges
  - Materials
  - File size
- **Native UI**: Beautiful HTML dialog that integrates seamlessly with SketchUp
- **Toolbar Integration**: Quick access buttons in the SketchUp toolbar
- **Menu Integration**: Full menu structure under Extensions > Auxin

## Requirements

- **SketchUp**: 2017 or later (HtmlDialog support required)
- **Auxin CLI**: Must be installed and available in system PATH
- **Oxen**: Auxin requires Oxen CLI (`pip install oxen-ai`)

## Installation

### Method 1: Copy to Plugins Folder

1. Locate your SketchUp Plugins folder:
   - **macOS**: `~/Library/Application Support/SketchUp [version]/SketchUp/Plugins`
   - **Windows**: `%APPDATA%\SketchUp\SketchUp [version]\SketchUp\Plugins`

2. Copy these files to the Plugins folder:
   - `auxin.rb`
   - `auxin/` (entire directory)

3. Restart SketchUp

### Method 2: Install via Extension Manager

1. Package the extension as an `.rbz` file:
   ```bash
   cd Auxin-SketchUp-Extension
   zip -r auxin.rbz auxin.rb auxin/
   ```

2. In SketchUp, go to **Window > Extension Manager**

3. Click **Install Extension** and select `auxin.rbz`

## Usage

### Opening the Auxin Panel

- **Menu**: Extensions > Auxin > Open Auxin Panel
- **Toolbar**: Click the Auxin button in the Auxin toolbar
- **Shortcut**: Can be assigned via Window > Preferences > Shortcuts

### Initializing a Repository

1. Save your SketchUp model first
2. Open the Auxin panel
3. Click **Initialize Repository**
4. The repository will be created in the same directory as your model

### Creating Commits

1. Make changes to your model
2. Save the model (Cmd/Ctrl + S)
3. Open the Auxin panel
4. Enter a descriptive commit message
5. Optionally add tags (comma-separated)
6. Click **Create Commit**

**Quick Commit** (faster workflow):
- Use Extensions > Auxin > Quick Commit
- Or click the Commit button in the toolbar

### Viewing History

1. Open the Auxin panel
2. Click the **History** tab
3. Browse commits with their messages and metadata
4. Click on any commit to see restore options

### Restoring Previous Versions

1. Go to the History tab
2. Click on the commit you want to restore
3. Confirm the restore operation
4. The model will be reloaded with the restored version

### Model Information

1. Click the **Info** tab in the Auxin panel
2. View current model metadata:
   - Units configuration
   - Layer count
   - Component and group counts
   - Geometry statistics
   - File size

## Keyboard Shortcuts

You can assign keyboard shortcuts to Auxin commands:

1. Go to **Window > Preferences > Shortcuts**
2. Search for "Auxin"
3. Assign shortcuts to:
   - Open Auxin Panel
   - Quick Commit
   - Initialize Repository

## Extension Structure

```
Auxin-SketchUp-Extension/
├── auxin.rb                    # Extension loader
├── auxin/
│   ├── main.rb                 # Core functionality
│   ├── dialogs/
│   │   └── main_dialog.html    # UI dialog
│   └── icons/
│       ├── auxin_small.svg     # 16x16 toolbar icon
│       ├── auxin_large.svg     # 24x24 toolbar icon
│       ├── commit_small.svg    # 16x16 commit icon
│       └── commit_large.svg    # 24x24 commit icon
└── README.md
```

## Icon Conversion

SketchUp requires PNG icons for toolbars. Convert the SVG icons to PNG:

```bash
# Using ImageMagick
convert auxin/icons/auxin_small.svg auxin/icons/auxin_small.png
convert auxin/icons/auxin_large.svg auxin/icons/auxin_large.png
convert auxin/icons/commit_small.svg auxin/icons/commit_small.png
convert auxin/icons/commit_large.svg auxin/icons/commit_large.png
```

Then update `main.rb` to reference `.png` files instead of `.svg`.

## Configuration

### Adjusting CLI Timeout

Edit `auxin/main.rb` and modify the `CLI_TIMEOUT` constant:

```ruby
CLI_TIMEOUT = 60  # seconds (default: 30)
```

### Custom Auxin CLI Path

If your Auxin CLI is not in the system PATH, you can modify the `execute` method in the CLI module to use an absolute path.

## Troubleshooting

### "Auxin CLI not found"

Ensure the Auxin CLI is installed and in your system PATH:

```bash
# Check if auxin is available
which auxin
auxin --version
```

### Dialog Not Opening

1. Check SketchUp's Ruby Console for errors (Window > Ruby Console)
2. Ensure the extension is enabled in Window > Extension Manager
3. Try restarting SketchUp

### Commits Failing

1. Ensure your model is saved before committing
2. Check that the repository is initialized
3. Look at the error message in the Auxin panel
4. Check the Auxin CLI output in Terminal/Command Prompt

### Icons Not Showing

Convert SVG icons to PNG format (see Icon Conversion section above).

## Development

### Testing Changes

1. Edit the Ruby or HTML files
2. In SketchUp Ruby Console, run:
   ```ruby
   load 'auxin/main.rb'
   ```
3. Reopen the Auxin panel to see changes

### Debugging

Enable verbose logging by checking SketchUp's Ruby Console:

```ruby
# In Ruby Console
Auxin::CLI.execute('--verbose', 'status')
```

### Adding New Features

1. Add CLI commands to the `Auxin::CLI` module
2. Add callbacks in `Dialog.register_callbacks`
3. Add JavaScript handlers in `main_dialog.html`
4. Update menus/toolbar in `UI_Integration.setup`

## API Reference

### Ruby API

```ruby
# Execute CLI command
Auxin::CLI.execute('status', '--path', '/path/to/project')

# Get model metadata
metadata = Auxin::ModelMetadata.extract

# Show dialog
Auxin::Dialog.show

# Check if path is repository
Auxin::CLI.is_repo?('/path/to/project')
```

### JavaScript Callbacks

```javascript
// Callbacks from Ruby
onModelPath(path)           // Receives model file path
onStatus(status)            // Receives repository status
onMetadata(metadata)        // Receives model metadata
onHistory(commits)          // Receives commit history
onInitSuccess()             // Repository initialized
onCommitSuccess(output)     // Commit created
onRestoreSuccess()          // Model restored
onError(message)            // Error occurred
```

## License

MIT License - See the main Auxin repository for full license text.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test in SketchUp
5. Submit a pull request

## Support

- **Issues**: https://github.com/jbacus/auxin/issues
- **Documentation**: See main Auxin repository

## Credits

- Auxin Project Contributors
- Powered by Oxen.ai
