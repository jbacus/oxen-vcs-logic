# Production Build Quick Start

Quick reference for building production releases of Auxin.

## One-Time Setup (15 minutes)

### 1. Join Apple Developer Program
- Go to https://developer.apple.com/programs/
- Enroll ($99/year)
- Wait for approval (usually 24-48 hours)

### 2. Get Developer ID Certificates

**Via Xcode (easiest):**
1. Open Xcode → Settings → Accounts
2. Add your Apple ID
3. Click "Manage Certificates"
4. Click + and select:
   - Developer ID Application
   - Developer ID Installer

**Via Website:**
1. Go to https://developer.apple.com/account/resources/certificates/list
2. Create "Developer ID Application" certificate
3. Create "Developer ID Installer" certificate
4. Download and install both

### 3. Setup Notarization Credentials

Create app-specific password:
1. Go to https://appleid.apple.com
2. Security → Generate app-specific password
3. Label it "Auxin Notarization"
4. Copy the password (format: `xxxx-xxxx-xxxx-xxxx`)

Store credentials:
```bash
xcrun notarytool store-credentials "auxin-notary" \
    --apple-id "your-email@example.com" \
    --team-id "YOUR-TEAM-ID" \
    --password "xxxx-xxxx-xxxx-xxxx"
```

**Find your Team ID:**
- Go to https://developer.apple.com/account
- Click "Membership" in sidebar
- Copy your 10-character Team ID

### 4. Verify Setup

```bash
# Check certificates
security find-identity -v -p codesigning

# Should show:
# 1) ABC... "Developer ID Application: Your Name (TEAMID)"
# 2) XYZ... "Developer ID Installer: Your Name (TEAMID)"

# Check notarization profile
xcrun notarytool history --keychain-profile "auxin-notary"
# Should list your notarization history (may be empty)
```

✅ **Setup complete!** You only need to do this once.

---

## Building a Release (5 minutes + 15 min notarization)

### Quick Build

```bash
# Build production release
./build-production.sh 1.0.0

# Wait for notarization (~15 minutes)
# Script handles everything automatically
```

### What It Does

1. ✓ Builds all components (CLI, Daemon, App, Server)
2. ✓ Signs all binaries with Developer ID
3. ✓ Creates installer package (.pkg)
4. ✓ Signs installer
5. ✓ Submits for notarization
6. ✓ Waits for Apple approval
7. ✓ Staples notarization ticket
8. ✓ Creates DMG for distribution
9. ✓ Signs and notarizes DMG

### Output

```
release-1.0.0/
├── Auxin-1.0.0-signed.pkg    ← Upload this to GitHub Release
├── Auxin-1.0.0.dmg            ← Or upload this (contains .pkg)
└── RELEASE_NOTES.txt          ← Include in release description
```

### Testing Build (Skip Notarization)

For testing the build process without waiting for notarization:

```bash
./build-production.sh 1.0.0 --skip-notarization
```

⚠️ **Warning:** Test builds will show Gatekeeper warnings on other Macs.

---

## Distribution Workflow

### 1. Prepare Release

```bash
# Update version in files
# - Auxin-CLI-Wrapper/Cargo.toml
# - auxin-server/Cargo.toml
# - Auxin-App/Sources/Info.plist
# - CHANGELOG.md

# Commit changes
git add .
git commit -m "chore: Bump version to 1.0.0"
git push
```

### 2. Build Production Release

```bash
./build-production.sh 1.0.0
```

**Time:** ~5 minutes build + ~15 minutes notarization = ~20 minutes total

### 3. Test the Release

```bash
# Install on test Mac
open release-1.0.0/Auxin-1.0.0.dmg

# Then run the installer
# Verify:
# - No Gatekeeper warnings
# - All components install correctly
# - App launches without errors
# - CLI works: auxin --version
```

### 4. Create GitHub Release

```bash
# Create and push tag
git tag v1.0.0
git push origin v1.0.0

# Go to GitHub → Releases → Create new release
# - Tag: v1.0.0
# - Title: Auxin 1.0.0
# - Description: (copy from CHANGELOG.md)
# - Attach: Auxin-1.0.0.dmg
# - Publish release
```

### 5. Announce

- Update README with download link
- Post on social media / forums
- Update documentation site
- Notify users

---

## Troubleshooting

### "Certificate not found"

**Problem:** Script can't find your certificates

**Solution:**
```bash
# List certificates
security find-identity -v -p codesigning

# Copy exact certificate name and set environment variable
export APP_SIGNING_CERT="Developer ID Application: John Doe (ABC123DEF4)"
export INSTALLER_SIGNING_CERT="Developer ID Installer: John Doe (ABC123DEF4)"

# Then run build
./build-production.sh 1.0.0
```

### "Notarization profile not found"

**Problem:** Credentials not stored

**Solution:**
```bash
# Store credentials again
xcrun notarytool store-credentials "auxin-notary" \
    --apple-id "your@email.com" \
    --team-id "TEAMID" \
    --password "xxxx-xxxx-xxxx-xxxx"
```

### "Notarization rejected"

**Problem:** Apple rejected your submission

**Solution:**
```bash
# Get rejection reason
xcrun notarytool log SUBMISSION-ID --keychain-profile "auxin-notary"

# Common issues:
# - Missing hardened runtime (script handles this)
# - Unsigned nested binaries (script handles this)
# - Invalid entitlements (check docs/PRODUCTION_SIGNING.md)
```

### "Developer cannot be verified" when installing

**Problem:** Not properly signed or notarized

**Solution:**
```bash
# Verify package signature
pkgutil --check-signature release-1.0.0/Auxin-1.0.0-signed.pkg

# Verify notarization
xcrun stapler validate release-1.0.0/Auxin-1.0.0-signed.pkg

# Check Gatekeeper
spctl --assess --type install --verbose release-1.0.0/Auxin-1.0.0-signed.pkg
```

---

## Common Scenarios

### Update Certificate

Certificates expire after 5 years. To update:

1. Request new certificate (same process as initial setup)
2. Old certificate will be automatically replaced
3. Next build will use new certificate
4. No other changes needed

### Multiple Developers

Each developer needs their own:
- Apple Developer account (individual or organization)
- Developer ID certificates
- Notarization credentials

Or use a shared:
- Organization Apple Developer account
- Distribution certificates (shared via Keychain export)
- Notarization credentials (shared securely)

### Automated Builds (CI/CD)

See `docs/PRODUCTION_SIGNING.md` for GitHub Actions example.

Key points:
- Store certificates as base64-encoded secrets
- Store notarization credentials as secrets
- Import certificates to temporary keychain
- Run build-production.sh
- Upload artifacts to release

---

## Quick Reference Commands

```bash
# List certificates
security find-identity -v -p codesigning

# Check notarization profile
xcrun notarytool history --keychain-profile "auxin-notary"

# Build production release
./build-production.sh 1.0.0

# Build without notarization (testing)
./build-production.sh 1.0.0 --skip-notarization

# Verify signed package
pkgutil --check-signature file.pkg

# Verify notarization
xcrun stapler validate file.pkg

# Check Gatekeeper
spctl --assess --type install --verbose file.pkg

# Manual notarization (if needed)
xcrun notarytool submit file.pkg --keychain-profile "auxin-notary" --wait
xcrun stapler staple file.pkg
```

---

## Cost Summary

| Item | Cost | Frequency |
|------|------|-----------|
| Apple Developer Program | $99 | Annually |
| Code Signing Certificate | Included | 5 years |
| Notarization | Free | Per submission |

**Total annual cost: $99/year**

---

## Help & Support

- **Full documentation:** `docs/PRODUCTION_SIGNING.md`
- **Apple Developer:** https://developer.apple.com/support/
- **Code signing issues:** https://developer.apple.com/forums/
- **Auxin issues:** https://github.com/jbacus/auxin/issues

---

## Checklist

### One-Time Setup
- [ ] Join Apple Developer Program
- [ ] Obtain Developer ID Application certificate
- [ ] Obtain Developer ID Installer certificate
- [ ] Create app-specific password
- [ ] Store notarization credentials
- [ ] Verify setup with test build

### Each Release
- [ ] Update version numbers
- [ ] Update CHANGELOG.md
- [ ] Commit and push changes
- [ ] Run `./build-production.sh X.Y.Z`
- [ ] Wait for notarization (~15 min)
- [ ] Test on clean Mac
- [ ] Create Git tag
- [ ] Create GitHub release
- [ ] Upload DMG to release
- [ ] Announce release

---

*Last Updated: 2025-11-21*
