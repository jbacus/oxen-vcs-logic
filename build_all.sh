#!/bin/bash
# Auxin - Build All Components
# Usage: ./build_all.sh

set -e  # Exit on error

echo "🔨 Building Auxin Components..."
echo ""

# 1. CLI
echo "📦 Building CLI..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-CLI-Wrapper"
cargo build --release
echo "✅ CLI built: Auxin-CLI-Wrapper/target/release/auxin"
echo ""

# 2. LaunchAgent
echo "📦 Building LaunchAgent..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-LaunchAgent"
swift build -c release
echo "✅ LaunchAgent built: Auxin-LaunchAgent/.build/release/Auxin-LaunchAgent"
echo ""

# 3. GUI App
echo "📦 Building GUI App..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-App"
swift build -c release
if [ -f "./create-app-bundle.sh" ]; then
    ./create-app-bundle.sh
    echo "✅ GUI App built: Auxin-App/Auxin.app"
else
    echo "✅ GUI App built: Auxin-App/.build/release/Auxin-App"
    echo "⚠️  Note: create-app-bundle.sh not found - no .app bundle created"
fi
echo ""

# 4. Server Frontend
echo "📦 Building Server Frontend..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server/frontend"
if [ ! -d "node_modules" ]; then
    echo "   Installing npm dependencies (first time)..."
    npm install
fi
npm run build
echo "✅ Frontend built: auxin-server/frontend/dist/"
echo ""

# 5. Server Backend
echo "📦 Building Server Backend..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server"
cargo build --release --features web-ui
echo "✅ Server built: auxin-server/target/release/auxin-server"
echo ""

echo "🎉 All components built successfully!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "Next steps:"
echo ""
echo "1. Start the server:"
echo "   cd auxin-server"
echo "   ./target/release/auxin-server"
echo ""
echo "2. Open web UI in browser:"
echo "   open http://localhost:3000"
echo ""
echo "3. Test CLI:"
echo "   cd Auxin-CLI-Wrapper"
echo "   ./target/release/auxin server health"
echo ""
echo "4. Open GUI app:"
echo "   cd Auxin-App"
echo "   open Auxin.app"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "Full instructions: See BUILD_AND_RUN.md"
