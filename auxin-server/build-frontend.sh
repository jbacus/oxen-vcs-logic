#!/bin/bash
set -e

echo "🎨 Building Auxin Frontend..."
echo ""

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js 18+ first."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js version: $(node --version)"
echo ""

# Navigate to frontend directory
cd frontend

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm install
    echo ""
fi

# Build production bundle
echo "🔨 Building production bundle..."
npm run build
echo ""

# Check if build succeeded
if [ -d "dist" ]; then
    echo "✅ Frontend build successful!"
    echo "   Output: frontend/dist/"
    echo ""
    echo "💡 Start the server with: cargo run --release"
    echo "   Then open: http://localhost:3000"
else
    echo "❌ Build failed!"
    exit 1
fi
