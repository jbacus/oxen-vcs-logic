# Auxin Server - Architecture (Oxen-Aligned)

**Project**: Self-hosted repository server for Auxin using liboxen
**Strategy**: Align with Oxen.ai's proven architecture
**Timeline**: 8 weeks (reduced from 24)
**Status**: Architecture Revision
**Last Updated**: 2025-11-17

---

## Executive Summary

### Strategic Pivot

After analyzing the actual Oxen.ai repository, we're **realigning our architecture** to match Oxen's proven design:

**Before (Auxin Server Original):**
- Axum web framework
- PostgreSQL + Redis + MinIO stack
- Custom database schema
- 24-week implementation
- High operational complexity

**After (Auxin Server Revised):**
- **Actix Web** (matches Oxen)
- **liboxen library** (reuse Oxen core)
- **Filesystem storage** with `.oxen` directories
- **8-week implementation**
- Simple one-binary deployment

### Why This Change?

1. **Reuse existing code**: liboxen handles repository logic
2. **Simpler deployment**: One binary vs. multi-service stack
3. **Oxen compatibility**: Can sync with Oxen Hub
4. **Lower maintenance**: File-based storage, simpler to debug
5. **Faster delivery**: 8 weeks instead of 24

---

## Architecture Overview

### System Design

```
┌──────────────────────────────────────────────────────────────┐
│              Auxin Server (Actix Web + liboxen)              │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐ │
│  │              Actix Web HTTP Server                     │ │
│  │  • Port 3000 (Oxen-compatible)                         │ │
│  │  • REST API endpoints                                  │ │
│  │  • Token authentication                                │ │
│  │  • CORS middleware                                     │ │
│  └────────────────┬───────────────────────────────────────┘ │
│                   │                                          │
│  ┌────────────────▼───────────────────────────────────────┐ │
│  │         Auxin Extensions Layer                         │ │
│  │  • Logic Pro metadata (BPM, sample rate, key)          │ │
│  │  • Distributed locking (file-based or Redis)           │ │
│  │  • Activity tracking                                   │ │
│  │  • Comment system                                      │ │
│  └────────────────┬───────────────────────────────────────┘ │
│                   │                                          │
│  ┌────────────────▼───────────────────────────────────────┐ │
│  │              liboxen Core                              │ │
│  │  • Repository management                               │ │
│  │  • Commit creation/retrieval                           │ │
│  │  • Push/pull protocol                                  │ │
│  │  • Block deduplication                                 │ │
│  │  • Merkle tree indexing                                │ │
│  └────────────────┬───────────────────────────────────────┘ │
└───────────────────┼──────────────────────────────────────────┘
                    │
       ┌────────────┴─────────────┐
       │                          │
┌──────▼──────┐          ┌────────▼────────┐
│  SYNC_DIR   │          │  Optional       │
│  (Filesystem)│          │  Redis          │
│             │          │  (Locks only)   │
│ /namespace/ │          └─────────────────┘
│   /repo/    │
│     /.oxen/ │
│       /history/
│       /refs/
│       /tree/
│       /versions/
└─────────────┘
```

### Technology Stack

**Backend:**
```toml
[dependencies]
actix-web = "4"           # Web framework (Oxen uses this)
actix-rt = "2"            # Async runtime
liboxen = "0.10"          # Core Oxen library
tokio = "1"               # Async runtime
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Optional (for Auxin extensions)
redis = { version = "0.24", optional = true }  # For distributed locks
sqlx = { version = "0.7", optional = true }    # For web UI metadata cache

# Utilities
uuid = "1.6"
chrono = "0.4"
anyhow = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

**Storage:**
- **Primary**: Filesystem (`.oxen` directories)
- **Optional**: Redis (distributed locks only)
- **Optional**: PostgreSQL (web UI metadata cache only)

---

## Implementation Plan (Revised)

### Phase 1: Core Server (Weeks 1-2)

**Goal**: Basic Actix server with liboxen integration

**Week 1: Server Setup**
```rust
// main.rs
use actix_web::{web, App, HttpServer};
use liboxen::{LocalRepository, RemoteRepository};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize server
    let sync_dir = std::env::var("SYNC_DIR")
        .unwrap_or_else(|_| "/var/oxen/data".to_string());

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(sync_dir.clone()))
            .route("/health", web::get().to(health_check))
            .route("/api/repos/{namespace}/{name}", web::post().to(create_repo))
            .route("/api/repos/{namespace}/{name}/push", web::post().to(push))
            .route("/api/repos/{namespace}/{name}/pull", web::get().to(pull))
    })
    .bind("0.0.0.0:3000")?
    .run()
    .await
}
```

**Week 2: Repository Operations**
- Implement create_repo using liboxen
- Implement push handler
- Implement pull handler
- Add authentication middleware
- Test with oxen CLI

**Deliverables:**
- ✅ Working Actix server on port 3000
- ✅ Can create repositories via API
- ✅ Basic push/pull working
- ✅ Compatible with oxen CLI

### Phase 2: Logic Pro Extensions (Weeks 3-4)

**Goal**: Add Auxin-specific features

**Week 3: Metadata Support**
```rust
// Extend commits with Logic Pro metadata
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct LogicProMetadata {
    bpm: Option<f64>,
    sample_rate: Option<i32>,
    key_signature: Option<String>,
    tags: Vec<String>,
}

// Store in .oxen/metadata/{commit_hash}.json
async fn store_metadata(
    repo: &LocalRepository,
    commit_hash: &str,
    metadata: LogicProMetadata,
) -> Result<()> {
    let metadata_path = repo.path()
        .join(".oxen/metadata")
        .join(format!("{}.json", commit_hash));

    tokio::fs::create_dir_all(metadata_path.parent().unwrap()).await?;
    tokio::fs::write(
        &metadata_path,
        serde_json::to_string_pretty(&metadata)?
    ).await?;

    Ok(())
}
```

**Week 4: Distributed Locking**
```rust
// Simple file-based locking (no Redis required initially)
use std::fs;
use std::time::SystemTime;

struct FileLock {
    lock_path: PathBuf,
}

impl FileLock {
    fn acquire(repo_path: &Path, timeout_hours: u64) -> Result<Self> {
        let lock_path = repo_path.join(".oxen/locks/project.lock");

        // Create lock file with metadata
        let lock_data = LockData {
            user: get_current_user(),
            machine_id: get_machine_id(),
            acquired_at: SystemTime::now(),
            expires_at: SystemTime::now() + Duration::from_secs(timeout_hours * 3600),
        };

        fs::write(&lock_path, serde_json::to_string(&lock_data)?)?;
        Ok(Self { lock_path })
    }

    fn release(self) -> Result<()> {
        fs::remove_file(&self.lock_path)?;
        Ok(())
    }
}
```

**Deliverables:**
- ✅ Logic Pro metadata storage/retrieval
- ✅ File-based distributed locking
- ✅ Lock heartbeat mechanism
- ✅ Lock status API

### Phase 3: Web UI & Polish (Weeks 5-8)

**Goal**: Lightweight web interface

**Week 5-6: Basic Web UI**
- Repository list view
- Commit history viewer
- File browser (read-only)
- Lock status display

**Week 7: Testing & Documentation**
- Integration tests with oxen CLI
- API documentation
- Deployment guide
- Performance benchmarking

**Week 8: Production Readiness**
- Monitoring setup
- Backup scripts
- Security hardening
- Launch! 🚀

---

## Comparison: Old vs New

| Aspect | Original Plan | Revised Plan |
|--------|---------------|--------------|
| **Timeline** | 24 weeks | 8 weeks |
| **Web Framework** | Axum | Actix Web |
| **Core Logic** | Custom | liboxen |
| **Storage** | PostgreSQL + S3 | Filesystem (.oxen) |
| **Cache** | Redis | Optional |
| **Deployment** | Docker Compose (4 services) | Single binary |
| **Infrastructure Cost** | $100-400/month | $20-50/month |
| **Lines of Code** | ~10,000 | ~2,000 |
| **Complexity** | High | Low |
| **Oxen Compatibility** | None | Full |
| **Maintenance** | High | Low |

---

## Storage Architecture

### .oxen Directory Structure

Following Oxen's proven design:

```
SYNC_DIR/
└── {namespace}/
    └── {repo_name}/
        ├── .oxen/
        │   ├── config.toml          # Repository config
        │   ├── HEAD                 # Current branch
        │   ├── history/             # Commit history
        │   │   └── {commit_hash}    # Commit objects
        │   ├── refs/                # Branch references
        │   │   └── heads/
        │   │       └── main
        │   ├── tree/                # File tree snapshots
        │   │   └── {tree_hash}      # Tree objects
        │   ├── versions/            # Deduplicated blocks
        │   │   └── {block_hash}     # Content blocks
        │   ├── metadata/            # Auxin extension
        │   │   └── {commit_hash}.json  # Logic Pro metadata
        │   └── locks/               # Auxin extension
        │       └── project.lock     # Distributed lock
        └── (no working directory - server only stores .oxen)
```

### Benefits of This Approach

1. **Reuse liboxen logic**: Don't reinvent the wheel
2. **Simple backups**: Just tar the SYNC_DIR
3. **Easy debugging**: Inspect files directly
4. **Oxen compatible**: Can migrate repos between servers
5. **Lower overhead**: No database queries

---

## API Endpoints (Simplified)

### Core (from liboxen)
```
POST   /api/repos/{namespace}/{name}          # Create repository
GET    /api/repos/{namespace}/{name}          # Get repository info
POST   /api/repos/{namespace}/{name}/push     # Push commits
GET    /api/repos/{namespace}/{name}/pull     # Pull commits
GET    /api/repos/{namespace}/{name}/commits  # List commits
```

### Auxin Extensions
```
GET    /api/repos/{namespace}/{name}/metadata/{commit}  # Logic Pro metadata
POST   /api/repos/{namespace}/{name}/locks/acquire      # Acquire lock
POST   /api/repos/{namespace}/{name}/locks/release      # Release lock
GET    /api/repos/{namespace}/{name}/locks/status       # Lock status
```

### Web UI (Optional)
```
GET    /                                       # Repository list
GET    /{namespace}/{name}                    # Repository view
GET    /{namespace}/{name}/commits            # Commit history
GET    /{namespace}/{name}/files/{commit}     # File browser
```

---

## Deployment

### Simple Deployment (One Binary)

```bash
# 1. Build server
cargo build --release

# 2. Set environment
export SYNC_DIR=/var/oxen/data
export OXEN_SERVER_PORT=3000

# 3. Run
./target/release/auxin-server

# That's it! No Docker Compose needed.
```

### Optional: Add Redis for Distributed Locks

```bash
# Only if you need distributed locking across multiple server instances
docker run -d -p 6379:6379 redis:7-alpine

export REDIS_URL=redis://localhost:6379
./target/release/auxin-server --enable-redis-locks
```

### Optional: Add PostgreSQL for Web UI

```bash
# Only if you want web UI with search/analytics
docker run -d -p 5432:5432 \
  -e POSTGRES_PASSWORD=password \
  postgres:16

export DATABASE_URL=postgres://postgres:password@localhost/auxin
./target/release/auxin-server --enable-web-ui
```

---

## Configuration

### Minimal Config (.env)

```bash
# Required
SYNC_DIR=/var/oxen/data
OXEN_SERVER_PORT=3000

# Optional
RUST_LOG=info
ENABLE_REDIS_LOCKS=false
ENABLE_WEB_UI=false
```

### Full Config (with all features)

```bash
# Server
SYNC_DIR=/var/oxen/data
OXEN_SERVER_PORT=3000
OXEN_SERVER_HOST=0.0.0.0

# Authentication
AUTH_TOKEN_SECRET=your_secret_here
AUTH_TOKEN_EXPIRY_HOURS=24

# Optional: Redis (distributed locks)
ENABLE_REDIS_LOCKS=true
REDIS_URL=redis://localhost:6379

# Optional: PostgreSQL (web UI)
ENABLE_WEB_UI=true
DATABASE_URL=postgres://user:pass@localhost/auxin

# Logging
RUST_LOG=info,auxin_server=debug
```

---

## Migration from Original Plan

### What to Keep

1. **Database migrations** → Convert to optional web UI cache
2. **Error handling** → Keep the error types
3. **Authentication concepts** → Simplify to token-based like Oxen
4. **Documentation** → Update to reflect new architecture

### What to Replace

1. **Axum → Actix Web** (simple find/replace)
2. **Custom storage → liboxen** (remove storage/ module)
3. **PostgreSQL models → .oxen files** (simplify)
4. **Docker Compose → Optional** (single binary first)

### Migration Script

```bash
# Backup current work
git checkout -b backup/original-design

# Start fresh aligned branch
git checkout -b feature/oxen-aligned-server

# Create new project structure
mkdir auxin-server
cd auxin-server

# Copy only what's needed
cp ../auxin-server/Cargo.toml .  # Update dependencies
cp ../auxin-server/README.md .   # Update docs

# Implement new architecture
# (See implementation guide below)
```

---

## Next Steps

### This Week
1. ✅ Study liboxen API (read source code)
2. ✅ Review Oxen server implementation
3. ⏳ Create new Cargo.toml with liboxen + actix-web
4. ⏳ Implement basic Actix server
5. ⏳ Test push/pull with oxen CLI

### Next Week
1. Add Logic Pro metadata support
2. Implement file-based locking
3. Write integration tests
4. Documentation

### Month 2
1. Optional: Web UI
2. Optional: Redis locks
3. Production deployment
4. Launch!

---

## Cost Savings

**Infrastructure:**
- Before: $100-400/month (Postgres + Redis + MinIO + Compute)
- After: $20-50/month (Just compute + disk)
- **Savings: $80-350/month**

**Development:**
- Before: 24 weeks × 40 hours = 960 hours
- After: 8 weeks × 40 hours = 320 hours
- **Savings: 640 hours (67% reduction)**

---

## Summary

**We're pivoting to align with Oxen's proven architecture:**

✅ **Simpler**: One binary vs. multi-service stack
✅ **Faster**: 8 weeks vs. 24 weeks
✅ **Cheaper**: $20-50/mo vs. $100-400/mo
✅ **Compatible**: Works with existing Oxen tools
✅ **Maintainable**: Less code, less complexity

**Trade-offs we accept:**
⚠️ Filesystem storage (vs. S3) - but Oxen is adding S3 support anyway
⚠️ Vertical scaling (vs. horizontal) - sufficient for most use cases
⚠️ Simpler web UI (vs. full SaaS) - can add later if needed

**This is the right architecture for a self-hosted repository server.**

---

*Last Updated: 2025-11-17*
*Version: 2.0 (Oxen-Aligned)*
