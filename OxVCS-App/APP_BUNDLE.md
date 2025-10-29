# OxVCS App Bundle Creation

This document explains how to build a double-clickable macOS application bundle (.app) for OxVCS.

## Overview

The OxVCS application is built using Swift Package Manager and packaged as a macOS `.app` bundle for easy distribution and use. Users can double-click the app in their Applications folder to launch the full GUI interface.

## Building the App Bundle

### Automated Build (Recommended)

The easiest way to build and install the app is using the main installer script:

```bash
cd /path/to/oxen-vcs-logic
./install.sh
```

This will:
1. Build the Swift executable in release mode
2. Create the OxVCS.app bundle structure
3. Install the app to `/Applications/OxVCS.app`

### Manual Build

If you want to build just the app bundle without installing:

```bash
cd OxVCS-App

# Build the executable
swift build -c release

# Create the app bundle
./create-app-bundle.sh
```

The app bundle will be created at `OxVCS-App/OxVCS.app`.

### Skip App Build

To install only the CLI tools and daemon without the GUI app:

```bash
./install.sh --skip-app
```

## App Bundle Structure

The created app bundle follows the standard macOS application structure:

```
OxVCS.app/
├── Contents/
│   ├── Info.plist           # App metadata and configuration
│   ├── PkgInfo              # Legacy type/creator codes
│   ├── MacOS/
│   │   └── OxVCS            # Main executable
│   └── Resources/
│       └── (icons, assets)  # Optional resources
```

### Info.plist

The Info.plist contains important metadata:

- **Bundle Identifier**: `com.oxen.oxvcs`
- **Display Name**: "OxVCS for Logic Pro"
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
cp -R OxVCS-App/OxVCS.app /Applications/

# Make it executable
chmod -R 755 /Applications/OxVCS.app
```

## Running the App

### From Applications Folder

1. Open Finder
2. Navigate to Applications
3. Double-click "OxVCS"

### From Terminal

```bash
open /Applications/OxVCS.app
```

Or run directly:

```bash
/Applications/OxVCS.app/Contents/MacOS/OxVCS
```

## First Launch

On first launch, macOS may show a security warning:

1. **"OxVCS cannot be opened because it is from an unidentified developer"**
   - Open System Settings
   - Go to Privacy & Security
   - Click "Open Anyway" next to the OxVCS warning

2. **Daemon Permission Required**
   - The app may request permission to run the background daemon
   - Go to System Settings → General → Login Items & Extensions
   - Enable "Oxen VCS Daemon"

## Code Signing (Optional)

For distribution, you should code sign the app:

```bash
# Sign with your Developer ID
codesign --deep --force --sign "Developer ID Application: Your Name" OxVCS.app

# Verify signature
codesign --verify --verbose OxVCS.app

# Check requirements
codesign --display --requirements - OxVCS.app
```

## Notarization (Optional)

For macOS Gatekeeper compatibility:

```bash
# Create a zip for notarization
ditto -c -k --keepParent OxVCS.app OxVCS.zip

# Submit for notarization
xcrun notarytool submit OxVCS.zip --keychain-profile "AC_PASSWORD"

# Staple the notarization ticket
xcrun stapler staple OxVCS.app
```

## Troubleshooting

### "App is damaged and can't be opened"

This usually means quarantine attributes need to be cleared:

```bash
xattr -cr /Applications/OxVCS.app
```

### App doesn't launch

Check the executable has proper permissions:

```bash
chmod +x /Applications/OxVCS.app/Contents/MacOS/OxVCS
```

Check the Info.plist is valid:

```bash
plutil -lint /Applications/OxVCS.app/Contents/Info.plist
```

### Daemon not connecting

Ensure the daemon is installed and running:

```bash
# Check if daemon binary exists
ls -l /usr/local/bin/oxvcs-daemon

# Check daemon status
oxvcs-daemon --status

# View daemon logs
tail -f /tmp/com.oxen.logic.daemon.stdout
```

## Uninstallation

To remove the app:

```bash
# Via installer script
./install.sh --uninstall

# Or manually
rm -rf /Applications/OxVCS.app
```

## Development

### Rebuilding After Code Changes

```bash
cd OxVCS-App

# Clean previous build
swift package clean

# Rebuild
swift build -c release

# Recreate bundle
./create-app-bundle.sh

# Test locally
open OxVCS.app
```

### Debugging

Run the app with console output visible:

```bash
/Applications/OxVCS.app/Contents/MacOS/OxVCS
```

Or attach lldb:

```bash
lldb /Applications/OxVCS.app/Contents/MacOS/OxVCS
(lldb) run
```

## Related Documentation

- [OxVCS-App README](README.md) - Complete app documentation
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
- OxVCS-LaunchAgent (background daemon)
- oxenvcs-cli (command-line tool)
- macOS 14.0+ (Sonoma or later)

### Bundle Metadata

| Property | Value |
|----------|-------|
| Bundle ID | com.oxen.oxvcs |
| Version | 1.0.0 |
| Min macOS | 14.0 |
| Category | Developer Tools |
| Executable | OxVCS |

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
