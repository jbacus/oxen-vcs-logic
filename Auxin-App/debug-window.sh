#!/bin/bash
#
# Debug script to check window state
#
echo "🔍 Debugging Auxin window state..."
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
if [ -d "Auxin.app" ]; then
    echo "  ✓ Auxin.app exists"
    if [ -f "Auxin.app/Contents/Info.plist" ]; then
        echo "  ✓ Info.plist exists"
    fi
    if [ -x "Auxin.app/Contents/MacOS/Auxin" ]; then
        echo "  ✓ Executable exists and is executable"
    fi
else
    echo "  ✗ Auxin.app not found"
fi

echo ""
echo "Run the app:"
echo "  open Auxin.app"
echo ""
