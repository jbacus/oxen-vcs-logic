#!/bin/bash
#
# Clean, rebuild, and reset Auxin app
# Useful when the app is in a broken state
#
set -e

echo "🧹 Cleaning Auxin app..."

# Clear saved preferences
echo "• Clearing saved preferences..."
defaults delete com.oxenvcs.app 2>/dev/null || echo "  (No saved preferences found)"

# Clean build artifacts
echo "• Cleaning build artifacts..."
rm -rf .build
rm -rf Auxin.app

# Rebuild
echo ""
echo "🔨 Building Auxin app..."
swift build -c release

# Create app bundle
echo ""
./create-app-bundle.sh

echo ""
echo "✅ Done! You can now run:"
echo "   open Auxin.app"
echo ""
