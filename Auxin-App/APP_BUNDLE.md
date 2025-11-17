# Auxin App Bundle Creation

This document explains how to build a double-clickable macOS application bundle (.app) for Auxin.

## Overview

The Auxin application is built using Swift Package Manager and packaged as a macOS `.app` bundle for easy distribution and use. Users can double-click the app in their Applications folder to launch the full GUI interface.

## Building the App Bundle

### Automated Build (Recommended)

The easiest way to build and install the app is using the main installer script:

```bash
cd /path/to/auxin
./install.sh
```

This will:
1. Build the Swift executable in release mode
2. Create the Auxin.app bundle structure
3. Install the app to `/Applications/Auxin.app`

### Manual Build

If you want to build just the app bundle without installing:

```bash
cd Auxin-App

# Build the executable
swift build -c release

# Create the app bundle
./create-app-bundle.sh
```

The app bundle will be created at `Auxin-App/Auxin.app`.

### Skip App Build

To install only the CLI tools and daemon without the GUI app:

```bash
./install.sh --skip-app
```

## App Bundle Structure

The created app bundle follows the standard macOS application structure:

```
Auxin.app/
├── Contents/
│   ├── Info.plist           # App metadata and configuration
│   ├── PkgInfo              # Legacy type/creator codes
│   ├── MacOS/
│   │   └── Auxin            # Main executable
│   └── Resources/
│       └── (icons, assets)  # Optional resources
```

### Info.plist

The Info.plist contains important metadata:

- **Bundle Identifier**: `com.oxen.oxvcs`
- **Display Name**: "Auxin for Logic Pro"
- **Minimum macOS Version**: 14.0 (Sonoma)
- **App Category**: Developer Tools
- **High Resolution Capable**: Yes

## Installation

### Via Installer Script

The install.sh script automatically copies the app to `/Applications/`:

```bash
./install.sh
```

### Manual Installation

To manually install the app:

```bash
# After building the app bundle
cp -R Auxin-App/Auxin.app /Applications/

# Make it executable
chmod -R 755 /Applications/Auxin.app
```

## Running the App

### From Applications Folder

1. Open Finder
2. Navigate to Applications
3. Double-click "Auxin"

### From Terminal

```bash
open /Applications/Auxin.app
```

Or run directly:

```bash
/Applications/Auxin.app/Contents/MacOS/Auxin
```

## First Launch

On first launch, macOS may show a security warning:

1. **"Auxin cannot be opened because it is from an unidentified developer"**
   - Open System Settings
   - Go to Privacy & Security
   - Click "Open Anyway" next to the Auxin warning

2. **Daemon Permission Required**
   - The app may request permission to run the background daemon
   - Go to System Settings → General → Login Items & Extensions
   - Enable "Oxen VCS Daemon"

## Code Signing (Optional)

For distribution, you should code sign the app:

```bash
# Sign with your Developer ID
codesign --deep --force --sign "Developer ID Application: Your Name" Auxin.app

# Verify signature
codesign --verify --verbose Auxin.app

# Check requirements
codesign --display --requirements - Auxin.app
```

## Notarization (Optional)

For macOS Gatekeeper compatibility:

```bash
# Create a zip for notarization
ditto -c -k --keepParent Auxin.app Auxin.zip

# Submit for notarization
xcrun notarytool submit Auxin.zip --keychain-profile "AC_PASSWORD"

# Staple the notarization ticket
xcrun stapler staple Auxin.app
```

## Troubleshooting

### "App is damaged and can't be opened"

This usually means quarantine attributes need to be cleared:

```bash
xattr -cr /Applications/Auxin.app
```

### App doesn't launch

Check the executable has proper permissions:

```bash
chmod +x /Applications/Auxin.app/Contents/MacOS/Auxin
```

Check the Info.plist is valid:

```bash
plutil -lint /Applications/Auxin.app/Contents/Info.plist
```

### Daemon not connecting

Ensure the daemon is installed and running:

```bash
# Check if daemon binary exists
ls -l /usr/local/bin/auxin-daemon

# Check daemon status
auxin-daemon --status

# View daemon logs
tail -f /tmp/com.auxin.daemon.stdout
```

## Uninstallation

To remove the app:

```bash
# Via installer script
./install.sh --uninstall

# Or manually
rm -rf /Applications/Auxin.app
```

## Development

### Rebuilding After Code Changes

```bash
cd Auxin-App

# Clean previous build
swift package clean

# Rebuild
swift build -c release

# Recreate bundle
./create-app-bundle.sh

# Test locally
open Auxin.app
```

### Debugging

Run the app with console output visible:

```bash
/Applications/Auxin.app/Contents/MacOS/Auxin
```

Or attach lldb:

```bash
lldb /Applications/Auxin.app/Contents/MacOS/Auxin
(lldb) run
```

## Related Documentation

- [Auxin-App README](README.md) - Complete app documentation
- [Installation Guide](../INSTALL.md) - Full installation instructions

## Technical Details

### Build System

- **Swift Package Manager**: Compiles the executable
- **create-app-bundle.sh**: Packages executable into .app structure
- **install.sh**: Orchestrates full build and installation

### Dependencies

**Build-time**:
- Swift 5.9+
- macOS 14.0+ SDK

**Runtime**:
- Auxin-LaunchAgent (background daemon)
- auxin (command-line tool)
- macOS 14.0+ (Sonoma or later)

### Bundle Metadata

| Property | Value |
|----------|-------|
| Bundle ID | com.oxen.oxvcs |
| Version | 1.0.0 |
| Min macOS | 14.0 |
| Category | Developer Tools |
| Executable | Auxin |

## Future Enhancements

- [ ] Custom app icon (.icns file)
- [ ] Dock menu with quick actions
- [ ] Sparkle framework for auto-updates
- [ ] DMG installer package
- [ ] Sandbox entitlements for App Store
- [ ] Help menu integration
- [ ] Localization support

## License

MIT License - See [LICENSE](../LICENSE) for details.
