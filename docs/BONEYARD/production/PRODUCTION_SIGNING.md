# Production Code Signing and Notarization Guide

This guide covers the complete process for preparing the Auxin installer for production distribution on macOS.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Obtaining Certificates](#obtaining-certificates)
3. [Code Signing Applications](#code-signing-applications)
4. [Code Signing Packages](#code-signing-packages)
5. [Notarization](#notarization)
6. [Verification](#verification)
7. [Automated Production Build](#automated-production-build)
8. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### 1. Apple Developer Program Membership

**Required:** Yes (costs $99/year)

**Sign up at:** https://developer.apple.com/programs/

**Why needed:**
- Required for Developer ID certificates
- Required for notarization
- Establishes trust with macOS Gatekeeper

### 2. Xcode and Command Line Tools

```bash
# Install Xcode from App Store
xcode-select --install

# Verify installation
xcodebuild -version
```

### 3. Check Your Team ID

After joining Apple Developer Program:

1. Go to https://developer.apple.com/account
2. Click on "Membership" in sidebar
3. Note your **Team ID** (10-character alphanumeric code like "AB12CD34EF")

You'll need this for notarization.

---

## Obtaining Certificates

### Step 1: Request Developer ID Certificates

You need **two** certificates:

1. **Developer ID Application** - For signing binaries and apps
2. **Developer ID Installer** - For signing installer packages

### Option A: Request via Xcode (Recommended)

1. Open **Xcode**
2. Go to **Xcode → Settings... → Accounts**
3. Add your Apple ID (if not already added)
4. Select your account, click **Manage Certificates...**
5. Click **+** button, select:
   - **Developer ID Application**
   - **Developer ID Installer**
6. Certificates will be downloaded and installed in Keychain

### Option B: Request via Developer Website

1. Go to https://developer.apple.com/account/resources/certificates/list
2. Click **+** to create new certificate
3. Select **Developer ID Application** → Continue
4. Follow instructions to create Certificate Signing Request (CSR):
   ```
   Applications → Utilities → Keychain Access
   Keychain Access → Certificate Assistant → Request a Certificate from a Certificate Authority

   Fill in:
   - User Email: your email
   - Common Name: Your Name
   - CA Email: (leave empty)
   - Select: "Saved to disk"
   ```
5. Upload CSR, download certificate
6. Double-click certificate to install in Keychain
7. Repeat for **Developer ID Installer** certificate

### Step 2: Verify Certificates in Keychain

```bash
# List application signing certificates
security find-identity -v -p codesigning

# Should see entries like:
# 1) HASH "Developer ID Application: Your Name (TEAMID)"
# 2) HASH "Developer ID Installer: Your Name (TEAMID)"
```

Copy the exact certificate name (including your name and Team ID) for use in signing commands.

---

## Code Signing Applications

Before signing packages, you must sign individual binaries and app bundles.

### Step 1: Sign Rust Binaries

```bash
# Sign the CLI binary
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
         --options runtime \
         --timestamp \
         Auxin-CLI-Wrapper/target/release/auxin

# Sign the daemon binary
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
         --options runtime \
         --timestamp \
         Auxin-LaunchAgent/.build/release/auxin-daemon

# Sign the server binary
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
         --options runtime \
         --timestamp \
         auxin-server/target/release/auxin-server

# Verify signatures
codesign --verify --verbose Auxin-CLI-Wrapper/target/release/auxin
codesign --verify --verbose Auxin-LaunchAgent/.build/release/auxin-daemon
codesign --verify --verbose auxin-server/target/release/auxin-server
```

**Important flags:**
- `--options runtime` - Enables hardened runtime (required for notarization)
- `--timestamp` - Adds secure timestamp (required for notarization)

### Step 2: Sign the App Bundle

```bash
# Sign the app bundle (includes entitlements if needed)
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
         --options runtime \
         --timestamp \
         --deep \
         Auxin-App/Auxin.app

# Verify
codesign --verify --verbose Auxin-App/Auxin.app
spctl --assess --verbose Auxin-App/Auxin.app
```

The `--deep` flag ensures all nested code is signed.

### Step 3: Add Entitlements (If Needed)

If your app needs special permissions, create an entitlements file:

```xml
<!-- Auxin-App/Resources/Entitlements.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Allow network connections -->
    <key>com.apple.security.network.client</key>
    <true/>

    <!-- Allow file system access -->
    <key>com.apple.security.files.user-selected.read-write</key>
    <true/>

    <!-- Disable library validation for Swift -->
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
</dict>
</plist>
```

Then sign with entitlements:

```bash
codesign --sign "Developer ID Application: Your Name (TEAMID)" \
         --options runtime \
         --timestamp \
         --entitlements Auxin-App/Resources/Entitlements.plist \
         Auxin-App/Auxin.app
```

---

## Code Signing Packages

### Step 1: Build Unsigned Installer

```bash
# Build the installer first
./build-installer.sh --version 1.0.0

# This creates: installer-build/Auxin-1.0.0.pkg
```

### Step 2: Sign the Product Package

```bash
# Sign the final installer package
productsign --sign "Developer ID Installer: Your Name (TEAMID)" \
            installer-build/Auxin-1.0.0.pkg \
            installer-build/Auxin-1.0.0-signed.pkg

# Verify signature
pkgutil --check-signature installer-build/Auxin-1.0.0-signed.pkg

# Should show:
#   Status: signed by a developer certificate issued by Apple
#   Certificate Chain:
#     1. Developer ID Installer: Your Name (TEAMID)
#     2. Developer ID Certification Authority
#     3. Apple Root CA
```

**Note:** You don't need to sign individual component packages separately. The `productsign` command on the final product package is sufficient.

---

## Notarization

Notarization is Apple's automated security check. It's required for distribution outside the Mac App Store.

### Step 1: Create App-Specific Password

1. Go to https://appleid.apple.com
2. Sign in with your Apple ID
3. Go to **Security** section
4. Under **App-Specific Passwords**, click **Generate Password**
5. Label it "Notarization" or "Auxin Notarization"
6. Copy the generated password (format: `xxxx-xxxx-xxxx-xxxx`)
7. Store it securely

### Step 2: Store Credentials in Keychain (Recommended)

This avoids typing your password repeatedly:

```bash
# Store credentials
xcrun notarytool store-credentials "auxin-notary" \
    --apple-id "your-email@example.com" \
    --team-id "YOUR-TEAM-ID" \
    --password "xxxx-xxxx-xxxx-xxxx"

# This creates a keychain profile named "auxin-notary"
```

### Step 3: Submit for Notarization

```bash
# Submit the signed package
xcrun notarytool submit installer-build/Auxin-1.0.0-signed.pkg \
    --keychain-profile "auxin-notary" \
    --wait

# The --wait flag makes it wait for results (typically 5-15 minutes)
```

**Output will show:**
```
Conducting pre-submission checks for Auxin-1.0.0-signed.pkg...
Submission ID received
  id: 12345678-1234-1234-1234-123456789012
Successfully uploaded file
  id: 12345678-1234-1234-1234-123456789012
  path: installer-build/Auxin-1.0.0-signed.pkg

Waiting for processing to complete...
Current status: In Progress...
Current status: Accepted

Processing complete
  id: 12345678-1234-1234-1234-123456789012
  status: Accepted
```

### Step 4: Check Notarization Status

If you didn't use `--wait`, check status later:

```bash
# Check status
xcrun notarytool info SUBMISSION-ID \
    --keychain-profile "auxin-notary"

# View detailed log (if rejected)
xcrun notarytool log SUBMISSION-ID \
    --keychain-profile "auxin-notary"
```

### Step 5: Staple the Notarization

After acceptance, "staple" the notarization ticket to the package:

```bash
# Staple the ticket
xcrun stapler staple installer-build/Auxin-1.0.0-signed.pkg

# Verify stapling
xcrun stapler validate installer-build/Auxin-1.0.0-signed.pkg

# Should show: "The validate action worked!"
```

**Why staple?**
- Embeds the notarization ticket in the package
- Allows offline installation
- Package works even if Apple's servers are down

---

## Verification

### Step 1: Verify Package Signature

```bash
# Check signature
pkgutil --check-signature installer-build/Auxin-1.0.0-signed.pkg

# Verify with spctl
spctl --assess --type install --verbose installer-build/Auxin-1.0.0-signed.pkg

# Should show: "accepted"
```

### Step 2: Test Installation

```bash
# Test on a clean Mac (or VM)
# Try to install normally
open installer-build/Auxin-1.0.0-signed.pkg

# Should install without Gatekeeper warnings
```

### Step 3: Verify Installed Binaries

After installation:

```bash
# Check CLI signature
codesign --verify --verbose /usr/local/bin/auxin
spctl --assess --type execute --verbose /usr/local/bin/auxin

# Check app signature
codesign --verify --verbose /Applications/Auxin.app
spctl --assess --verbose /Applications/Auxin.app
```

---

## Automated Production Build

Create a production build script that handles signing and notarization automatically.

### Create `build-production.sh`

```bash
#!/bin/bash
#
# Auxin Production Build Script
# Builds, signs, and notarizes the installer
#
set -e

# Configuration
VERSION="${1:-1.0.0}"
APP_CERT="Developer ID Application: Your Name (TEAMID)"
INSTALLER_CERT="Developer ID Installer: Your Name (TEAMID)"
NOTARY_PROFILE="auxin-notary"

echo "Building Auxin $VERSION for production..."
echo ""

# Step 1: Build components
echo "Step 1: Building components..."
cd Auxin-CLI-Wrapper && cargo build --release && cd ..
cd Auxin-LaunchAgent && swift build -c release && cd ..
cd Auxin-App && swift build -c release && ./create-app-bundle.sh && cd ..
cd auxin-server && cargo build --release && cd ..

# Step 2: Sign binaries
echo ""
echo "Step 2: Code signing binaries..."
codesign --sign "$APP_CERT" --options runtime --timestamp \
    Auxin-CLI-Wrapper/target/release/auxin
codesign --sign "$APP_CERT" --options runtime --timestamp \
    Auxin-LaunchAgent/.build/release/auxin-daemon
codesign --sign "$APP_CERT" --options runtime --timestamp \
    auxin-server/target/release/auxin-server

# Step 3: Sign app bundle
echo ""
echo "Step 3: Code signing app bundle..."
codesign --sign "$APP_CERT" --options runtime --timestamp --deep \
    Auxin-App/Auxin.app

# Step 4: Verify signatures
echo ""
echo "Step 4: Verifying signatures..."
codesign --verify --verbose Auxin-CLI-Wrapper/target/release/auxin
codesign --verify --verbose Auxin-LaunchAgent/.build/release/auxin-daemon
codesign --verify --verbose auxin-server/target/release/auxin-server
codesign --verify --verbose Auxin-App/Auxin.app

# Step 5: Build installer package
echo ""
echo "Step 5: Building installer package..."
./build-installer.sh --version "$VERSION"

# Step 6: Sign installer
echo ""
echo "Step 6: Signing installer package..."
productsign --sign "$INSTALLER_CERT" \
    "installer-build/Auxin-$VERSION.pkg" \
    "installer-build/Auxin-$VERSION-signed.pkg"

pkgutil --check-signature "installer-build/Auxin-$VERSION-signed.pkg"

# Step 7: Notarize
echo ""
echo "Step 7: Submitting for notarization..."
echo "(This may take 5-15 minutes)"
xcrun notarytool submit "installer-build/Auxin-$VERSION-signed.pkg" \
    --keychain-profile "$NOTARY_PROFILE" \
    --wait

# Step 8: Staple
echo ""
echo "Step 8: Stapling notarization..."
xcrun stapler staple "installer-build/Auxin-$VERSION-signed.pkg"
xcrun stapler validate "installer-build/Auxin-$VERSION-signed.pkg"

# Step 9: Create DMG
echo ""
echo "Step 9: Creating distribution DMG..."
./create-dmg.sh --version "$VERSION" \
    --installer "installer-build/Auxin-$VERSION-signed.pkg"

# Step 10: Sign and notarize DMG
echo ""
echo "Step 10: Signing and notarizing DMG..."
codesign --sign "$APP_CERT" --timestamp "Auxin-$VERSION.dmg"

xcrun notarytool submit "Auxin-$VERSION.dmg" \
    --keychain-profile "$NOTARY_PROFILE" \
    --wait

xcrun stapler staple "Auxin-$VERSION.dmg"

# Done!
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✓ Production build complete!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Signed and notarized files:"
echo "  • installer-build/Auxin-$VERSION-signed.pkg"
echo "  • Auxin-$VERSION.dmg"
echo ""
echo "Ready for distribution!"
echo ""
```

Make it executable:

```bash
chmod +x build-production.sh
```

### Usage

```bash
# Build production release
./build-production.sh 1.0.0

# Automated process:
# 1. Builds all components
# 2. Signs binaries and app
# 3. Builds installer
# 4. Signs installer
# 5. Notarizes installer
# 6. Staples notarization
# 7. Creates DMG
# 8. Signs and notarizes DMG
# 9. Ready for release!
```

---

## Troubleshooting

### Common Issues

#### 1. Certificate Not Found

**Error:**
```
error: The specified item could not be found in the keychain.
```

**Solution:**
```bash
# List available certificates
security find-identity -v -p codesigning

# Use exact name from output, including Team ID
codesign --sign "Developer ID Application: Your Name (TEAMID)" ...
```

#### 2. Notarization Rejected

**Error:**
```
status: Invalid
```

**Solution:**
```bash
# Get detailed rejection reason
xcrun notarytool log SUBMISSION-ID --keychain-profile "auxin-notary"

# Common reasons:
# - Missing --options runtime flag
# - Missing --timestamp flag
# - Unsigned nested binaries (use --deep for apps)
```

#### 3. "Developer cannot be verified" Error

**Problem:** User sees this when opening installer

**Solution:**
- Ensure package is signed with Developer ID Installer certificate
- Ensure package is notarized
- Ensure notarization is stapled
- Verify with: `spctl --assess --type install --verbose installer.pkg`

#### 4. Hardened Runtime Issues

**Error:**
```
The executable does not have the hardened runtime enabled.
```

**Solution:**
Add `--options runtime` to all codesign commands:
```bash
codesign --sign "..." --options runtime --timestamp binary
```

#### 5. Entitlements Required

Some features require entitlements (network access, file access, etc.)

**Solution:**
Create entitlements.plist and sign with:
```bash
codesign --sign "..." --entitlements entitlements.plist --options runtime app.app
```

### Verification Commands Cheat Sheet

```bash
# Verify binary signature
codesign --verify --verbose /path/to/binary

# Display signature info
codesign --display --verbose=4 /path/to/binary

# Check Gatekeeper assessment
spctl --assess --type execute --verbose /path/to/binary

# Check package signature
pkgutil --check-signature installer.pkg

# Check notarization stapling
xcrun stapler validate installer.pkg

# View notarization history
xcrun notarytool history --keychain-profile "auxin-notary"
```

---

## Best Practices

### 1. Version Control

**Don't commit:**
- Signed binaries
- App-specific passwords
- Keychain profiles

**Do commit:**
- Build scripts
- Entitlements files
- Documentation

### 2. Security

- Use app-specific passwords (never your main Apple ID password)
- Store credentials in Keychain using `notarytool store-credentials`
- Keep certificates secure
- Rotate app-specific passwords periodically

### 3. Testing

Always test signed builds:
1. Install on clean Mac
2. Verify no Gatekeeper warnings
3. Test all functionality
4. Check binary signatures remain intact

### 4. Automation

- Use CI/CD for production builds (GitHub Actions, etc.)
- Automate signing and notarization
- Store credentials as secrets (not in code)

---

## GitHub Actions Example

For automated production builds:

```yaml
name: Build Production Release
on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: macos-14
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Import Certificates
        env:
          APP_CERT_BASE64: ${{ secrets.APP_CERT_BASE64 }}
          INSTALLER_CERT_BASE64: ${{ secrets.INSTALLER_CERT_BASE64 }}
          CERT_PASSWORD: ${{ secrets.CERT_PASSWORD }}
        run: |
          # Import certificates from secrets
          echo "$APP_CERT_BASE64" | base64 --decode > app_cert.p12
          echo "$INSTALLER_CERT_BASE64" | base64 --decode > installer_cert.p12

          # Import to keychain
          security create-keychain -p actions temp.keychain
          security default-keychain -s temp.keychain
          security unlock-keychain -p actions temp.keychain
          security import app_cert.p12 -k temp.keychain -P "$CERT_PASSWORD" -T /usr/bin/codesign
          security import installer_cert.p12 -k temp.keychain -P "$CERT_PASSWORD" -T /usr/bin/productsign
          security set-key-partition-list -S apple-tool:,apple: -s -k actions temp.keychain

      - name: Setup Notarization
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          TEAM_ID: ${{ secrets.TEAM_ID }}
          NOTARY_PASSWORD: ${{ secrets.NOTARY_PASSWORD }}
        run: |
          xcrun notarytool store-credentials "auxin-notary" \
            --apple-id "$APPLE_ID" \
            --team-id "$TEAM_ID" \
            --password "$NOTARY_PASSWORD"

      - name: Build Production Release
        run: ./build-production.sh ${GITHUB_REF#refs/tags/v}

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            installer-build/Auxin-*-signed.pkg
            Auxin-*.dmg
```

---

## Summary Checklist

Before first release:
- [ ] Join Apple Developer Program ($99/year)
- [ ] Obtain Developer ID Application certificate
- [ ] Obtain Developer ID Installer certificate
- [ ] Create app-specific password
- [ ] Store notarization credentials in keychain
- [ ] Test signing process with development build
- [ ] Test notarization with development build
- [ ] Document your Team ID and certificate names

For each release:
- [ ] Build all components
- [ ] Sign all binaries with hardened runtime
- [ ] Sign app bundle
- [ ] Build installer package
- [ ] Sign installer package
- [ ] Submit for notarization
- [ ] Wait for acceptance
- [ ] Staple notarization
- [ ] Create distribution DMG
- [ ] Test on clean Mac
- [ ] Verify no Gatekeeper warnings
- [ ] Upload to GitHub Releases
- [ ] Update documentation

---

## Resources

- **Apple Developer:** https://developer.apple.com
- **Code Signing Guide:** https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/
- **Notarization Guide:** https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution
- **App-Specific Passwords:** https://support.apple.com/en-us/HT204397

---

*Last Updated: 2025-11-21*
