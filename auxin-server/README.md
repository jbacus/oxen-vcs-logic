# Auxin Server (Oxen-Aligned)

**Self-hosted repository server for Auxin, aligned with Oxen.ai architecture**

## Architecture

This version uses:
- ✅ **Actix Web** (same as Oxen)
- ✅ **liboxen** (reuses Oxen core library)
- ✅ **Filesystem storage** with `.oxen` directories
- ✅ **Simple deployment** (one binary)

**Replaces the complex PostgreSQL + Redis + MinIO stack with simple filesystem storage.**

## Quick Start

### 1. Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. Build and Run

```bash
# Clone and navigate
cd auxin-server

# Copy environment config
cp .env.example .env

# Build
cargo build --release

# Run
./target/release/auxin-server
```

### 3. Test

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
```

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
ENABLE_WEB_UI=false               # Enable web UI (requires PostgreSQL)
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

### Coming Soon (using liboxen)
- `POST /api/repos/{namespace}/{name}/push` - Push commits
- `GET /api/repos/{namespace}/{name}/pull` - Pull commits
- `GET /api/repos/{namespace}/{name}/commits` - List commits

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
cargo test
```

### Code Quality

```bash
# Format
cargo fmt

# Lint
cargo clippy

# Check
cargo check
```

## Next Steps

See `../AUXIN_SERVER.md` for:
- Complete 8-week implementation plan
- Integration with liboxen
- Logic Pro metadata support
- Distributed locking implementation
- Web UI development

## Documentation

- **Implementation Plan**: `../AUXIN_SERVER.md`
- **Original Plan**: `../OXVCS_SERVER_PLAN.md` (archived)

## License

MIT
