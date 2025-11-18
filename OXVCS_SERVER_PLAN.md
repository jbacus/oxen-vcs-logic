# OxVCS Server: Complete Development Plan

**Project**: Self-hosted repository server for OxVCS (Logic Pro version control)
**Goal**: Build an Oxen.ai-compatible server for secure, collaborative Logic Pro project hosting
**Timeline**: 24 weeks (6 months)
**Status**: Planning Phase
**Last Updated**: 2025-11-17

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [Feature Requirements](#feature-requirements)
3. [Technical Architecture](#technical-architecture)
4. [Development Phases](#development-phases)
5. [Project Structure](#project-structure)
6. [Implementation Details](#implementation-details)
7. [Testing Strategy](#testing-strategy)
8. [Deployment Guide](#deployment-guide)
9. [Maintenance & Operations](#maintenance--operations)

---

## Executive Summary

### What We're Building

**OxVCS Server** is a self-hosted Git-like repository server optimized for Logic Pro projects. It provides:

- **Repository hosting** with block-level deduplication (10-100x more efficient than Git-LFS)
- **Push/Pull protocol** compatible with existing OxVCS CLI
- **Distributed locking** for collision-free collaboration
- **Web UI** for browsing commits, managing teams, and viewing activity
- **REST API** for programmatic access
- **Real-time WebSocket** notifications for team awareness

### Why Build This?

**Current State**: OxVCS relies on Oxen Hub (external SaaS)
- ✅ Works well for individuals
- ❌ External dependency (no control over data)
- ❌ Can't customize for Logic Pro workflows
- ❌ Monthly costs per user
- ❌ Can't run on-premise

**Future State**: OxVCS Server (self-hosted)
- ✅ Full data control
- ✅ Custom Logic Pro features
- ✅ One-time infrastructure cost
- ✅ On-premise deployment option
- ✅ Unlimited users

### Success Metrics

| Metric | Target | Measurement |
|--------|--------|-------------|
| **Push Speed** | <2min for 10GB project | Benchmark test |
| **Pull Speed** | <30sec for 10GB project (shallow) | Benchmark test |
| **Lock Acquisition** | <500ms | API response time |
| **Web UI Load Time** | <2sec | Lighthouse score |
| **Storage Efficiency** | 70%+ deduplication | Actual vs raw size |
| **Uptime** | 99.9% | Monitoring |

---

## Feature Requirements

### Core Features (Oxen.ai Parity)

Based on Oxen.ai's capabilities, we need to support:

#### 1. Repository Management

**CLI Commands:**
```bash
oxen init                    # Initialize local repository
oxen clone <url>             # Clone from server
oxen clone --shallow <url>   # Clone metadata only (fast)
oxen remote add origin <url> # Configure remote
oxen remote -v               # List remotes
```

**Server Features:**
- Repository CRUD (create, read, update, delete)
- Public/private visibility
- Repository metadata (size, commit count, last activity)
- Default branch configuration
- Archive/unarchive repositories

#### 2. Version Control Operations

**CLI Commands:**
```bash
oxen add <file>         # Stage file
oxen add .              # Stage all changes
oxen status             # View working directory status
oxen commit -m "msg"    # Create commit
oxen log                # View commit history
oxen log -n 10          # Limit to 10 commits
oxen restore <file>     # Restore file to HEAD
oxen rm <file>          # Remove file
```

**Server Features:**
- Commit storage and retrieval
- Commit metadata (author, timestamp, message, parent)
- File tree tracking
- Diff computation
- Binary file support
- Large file handling (>1GB)

#### 3. Collaboration (Push/Pull)

**CLI Commands:**
```bash
oxen push origin main    # Push commits to server
oxen pull origin main    # Pull commits from server
oxen fetch              # Fetch updates without merging
```

**Server Features:**
- Push protocol (receive commits + blocks)
- Pull protocol (send commits + blocks)
- Conflict detection
- Presigned URL generation for S3 uploads
- Block-level deduplication
- Missing block identification
- Bandwidth optimization

#### 4. Branching

**CLI Commands:**
```bash
oxen branch                  # List branches
oxen branch <name>          # Create branch
oxen checkout <name>        # Switch branch
oxen checkout -b <name>     # Create and switch
oxen branch -d <name>       # Delete branch
```

**Server Features:**
- Branch CRUD operations
- Branch HEAD tracking
- Merge base computation
- Protected branches
- Default branch configuration

#### 5. Advanced Features (Oxen.ai-specific)

**Remote Workspace:**
```bash
oxen clone --shallow <url>   # Metadata-only clone
# Work with files without downloading them
oxen workspace create        # Create remote workspace
oxen workspace sync          # Sync workspace changes
```

**Server Features:**
- Workspace management
- Remote file access
- Lazy file downloading
- Workspace isolation

**DataFrame Support** (Future Phase):
```bash
oxen info <file.csv>         # Show CSV metadata
# Query CSV files via SQL (server-side)
```

**Server Features:**
- Index CSV/Parquet files into DuckDB
- SQL query endpoint
- DataFrame editing API
- Row-level versioning

#### 6. Authentication & Authorization

**CLI Commands:**
```bash
oxen config user.name "John"
oxen config user.email "john@example.com"
# API key stored in ~/.oxen/user_config.toml
```

**Server Features:**
- User registration/login
- API key generation
- JWT token authentication
- Password hashing (Argon2)
- Session management
- OAuth2 support (future)

#### 7. Team Collaboration

**Web UI Features:**
- Repository access control (public/private/internal)
- Collaborator management
- Team creation
- Role-based permissions (owner/admin/write/read)
- Activity feed
- Commit comments
- User profiles

#### 8. Web Dashboard

**Pages:**
- Repository browser
- Commit history viewer
- File explorer with preview
- Diff viewer (text files)
- Team management
- User settings
- Activity feed
- Search (repositories, commits, users)

---

## Technical Architecture

### System Components

```
┌─────────────────────────────────────────────────────────────────────┐
│                         Load Balancer (Nginx)                       │
│                     TLS Termination, Rate Limiting                  │
└────────────────────────────────┬────────────────────────────────────┘
                                 │
        ┌────────────────────────┼────────────────────────┐
        │                        │                        │
┌───────▼────────┐      ┌────────▼────────┐     ┌───────▼────────┐
│  Web Frontend  │      │   API Server    │     │  WS Server     │
│   (Next.js)    │      │  (Rust/Axum)    │     │ (Rust/Tokio)   │
│  Port 3000     │      │   Port 8080     │     │  Port 8081     │
└────────────────┘      └────────┬────────┘     └───────┬────────┘
                                 │                      │
              ┌──────────────────┼──────────────────────┤
              │                  │                      │
         ┌────▼─────┐      ┌─────▼─────┐        ┌─────▼──────┐
         │PostgreSQL│      │   Redis   │        │  MinIO/S3  │
         │          │      │           │        │            │
         │ Metadata │      │ Cache +   │        │ Block      │
         │ Users    │      │ Sessions  │        │ Storage    │
         │ Repos    │      │ Locks     │        │ Deduped    │
         └──────────┘      └───────────┘        └────────────┘
```

### Technology Stack

**Backend (Rust):**
```toml
[dependencies]
axum = "0.7"                  # Web framework
tokio = { version = "1", features = ["full"] }
sqlx = "0.7"                  # PostgreSQL driver
redis = { version = "0.24", features = ["tokio-comp"] }
aws-sdk-s3 = "1.0"            # S3/MinIO client
tower = "0.4"                 # Middleware
tower-http = "0.5"            # HTTP middleware
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1.6", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
jsonwebtoken = "9"            # JWT
argon2 = "0.5"                # Password hashing
sha2 = "0.10"                 # SHA-256 for content addressing
async-trait = "0.1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
tokio-tungstenite = "0.21"    # WebSocket
futures = "0.3"
bytes = "1"
```

**Frontend (TypeScript/React):**
```json
{
  "dependencies": {
    "next": "14",
    "react": "18",
    "react-dom": "18",
    "typescript": "5",
    "@tanstack/react-query": "5",
    "axios": "1",
    "socket.io-client": "4",
    "tailwindcss": "3",
    "@shadcn/ui": "latest",
    "zustand": "4",
    "react-markdown": "9",
    "date-fns": "3"
  }
}
```

**Infrastructure:**
- **Database**: PostgreSQL 16
- **Cache**: Redis 7
- **Storage**: MinIO (S3-compatible) or AWS S3
- **Reverse Proxy**: Nginx
- **Container**: Docker + Docker Compose
- **Orchestration**: Kubernetes (production)
- **CI/CD**: GitHub Actions
- **Monitoring**: Prometheus + Grafana
- **Logging**: Loki
- **Tracing**: Jaeger

### Data Models

**Key Entities:**
```rust
// User
struct User {
    id: Uuid,
    username: String,
    email: String,
    password_hash: String,
    created_at: DateTime<Utc>,
}

// Repository
struct Repository {
    id: Uuid,
    namespace: String,      // username or org
    name: String,
    owner_id: Uuid,
    visibility: Visibility,  // Public, Private, Internal
    storage_path: String,
    size_bytes: i64,
    default_branch: String,
}

// Commit
struct Commit {
    id: Uuid,
    repository_id: Uuid,
    commit_hash: String,     // SHA-256
    parent_hash: Option<String>,
    message: String,
    author_name: String,
    author_email: String,
    branch: String,
    created_at: DateTime<Utc>,
    // Logic Pro metadata
    metadata: CommitMetadata,
}

// Block (content-addressed storage)
struct Block {
    hash: String,            // SHA-256 of content
    size: u64,
    storage_key: String,     // S3 key
    created_at: DateTime<Utc>,
}

// Lock
struct Lock {
    id: Uuid,
    repository_id: Uuid,
    project_path: String,
    locked_by_user_id: Uuid,
    machine_id: String,
    acquired_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    last_heartbeat: DateTime<Utc>,
}
```

### Storage Strategy

**Block-Level Deduplication:**

1. **Chunking**: Files split into 4MB blocks
2. **Content Addressing**: SHA-256 hash per block
3. **Deduplication**: Identical blocks stored once
4. **Storage Layout**:
   ```
   s3://oxvcs-storage/
   ├── blocks/
   │   ├── ab/cd/abcd1234...  # Content-addressed blocks
   │   └── ef/gh/efgh5678...
   └── repositories/
       └── {namespace}/
           └── {repo}/
               ├── commits/    # Commit metadata
               └── trees/      # File tree manifests
   ```

**Example Deduplication:**
```
Project A (50GB) → Blocks: {A1, A2, A3, A4, A5}
Project B (50GB) → Blocks: {A1, A2, B3, B4, B5}  # Shares A1, A2
Storage Used: Not 100GB, only 70GB (30% savings)
```

### API Endpoints

**Authentication:**
```
POST   /api/v1/auth/register
POST   /api/v1/auth/login
POST   /api/v1/auth/logout
POST   /api/v1/auth/refresh
GET    /api/v1/auth/me
POST   /api/v1/auth/api-keys
DELETE /api/v1/auth/api-keys/{id}
```

**Repositories:**
```
GET    /api/v1/repositories
POST   /api/v1/repositories
GET    /api/v1/repositories/{namespace}/{name}
PATCH  /api/v1/repositories/{namespace}/{name}
DELETE /api/v1/repositories/{namespace}/{name}
GET    /api/v1/repositories/{namespace}/{name}/commits
GET    /api/v1/repositories/{namespace}/{name}/commits/{hash}
GET    /api/v1/repositories/{namespace}/{name}/tree/{hash}
GET    /api/v1/repositories/{namespace}/{name}/branches
POST   /api/v1/repositories/{namespace}/{name}/branches
DELETE /api/v1/repositories/{namespace}/{name}/branches/{name}
```

**Sync Protocol:**
```
POST   /api/v1/repositories/{namespace}/{name}/push
GET    /api/v1/repositories/{namespace}/{name}/pull
POST   /api/v1/repositories/{namespace}/{name}/blocks/check
GET    /api/v1/repositories/{namespace}/{name}/blocks/{hash}
```

**Locks:**
```
POST   /api/v1/repositories/{namespace}/{name}/locks/acquire
POST   /api/v1/repositories/{namespace}/{name}/locks/release
POST   /api/v1/repositories/{namespace}/{name}/locks/heartbeat
GET    /api/v1/repositories/{namespace}/{name}/locks/status
POST   /api/v1/repositories/{namespace}/{name}/locks/break
```

**Collaboration:**
```
GET    /api/v1/repositories/{namespace}/{name}/activity
POST   /api/v1/repositories/{namespace}/{name}/comments
GET    /api/v1/repositories/{namespace}/{name}/comments/{commit_hash}
GET    /api/v1/repositories/{namespace}/{name}/collaborators
POST   /api/v1/repositories/{namespace}/{name}/collaborators
DELETE /api/v1/repositories/{namespace}/{name}/collaborators/{user_id}
```

**WebSocket:**
```
WS     /api/v1/repositories/{namespace}/{name}/events
```

---

## Development Phases

### Phase 1: Foundation (Weeks 1-4)

**Goal**: Basic server with auth, repos, and database

**Deliverables:**
- ✅ Project structure and build system
- ✅ PostgreSQL schema and migrations
- ✅ User registration/login with JWT
- ✅ API key generation
- ✅ Repository CRUD operations
- ✅ Basic error handling
- ✅ Docker Compose development environment
- ✅ Unit tests (>80% coverage)

**Tasks:**

**Week 1: Project Setup**
```bash
# Create workspace
mkdir -p oxvcs-server
cd oxvcs-server

# Initialize Rust project
cargo init --name oxvcs-server

# Add core dependencies
cargo add axum tokio sqlx tower tower-http
cargo add serde serde_json uuid chrono
cargo add jsonwebtoken argon2 anyhow thiserror
cargo add tracing tracing-subscriber

# Create module structure
mkdir -p src/{api,auth,db,models,storage,error}

# Initialize database
mkdir migrations
# Write SQL migrations (see database schema below)

# Docker setup
touch docker-compose.yml Dockerfile
```

**Week 2: Authentication System**
- Implement `src/auth/mod.rs`
  - User registration with email validation
  - Password hashing (Argon2)
  - JWT token generation/verification
  - API key generation with scopes
- Implement `src/models/user.rs`
- Write auth integration tests
- Create auth middleware for protected routes

**Week 3: Repository Management**
- Implement `src/api/repositories.rs`
  - POST /repositories (create)
  - GET /repositories (list)
  - GET /repositories/{ns}/{name} (get)
  - PATCH /repositories/{ns}/{name} (update)
  - DELETE /repositories/{ns}/{name} (delete)
- Implement `src/models/repository.rs`
- Add authorization checks (owner/collaborator)
- Write repository integration tests

**Week 4: Database & Testing**
- Finalize PostgreSQL migrations
- Set up SQLx for compile-time query checking
- Write comprehensive unit tests
- Set up CI pipeline (GitHub Actions)
- Create developer documentation

### Phase 2: Storage & Sync Protocol (Weeks 5-10)

**Goal**: Implement push/pull with deduplication

**Deliverables:**
- ✅ S3/MinIO integration
- ✅ Block-level deduplication engine
- ✅ Push endpoint (receive commits + blocks)
- ✅ Pull endpoint (send commits + blocks)
- ✅ Presigned URL generation
- ✅ Client integration in OxVCS CLI

**Tasks:**

**Week 5: S3 Storage Backend**
- Implement `src/storage/s3.rs`
  - S3 client initialization
  - Upload/download blocks
  - Presigned URL generation
  - Error handling for network failures
- Set up MinIO in Docker Compose
- Write storage integration tests

**Week 6-7: Deduplication Engine**
- Implement `src/storage/deduplication.rs`
  - File chunking (4MB blocks)
  - SHA-256 content addressing
  - Block registry (PostgreSQL)
  - Duplicate detection
- Implement `src/storage/block.rs`
  - Block CRUD operations
  - Reference counting
  - Garbage collection logic
- Write deduplication tests with real files

**Week 8-9: Push/Pull Protocol**
- Implement `src/api/sync.rs`
  - POST /push endpoint
    - Receive commit metadata
    - Identify missing blocks
    - Generate upload URLs
    - Atomically update repository HEAD
  - GET /pull endpoint
    - Send commits since last pull
    - Generate download URLs for blocks
    - Support shallow clones (metadata only)
- Implement commit storage in S3
- Write sync integration tests

**Week 10: CLI Integration**
- Update `OxVCS-CLI-Wrapper/src/cloud_client.rs`
  - Push implementation
  - Pull implementation
  - Error handling
  - Progress reporting
- Test with real Logic Pro projects
- Performance benchmarking

### Phase 3: Distributed Locking (Weeks 11-12)

**Goal**: Collaborative locking system

**Deliverables:**
- ✅ Lock acquisition/release API
- ✅ Redis-backed lock cache
- ✅ Heartbeat mechanism
- ✅ Automatic expiration
- ✅ Race condition handling

**Tasks:**

**Week 11: Lock Service**
- Implement `src/api/locks.rs`
  - POST /locks/acquire
  - POST /locks/release
  - POST /locks/heartbeat
  - GET /locks/status
  - POST /locks/break (admin only)
- Implement `src/locks/manager.rs`
  - PostgreSQL transactions for atomicity
  - Redis cache for fast lookups
  - Lock expiration worker
  - Stale lock detection
- Write lock integration tests

**Week 12: Testing & Integration**
- Race condition testing (concurrent acquires)
- Network partition testing
- Heartbeat failure scenarios
- CLI integration
- Load testing (100 concurrent lock attempts)

### Phase 4: Web Frontend (Weeks 13-18)

**Goal**: Browser-based UI for management

**Deliverables:**
- ✅ Next.js application
- ✅ Authentication flow
- ✅ Repository browser
- ✅ Commit history viewer
- ✅ File explorer
- ✅ Team management
- ✅ Activity feed

**Tasks:**

**Week 13: Frontend Setup**
```bash
npx create-next-app@latest oxvcs-web
cd oxvcs-web
npm install @tanstack/react-query axios
npm install tailwindcss @shadcn/ui
npm install socket.io-client zustand
```

**Week 14: Authentication Pages**
- `/login` - Login form
- `/register` - Registration form
- `/settings` - User settings, API keys
- Auth context with JWT storage
- Protected route wrapper

**Week 15-16: Repository Views**
- `/` - Dashboard (repository list)
- `/[namespace]/[repo]` - Repository overview
- `/[namespace]/[repo]/commits` - Commit history
- `/[namespace]/[repo]/tree/[hash]` - File explorer
- `/[namespace]/[repo]/commit/[hash]` - Commit detail with diff

**Week 17: Team & Collaboration**
- `/[namespace]/[repo]/settings` - Repository settings
- `/[namespace]/[repo]/collaborators` - Manage access
- `/[namespace]/[repo]/activity` - Activity feed
- Comment system on commits
- Lock status indicator

**Week 18: Polish & Testing**
- Responsive design (mobile-friendly)
- Loading states and error handling
- E2E tests (Playwright)
- Accessibility audit
- Performance optimization

### Phase 5: Real-Time Features (Weeks 19-20)

**Goal**: WebSocket events and notifications

**Deliverables:**
- ✅ WebSocket server
- ✅ Event broadcasting
- ✅ Desktop notifications
- ✅ Real-time activity updates

**Tasks:**

**Week 19: WebSocket Backend**
- Implement `src/websocket/mod.rs`
  - Connection handling
  - Room-based subscriptions (per repository)
  - Event broadcasting
  - Authentication
- Integrate with lock service
- Integrate with commit service

**Week 20: Frontend Integration**
- Socket.IO client in Next.js
- Real-time activity feed updates
- Lock status updates
- Browser notifications
- Presence indicators ("3 people online")

### Phase 6: Advanced Features (Weeks 21-24)

**Goal**: Production readiness and nice-to-haves

**Deliverables:**
- ✅ Search functionality
- ✅ Workspace support (shallow clones)
- ✅ Audit logging
- ✅ Monitoring & observability
- ✅ Performance optimization
- ✅ Documentation

**Tasks:**

**Week 21: Search & Discovery**
- Full-text search for repositories
- Commit message search
- User search
- Elasticsearch integration (optional)

**Week 22: Workspace API**
- Remote workspace CRUD
- Lazy file downloading
- Workspace sync
- Conflict resolution

**Week 23: Production Readiness**
- Prometheus metrics
- Grafana dashboards
- Distributed tracing (Jaeger)
- Error tracking (Sentry)
- Rate limiting
- DDoS protection

**Week 24: Documentation & Launch**
- API documentation (OpenAPI/Swagger)
- Deployment guide
- Operations runbook
- Security audit
- Load testing
- Launch 🚀

---

## Project Structure

```
oxvcs-server/
├── Cargo.toml
├── Cargo.lock
├── Dockerfile
├── docker-compose.yml
├── .env.example
├── README.md
│
├── migrations/
│   ├── 20250101_create_users.sql
│   ├── 20250102_create_repositories.sql
│   ├── 20250103_create_commits.sql
│   ├── 20250104_create_blocks.sql
│   ├── 20250105_create_locks.sql
│   └── 20250106_create_activity.sql
│
├── src/
│   ├── main.rs                   # Server entry point
│   ├── lib.rs                    # Library exports
│   ├── config.rs                 # Configuration
│   │
│   ├── api/
│   │   ├── mod.rs
│   │   ├── auth.rs               # Auth endpoints
│   │   ├── repositories.rs       # Repo CRUD
│   │   ├── commits.rs            # Commit operations
│   │   ├── sync.rs               # Push/pull protocol
│   │   ├── locks.rs              # Lock management
│   │   ├── collaborators.rs     # Team management
│   │   ├── activity.rs           # Activity feed
│   │   └── search.rs             # Search API
│   │
│   ├── auth/
│   │   ├── mod.rs
│   │   ├── jwt.rs                # JWT generation/validation
│   │   ├── password.rs           # Password hashing
│   │   ├── api_key.rs            # API key management
│   │   └── middleware.rs         # Auth middleware
│   │
│   ├── db/
│   │   ├── mod.rs
│   │   ├── users.rs              # User queries
│   │   ├── repositories.rs       # Repo queries
│   │   ├── commits.rs            # Commit queries
│   │   ├── blocks.rs             # Block registry
│   │   ├── locks.rs              # Lock queries
│   │   └── activity.rs           # Activity queries
│   │
│   ├── models/
│   │   ├── mod.rs
│   │   ├── user.rs
│   │   ├── repository.rs
│   │   ├── commit.rs
│   │   ├── block.rs
│   │   ├── lock.rs
│   │   └── activity.rs
│   │
│   ├── storage/
│   │   ├── mod.rs
│   │   ├── s3.rs                 # S3 client
│   │   ├── deduplication.rs     # Chunking + dedup
│   │   ├── block.rs              # Block operations
│   │   └── garbage_collection.rs # GC for orphaned blocks
│   │
│   ├── locks/
│   │   ├── mod.rs
│   │   ├── manager.rs            # Lock manager
│   │   ├── heartbeat.rs          # Heartbeat worker
│   │   └── expiration.rs         # Expiration worker
│   │
│   ├── websocket/
│   │   ├── mod.rs
│   │   ├── server.rs             # WebSocket server
│   │   ├── events.rs             # Event types
│   │   └── rooms.rs              # Room management
│   │
│   ├── workers/
│   │   ├── mod.rs
│   │   ├── lock_expiration.rs   # Background worker
│   │   └── garbage_collection.rs # GC worker
│   │
│   ├── middleware/
│   │   ├── mod.rs
│   │   ├── auth.rs               # Auth middleware
│   │   ├── rate_limit.rs         # Rate limiting
│   │   └── logging.rs            # Request logging
│   │
│   ├── error.rs                  # Error types
│   ├── telemetry.rs              # Tracing setup
│   └── utils.rs                  # Utilities
│
├── tests/
│   ├── integration/
│   │   ├── auth_test.rs
│   │   ├── repository_test.rs
│   │   ├── sync_test.rs
│   │   ├── lock_test.rs
│   │   └── api_test.rs
│   └── fixtures/
│       └── test_data/
│
├── benches/
│   ├── deduplication_bench.rs
│   └── sync_bench.rs
│
└── docs/
    ├── API.md
    ├── ARCHITECTURE.md
    ├── DEPLOYMENT.md
    └── OPERATIONS.md
```

**Web Frontend Structure:**

```
oxvcs-web/
├── package.json
├── tsconfig.json
├── tailwind.config.js
├── next.config.js
│
├── src/
│   ├── app/
│   │   ├── layout.tsx
│   │   ├── page.tsx              # Dashboard
│   │   ├── login/
│   │   ├── register/
│   │   ├── settings/
│   │   └── [namespace]/
│   │       └── [repo]/
│   │           ├── page.tsx       # Repo overview
│   │           ├── commits/
│   │           ├── tree/
│   │           ├── settings/
│   │           └── activity/
│   │
│   ├── components/
│   │   ├── ui/                   # shadcn components
│   │   ├── auth/
│   │   ├── repository/
│   │   ├── commit/
│   │   └── layout/
│   │
│   ├── lib/
│   │   ├── api.ts                # API client
│   │   ├── auth.ts               # Auth utilities
│   │   ├── websocket.ts          # WS client
│   │   └── utils.ts
│   │
│   ├── hooks/
│   │   ├── useAuth.ts
│   │   ├── useRepositories.ts
│   │   └── useWebSocket.ts
│   │
│   └── types/
│       ├── api.ts
│       └── models.ts
│
└── public/
    └── assets/
```

---

## Implementation Details

### Database Schema

See earlier in this document for full PostgreSQL schema. Key tables:
- `users`
- `api_keys`
- `repositories`
- `repository_collaborators`
- `commits`
- `blocks`
- `locks`
- `activities`
- `comments`

### Push Protocol Implementation

**Sequence:**

```
Client                              Server
  |                                    |
  |------ POST /push ----------------->|
  |  commits: [...]                    |
  |  blocks: [{hash, size}, ...]       |
  |                                    |
  |<----- 200 OK ---------------------|
  |  missing_blocks: [hash1, hash2]    |
  |  upload_urls: {                    |
  |    hash1: "https://s3.../abc",     |
  |    hash2: "https://s3.../def"      |
  |  }                                 |
  |                                    |
  |------ PUT https://s3.../abc ------>| (Direct to S3)
  |       (block data)                 |
  |                                    |
  |<----- 200 OK ----------------------|
  |                                    |
  |------ PUT https://s3.../def ------>|
  |       (block data)                 |
  |                                    |
  |<----- 200 OK ----------------------|
  |                                    |
  |------ POST /push/complete -------->|
  |                                    |
  |<----- 200 OK ----------------------|
  |  success: true                     |
```

**Server-side Logic:**

```rust
// src/api/sync.rs

#[derive(Deserialize)]
pub struct PushRequest {
    pub branch: String,
    pub commits: Vec<CommitData>,
    pub blocks: Vec<BlockInfo>,
}

pub async fn push_handler(
    State(app): State<AppState>,
    Path((namespace, repo_name)): Path<(String, String)>,
    Json(req): Json<PushRequest>,
) -> Result<Json<PushResponse>> {
    // 1. Verify user has write access
    let repo = app.db.get_repository(&namespace, &repo_name).await?;
    require_write_access(&app, &repo)?;

    // 2. Check which blocks are missing
    let missing_blocks = app.storage
        .check_missing_blocks(&req.blocks)
        .await?;

    // 3. Generate presigned upload URLs
    let upload_urls = app.storage
        .generate_upload_urls(&missing_blocks)
        .await?;

    // 4. Store commit metadata
    for commit in &req.commits {
        app.db.create_commit(&repo.id, commit).await?;
    }

    // 5. Update branch HEAD
    app.db.update_branch(&repo.id, &req.branch,
                         &req.commits.last().unwrap().hash)
        .await?;

    // 6. Broadcast event
    app.ws.broadcast(RepositoryEvent::CommitPushed {
        repo_id: repo.id,
        branch: req.branch.clone(),
        commit: req.commits.last().unwrap().clone(),
    }).await;

    Ok(Json(PushResponse {
        success: true,
        missing_blocks,
        upload_urls,
    }))
}
```

### Deduplication Algorithm

```rust
// src/storage/deduplication.rs

use sha2::{Sha256, Digest};

const CHUNK_SIZE: usize = 4 * 1024 * 1024; // 4MB

pub struct DeduplicationEngine {
    s3: S3Client,
    db: DbPool,
}

impl DeduplicationEngine {
    /// Store file with deduplication
    pub async fn store_file(
        &self,
        file_path: &Path,
    ) -> Result<Vec<BlockInfo>> {
        let mut blocks = Vec::new();
        let mut file = tokio::fs::File::open(file_path).await?;
        let mut offset = 0u64;

        loop {
            // Read chunk
            let mut chunk = vec![0u8; CHUNK_SIZE];
            let n = file.read(&mut chunk).await?;
            if n == 0 { break; }
            chunk.truncate(n);

            // Compute hash
            let hash = format!("{:x}", Sha256::digest(&chunk));

            // Check if block exists
            if !self.block_exists(&hash).await? {
                // Upload to S3
                let key = format!("blocks/{}/{}/{}",
                                  &hash[..2],
                                  &hash[2..4],
                                  hash);

                self.s3.put_object()
                    .bucket("oxvcs-storage")
                    .key(&key)
                    .body(chunk.into())
                    .send()
                    .await?;

                // Register block
                self.db.create_block(&hash, n as i64, &key).await?;
            }

            blocks.push(BlockInfo {
                hash,
                size: n as u64,
                offset,
            });

            offset += n as u64;
        }

        Ok(blocks)
    }

    async fn block_exists(&self, hash: &str) -> Result<bool> {
        Ok(self.db.get_block(hash).await?.is_some())
    }
}
```

---

## Testing Strategy

### Unit Tests

**Coverage Target**: >80%

```rust
// tests/unit/deduplication_test.rs

#[tokio::test]
async fn test_deduplication_identical_files() {
    let engine = DeduplicationEngine::new_test();

    // Create two identical files
    let file1 = create_test_file(b"hello world");
    let file2 = create_test_file(b"hello world");

    // Store both
    let blocks1 = engine.store_file(&file1).await.unwrap();
    let blocks2 = engine.store_file(&file2).await.unwrap();

    // Should have identical block hashes
    assert_eq!(blocks1.len(), blocks2.len());
    for (b1, b2) in blocks1.iter().zip(blocks2.iter()) {
        assert_eq!(b1.hash, b2.hash);
    }

    // Should only store blocks once
    let storage_count = engine.count_blocks().await.unwrap();
    assert_eq!(storage_count, blocks1.len());
}
```

### Integration Tests

**Run against real database and S3:**

```rust
// tests/integration/push_pull_test.rs

#[tokio::test]
async fn test_push_pull_workflow() {
    let server = TestServer::new().await;
    let client = TestClient::new(&server.url);

    // Create repository
    let repo = client.create_repository("testuser", "testrepo")
        .await.unwrap();

    // Create local files
    let files = vec![
        ("file1.txt", b"content1"),
        ("file2.txt", b"content2"),
    ];

    // Push
    client.push(&repo, "main", files).await.unwrap();

    // Pull
    let pulled_files = client.pull(&repo, "main").await.unwrap();

    // Verify
    assert_eq!(pulled_files.len(), 2);
    assert_eq!(pulled_files[0].content, b"content1");
}
```

### Load Tests

**Using `k6` for load testing:**

```javascript
// tests/load/push_load_test.js

import http from 'k6/http';
import { check } from 'k6';

export let options = {
  stages: [
    { duration: '2m', target: 100 },  // Ramp up to 100 users
    { duration: '5m', target: 100 },  // Stay at 100 users
    { duration: '2m', target: 0 },    // Ramp down
  ],
};

export default function() {
  const payload = {
    branch: 'main',
    commits: [...],
    blocks: [...],
  };

  const res = http.post(
    'http://localhost:8080/api/v1/repositories/test/repo/push',
    JSON.stringify(payload),
    { headers: { 'Authorization': `Bearer ${__ENV.API_TOKEN}` } }
  );

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 2s': (r) => r.timings.duration < 2000,
  });
}
```

---

## Deployment Guide

### Docker Compose (Development)

```yaml
# docker-compose.yml

version: '3.8'

services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_USER: oxvcs
      POSTGRES_PASSWORD: dev_password
      POSTGRES_DB: oxvcs_server
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./migrations:/docker-entrypoint-initdb.d

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"

  minio:
    image: minio/minio
    command: server /data --console-address ":9001"
    environment:
      MINIO_ROOT_USER: minioadmin
      MINIO_ROOT_PASSWORD: minioadmin
    ports:
      - "9000:9000"
      - "9001:9001"
    volumes:
      - minio_data:/data

  server:
    build: .
    depends_on:
      - postgres
      - redis
      - minio
    environment:
      DATABASE_URL: postgres://oxvcs:dev_password@postgres:5432/oxvcs_server
      REDIS_URL: redis://redis:6379
      S3_ENDPOINT: http://minio:9000
      S3_ACCESS_KEY: minioadmin
      S3_SECRET_KEY: minioadmin
      S3_BUCKET: oxvcs-storage
      JWT_SECRET: dev_jwt_secret
    ports:
      - "8080:8080"
      - "8081:8081"

  web:
    build: ./oxvcs-web
    depends_on:
      - server
    environment:
      NEXT_PUBLIC_API_URL: http://localhost:8080
    ports:
      - "3000:3000"

volumes:
  postgres_data:
  minio_data:
```

### Kubernetes (Production)

See earlier architecture section for full K8s manifests.

**Quick deploy:**
```bash
# Create namespace
kubectl create namespace oxvcs

# Apply secrets
kubectl apply -f k8s/secrets.yaml

# Deploy services
kubectl apply -f k8s/postgres.yaml
kubectl apply -f k8s/redis.yaml
kubectl apply -f k8s/server.yaml
kubectl apply -f k8s/web.yaml
kubectl apply -f k8s/ingress.yaml
```

---

## Maintenance & Operations

### Monitoring

**Prometheus Metrics:**
```rust
// src/telemetry.rs

use prometheus::{
    Registry, Counter, Histogram, Gauge,
    register_counter, register_histogram, register_gauge,
};

lazy_static! {
    pub static ref HTTP_REQUESTS: Counter = register_counter!(
        "oxvcs_http_requests_total",
        "Total HTTP requests"
    ).unwrap();

    pub static ref PUSH_DURATION: Histogram = register_histogram!(
        "oxvcs_push_duration_seconds",
        "Push operation duration"
    ).unwrap();

    pub static ref ACTIVE_LOCKS: Gauge = register_gauge!(
        "oxvcs_active_locks",
        "Number of active locks"
    ).unwrap();
}
```

### Backup Strategy

**Automated Backups:**
```bash
#!/bin/bash
# scripts/backup.sh

# Backup PostgreSQL
pg_dump -h localhost -U oxvcs oxvcs_server | \
  gzip > backups/postgres-$(date +%Y%m%d).sql.gz

# Backup S3 (if using MinIO)
mc mirror minio/oxvcs-storage backups/s3-$(date +%Y%m%d)/

# Retention: Keep last 30 days
find backups/ -type f -mtime +30 -delete
```

### Scaling Considerations

**Horizontal Scaling:**
- Run multiple API server instances behind load balancer
- Use Redis for shared session state
- PostgreSQL read replicas for queries
- S3 handles storage scaling automatically

**Vertical Scaling:**
- Database: Increase CPU/RAM for large deployments
- Redis: Increase memory for more cache
- Server: More CPU for deduplication computations

---

## Summary

This development plan provides a complete roadmap for building OxVCS Server over 24 weeks. Key milestones:

- **Week 4**: MVP with auth and repository management
- **Week 10**: Push/pull working with deduplication
- **Week 12**: Distributed locking functional
- **Week 18**: Web UI complete
- **Week 20**: Real-time features live
- **Week 24**: Production ready 🚀

**Next Steps:**
1. Review and approve this plan
2. Set up development environment
3. Begin Phase 1 implementation
4. Weekly progress reviews

---

*Last Updated: 2025-11-17*
*Version: 1.0*
