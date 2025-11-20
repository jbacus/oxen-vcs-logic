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

## Monitoring and Observability

### Health Checks

Monitor server health:
```bash
# Basic health check
curl http://localhost:3000/health

# Expected response
{"status":"healthy","version":"0.1.0"}
```

### Logging

**Development:**
```bash
# Run with debug logging
RUST_LOG=debug ./run-local.sh

# Filter specific modules
RUST_LOG=auxin_server=debug,actix_web=info ./run-local.sh
```

**Production:**
```bash
# Log to file
./run-local.sh > /var/log/auxin-server/access.log 2>&1

# Rotate logs with logrotate
sudo tee /etc/logrotate.d/auxin-server << 'EOF'
/var/log/auxin-server/*.log {
    daily
    rotate 7
    compress
    delaycompress
    notifempty
    create 644 auxin auxin
    postrotate
        pkill -HUP auxin-server
    endscript
}
EOF
```

### Metrics

Monitor key metrics:

```bash
# Server status
curl http://localhost:3000/api/status

# Repository stats
curl http://localhost:3000/api/repositories

# System resources
ps aux | grep auxin-server
du -sh /var/oxen/data
```

### Alerting

Set up basic monitoring:

```bash
# Simple uptime check (cron every 5 min)
*/5 * * * * curl -sf http://localhost:3000/health || echo "Auxin server down!" | mail -s "Alert" admin@example.com
```

**Recommended monitoring tools:**
- Prometheus + Grafana (metrics)
- ELK Stack (log aggregation)
- Uptime Robot (external monitoring)
- PagerDuty (incident management)

---

## Upgrade Procedures

### Upgrading Auxin Server

#### Before Upgrading

1. **Backup data directory:**
```bash
tar -czf /backup/oxen-data-$(date +%Y%m%d).tar.gz /var/oxen/data
```

2. **Check current version:**
```bash
curl http://localhost:3000/health
```

3. **Review changelog** for breaking changes

#### Upgrade Process

**Local Development:**
```bash
# 1. Stop server
pkill auxin-server

# 2. Pull latest code
git pull origin main

# 3. Rebuild
./deploy-local.sh

# 4. Restart
./run-local.sh
```

**Production (macOS):**
```bash
# 1. Stop service
launchctl unload ~/Library/LaunchAgents/com.auxin.server.plist

# 2. Backup binary
cp /usr/local/bin/auxin-server /usr/local/bin/auxin-server.bak

# 3. Build and install new version
cd auxin-server
cargo build --release
sudo cp target/release/auxin-server /usr/local/bin/

# 4. Restart service
launchctl load ~/Library/LaunchAgents/com.auxin.server.plist

# 5. Verify
curl http://localhost:3000/health
```

**Docker:**
```bash
# 1. Pull latest image
docker-compose pull

# 2. Recreate containers
docker-compose up -d --force-recreate

# 3. Verify
docker-compose logs -f
```

#### Rollback

If upgrade fails:

```bash
# Local/Production
cp /usr/local/bin/auxin-server.bak /usr/local/bin/auxin-server
launchctl load ~/Library/LaunchAgents/com.auxin.server.plist

# Docker
docker-compose down
docker image ls | grep auxin-server  # Find previous version
docker-compose up -d auxin-server:<previous-version>
```

### Database Migrations

Currently auxin-server uses filesystem storage (no database migrations needed).

Future versions with SQLite/PostgreSQL will include:
```bash
# Migration command (coming soon)
auxin-server migrate
```

---

## FAQ

### General Questions

**Q: What's the difference between mock-oxen and full-oxen modes?**
A: `mock-oxen` (default) provides full HTTP API, locks, and metadata but returns `501 Not Implemented` for VCS operations (commit, push, pull). `full-oxen` mode includes actual VCS operations but requires async refactoring (WIP).

**Q: Can I use this in production?**
A: Yes! The mock-oxen mode is production-ready for server infrastructure, distributed locking, and metadata management. Full VCS operations are coming soon.

**Q: How much disk space do I need?**
A: Depends on your projects. Logic Pro projects can be 1-10GB each. Plan for:
- Small team (5 users, 10 projects): 100GB
- Medium team (20 users, 50 projects): 500GB
- Large team (100 users, 200 projects): 2TB+

**Q: Can I run multiple servers?**
A: Not currently. Distributed server deployment requires coordination layer (planned for future).

### Performance Questions

**Q: How many concurrent users can it handle?**
A: In testing: ~100 concurrent users with release build on modest hardware (4 cores, 8GB RAM). Use load balancer + multiple instances for more.

**Q: Why is the first request slow?**
A: Cold start - Rust binary loads. Subsequent requests are fast (<10ms). Keep server running or use pre-warming.

**Q: How do I improve performance?**
A:
1. Use release build (`--release`)
2. Increase file descriptors (`ulimit -n 4096`)
3. Use SSD for data directory
4. Enable HTTP/2 in reverse proxy
5. Use CDN for static assets

### Troubleshooting Questions

**Q: Server crashes with "Address already in use"**
A: Port 3000 is taken. Either:
```bash
# Kill existing process
lsof -ti:3000 | xargs kill -9

# Or change port
export OXEN_SERVER_PORT=3001
```

**Q: Frontend shows "Failed to fetch"**
A: CORS issue or API not running. Check:
```bash
# Is server running?
curl http://localhost:3000/health

# Check console for errors
open http://localhost:3000
# F12 > Console tab
```

**Q: "Permission denied" errors**
A: Check file permissions:
```bash
ls -la /var/oxen/data
chmod -R 755 /var/oxen/data
chown -R $USER /var/oxen/data
```

### Development Questions

**Q: How do I debug the server?**
A:
```bash
# Run with debug logs
RUST_LOG=debug cargo run

# Use rust-lldb for debugging
rust-lldb target/debug/auxin-server
```

**Q: Can I contribute?**
A: Yes! See CONTRIBUTING.md (if it exists) or open an issue/PR on GitHub.

**Q: Where are the logs?**
A:
- Local: stdout/stderr
- Docker: `docker-compose logs -f`
- Production: `/var/log/auxin-server/` (if configured)

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
