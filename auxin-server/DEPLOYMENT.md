# Auxin Server - Deployment Guide

Complete guide for deploying auxin-server for local development, testing, and production.

## Table of Contents

- [Quick Start (Local Testing)](#quick-start-local-testing)
- [Development Deployment](#development-deployment)
- [Docker Deployment](#docker-deployment)
- [Production Deployment (macOS)](#production-deployment-macos)
- [Testing](#testing)
- [Troubleshooting](#troubleshooting)

---

## Quick Start (Local Testing)

**Fastest way to get auxin-server running locally:**

```bash
# 1. One-command deployment
./deploy-local.sh

# 2. Start the server
./run-local.sh

# 3. Test it
curl http://localhost:3000/health
open http://localhost:3000  # If frontend is built
```

That's it! The server is now running with web UI (if Node.js is installed).

---

## Development Deployment

### Prerequisites

- **Rust 1.70+** - [Install](https://rustup.rs/)
- **Node.js 18+** - [Install](https://nodejs.org/) (optional, for web UI)

### Step-by-Step Setup

#### 1. Deploy Locally

```bash
./deploy-local.sh
```

This script will:
- ✅ Check prerequisites (Rust, Node.js)
- ✅ Set up local data directory (`.local-data/`)
- ✅ Create `.env` configuration
- ✅ Build the frontend (if Node.js available)
- ✅ Build the Rust backend (release mode)
- ✅ Create sample test repository

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🎵 Auxin Server - Local Deployment Complete!
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

📁 Data directory:    ./auxin-server/.local-data
🔧 Configuration:     ./auxin-server/.env
🚀 Binary:            ./auxin-server/target/release/auxin-server
🎨 Web UI:            ✓ Built (frontend/dist)
```

#### 2. Run the Server

```bash
./run-local.sh
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🎵 Starting Auxin Server
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  🌐 Server:      http://127.0.0.1:3000
  📁 Data:        /path/to/.local-data
  📝 Logs:        RUST_LOG=info

  Press Ctrl+C to stop
```

#### 3. Access the Server

**API:**
```bash
curl http://localhost:3000/health
curl http://localhost:3000/api/repos
```

**Web UI:**
```
http://localhost:3000
```

### Development Workflow

**Backend Development:**
```bash
# Make changes to Rust code in src/

# Rebuild and run
cargo build --release && ./run-local.sh
```

**Frontend Development:**
```bash
# Terminal 1: Run backend
./run-local.sh

# Terminal 2: Frontend dev server (hot reload)
cd frontend
npm run dev
# Opens http://localhost:5173
```

The dev server proxies API requests to `:3000`, so you get instant updates!

---

## Docker Deployment

### Option 1: Docker Compose (Recommended)

**Build and run everything:**

```bash
# Build frontend first
cd frontend
npm install
npm run build
cd ..

# Start with Docker Compose
docker-compose up -d

# View logs
docker-compose logs -f

# Stop
docker-compose down
```

**Access:**
- API: `http://localhost:3000/api/repos`
- Web UI: `http://localhost:3000`

### Option 2: Plain Docker

**Build image:**

```bash
# Build frontend first
cd frontend && npm install && npm run build && cd ..

# Build Docker image
docker build -t auxin-server:latest .
```

**Run container:**

```bash
docker run -d \
  --name auxin-server \
  -p 3000:3000 \
  -v auxin-data:/var/oxen/data \
  auxin-server:latest
```

**Check status:**

```bash
docker ps
docker logs auxin-server
```

**Stop container:**

```bash
docker stop auxin-server
docker rm auxin-server
```

### Docker Configuration

**Environment variables:**

```bash
docker run -d \
  -p 3000:3000 \
  -e SYNC_DIR=/var/oxen/data \
  -e OXEN_SERVER_PORT=3000 \
  -e RUST_LOG=debug \
  -v auxin-data:/var/oxen/data \
  auxin-server:latest
```

**With Redis (distributed locks):**

Uncomment the `redis` service in `docker-compose.yml`:

```yaml
services:
  auxin-server:
    environment:
      - ENABLE_REDIS_LOCKS=true
      - REDIS_URL=redis://redis:6379

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
```

---

## Production Deployment (macOS)

### System Installation

For production use on macOS with LaunchAgent (auto-start on boot):

```bash
cd scripts
./setup.sh
```

This will:
- ✅ Build release binary
- ✅ Install to `/usr/local/bin/auxin-server`
- ✅ Create config at `~/.config/auxin-server/.env`
- ✅ Set up LaunchAgent for auto-start
- ✅ Create data directory at `/var/oxen/data`

### Managing the Service

```bash
# Start service
./scripts/start.sh

# Stop service
./scripts/stop.sh

# Check status
./scripts/status.sh

# View logs
tail -f ~/Library/Logs/auxin-server.log

# Restart
./scripts/restart.sh

# Uninstall
./scripts/uninstall.sh
```

### Production Configuration

Edit `~/.config/auxin-server/.env`:

```bash
# Server
SYNC_DIR=/var/oxen/data
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=0.0.0.0  # Allow external connections

# Authentication (optional)
AUTH_TOKEN_SECRET=your-random-secret-here
AUTH_TOKEN_EXPIRY_HOURS=24

# Logging
RUST_LOG=info,auxin_server=info

# Features
ENABLE_REDIS_LOCKS=false
```

### Nginx Reverse Proxy (Optional)

If deploying behind nginx:

```nginx
server {
    listen 80;
    server_name auxin.example.com;

    location / {
        proxy_pass http://127.0.0.1:3000;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

---

## Testing

### Automated Test Suite

```bash
# Make sure server is running first
./run-local.sh &

# Run tests
./test-deployment.sh
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  🧪 Auxin Server - Deployment Test Suite
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

Testing: Health check... ✓ PASS (HTTP 200)
Testing: List repositories... ✓ PASS (HTTP 200)
Testing: Create repository... ✓ PASS (HTTP 201)
Testing: Get repository info... ✓ PASS (HTTP 200)
Testing: List commits... ✓ PASS (HTTP 200)
Testing: List branches... ✓ PASS (HTTP 200)
Testing: Get lock status... ✓ PASS (HTTP 200)
Testing: Acquire lock... ✓ PASS (HTTP 200)
Testing: Lock heartbeat... ✓ PASS (HTTP 200)
Testing: Release lock... ✓ PASS (HTTP 200)
Testing: Web UI availability... ✓ PASS (HTTP 200)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Test Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  ✓ Passed: 11
  ✗ Failed: 0

✓ All tests passed!
```

### Manual API Testing

```bash
# Health check
curl http://localhost:3000/health

# List repositories
curl http://localhost:3000/api/repos

# Create repository
curl -X POST http://localhost:3000/api/repos/myuser/myrepo \
  -H "Content-Type: application/json" \
  -d '{"description": "My test repository"}'

# Get repository
curl http://localhost:3000/api/repos/myuser/myrepo

# List commits
curl http://localhost:3000/api/repos/myuser/myrepo/commits

# Acquire lock
curl -X POST http://localhost:3000/api/repos/myuser/myrepo/locks/acquire \
  -H "Content-Type: application/json" \
  -d '{"timeout_hours": 2}'

# Check lock status
curl http://localhost:3000/api/repos/myuser/myrepo/locks/status

# Release lock
curl -X POST http://localhost:3000/api/repos/myuser/myrepo/locks/release
```

### Web UI Testing

1. **Open browser:** `http://localhost:3000`
2. **Create repository:** Click "New Repository"
3. **View repository:** Click on any repository card
4. **Manage locks:** Go to "Locks" tab
5. **View metadata:** Go to "Metadata" tab

---

## Troubleshooting

### Server Won't Start

**Problem:** Binary not found

```bash
✗ Server binary not found!
```

**Solution:**
```bash
./deploy-local.sh  # Build everything
```

---

**Problem:** Port already in use

```
Error: Address already in use (os error 48)
```

**Solution:**
```bash
# Find process using port 3000
lsof -ti:3000

# Kill it
kill -9 $(lsof -ti:3000)

# Or change port in .env
echo "OXEN_SERVER_PORT=3001" >> .env
```

---

### Frontend Not Loading

**Problem:** Web UI returns 404

**Solution:**
```bash
# Check if frontend is built
ls -la frontend/dist/

# If not, build it
cd frontend
npm install
npm run build
```

---

**Problem:** "Cannot GET /" error

**Check:** Server logs should show:
```
Frontend static files found at: frontend/dist
Web UI will be available at http://127.0.0.1:3000/
```

If not:
```bash
./build-frontend.sh
./run-local.sh
```

---

### API Errors

**Problem:** CORS errors in browser console

**Check:** `src/main.rs` has CORS enabled:
```rust
.wrap(
    actix_cors::Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header(),
)
```

---

**Problem:** Connection refused

**Check:**
1. Server is running: `curl http://localhost:3000/health`
2. Firewall allows port 3000
3. `.env` has correct `OXEN_SERVER_HOST`

---

### Docker Issues

**Problem:** Build fails

```bash
# Clean rebuild
docker-compose down -v
docker-compose build --no-cache
docker-compose up
```

---

**Problem:** Permission denied errors

```bash
# Fix volume permissions
docker-compose down
docker volume rm auxin-server_auxin-data
docker-compose up -d
```

---

### Data Issues

**Problem:** Lost all repositories

**Check:** Data directory location
```bash
# Local deployment
ls -la .local-data/

# Production (macOS)
ls -la /var/oxen/data/

# Docker
docker exec auxin-server ls -la /var/oxen/data/
```

**Backup:**
```bash
# Local
tar -czf auxin-backup-$(date +%Y%m%d).tar.gz .local-data/

# Production
tar -czf auxin-backup-$(date +%Y%m%d).tar.gz /var/oxen/data/
```

---

## Deployment Checklist

### Before Deploying

- [ ] Rust installed and up to date
- [ ] Node.js installed (for web UI)
- [ ] Ports 3000 available
- [ ] Sufficient disk space (repositories can be large)

### Local Development

- [ ] Run `./deploy-local.sh`
- [ ] Verify `.env` exists
- [ ] Test with `./test-deployment.sh`
- [ ] Frontend builds successfully

### Production (macOS)

- [ ] Run `scripts/setup.sh`
- [ ] Configure `~/.config/auxin-server/.env`
- [ ] Test with `scripts/status.sh`
- [ ] Check logs: `~/Library/Logs/auxin-server.log`
- [ ] Set up backups for `/var/oxen/data`

### Docker

- [ ] Build frontend first
- [ ] Test with `docker-compose up`
- [ ] Configure volumes for persistence
- [ ] Set up health checks
- [ ] Configure resource limits

---

## Performance Tuning

### Rust Binary Optimization

Already optimized in `Cargo.toml`:
```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Single codegen unit
opt-level = 3           # Maximum optimization
```

### Frontend Optimization

```bash
cd frontend

# Analyze bundle size
npm run build -- --report

# Production build optimizations already enabled:
# - Tree shaking
# - Minification
# - Code splitting
# - Asset optimization
```

### Data Directory

**SSD recommended** for `/var/oxen/data` or `SYNC_DIR`:
- Faster repository operations
- Better performance for large files
- Improved commit/checkout speed

---

## Security Considerations

### Authentication

Generate strong auth token:
```bash
openssl rand -hex 32
```

Add to `.env`:
```bash
AUTH_TOKEN_SECRET=your-generated-secret-here
```

### Network Security

**Local development:**
```bash
OXEN_SERVER_HOST=127.0.0.1  # Localhost only
```

**Production:**
```bash
OXEN_SERVER_HOST=0.0.0.0     # All interfaces
# + Use firewall rules
# + Use reverse proxy (nginx)
# + Use HTTPS (Let's Encrypt)
```

### File Permissions

```bash
# Data directory
chmod 700 /var/oxen/data
chown $USER /var/oxen/data

# Config file
chmod 600 ~/.config/auxin-server/.env
```

---

## Summary

**Three deployment options:**

1. **Local Development** (Fastest)
   ```bash
   ./deploy-local.sh && ./run-local.sh
   ```

2. **Docker** (Portable)
   ```bash
   docker-compose up -d
   ```

3. **Production macOS** (Auto-start)
   ```bash
   scripts/setup.sh && scripts/start.sh
   ```

**All options include:**
- ✅ Full REST API
- ✅ Web frontend (if built)
- ✅ Logic Pro metadata support
- ✅ Distributed locking
- ✅ Health checks
- ✅ Logging

Choose based on your needs and enjoy version control for Logic Pro! 🎵

---

*For more help, see:*
- Main README: `README.md`
- Frontend docs: `frontend/README.md`
- Setup guide: `FRONTEND_SETUP.md`
- Known issues: `scripts/KNOWN_ISSUES.md`
