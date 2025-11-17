#!/bin/bash
#
# Auxin App Bundle Creation Script
# Creates a proper macOS .app bundle from the Swift executable
#
set -e

echo "Creating Auxin.app bundle..."

# Directories
BUILD_DIR=".build/release"
EXECUTABLE_NAME="Auxin"
APP_NAME="Auxin.app"
APP_DIR="$APP_NAME"
CONTENTS_DIR="$APP_DIR/Contents"
MACOS_DIR="$CONTENTS_DIR/MacOS"
RESOURCES_DIR="$CONTENTS_DIR/Resources"

# Clean and create directories
echo "• Creating app bundle structure..."
rm -rf "$APP_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# Copy executable
echo "• Copying executable..."
if [ ! -f "$BUILD_DIR/$EXECUTABLE_NAME" ]; then
    echo "Error: Executable not found at $BUILD_DIR/$EXECUTABLE_NAME"
    echo "Run 'swift build -c release' first"
    exit 1
fi
cp "$BUILD_DIR/$EXECUTABLE_NAME" "$MACOS_DIR/"
chmod +x "$MACOS_DIR/$EXECUTABLE_NAME"

# Copy Info.plist
echo "• Copying Info.plist..."
if [ ! -f "Resources/Info.plist" ]; then
    echo "Error: Info.plist not found at Resources/Info.plist"
    exit 1
fi
cp "Resources/Info.plist" "$CONTENTS_DIR/"

# Create PkgInfo
echo "• Creating PkgInfo..."
echo -n "APPL????" > "$CONTENTS_DIR/PkgInfo"

# Copy any additional resources if they exist
if [ -d "Resources/Assets.xcassets" ]; then
    echo "• Copying Assets..."
    cp -R "Resources/Assets.xcassets" "$RESOURCES_DIR/"
fi

echo "✅ App bundle created successfully: $APP_DIR"
echo ""
echo "You can now:"
echo "  • Run: open $APP_DIR"
echo "  • Or install: cp -R $APP_DIR /Applications/"
echo ""
