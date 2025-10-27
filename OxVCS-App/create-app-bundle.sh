#!/bin/bash
#
# Create OxVCS.app Bundle
# Packages the Swift executable into a double-clickable macOS application bundle
#

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

print_info() {
    echo -e "${BLUE}ℹ${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

print_error() {
    echo -e "${RED}✗${NC} $1"
}

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/.build/release"
APP_NAME="OxVCS"
APP_BUNDLE="$SCRIPT_DIR/$APP_NAME.app"
EXECUTABLE_NAME="OxVCS"

print_info "Building OxVCS.app bundle..."
echo ""

# Step 1: Build the executable if needed
if [ ! -f "$BUILD_DIR/$EXECUTABLE_NAME" ]; then
    print_info "Building Swift executable in release mode..."
    cd "$SCRIPT_DIR"
    swift build -c release
    print_success "Executable built"
else
    print_info "Using existing executable at $BUILD_DIR/$EXECUTABLE_NAME"
fi

# Step 2: Create app bundle structure
print_info "Creating app bundle structure..."
rm -rf "$APP_BUNDLE"
mkdir -p "$APP_BUNDLE/Contents/MacOS"
mkdir -p "$APP_BUNDLE/Contents/Resources"
print_success "Bundle structure created"

# Step 3: Copy executable
print_info "Copying executable..."
cp "$BUILD_DIR/$EXECUTABLE_NAME" "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
chmod +x "$APP_BUNDLE/Contents/MacOS/$APP_NAME"
print_success "Executable copied"

# Step 4: Create Info.plist
print_info "Creating Info.plist..."
cat > "$APP_BUNDLE/Contents/Info.plist" << 'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>OxVCS</string>
    <key>CFBundleIdentifier</key>
    <string>com.oxen.oxvcs</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>OxVCS</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
    <key>LSMinimumSystemVersion</key>
    <string>14.0</string>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSApplicationCategoryType</key>
    <string>public.app-category.developer-tools</string>
    <key>NSHumanReadableCopyright</key>
    <string>Copyright © 2025 Oxen.ai. All rights reserved.</string>
    <key>CFBundleDisplayName</key>
    <string>OxVCS for Logic Pro</string>
    <key>LSUIElement</key>
    <false/>
</dict>
</plist>
EOF
print_success "Info.plist created"

# Step 5: Create PkgInfo (optional but traditional)
print_info "Creating PkgInfo..."
echo -n "APPL????" > "$APP_BUNDLE/Contents/PkgInfo"
print_success "PkgInfo created"

# Step 6: Set proper permissions
print_info "Setting permissions..."
chmod -R 755 "$APP_BUNDLE"
chmod 644 "$APP_BUNDLE/Contents/Info.plist"
chmod 644 "$APP_BUNDLE/Contents/PkgInfo"
print_success "Permissions set"

# Step 7: Verify bundle
print_info "Verifying bundle structure..."
if [ -d "$APP_BUNDLE" ] && \
   [ -f "$APP_BUNDLE/Contents/Info.plist" ] && \
   [ -x "$APP_BUNDLE/Contents/MacOS/$APP_NAME" ]; then
    print_success "Bundle structure verified"
else
    print_error "Bundle structure is invalid"
    exit 1
fi

echo ""
print_success "✨ OxVCS.app created successfully!"
echo ""
echo "Location: $APP_BUNDLE"
echo ""
echo "To install:"
echo "  1. Copy to Applications folder:"
echo "     cp -r '$APP_BUNDLE' /Applications/"
echo ""
echo "  2. Or run directly:"
echo "     open '$APP_BUNDLE'"
echo ""
echo "Note: On first launch, you may need to allow the app in System Settings"
echo "      (System Settings → Privacy & Security → Allow)"
echo ""
