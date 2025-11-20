# Auxin Server - Installation Guide

**Version:** 0.2.0
**Last Updated:** 2025-11-20
**Status:** Production-Ready

This guide covers all installation methods for Auxin server, from quick local development to production deployment.

---

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Quick Start (5 Minutes)](#quick-start-5-minutes)
3. [Installation Methods](#installation-methods)
4. [Verification](#verification)
5. [Configuration](#configuration)
6. [Troubleshooting](#troubleshooting)

---

## Prerequisites

### Required

✅ **Rust** (1.70 or later)
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### Optional

⚪ **Node.js** (18+) - For web dashboard
```bash
# macOS (Homebrew)
brew install node

# Linux
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Verify
node --version
npm --version
```

⚪ **Docker** - For containerized deployment
```bash
# macOS: Install Docker Desktop from docker.com
# Linux: Use docker.io package

docker --version
docker-compose --version
```

---

## Quick Start (5 Minutes)

The fastest way to get Auxin server running:

```bash
# 1. Clone repository (if not already done)
git clone https://github.com/yourusername/auxin.git
cd auxin/auxin-server

# 2. Deploy locally
./deploy-local.sh

# 3. Start server
./run-local.sh

# 4. Test in another terminal
curl http://localhost:3000/health
# Expected: OK

curl http://localhost:3000/api/repos
# Expected: []
```

**That's it!** Server is running on http://localhost:3000

---

## Installation Methods

### Method 1: Local Development (Recommended for Testing)

**Best for:** Development, testing, quick experiments

**Steps:**

```bash
cd auxin-server

# One-command deployment
./deploy-local.sh
```

**What it does:**
- ✅ Checks prerequisites (Rust, optionally Node.js)
- ✅ Creates `.local-data/` directory for repositories
- ✅ Generates `.env` configuration file
- ✅ Builds frontend (if Node.js available)
- ✅ Builds Rust backend (release mode)
- ✅ Creates sample test repository

**Start the server:**
```bash
./run-local.sh
```

**Data location:** `./local-data/`
**Configuration:** `./.env`
**Binary:** `./target/release/auxin-server`

**Pros:**
- ✅ No system installation required
- ✅ Easy to clean up (just delete directory)
- ✅ Perfect for development
- ✅ No sudo required

**Cons:**
- ❌ Not auto-started on boot
- ❌ Runs only when you start it manually

---

### Method 2: Docker (Recommended for Production)

**Best for:** Production deployment, isolated environments, easy scaling

**Steps:**

```bash
cd auxin-server

# Build frontend (one-time)
cd frontend
npm install
npm run build
cd ..

# Start with Docker Compose
docker-compose up -d
```

**docker-compose.yml** (already provided):
```yaml
version: '3.8'

services:
  auxin-server:
    build: .
    ports:
      - "3000:3000"
    volumes:
      - auxin-data:/var/oxen/data
    environment:
      - RUST_LOG=info,auxin_server=debug
      - ENABLE_WEB_UI=true
    restart: unless-stopped

volumes:
  auxin-data:
```

**Manage the service:**
```bash
# View logs
docker-compose logs -f

# Stop server
docker-compose down

# Restart
docker-compose restart

# Rebuild after code changes
docker-compose build
docker-compose up -d
```

**Data location:** Docker volume `auxin-data`
**Access:** http://localhost:3000

**Pros:**
- ✅ Portable across platforms
- ✅ Isolated environment
- ✅ Easy to scale (run multiple instances)
- ✅ Auto-restart on failure
- ✅ Simple backup (just export volume)

**Cons:**
- ❌ Requires Docker installation
- ❌ Slightly more complex debugging

---

### Method 3: macOS System Service (Production on macOS)

**Best for:** Production deployment on macOS, auto-start on boot

**Steps:**

```bash
cd auxin-server/scripts

# Install system-wide
./setup.sh

# Start service
./start.sh
```

**What it does:**
- ✅ Builds release binary
- ✅ Installs to `/usr/local/bin/auxin-server`
- ✅ Creates data directory `/var/oxen/data`
- ✅ Generates config in `~/.config/auxin-server/.env`
- ✅ Creates LaunchAgent for auto-start
- ✅ Starts the service

**Manage the service:**
```bash
# Check status
./status.sh

# Stop service
./stop.sh

# Restart service
./restart.sh

# Uninstall
./uninstall.sh
```

**Data location:** `/var/oxen/data`
**Configuration:** `~/.config/auxin-server/.env`
**Binary:** `/usr/local/bin/auxin-server`
**LaunchAgent:** `~/Library/LaunchAgents/com.auxin.server.plist`

**Pros:**
- ✅ Auto-starts on boot
- ✅ System-wide installation
- ✅ Service management (start/stop/restart)
- ✅ Production-ready

**Cons:**
- ❌ macOS only
- ❌ Requires sudo for some directories
- ❌ More complex to uninstall

---

## Verification

After installation, verify the server is working:

### 1. Health Check

```bash
curl http://localhost:3000/health
```

**Expected:** `OK`

### 2. API Test

```bash
# List repositories (should be empty initially)
curl http://localhost:3000/api/repos

# Create a test repository
curl -X POST http://localhost:3000/api/repos/testuser/testrepo \
  -H "Content-Type: application/json" \
  -d '{"description": "Test repository"}'

# List again (should show the new repo)
curl http://localhost:3000/api/repos
```

### 3. Web UI (if frontend is built)

Open http://localhost:3000 in your browser.

You should see the Auxin server dashboard.

### 4. Run Tests

```bash
cd auxin-server

# Run end-to-end collaboration tests
cargo test --test collaboration_e2e_tests -- --nocapture

# Run all tests
cargo test --all
```

**Expected:** All tests passing ✅

---

## Configuration

### Environment Variables

Auxin server is configured via environment variables or a `.env` file.

**Common Settings:**

```bash
# Server Configuration
SYNC_DIR=/var/oxen/data              # Where repositories are stored
OXEN_SERVER_HOST=0.0.0.0            # Listen address (0.0.0.0 for all interfaces)
OXEN_SERVER_PORT=3000                # Port to listen on

# Authentication
AUTH_TOKEN_SECRET=your-secret-key    # JWT token secret (keep secret!)
AUTH_TOKEN_EXPIRY_HOURS=24          # Token expiration time

# Logging
RUST_LOG=info,auxin_server=debug    # Log level

# Optional Features
ENABLE_REDIS_LOCKS=false             # Use Redis for distributed locks
ENABLE_WEB_UI=true                   # Enable web dashboard
```

### Configuration Locations

| Method | Config Location | Priority |
|--------|----------------|----------|
| Local Dev | `./env` | Environment vars override |
| Docker | `docker-compose.yml` environment | Container env vars |
| macOS Service | `~/.config/auxin-server/.env` | LaunchAgent env vars |

### Generating Secure Token Secret

**Important:** Use a cryptographically secure secret for production!

```bash
# Generate random 32-byte hex string
openssl rand -hex 32

# Or use this in your .env
AUTH_TOKEN_SECRET=$(openssl rand -hex 32)
```

### Example .env File

```bash
# Auxin Server Configuration

# Server
SYNC_DIR=/var/oxen/data
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=0.0.0.0

# Authentication (CHANGE THIS!)
AUTH_TOKEN_SECRET=your-very-secret-random-string-here
AUTH_TOKEN_EXPIRY_HOURS=24

# Logging
RUST_LOG=info,auxin_server=debug

# Features
ENABLE_REDIS_LOCKS=false
ENABLE_WEB_UI=true
```

---

## Troubleshooting

### Build Errors

#### "cargo: command not found"

**Cause:** Rust not installed

**Fix:**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### "error: linker `cc` not found"

**Cause:** C compiler not installed

**Fix:**
```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf install gcc
```

### Runtime Errors

#### "Address already in use" (port 3000)

**Cause:** Another service is using port 3000

**Fix:**
```bash
# Find what's using port 3000
lsof -i :3000

# Kill the process
kill -9 <PID>

# Or change port in .env
OXEN_SERVER_PORT=3001
```

#### "Permission denied" (data directory)

**Cause:** No write access to SYNC_DIR

**Fix:**
```bash
# Change ownership
sudo chown -R $USER /var/oxen/data

# Or use a different directory
SYNC_DIR=$HOME/.auxin/data
```

#### "Failed to connect to Redis"

**Cause:** Redis not running (only if ENABLE_REDIS_LOCKS=true)

**Fix:**
```bash
# Disable Redis locks
ENABLE_REDIS_LOCKS=false

# Or install and start Redis
brew install redis          # macOS
sudo apt install redis      # Linux
redis-server
```

### Test Failures

#### Collaboration tests fail

**Cause:** Port already in use or data directory issues

**Fix:**
```bash
# Make sure no server is running
pkill auxin-server

# Run tests with clean environment
cargo test --test collaboration_e2e_tests -- --nocapture
```

### Frontend Issues

#### "Frontend not built" warning

**Cause:** Frontend not compiled

**Fix:**
```bash
cd frontend
npm install
npm run build
cd ..
```

#### "npm: command not found"

**Cause:** Node.js not installed

**Fix:**
```bash
# macOS
brew install node

# Ubuntu
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
```

---

## Upgrading

### Local Development

```bash
cd auxin-server

# Pull latest changes
git pull origin main

# Rebuild
cargo build --release

# Rebuild frontend (if changed)
cd frontend && npm run build && cd ..

# Restart server
# (Ctrl+C to stop, then ./run-local.sh)
```

### Docker

```bash
cd auxin-server

# Pull latest code
git pull origin main

# Rebuild container
docker-compose build

# Restart
docker-compose down
docker-compose up -d
```

### macOS Service

```bash
cd auxin-server/scripts

# Stop service
./stop.sh

# Rebuild and reinstall
cd ..
cargo build --release
cd scripts
sudo cp ../target/release/auxin-server /usr/local/bin/

# Start service
./start.sh
```

---

## Next Steps

After installation:

1. **Read the documentation:**
   - [README.md](README.md) - Overview and quick start
   - [TESTING.md](TESTING.md) - Testing guide
   - [STATUS.md](STATUS.md) - Production readiness

2. **Test collaboration features:**
   ```bash
   cargo test --test collaboration_e2e_tests -- --nocapture
   ```

3. **Try the API:**
   - See [README.md#api-endpoints](README.md#api-endpoints)
   - Use curl or Postman to test endpoints

4. **Set up authentication:**
   - Register users via `/api/auth/register`
   - Test login via `/api/auth/login`

5. **Configure for production:**
   - Set secure `AUTH_TOKEN_SECRET`
   - Configure firewall if exposing externally
   - Set up HTTPS (use nginx/caddy reverse proxy)

---

## Support

- **Documentation:** [README.md](README.md)
- **Testing Guide:** [TESTING.md](TESTING.md)
- **GitHub Issues:** https://github.com/jbacus/auxin/issues

---

## Summary

| Method | Best For | Auto-Start | Complexity | Production-Ready |
|--------|----------|------------|------------|------------------|
| **Local Dev** | Testing | ❌ | ⭐ Easy | ❌ |
| **Docker** | Production | ✅ | ⭐⭐ Medium | ✅ |
| **macOS Service** | macOS Production | ✅ | ⭐⭐⭐ Advanced | ✅ |

**Recommended:**
- Development: Local Dev
- Production: Docker or macOS Service

---

**Installation successful!** 🎉

Start the server and visit http://localhost:3000 to get started with remote collaboration.
