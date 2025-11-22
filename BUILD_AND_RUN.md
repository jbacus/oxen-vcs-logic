# Auxin Local Build & Run Guide

**Last Updated**: 2025-11-22
**Purpose**: Complete guide to building and running all Auxin components locally for testing
**Time Required**: ~15-20 minutes (first time)

---

## Prerequisites

### Required Tools

```bash
# Check you have everything installed
xcode-select --version          # Xcode Command Line Tools
swift --version                 # Swift 5.9+
cargo --version                 # Rust 1.70+
node --version                  # Node.js 18+
npm --version                   # npm 9+
oxen --version                  # Oxen CLI (pip install oxenai)
```

### Install Missing Tools

```bash
# Xcode Command Line Tools (if needed)
xcode-select --install

# Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js (if needed - use Homebrew)
brew install node

# Oxen CLI (required for VCS operations)
pip install oxenai
```

---

## Component 1: Auxin CLI (Rust)

**What**: Command-line interface for all Auxin operations
**Location**: `Auxin-CLI-Wrapper/`
**Language**: Rust
**Build Time**: ~2-3 minutes (first time)

### Build Steps

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-CLI-Wrapper"

# Clean previous builds (optional)
cargo clean

# Build in release mode (optimized)
cargo build --release

# Verify build succeeded
ls -lh target/release/auxin

# Expected output: Binary ~15-20 MB
```

### Test It Works

```bash
# Check version and help
./target/release/auxin --version
./target/release/auxin --help

# Test server connectivity
./target/release/auxin server health
```

### Add to PATH (Optional)

```bash
# Add to your shell profile (~/.zshrc or ~/.bash_profile)
export PATH="/Users/johnbacus/My Projects/Unit3/auxin/Auxin-CLI-Wrapper/target/release:$PATH"

# Reload shell
source ~/.zshrc

# Now you can use 'auxin' directly
auxin --version
```

---

## Component 2: Auxin LaunchAgent (Swift Daemon)

**What**: Background daemon for automatic commits and file monitoring
**Location**: `Auxin-LaunchAgent/`
**Language**: Swift
**Build Time**: ~1-2 minutes

### Build Steps

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-LaunchAgent"

# Clean previous builds (optional)
swift package clean

# Build in release mode
swift build -c release

# Verify build succeeded
ls -lh .build/release/Auxin-LaunchAgent

# Expected output: Binary ~3-5 MB
```

### Install as LaunchAgent (Optional - for production use)

```bash
# Copy to system location
sudo cp .build/release/Auxin-LaunchAgent /usr/local/bin/

# Install LaunchAgent plist
mkdir -p ~/Library/LaunchAgents
cp com.auxin.agent.plist ~/Library/LaunchAgents/

# Load the agent
launchctl load ~/Library/LaunchAgents/com.auxin.agent.plist

# Check if running
launchctl list | grep auxin
```

### Test It Works (Without Installing)

```bash
# Run daemon in foreground for testing
.build/release/Auxin-LaunchAgent --help

# Note: Full daemon features require installation as LaunchAgent
```

---

## Component 3: Auxin GUI App (Swift/SwiftUI)

**What**: Native macOS application for project management
**Location**: `Auxin-App/`
**Language**: Swift/SwiftUI
**Build Time**: ~2-3 minutes

### Build Steps

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-App"

# Clean previous builds (optional)
swift package clean

# Build in release mode
swift build -c release

# Create app bundle (macOS application)
./create-app-bundle.sh

# Expected output: Auxin.app in current directory
```

### Run the App

```bash
# Option 1: Open the app bundle
open Auxin.app

# Option 2: Run from command line
./Auxin.app/Contents/MacOS/Auxin-App

# Expected: macOS app window opens with project list
```

### Install to Applications (Optional)

```bash
# Copy to /Applications
cp -R Auxin.app /Applications/

# Now accessible via Spotlight/Launchpad
```

---

## Component 4: Auxin Server Backend (Rust)

**What**: Web server API for team collaboration
**Location**: `auxin-server/`
**Language**: Rust (Actix Web)
**Build Time**: ~3-5 minutes (first time)
**Port**: 3000 (default)

### Build Steps

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server"

# Clean previous builds (optional)
cargo clean

# Build in release mode with web UI support
cargo build --release --features web-ui

# Verify build succeeded
ls -lh target/release/auxin-server

# Expected output: Binary ~30-40 MB
```

### Configure Environment

```bash
# Create .env file for local development (optional)
cat > .env << 'EOF'
# Server configuration
SYNC_DIR=.local-data
OXEN_SERVER_HOST=127.0.0.1
OXEN_SERVER_PORT=3000
AUTH_TOKEN_SECRET=dev_secret_change_in_production
AUTH_TOKEN_EXPIRY_HOURS=24
ENABLE_WEB_UI=true
EOF
```

### Run the Server

```bash
# Start server (stays in foreground)
./target/release/auxin-server

# Expected output:
# INFO Starting Auxin Server...
# INFO Server will listen on 127.0.0.1:3000
# INFO Web UI will be available at http://127.0.0.1:3000/
```

**Server will run on**: `http://localhost:3000`

### Test Backend Works

```bash
# In a new terminal, test health endpoint
curl http://localhost:3000/health

# Expected: "OK"

# Test API endpoint
curl http://localhost:3000/api/repos

# Expected: JSON response (empty array [] if no repos)
```

---

## Component 5: Auxin Server Frontend (React/TypeScript)

**What**: Web UI for team collaboration
**Location**: `auxin-server/frontend/`
**Language**: React/TypeScript (Vite)
**Build Time**: ~1-2 minutes
**Serves From**: Backend on port 3000

### Build Steps

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server/frontend"

# Install dependencies (first time only)
npm install

# Build production bundle
npm run build

# Verify build succeeded
ls -lh dist/

# Expected: dist/ folder with index.html and assets/
```

### Development Mode (Optional - Hot Reload)

```bash
# Run development server with hot reload
npm run dev

# Expected output:
# VITE ready in XXX ms
# Local: http://localhost:5173/

# Note: Dev mode runs on port 5173, proxies API to port 3000
```

### Test Frontend Works

```bash
# With server running, open browser to:
open http://localhost:3000

# Expected: Auxin web UI loads, shows login/register page
```

---

## Quick Start: Build Everything

**One-command build script** (run from project root):

```bash
#!/bin/bash
# File: build_all.sh

set -e  # Exit on error

echo "🔨 Building Auxin Components..."
echo ""

# 1. CLI
echo "📦 Building CLI..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-CLI-Wrapper"
cargo build --release
echo "✅ CLI built"
echo ""

# 2. LaunchAgent
echo "📦 Building LaunchAgent..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-LaunchAgent"
swift build -c release
echo "✅ LaunchAgent built"
echo ""

# 3. GUI App
echo "📦 Building GUI App..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-App"
swift build -c release
./create-app-bundle.sh
echo "✅ GUI App built"
echo ""

# 4. Server Frontend
echo "📦 Building Server Frontend..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server/frontend"
npm install
npm run build
echo "✅ Frontend built"
echo ""

# 5. Server Backend
echo "📦 Building Server Backend..."
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server"
cargo build --release --features web-ui
echo "✅ Server built"
echo ""

echo "🎉 All components built successfully!"
echo ""
echo "Next steps:"
echo "  1. Start server: cd auxin-server && ./target/release/auxin-server"
echo "  2. Open browser: http://localhost:3000"
echo "  3. Open GUI app: open Auxin-App/Auxin.app"
echo "  4. Use CLI: Auxin-CLI-Wrapper/target/release/auxin --help"
```

### Run the build script

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin"
chmod +x build_all.sh
./build_all.sh
```

---

## Running Everything Together

### Terminal 1: Auxin Server

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/auxin-server"
./target/release/auxin-server

# Leave running - provides web UI and API
```

### Terminal 2: CLI Testing

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-CLI-Wrapper"

# Test CLI commands
./target/release/auxin server health
./target/release/auxin server status
./target/release/auxin --help
```

### Browser: Web UI

```bash
# Open web interface
open http://localhost:3000

# Expected: Web UI with login/register
```

### Finder: GUI App

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin/Auxin-App"
open Auxin.app

# Expected: Native macOS app opens
```

---

## Testing Workflow

### 1. Create a Test User (Web UI)

1. Open http://localhost:3000
2. Click "Register"
3. Enter:
   - Username: `test_user`
   - Email: `test@example.com`
   - Password: `test123`
4. Click "Register" button
5. Should redirect to login or dashboard

### 2. Create a Test Repository (Web UI)

1. Click "Create Repository" or "+"
2. Enter:
   - Namespace: `test_user`
   - Name: `TestProject`
   - Description: "My test project"
3. Click "Create"
4. Should see new repository in list

### 3. Test CLI Integration

```bash
# Configure CLI to use server
mkdir -p ~/.auxin
cat > ~/.auxin/config.toml << 'EOF'
[cli]
url = "http://localhost:3000"
token = ""  # Will be set after login
timeout_secs = 30
use_server_locks = true
use_server_metadata = true
default_namespace = "test_user"
EOF

# Test server connectivity
auxin server health
# Expected: ✓ Connected to http://localhost:3000

auxin server status
# Expected: Shows server config with "● Connected"
```

### 4. Initialize a Local Project

```bash
# Create a test Logic Pro project
cd /tmp
mkdir -p TestProject.logicx/Alternatives/000
touch TestProject.logicx/Alternatives/000/ProjectData

# Initialize with Auxin
auxin init --type logicpro TestProject.logicx

# Expected: Repository initialized with .oxenignore
```

### 5. Test Locking

```bash
cd /tmp/TestProject.logicx

# Check lock status
auxin lock status
# Expected: 🔓 Repository is UNLOCKED

# Acquire lock (note: may fail due to auth bug - that's expected)
auxin lock acquire --timeout 1
# Expected: Either success or fallback to local lock
```

### 6. Test GUI App

1. Open Auxin.app
2. Click "Add Project" or "+"
3. Browse to `/tmp/TestProject.logicx`
4. Should see project in sidebar
5. Click project to view commit history

---

## Troubleshooting

### Build Errors

**Problem**: Rust build fails with "linker error"
```bash
# Solution: Install Xcode Command Line Tools
xcode-select --install
```

**Problem**: Swift build fails with "missing module"
```bash
# Solution: Update Swift toolchain
# Download from https://swift.org/download/
```

**Problem**: npm install fails
```bash
# Solution: Clear npm cache
npm cache clean --force
rm -rf node_modules package-lock.json
npm install
```

### Runtime Errors

**Problem**: Server fails to start - "Address already in use"
```bash
# Solution: Kill process on port 3000
lsof -ti:3000 | xargs kill -9

# Or use a different port
OXEN_SERVER_PORT=3001 ./target/release/auxin-server
```

**Problem**: GUI app crashes on launch
```bash
# Solution: Check LaunchAgent is not running
launchctl list | grep auxin
launchctl unload ~/Library/LaunchAgents/com.auxin.agent.plist

# Run app again
open Auxin.app
```

**Problem**: CLI can't connect to server
```bash
# Solution: Verify server is running
curl http://localhost:3000/health

# Check CLI config
cat ~/.auxin/config.toml
```

### Clean Rebuild (Nuclear Option)

```bash
cd "/Users/johnbacus/My Projects/Unit3/auxin"

# Clean all Rust builds
find . -name "target" -type d -exec rm -rf {} +

# Clean all Swift builds
find . -name ".build" -type d -exec rm -rf {} +

# Clean frontend
cd auxin-server/frontend
rm -rf node_modules dist

# Rebuild everything
cd "/Users/johnbacus/My Projects/Unit3/auxin"
./build_all.sh
```

---

## Configuration Files

### CLI Config: `~/.auxin/config.toml`

```toml
[cli]
url = "http://localhost:3000"
token = ""  # Get from web UI after login
timeout_secs = 30
use_server_locks = true
use_server_metadata = true
default_namespace = "test_user"

[defaults]
verbose = false
color = "auto"

[lock]
timeout_hours = 4

[ui]
progress = true
emoji = true
```

### Server Config: `auxin-server/.env`

```env
SYNC_DIR=.local-data
OXEN_SERVER_HOST=127.0.0.1
OXEN_SERVER_PORT=3000
AUTH_TOKEN_SECRET=dev_secret_change_in_production
AUTH_TOKEN_EXPIRY_HOURS=24
ENABLE_WEB_UI=true
```

---

## Port Reference

| Component | Port | URL |
|-----------|------|-----|
| Auxin Server | 3000 | http://localhost:3000 |
| Frontend Dev Server | 5173 | http://localhost:5173 (dev mode only) |

---

## Next Steps

After building and running everything:

1. **Read the docs**:
   - `docs/user/getting-started.md` - User guide
   - `docs/developer/architecture.md` - Architecture overview
   - `V0.3_VALIDATION.md` - What's working in v0.3

2. **Try the workflows**:
   - Initialize a Logic Pro/SketchUp project
   - Make some commits
   - Test locking
   - Try the web UI
   - Explore the GUI app

3. **Report issues**:
   - Known issue: Server lock acquire has auth bug (falls back to local)
   - File other issues on GitHub

---

**Happy Testing!** 🚀

For questions or issues, see:
- `TROUBLESHOOTING.md` (if it exists)
- GitHub Issues: https://github.com/jbacus/auxin/issues
- Documentation: `docs/`
