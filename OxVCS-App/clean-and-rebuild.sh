#!/bin/bash
#
# Clean, rebuild, and reset OxVCS app
# Useful when the app is in a broken state
#
set -e

echo "🧹 Cleaning OxVCS app..."

# Clear saved preferences
echo "• Clearing saved preferences..."
defaults delete com.oxenvcs.app 2>/dev/null || echo "  (No saved preferences found)"

# Clean build artifacts
echo "• Cleaning build artifacts..."
rm -rf .build
rm -rf OxVCS.app

# Rebuild
echo ""
echo "🔨 Building OxVCS app..."
swift build -c release

# Create app bundle
echo ""
./create-app-bundle.sh

echo ""
echo "✅ Done! You can now run:"
echo "   open OxVCS.app"
echo ""
