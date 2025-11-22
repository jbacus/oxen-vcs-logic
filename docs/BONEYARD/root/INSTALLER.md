# Auxin Installer Documentation

This document describes how to build and distribute the Auxin macOS installer.

## Overview

The Auxin installer is a native macOS `.pkg` package that installs all Auxin components:

- **Auxin CLI** - Command-line interface (required)
- **Auxin Daemon** - Background service for automatic commits (required)
- **Auxin Application** - Native macOS GUI app (recommended)
- **Auxin Server** - Optional collaboration server for LAN teams (optional)

## Prerequisites

### For Building the Installer

- macOS 14.0 or later
- Xcode Command Line Tools (`xcode-select --install`)
- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- Swift 5.9+ (included with Xcode)

### For End Users

- macOS 14.0 or later
- Oxen CLI (`pip3 install oxen-ai`)

## Building the Installer

### Quick Build

Build the complete installer package with default settings:

```bash
./build-installer.sh
```

This will:
1. Build all components (CLI, Daemon, App, Server)
2. Create component packages
3. Generate installer resources
4. Build the final product package
5. Output: `installer-build/Auxin-1.0.0.pkg`

### Custom Build Options

Specify a custom version:

```bash
./build-installer.sh --version 1.2.3
```

Specify a custom output directory:

```bash
./build-installer.sh --output /path/to/output
```

Both options combined:

```bash
./build-installer.sh --version 1.2.3 --output ~/Desktop/auxin-installer
```

### Build Output

The build process creates the following structure:

```
installer-build/
├── Auxin-1.0.0.pkg          # Final installer package
├── build/                    # Component payloads
│   ├── cli/                 # CLI files
│   ├── daemon/              # Daemon files
│   ├── app/                 # App bundle
│   └── server/              # Server files
├── packages/                 # Component packages
│   ├── auxin-cli.pkg
│   ├── auxin-daemon.pkg
│   ├── auxin-app.pkg
│   └── auxin-server.pkg
├── resources/               # Installer resources
│   ├── welcome.html
│   ├── readme.html
│   ├── license.txt
│   └── background.png
├── scripts/                 # Postinstall scripts
│   ├── cli/
│   ├── daemon/
│   ├── app/
│   └── server/
└── distribution.xml         # Product configuration
```

## Creating a DMG for Distribution

After building the installer, create a distributable DMG:

```bash
./create-dmg.sh
```

This creates a compressed disk image containing:
- The installer package
- README with instructions
- LICENSE file
- Symbolic link to /Applications folder

### DMG Options

Specify version:

```bash
./create-dmg.sh --version 1.2.3
```

Specify installer path:

```bash
./create-dmg.sh --installer /path/to/Auxin-1.0.0.pkg
```

Custom output directory:

```bash
./create-dmg.sh --output ~/Desktop
```

### DMG Output

- File: `Auxin-1.2.3.dmg`
- Format: UDZO (compressed)
- Volume name: "Auxin 1.2.3"

## Installer Components

### 1. CLI Package (`com.auxin.cli`)

**What it installs:**
- `/usr/local/bin/auxin` - CLI binary

**Postinstall:**
- Verifies installation
- Shows version information

**Required:** Yes

### 2. Daemon Package (`com.auxin.daemon`)

**What it installs:**
- `/usr/local/bin/auxin-daemon` - Daemon binary
- `~/Library/LaunchAgents/com.auxin.daemon.plist` - LaunchAgent configuration

**Postinstall:**
- Copies plist to user's LaunchAgents directory
- Sets proper permissions
- Provides instructions for enabling in System Settings

**Required:** Yes

**Note:** The daemon requires user approval in System Settings:
1. Open System Settings
2. Go to General → Login Items & Extensions
3. Enable "Auxin Daemon"

### 3. App Package (`com.auxin.app`)

**What it installs:**
- `/Applications/Auxin.app` - Native macOS application

**Postinstall:**
- Verifies app bundle installation

**Required:** No (but recommended)

### 4. Server Package (`com.auxin.server`)

**What it installs:**
- `/usr/local/bin/auxin-server` - Server binary
- `/var/oxen/data` - Data directory
- `~/.config/auxin-server/.env` - Configuration file

**Postinstall:**
- Creates data directories
- Generates secure auth token
- Creates default configuration
- Provides startup instructions

**Required:** No (optional for LAN collaboration)

## Customizing the Installer

### Modifying Welcome Screen

Edit `build-installer.sh` and modify the `create_resources()` function:

```bash
cat > "$RESOURCES_DIR/welcome.html" << 'EOF'
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <style>
        body { font-family: -apple-system; }
        /* Your custom styles */
    </style>
</head>
<body>
    <!-- Your custom content -->
</body>
</html>
EOF
```

### Adding Custom Postinstall Actions

Edit the postinstall scripts in `build-installer.sh`:

```bash
cat > "$SCRIPTS_DIR/component/postinstall" << 'EOF'
#!/bin/bash
# Your custom postinstall script
EOF
```

### Customizing Package Choices

Edit `distribution.xml` configuration in `build-installer.sh`:

```xml
<choice id="auxin.component"
        visible="true"
        title="Component Name"
        description="Component description"
        start_selected="true">
    <pkg-ref id="com.auxin.component"/>
</choice>
```

## Code Signing and Notarization

### Code Signing (Recommended for Distribution)

Sign the installer with your Apple Developer ID:

```bash
# Sign individual packages first
productsign --sign "Developer ID Installer: Your Name (TEAMID)" \
    installer-build/packages/auxin-cli.pkg \
    installer-build/packages/auxin-cli-signed.pkg

# Then sign the final product
productsign --sign "Developer ID Installer: Your Name (TEAMID)" \
    installer-build/Auxin-1.0.0.pkg \
    installer-build/Auxin-1.0.0-signed.pkg
```

### Notarization (Required for Distribution)

Notarize the signed installer with Apple:

```bash
# Submit for notarization
xcrun notarytool submit Auxin-1.0.0-signed.pkg \
    --apple-id your-email@example.com \
    --team-id TEAMID \
    --password app-specific-password \
    --wait

# Staple the notarization ticket
xcrun stapler staple Auxin-1.0.0-signed.pkg
```

### Creating an App-Specific Password

1. Go to https://appleid.apple.com
2. Sign in with your Apple ID
3. Go to Security section
4. Click "Generate Password" under App-Specific Passwords
5. Use this password with `notarytool`

## Testing the Installer

### Basic Testing

1. Build the installer
2. Install on a test machine:
   ```bash
   open installer-build/Auxin-1.0.0.pkg
   ```
3. Verify all components are installed
4. Test functionality
5. Uninstall and verify cleanup

### Automated Testing

Create a test script:

```bash
#!/bin/bash
# test-installer.sh

# Install
sudo installer -pkg installer-build/Auxin-1.0.0.pkg -target /

# Verify binaries
test -x /usr/local/bin/auxin || exit 1
test -x /usr/local/bin/auxin-daemon || exit 1
test -d /Applications/Auxin.app || exit 1

# Test CLI
auxin --version || exit 1

# Verify daemon plist
test -f ~/Library/LaunchAgents/com.auxin.daemon.plist || exit 1

echo "✓ All tests passed"
```

### Testing the DMG

1. Build the DMG
2. Mount it:
   ```bash
   open Auxin-1.0.0.dmg
   ```
3. Run the installer from the mounted volume
4. Verify installation

## Uninstalling

Users can uninstall Auxin using the provided uninstall script:

```bash
./uninstall.sh
```

Options:
- `--yes` - Skip confirmation prompt
- `--keep-data` - Preserve repository data

The uninstaller removes:
- All binaries
- Application bundle
- LaunchAgent plists
- Configuration files
- Log files
- Server data directory
- Shell completions

**Note:** Repository data (`.oxen` directories) are preserved by default.

## Distribution Checklist

Before distributing the installer:

- [ ] Build with release configuration
- [ ] Test installation on clean macOS system
- [ ] Test all components work correctly
- [ ] Verify daemon can be enabled in System Settings
- [ ] Test server installation (if included)
- [ ] Code sign the installer
- [ ] Notarize with Apple
- [ ] Staple notarization ticket
- [ ] Create DMG for distribution
- [ ] Test DMG on different macOS versions
- [ ] Write release notes
- [ ] Update CHANGELOG.md
- [ ] Create GitHub release
- [ ] Upload DMG to release
- [ ] Update documentation

## Troubleshooting

### Build Fails: "pkgbuild not found"

Install Xcode Command Line Tools:
```bash
xcode-select --install
```

### Build Fails: Swift or Rust not found

- **Rust:** Install from https://rustup.rs
- **Swift:** Install Xcode from App Store

### Installer Fails: "Package is damaged"

This usually means the package needs to be signed and notarized. Either:
1. Sign and notarize the package
2. Or have users right-click → Open to bypass Gatekeeper

### Daemon Not Loading

The daemon requires user approval in System Settings:
1. System Settings → General → Login Items & Extensions
2. Enable "Auxin Daemon"

On older macOS versions, use:
```bash
launchctl load ~/Library/LaunchAgents/com.auxin.daemon.plist
```

### Server Won't Start

Check configuration:
```bash
cat ~/.config/auxin-server/.env
```

Verify data directory exists and has proper permissions:
```bash
ls -la /var/oxen/data
```

Check logs:
```bash
tail -f ~/Library/Logs/auxin-server.log
```

## Advanced Topics

### Custom Installation Locations

To install to custom locations, modify the `--install-location` parameter in `pkgbuild` commands:

```bash
pkgbuild --root "$BUILD_DIR/cli" \
         --install-location "/opt/auxin" \  # Custom location
         "$PACKAGES_DIR/auxin-cli.pkg"
```

### Multi-User Installation

The installer currently installs binaries system-wide but configurations per-user. To change this:

1. Modify install locations in `build-installer.sh`
2. Update postinstall scripts to handle multi-user scenarios
3. Consider using `/Library/LaunchDaemons` instead of `~/Library/LaunchAgents`

### Creating an Installer Bundle

For a more app-like installer experience, wrap the PKG in an application:

1. Create an app bundle with AppleScript or Swift
2. Embed the `.pkg` file as a resource
3. Use `installer` command to run it programmatically

## Continuous Integration

### GitHub Actions Example

```yaml
name: Build Installer
on: [push, pull_request]

jobs:
  build:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build Installer
        run: ./build-installer.sh --version ${{ github.ref_name }}

      - name: Create DMG
        run: ./create-dmg.sh --version ${{ github.ref_name }}

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: installer
          path: |
            installer-build/Auxin-*.pkg
            Auxin-*.dmg
```

## Release Workflow

1. **Update Version Numbers**
   - `Auxin-CLI-Wrapper/Cargo.toml`
   - `auxin-server/Cargo.toml`
   - `Auxin-App/Sources/Info.plist`
   - `CHANGELOG.md`

2. **Build Release**
   ```bash
   ./build-installer.sh --version X.Y.Z
   ./create-dmg.sh --version X.Y.Z
   ```

3. **Sign and Notarize**
   ```bash
   productsign --sign "Developer ID" ...
   xcrun notarytool submit ...
   xcrun stapler staple ...
   ```

4. **Test**
   - Clean macOS installation
   - Verify all components
   - Test upgrade from previous version

5. **Release**
   - Create Git tag: `git tag vX.Y.Z`
   - Push tag: `git push origin vX.Y.Z`
   - Create GitHub Release
   - Upload DMG
   - Update documentation

## Support

For issues with the installer:
- GitHub Issues: https://github.com/jbacus/auxin/issues
- Documentation: https://github.com/jbacus/auxin

## License

The installer scripts are part of the Auxin project and are licensed under the MIT License.

---

*Last Updated: 2025-11-21*
