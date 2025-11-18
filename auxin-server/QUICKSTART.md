# Auxin Server - Quick Start Guide

Get auxin-server running in **under 5 minutes**.

## The Fastest Way

```bash
# 1. Clone repository (if you haven't)
git clone https://github.com/jbacus/auxin.git
cd auxin/auxin-server

# 2. One-command deployment
./deploy-local.sh

# 3. Start server
./run-local.sh

# 4. Test it (in another terminal)
curl http://localhost:3000/health
# Expected: OK

# 5. Open web UI
open http://localhost:3000
```

**Done!** 🎉

---

## What Just Happened?

The `deploy-local.sh` script:

1. ✅ Checked for Rust and Node.js
2. ✅ Created local data directory
3. ✅ Built the frontend (React app)
4. ✅ Built the backend (Rust server)
5. ✅ Created sample test data
6. ✅ Generated configuration

The `run-local.sh` script:

1. ✅ Started the auxin-server
2. ✅ Serves API at `http://localhost:3000/api`
3. ✅ Serves Web UI at `http://localhost:3000`

---

## Next Steps

### Create Your First Repository

**Using Web UI:**
1. Open `http://localhost:3000`
2. Click "New Repository"
3. Enter namespace (e.g., `myuser`)
4. Enter name (e.g., `my-logic-project`)
5. Add description
6. Click "Create Repository"

**Using API:**
```bash
curl -X POST http://localhost:3000/api/repos/myuser/my-logic-project \
  -H "Content-Type: application/json" \
  -d '{"description": "My Logic Pro project"}'
```

### Explore the Web UI

**Repository List:**
- Browse all repositories
- Search by name or namespace
- Create new repositories

**Repository Detail:**
- View commit history
- Check lock status
- See Logic Pro metadata (BPM, sample rate, key)
- Manage locks (acquire/release)

### Test the API

```bash
# List all repositories
curl http://localhost:3000/api/repos

# Get repository info
curl http://localhost:3000/api/repos/myuser/my-logic-project

# List commits
curl http://localhost:3000/api/repos/myuser/my-logic-project/commits

# Acquire lock
curl -X POST http://localhost:3000/api/repos/myuser/my-logic-project/locks/acquire \
  -H "Content-Type: application/json" \
  -d '{"timeout_hours": 2}'

# Check lock status
curl http://localhost:3000/api/repos/myuser/my-logic-project/locks/status

# Release lock
curl -X POST http://localhost:3000/api/repos/myuser/my-logic-project/locks/release
```

---

## Configuration

Edit `.env` file in `auxin-server/`:

```bash
# Server settings
SYNC_DIR=./.local-data          # Where repositories are stored
OXEN_SERVER_PORT=3000           # Server port
OXEN_SERVER_HOST=127.0.0.1      # Bind address

# Logging
RUST_LOG=info,auxin_server=debug

# Features
ENABLE_REDIS_LOCKS=false
```

Restart server after changes:
```bash
# Ctrl+C to stop, then:
./run-local.sh
```

---

## Development Mode

**Backend (Rust):**
```bash
# Make changes to src/

# Rebuild and run
cargo build --release
./run-local.sh
```

**Frontend (React with Hot Reload):**
```bash
# Terminal 1: Run backend
./run-local.sh

# Terminal 2: Frontend dev server
cd frontend
npm run dev
# Opens http://localhost:5173 with instant updates!
```

---

## Testing

Run automated tests:

```bash
# Make sure server is running
./run-local.sh &

# Run test suite
./test-deployment.sh
```

Expected output:
```
✓ Passed: 11
✗ Failed: 0
✓ All tests passed!
```

---

## Common Issues

### "Rust is not installed"

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### "Node.js not found"

Option 1: Install Node.js from https://nodejs.org/

Option 2: Skip frontend (API still works):
```bash
# Just build backend
cargo build --release
./run-local.sh
# Web UI won't be available, but API works fine
```

### "Port 3000 already in use"

```bash
# Kill process on port 3000
lsof -ti:3000 | xargs kill -9

# Or change port in .env
echo "OXEN_SERVER_PORT=3001" >> .env
```

### Frontend not loading

```bash
# Rebuild frontend
cd frontend
npm install
npm run build
cd ..
./run-local.sh
```

---

## Directory Structure

After deployment:

```
auxin-server/
├── .local-data/          # Repository data (auto-created)
│   └── demo/
│       └── my-logic-project/
├── frontend/
│   └── dist/            # Built web UI
├── target/
│   └── release/
│       └── auxin-server # Server binary
├── .env                 # Configuration
├── deploy-local.sh      # Deployment script
├── run-local.sh         # Start script
└── test-deployment.sh   # Test script
```

---

## What's Next?

### Learn More

- **Full Deployment Guide:** [DEPLOYMENT.md](DEPLOYMENT.md)
- **Frontend Development:** [FRONTEND_SETUP.md](FRONTEND_SETUP.md)
- **API Documentation:** [README.md#api-endpoints](README.md#api-endpoints)

### Deploy for Production

**macOS (with auto-start):**
```bash
cd scripts
./setup.sh
./start.sh
```

**Docker:**
```bash
docker-compose up -d
```

### Integrate with Logic Pro

1. Initialize repository in Logic Pro project folder
2. Use Auxin CLI to push changes to server
3. Manage locks via Web UI before editing
4. View commit history and metadata

---

## Support

**Questions?**
- Check [DEPLOYMENT.md](DEPLOYMENT.md) for detailed docs
- Review [Troubleshooting](DEPLOYMENT.md#troubleshooting)
- Open issue on [GitHub](https://github.com/jbacus/auxin)

**Happy music production with version control!** 🎵🚀

---

*Last updated: 2025-01-18*
