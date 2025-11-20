<div align="center">
  <img src="../assets/icon/icon-128.png" alt="Auxin Logo" width="96" height="96">
  <h1>Auxin Server (Oxen-Aligned)</h1>
  <p><strong>Self-hosted repository server for Auxin, aligned with Oxen.ai architecture</strong></p>
</div>

## ⚡ Quick Note: Build Modes

Auxin-server supports two build modes via feature flags:

| Mode | Status | Use Case |
|------|--------|----------|
| **`mock-oxen`** (default) | ✅ **Ready** | Development, testing, deployment - full HTTP API + Auxin features |
| **`full-oxen`** | 🟡 **WIP** | Full VCS operations (requires async refactoring for liboxen 0.38) |

**TL;DR:** The default build works perfectly for server infrastructure, locks, and metadata. VCS operations (commit, push, pull) return `501 Not Implemented`.

See [BUILD_MACOS_26.md](BUILD_MACOS_26.md) for details on building with either mode.

## Architecture

This version uses:
- ✅ **Actix Web** (same as Oxen)
- ✅ **liboxen 0.38** (optional, for full VCS operations)
- ✅ **Filesystem storage** with `.oxen` directories
- ✅ **Simple deployment** (one binary)
- ✅ **Feature flags** for flexible building
- ✅ **Modern Web UI** (React + TypeScript + Tailwind CSS)

**Replaces the complex PostgreSQL + Redis + MinIO stack with simple filesystem storage.**

## Quick Start

**Fastest way to get started (< 5 minutes):**

```bash
# 1. One-command deployment
./deploy-local.sh

# 2. Start server
./run-local.sh

# 3. Test it
curl http://localhost:3000/health
open http://localhost:3000  # Web UI
```

**See [QUICKSTART.md](QUICKSTART.md) for detailed quick start guide.**

## Deployment Options

Auxin-server supports multiple deployment methods:

### 1. Local Development (Recommended for Testing)

```bash
./deploy-local.sh  # Builds everything automatically
./run-local.sh     # Starts server
```

- ✅ No system installation required
- ✅ Runs from project directory
- ✅ Includes sample test data
- ✅ Perfect for development

### 2. Docker

```bash
cd frontend && npm install && npm run build && cd ..
docker-compose up -d
```

- ✅ Portable across platforms
- ✅ Isolated environment
- ✅ Easy scaling

### 3. Production (macOS)

```bash
cd scripts
./setup.sh   # Installs to /usr/local/bin
./start.sh   # Auto-starts with LaunchAgent
```

- ✅ System-wide installation
- ✅ Auto-start on boot
- ✅ Service management

**macOS 26.x users:** See [BUILD_MACOS_26.md](BUILD_MACOS_26.md) for platform-specific notes.

**See [DEPLOYMENT.md](DEPLOYMENT.md) for comprehensive deployment guide.**

## Testing

Run the automated test suite:

```bash
# Make sure server is running
./run-local.sh &

# Run tests
./test-deployment.sh
```

### Manual API Tests

```bash
# Health check
curl http://localhost:3000/health
# Expected: OK

# List repositories
curl http://localhost:3000/api/repos
# Expected: []

# Create a repository
curl -X POST http://localhost:3000/api/repos/myuser/myrepo \
  -H "Content-Type: application/json" \
  -d '{"description": "My first repository"}'

# Get repository info
curl http://localhost:3000/api/repos/myuser/myrepo

# List repositories again
curl http://localhost:3000/api/repos
# Expected: [{"namespace":"myuser","name":"myrepo",...}]

# Or use the Web UI
# Open http://localhost:3000 in your browser
```

## Web Frontend

The auxin-server includes a modern web frontend built with React, TypeScript, and Tailwind CSS.

### Features

- 📦 **Repository Management** - Browse and create repositories
- 📝 **Commit History** - View detailed commit timeline
- 🎵 **Logic Pro Metadata** - Display BPM, sample rate, key signature
- 🔒 **Lock Management** - Acquire/release distributed locks with visual status
- 🎨 **Modern UI** - Responsive design with dark mode support

### Development

```bash
cd frontend

# Install dependencies
npm install

# Start dev server (with hot reload)
npm run dev
# Opens at http://localhost:5173 with API proxy to :3000

# Build for production
npm run build
# Output: frontend/dist/
```

See [frontend/README.md](frontend/README.md) for detailed frontend documentation.

## Configuration

Edit `.env` file:

```bash
# Required
SYNC_DIR=/var/oxen/data           # Where repositories are stored
OXEN_SERVER_PORT=3000             # Server port (Oxen-compatible)
OXEN_SERVER_HOST=0.0.0.0          # Bind address

# Optional
RUST_LOG=info                     # Logging level
ENABLE_REDIS_LOCKS=false          # Use Redis for distributed locks
```

## Repository Structure

Repositories are stored as `.oxen` directories:

```
SYNC_DIR/
└── {namespace}/
    └── {repo_name}/
        └── .oxen/
            ├── config.toml        # Repository config
            ├── HEAD               # Current branch
            ├── history/           # Commit history
            ├── refs/              # Branch references
            │   └── heads/
            ├── tree/              # File tree snapshots
            ├── versions/          # Deduplicated blocks
            ├── metadata/          # Logic Pro metadata (Auxin extension)
            └── locks/             # Distributed locks (Auxin extension)
```

## API Endpoints

### Core
- `GET /health` - Health check
- `GET /api/repos` - List all repositories
- `POST /api/repos/{namespace}/{name}` - Create repository
- `GET /api/repos/{namespace}/{name}` - Get repository info

### VCS Operations (full-oxen feature required)
**Status:** Returns `501 Not Implemented` in mock-oxen mode (default)
- `POST /api/repos/{namespace}/{name}/push` - Push commits
- `GET /api/repos/{namespace}/{name}/pull` - Pull commits
- `GET /api/repos/{namespace}/{name}/commits` - List commits
- `POST /api/repos/{namespace}/{name}/add` - Stage files
- `POST /api/repos/{namespace}/{name}/commit` - Create commit
- `GET /api/repos/{namespace}/{name}/branches` - List branches
- `POST /api/repos/{namespace}/{name}/branches` - Create branch

### Auxin Extensions
- `GET /api/repos/{namespace}/{name}/metadata/{commit}` - Logic Pro metadata
- `POST /api/repos/{namespace}/{name}/locks/acquire` - Acquire lock
- `POST /api/repos/{namespace}/{name}/locks/release` - Release lock
- `GET /api/repos/{namespace}/{name}/locks/status` - Lock status

## Deployment

### Simple (One Binary)

```bash
# Build release
cargo build --release

# Copy binary
sudo cp target/release/auxin-server /usr/local/bin/

# Create systemd service
sudo tee /etc/systemd/system/auxin-server.service > /dev/null <<EOF
[Unit]
Description=Auxin Server
After=network.target

[Service]
Type=simple
User=auxin
Environment="SYNC_DIR=/var/oxen/data"
Environment="OXEN_SERVER_PORT=3000"
ExecStart=/usr/local/bin/auxin-server
Restart=always

[Install]
WantedBy=multi-user.target
EOF

# Start service
sudo systemctl daemon-reload
sudo systemctl enable auxin-server
sudo systemctl start auxin-server
```

### With Docker (Optional)

```bash
# Build image
docker build -t auxin-server:latest .

# Run
docker run -d \
  -p 3000:3000 \
  -v /var/oxen/data:/var/oxen/data \
  -e SYNC_DIR=/var/oxen/data \
  auxin-server:latest
```

## Comparison: v1 vs v2

| Feature | v1 (Original) | v2 (Oxen-Aligned) |
|---------|---------------|-------------------|
| **Web Framework** | Axum | Actix Web |
| **Core Library** | Custom | liboxen |
| **Storage** | PostgreSQL + S3 | Filesystem |
| **Deployment** | Multi-service (4 containers) | Single binary |
| **Development Time** | 24 weeks | 8 weeks |
| **Infrastructure Cost** | $100-400/month | $20-50/month |
| **Oxen Compatible** | No | Yes |
| **Lines of Code** | ~10,000 | ~2,000 |

## Development

### Running in Development

```bash
# Watch mode (auto-restart on changes)
cargo install cargo-watch
cargo watch -x run

# Run with debug logging
RUST_LOG=debug cargo run
```

### Running Tests

```bash
# Run all tests (default: mock-oxen)
cargo test

# Run with verbose output
cargo test -- --nocapture

# Run specific test suite
cargo test --test mock_repository_tests
cargo test --test feature_flag_tests
cargo test --test error_handling_tests

# Test coverage (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

**Test Coverage:** New tests added for mock implementation:
- `mock_repository_tests.rs`: Repository operations (17 tests)
- `feature_flag_tests.rs`: Feature flag behavior (5 tests)
- `error_handling_tests.rs`: Error handling (8 tests)

Total: **30+ integration tests** for mock mode functionality.

### Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy

# Check
cargo check
```

## Feature Flags

Auxin-server uses Cargo feature flags for flexible building:

```toml
[features]
default = ["mock-oxen"]              # Mock VCS (works everywhere)
full-oxen = ["liboxen"]              # Full Oxen (WIP - async refactoring)
mock-oxen = []                       # Mock implementation
redis-locks = ["redis"]              # Redis for distributed locks
web-ui = ["sqlx"]                    # PostgreSQL for web UI
```

Build with specific features:
```bash
# Default (mock mode)
cargo build --release

# Full Oxen (WIP)
cargo build --release --no-default-features --features full-oxen

# With Redis locks
cargo build --release --features redis-locks

# With Web UI
cargo build --release --features web-ui
```

## What Works in Mock Mode

✅ **Fully Functional:**
- HTTP server (Actix Web)
- All API endpoints
- Repository discovery
- **Auxin extensions** (locks, Logic Pro metadata) - **These are the core value!**
- Authentication & JWT tokens
- Configuration management
- LaunchAgent service management
- All deployment scripts

❌ **Returns 501 Not Implemented:**
- VCS operations (add, commit, push, pull, clone)
- Branch management (create, checkout, merge)

**Use Case:** Perfect for development, testing, and production deployment where Auxin-specific features (locks and metadata) are the primary value.

## Documentation

- **[BUILD_MACOS_26.md](BUILD_MACOS_26.md)**: macOS 26.x build instructions and feature flag details
- **[scripts/README.md](scripts/README.md)**: Deployment script documentation
- **[AUXIN_SERVER_PLAN.md](../AUXIN_SERVER_PLAN.md)**: Complete implementation plan

## License

MIT
