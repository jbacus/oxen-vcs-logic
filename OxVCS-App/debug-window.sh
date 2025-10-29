#!/bin/bash
#
# Debug script to check window state
#
echo "🔍 Debugging OxVCS window state..."
echo ""

# Check for saved preferences
echo "Saved preferences:"
defaults read com.oxenvcs.app 2>/dev/null || echo "  (None found - this is good!)"

echo ""
echo "To clear saved preferences:"
echo "  defaults delete com.oxenvcs.app"
echo ""

# Check app bundle structure
echo "App bundle check:"
if [ -d "OxVCS.app" ]; then
    echo "  ✓ OxVCS.app exists"
    if [ -f "OxVCS.app/Contents/Info.plist" ]; then
        echo "  ✓ Info.plist exists"
    fi
    if [ -x "OxVCS.app/Contents/MacOS/OxVCS" ]; then
        echo "  ✓ Executable exists and is executable"
    fi
else
    echo "  ✗ OxVCS.app not found"
fi

echo ""
echo "Run the app:"
echo "  open OxVCS.app"
echo ""
