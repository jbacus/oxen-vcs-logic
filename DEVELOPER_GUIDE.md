# Auxin Developer & Deployment Guide

<div align="center">
  <img src="assets/icon/icon-128.png" alt="Auxin Logo" width="80" height="80">
  <h2>Development, Deployment, and Operations</h2>
</div>

**For**: Contributors, maintainers, deployers, and DevOps engineers

This comprehensive guide covers everything from setting up a development environment to deploying Auxin Server in production.

---

## Quick Navigation

### For Developers
- [Development Setup](#development-setup)
- [Architecture](#architecture)
- [Building and Testing](#building-and-testing)
- [Contributing Code](#contributing-code)
- [Adding New Application Support](#adding-new-application-support)

### For DevOps/Deployment
- [Server Deployment](#server-deployment)
- [Production Configuration](#production-configuration)
- [Monitoring and Logging](#monitoring-and-logging)
- [Backup and Recovery](#backup-and-recovery)
- [Security Hardening](#security-hardening)

---

# Part 1: Development

## Development Setup

### Prerequisites

**Required**:
- macOS 14.0+ (for Swift components)
- Xcode 15+ (for Swift/SwiftUI development)
- Rust stable toolchain (for CLI wrapper and server)
- Oxen CLI: `pip install oxen-ai`

**Optional**:
- Docker (for server development)
- Node.js 18+ (for server frontend development)

### Quick Start

```bash
# 1. Clone repository
git clone https://github.com/jbacus/auxin.git
cd auxin

# 2. Install Rust (if needed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install Oxen CLI
pip install oxen-ai

# 4. Build all components
./install.sh

# 5. Run tests
./run_all_tests.sh
```

### Component-Specific Setup

#### CLI Wrapper (Rust)

```bash
cd Auxin-CLI-Wrapper

# Build
cargo build

# Run tests
cargo test --all-features

# Generate coverage report
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage/

# Run with debugging
RUST_BACKTRACE=1 cargo run -- --help
```

#### LaunchAgent (Swift)

```bash
cd Auxin-LaunchAgent

# Build
swift build

# Run tests (216 tests)
swift test --filter 'XPCServiceTests|PowerManagementTests|LockManagerTests'

# Run daemon manually (for debugging)
swift run
```

#### GUI App (Swift)

```bash
cd Auxin-App

# Build
swift build -c release

# Create app bundle
./create-app-bundle.sh

# Run tests
swift test
```

#### Server (Rust + React)

```bash
cd auxin-server

# Build backend
cargo build

# Install frontend dependencies
cd frontend && npm install && cd ..

# Run development server
cargo run &
cd frontend && npm start

# Run tests
cargo test --all-features
```

---

## Architecture

### System Overview

```
┌─────────────────────────────────────────────────────────┐
│                 Auxin Complete System                    │
├─────────────────────────────────────────────────────────┤
│                                                           │
│  ┌─────────────┐      ┌──────────────┐      ┌────────┐ │
│  │  Auxin.app  │◄────►│LaunchAgent   │◄────►│  CLI   │ │
│  │ (SwiftUI)   │ XPC  │ (FSEvents)   │ Exec │ (Rust) │ │
│  └─────────────┘      └──────────────┘      └────────┘ │
│         │                     │                    │     │
│         └─────────────────────┴────────────────────┘     │
│                              │                            │
│                    ┌─────────▼────────┐                  │
│                    │  Oxen CLI        │                  │
│                    │  (subprocess)    │                  │
│                    └──────────────────┘                  │
│                              │                            │
│                    ┌─────────▼────────┐                  │
│                    │ Auxin Server     │ (Optional)       │
│                    │ (Collaboration)  │                  │
│                    └──────────────────┘                  │
└─────────────────────────────────────────────────────────┘
```

### Design Principles

1. **Separation of Concerns**: GUI → Daemon → CLI → Oxen
2. **Oxen-first**: All VCS operations through Oxen subprocess (never direct liboxen calls)
3. **Binary-aware**: Never attempt algorithmic merge of binary files
4. **Pessimistic locking**: Prevent conflicts rather than resolve them
5. **Application-specific**: Custom metadata and ignore patterns per app type
6. **Power-safe**: Emergency commits before sleep/shutdown

### Technology Stack

| Component | Language | Key Libraries |
|-----------|----------|---------------|
| CLI Wrapper | Rust | clap, tokio, serde, anyhow |
| LaunchAgent | Swift | FSEvents, XPC, IOKit |
| GUI App | Swift | SwiftUI, ServiceManagement |
| Server | Rust | actix-web, tokio, jsonwebtoken |
| Server Frontend | TypeScript | React, TanStack Query |

### Project Structure

```
auxin/
├── Auxin-CLI-Wrapper/          # Rust CLI (88% coverage)
│   ├── src/
│   │   ├── oxen_subprocess.rs  # ⚠️ CRITICAL: Oxen integration
│   │   ├── config.rs           # ProjectType enum
│   │   ├── logic_project.rs    # Logic Pro detection
│   │   └── ...
│   └── tests/
│
├── Auxin-LaunchAgent/          # Background daemon (216 tests)
│   ├── Sources/
│   │   ├── Daemon.swift        # ⚠️ CRITICAL: Orchestration
│   │   ├── FSEventsMonitor.swift # ⚠️ CRITICAL: File monitoring
│   │   ├── PowerManagement.swift # ⚠️ CRITICAL: Power events
│   │   └── ...
│   └── Tests/
│
├── Auxin-App/                  # SwiftUI application
│   └── Sources/Views/
│
├── auxin-server/               # Collaboration server
│   ├── src/api/                # REST endpoints
│   ├── frontend/               # React dashboard
│   └── docs/
│       ├── api/openapi.yaml    # OpenAPI 3.0 spec
│       └── deployment/         # Production guides
│
├── auxin-config/               # Unified TOML configuration
│
└── docs/
    ├── user/                   # End-user documentation
    ├── developer/              # Technical reference
    └── BONEYARD/               # Deprecated docs
```

---

## Building and Testing

### Build All Components

```bash
# Development build
./install.sh

# Release build (optimized)
./install.sh --release
```

### Run All Tests

```bash
# Comprehensive test suite
./run_all_tests.sh

# Expected results:
# - Rust CLI: 50+ tests, 88% coverage
# - Swift LaunchAgent: 216 tests passing
# - Swift App: All tests passing
# - Server: Integration tests passing
```

### Test Individual Components

**Rust CLI**:
```bash
cd Auxin-CLI-Wrapper
cargo test --all-features -- --nocapture
cargo tarpaulin --out Html  # Generate coverage
```

**Swift LaunchAgent**:
```bash
cd Auxin-LaunchAgent
swift test --filter XPCServiceTests  # 30 tests
swift test --filter PowerManagementTests
swift test --filter LockManagerTests
```

**Server**:
```bash
cd auxin-server
cargo test --all-features
cargo test --test collaboration_e2e_tests  # E2E tests
```

### Debugging

**Daemon Logs**:
```bash
# View live logs
log show --predicate 'process == "Auxin-LaunchAgent"' --last 1h --info

# Check daemon status
launchctl list | grep com.auxin.daemon
launchctl print gui/$(id -u)/com.auxin.daemon
```

**CLI Debugging**:
```bash
# Verbose output
auxin --verbose commit -m "Test"

# Rust backtrace
RUST_BACKTRACE=1 cargo run -- commit -m "Test"
```

---

## Contributing Code

### Code Style

**Swift**:
- Follow [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)
- Max line length: 120 characters
- Use `// MARK: -` for sections
- Prefer `async/await` over closures

**Rust**:
- Run `cargo fmt` before committing (enforced by CI)
- Run `cargo clippy -- -D warnings`
- Max line length: 100 characters
- Document public APIs with rustdoc

### Commit Message Format

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

**Example**:
```
feat(cli): Add support for Blender projects

Implemented Blender project detection and metadata extraction.

- Added is_blender_project() detection
- Created BlenderMetadata struct
- Added .auxinignore patterns

Closes #123
```

### Pull Request Process

1. **Fork** repository and create feature branch
2. **Develop** following code style guidelines
3. **Test** thoroughly (maintain or improve coverage)
4. **Document** changes in code and user guides
5. **Submit PR** with clear description

**PR Checklist**:
- [ ] Code follows style guidelines
- [ ] All tests pass locally
- [ ] New tests added for new features
- [ ] Documentation updated
- [ ] No compiler warnings
- [ ] Commit messages follow conventions

### CI/CD Requirements

All PRs must pass:
- ✅ All unit tests (Rust + Swift)
- ✅ Code coverage ≥70%
- ✅ No compiler warnings
- ✅ Clippy lints passing
- ✅ Security scan (CodeQL)
- ✅ Dependency vulnerability scan

---

## Adding New Application Support

### Quick Overview

1. Add `ProjectType` variant
2. Create detection module
3. Create metadata extractor
4. Add ignore patterns
5. Write tests
6. Update documentation

### Step-by-Step Example: Adding "MyApp" Support

#### 1. Add ProjectType Variant

**File**: `Auxin-CLI-Wrapper/src/config.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectType {
    Auto,
    LogicPro,
    SketchUp,
    Blender,
    MyApp,  // ← Add here
}

impl ProjectType {
    pub fn file_extension(&self) -> &str {
        match self {
            Self::MyApp => "myapp",  // ← Add extension
            // ...
        }
    }
}
```

#### 2. Create Detection Module

**File**: `Auxin-CLI-Wrapper/src/myapp_project.rs`

```rust
use std::path::Path;

/// Detects if the given path is a MyApp project
pub fn is_myapp_project(path: &Path) -> bool {
    // Check file extension
    if !path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext == "myapp")
        .unwrap_or(false)
    {
        return false;
    }

    // Check for required files
    let manifest_path = path.join("project.json");
    manifest_path.exists()
}
```

#### 3. Create Metadata Extractor

**File**: `Auxin-CLI-Wrapper/src/myapp_metadata.rs`

```rust
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct MyAppMetadata {
    pub version: String,
    pub author: Option<String>,
    pub created_date: Option<String>,
}

pub fn extract_myapp_metadata(path: &Path) -> anyhow::Result<MyAppMetadata> {
    let manifest_path = path.join("project.json");
    let content = std::fs::read_to_string(manifest_path)?;
    let metadata: MyAppMetadata = serde_json::from_str(&content)?;
    Ok(metadata)
}
```

#### 4. Add Ignore Patterns

**File**: `Auxin-CLI-Wrapper/src/ignore_template.rs`

```rust
pub fn get_default_ignores(project_type: ProjectType) -> Vec<String> {
    match project_type {
        ProjectType::MyApp => vec![
            "*.cache".to_string(),
            "**/temp/**".to_string(),
            "**/.myapp_temp/**".to_string(),
        ],
        // ...
    }
}
```

#### 5. Write Tests

**File**: `Auxin-CLI-Wrapper/tests/myapp_tests.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_detect_myapp_project() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().join("test.myapp");
        std::fs::create_dir(&project_path).unwrap();
        std::fs::write(project_path.join("project.json"), "{}").unwrap();

        assert!(is_myapp_project(&project_path));
    }
}
```

#### 6. Update Documentation

**File**: `docs/user/for-myapp-users.md`

Create user guide with:
- Installation steps
- Quick start tutorial
- Common workflows
- Troubleshooting

See **[Extensibility Guide](docs/developer/extensibility.md)** for complete details.

---

# Part 2: Server Deployment & Operations

## Server Deployment

### Deployment Options

| Option | Use Case | Complexity |
|--------|----------|------------|
| **Docker** | Recommended for production | Low |
| **Systemd** | Direct installation on Linux | Medium |
| **GCP** | Cloud deployment with CI/CD | High |

### Quick Start: Docker Deployment

**Prerequisites**:
- Docker 24+ and Docker Compose
- Domain name with DNS configured
- SSL certificate (Let's Encrypt recommended)

**1. Clone and Configure**:

```bash
# Clone repository
git clone https://github.com/jbacus/auxin.git
cd auxin/auxin-server

# Copy configuration template
cp config.toml.example config.toml

# Edit configuration
nano config.toml
```

**2. Configure Environment**:

**File**: `config.toml`

```toml
[server]
host = "0.0.0.0"
port = 3000
data_dir = "/var/oxen/data"

[auth]
token_secret = "CHANGE_THIS_TO_A_SECURE_RANDOM_STRING"
token_expiry_hours = 720  # 30 days

[cors]
allowed_origins = ["https://auxin.yourdomain.com"]

[rate_limiting]
requests_per_minute = 60
```

**3. Launch with Docker Compose**:

```bash
# Start services
docker-compose up -d

# Check status
docker-compose ps

# View logs
docker-compose logs -f auxin-server
```

**4. Set Up Nginx Reverse Proxy**:

**File**: `/etc/nginx/sites-available/auxin`

```nginx
upstream auxin_backend {
    server 127.0.0.1:3000;
    keepalive 32;
}

server {
    listen 443 ssl http2;
    server_name auxin.yourdomain.com;

    # SSL configuration
    ssl_certificate /etc/letsencrypt/live/auxin.yourdomain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/auxin.yourdomain.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;

    # Large file uploads (for creative projects)
    client_max_body_size 10G;
    client_body_timeout 300s;

    # WebSocket support
    location /ws {
        proxy_pass http://auxin_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_read_timeout 86400;
    }

    # API endpoints
    location /api {
        proxy_pass http://auxin_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Frontend
    location / {
        proxy_pass http://auxin_backend;
        proxy_set_header Host $host;
    }
}

# HTTP to HTTPS redirect
server {
    listen 80;
    server_name auxin.yourdomain.com;
    return 301 https://$server_name$request_uri;
}
```

**5. Enable and Start Nginx**:

```bash
# Enable site
sudo ln -s /etc/nginx/sites-available/auxin /etc/nginx/sites-enabled/

# Test configuration
sudo nginx -t

# Restart Nginx
sudo systemctl restart nginx
```

**6. Set Up SSL with Let's Encrypt**:

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d auxin.yourdomain.com

# Auto-renewal (runs twice daily)
sudo systemctl enable certbot.timer
```

---

## Production Configuration

### Configuration Options

Auxin uses a unified TOML configuration system with layered precedence:

1. Environment variables (`AUXIN_*`)
2. Project config: `./config.toml`
3. User config: `~/.config/auxin/config.toml`
4. System defaults

### Example Production Config

**File**: `config.toml`

```toml
[server]
host = "0.0.0.0"
port = 3000
data_dir = "/var/oxen/data"
log_level = "info"
workers = 4  # Number of worker threads

[auth]
token_secret = "${AUXIN_AUTH_SECRET}"  # Use environment variable
token_expiry_hours = 720
session_timeout_minutes = 60

[cors]
allowed_origins = [
    "https://auxin.yourdomain.com",
    "https://app.yourdomain.com"
]
allowed_methods = ["GET", "POST", "PUT", "DELETE"]
allowed_headers = ["Content-Type", "Authorization"]

[rate_limiting]
enabled = true
requests_per_minute = 60
burst_size = 100

[storage]
max_file_size_gb = 10
cleanup_interval_hours = 24
retention_days = 90

[database]
# Future: PostgreSQL configuration
# Currently uses file-based storage

[monitoring]
prometheus_enabled = true
prometheus_port = 9090
health_check_path = "/health"

[logging]
format = "json"  # or "text"
destination = "stdout"  # or "file"
file_path = "/var/log/auxin/server.log"
rotation_size_mb = 100
```

### Environment Variables

Override any config value with environment variables:

```bash
# Server configuration
export AUXIN_SERVER_HOST="0.0.0.0"
export AUXIN_SERVER_PORT="3000"
export AUXIN_SERVER_DATA_DIR="/data/oxen"

# Authentication
export AUXIN_AUTH_TOKEN_SECRET="your-secure-secret-here"
export AUXIN_AUTH_TOKEN_EXPIRY_HOURS="720"

# CORS
export AUXIN_CORS_ALLOWED_ORIGINS="https://auxin.example.com"

# Rate limiting
export AUXIN_RATE_LIMITING_REQUESTS_PER_MINUTE="100"
```

### Docker Configuration

**File**: `docker-compose.yml`

```yaml
version: '3.8'

services:
  auxin-server:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: auxin-server
    restart: unless-stopped

    ports:
      - "3000:3000"

    volumes:
      - auxin-data:/var/oxen/data
      - ./config.toml:/app/config.toml:ro
      - ./logs:/var/log/auxin

    environment:
      - RUST_LOG=info,auxin_server=info
      - AUXIN_AUTH_TOKEN_SECRET=${AUTH_SECRET}

    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s

    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"

volumes:
  auxin-data:
    driver: local
```

---

## Monitoring and Logging

### Prometheus Metrics

Auxin Server exposes Prometheus metrics at `/metrics`:

**Key Metrics**:
- `auxin_http_requests_total` - Total HTTP requests
- `auxin_http_request_duration_seconds` - Request latency
- `auxin_active_connections` - Current connections
- `auxin_repository_operations_total` - VCS operations
- `auxin_lock_acquisitions_total` - Lock operations
- `auxin_storage_bytes_used` - Disk usage

**Prometheus Configuration**:

**File**: `prometheus.yml`

```yaml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'auxin-server'
    static_configs:
      - targets: ['auxin-server:3000']
    metrics_path: '/metrics'

  - job_name: 'node'
    static_configs:
      - targets: ['node-exporter:9100']
```

### Grafana Dashboard

**Pre-built Dashboard**: `auxin-server/monitoring/grafana-dashboard.json`

**Key Panels**:
- Request rate and latency
- Active users and connections
- Repository operations (clone, commit, push, pull)
- Storage usage trends
- Error rate

**Import Dashboard**:
1. Open Grafana UI
2. Go to Dashboards → Import
3. Upload `grafana-dashboard.json`
4. Select Prometheus data source

### Application Logs

**Log Locations**:
- Docker: `docker-compose logs -f auxin-server`
- Systemd: `journalctl -u auxin-server -f`
- File: `/var/log/auxin/server.log`

**Log Format (JSON)**:
```json
{
  "timestamp": "2025-11-22T10:30:45Z",
  "level": "INFO",
  "target": "auxin_server::api",
  "message": "Repository cloned successfully",
  "user_id": "user_123",
  "repo": "team/project",
  "duration_ms": 1234
}
```

**Useful Log Queries**:

```bash
# Filter by level
journalctl -u auxin-server | grep ERROR

# Filter by user
journalctl -u auxin-server | grep user_123

# Follow specific operation
journalctl -u auxin-server -f | grep "clone"

# Last hour of errors
journalctl -u auxin-server --since "1 hour ago" | grep ERROR
```

---

## Backup and Recovery

### Automated Backup Script

**File**: `auxin-server/docs/deployment/backup-auxin.sh`

```bash
#!/bin/bash
# Automated backup with encryption and S3 upload

BACKUP_DIR="/backup/auxin"
DATE=$(date +%Y%m%d-%H%M%S)
DOCKER_VOLUME="auxin_auxin-data"
ENCRYPTION_PASSWORD="${BACKUP_ENCRYPTION_PASSWORD}"
S3_BUCKET="${BACKUP_S3_BUCKET}"

# Backup Docker volume
docker run --rm \
    -v "$DOCKER_VOLUME":/data:ro \
    -v "$BACKUP_DIR":/backup \
    alpine tar czf /backup/auxin-$DATE.tar.gz -C /data .

# Encrypt
openssl enc -aes-256-cbc -salt -pbkdf2 \
    -pass pass:"$ENCRYPTION_PASSWORD" \
    -in "$BACKUP_DIR/auxin-$DATE.tar.gz" \
    -out "$BACKUP_DIR/auxin-$DATE.tar.gz.enc"

# Upload to S3
aws s3 cp "$BACKUP_DIR/auxin-$DATE.tar.gz.enc" \
    "s3://$S3_BUCKET/auxin-backups/"

# Cleanup old backups (keep 30 days)
find "$BACKUP_DIR" -name "auxin-*.tar.gz*" -mtime +30 -delete
```

**Schedule with Cron**:

```bash
# Run daily at 2 AM
0 2 * * * /usr/local/bin/backup-auxin.sh >> /var/log/auxin-backup.log 2>&1
```

### Manual Backup

```bash
# Backup Docker volume
docker run --rm \
    -v auxin_auxin-data:/data:ro \
    -v $(pwd):/backup \
    alpine tar czf /backup/auxin-backup.tar.gz -C /data .

# Backup configuration
cp config.toml config.toml.backup
```

### Restore from Backup

```bash
# Stop service
docker-compose down

# Extract backup to volume
docker run --rm \
    -v auxin_auxin-data:/data \
    -v $(pwd):/backup \
    alpine tar xzf /backup/auxin-backup.tar.gz -C /data

# Restart service
docker-compose up -d
```

---

## Security Hardening

### SSL/TLS Configuration

**Strong Cipher Suite** (Nginx):
```nginx
ssl_protocols TLSv1.2 TLSv1.3;
ssl_ciphers 'ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256:ECDHE-ECDSA-AES256-GCM-SHA384:ECDHE-RSA-AES256-GCM-SHA384';
ssl_prefer_server_ciphers off;
ssl_session_timeout 1d;
ssl_session_cache shared:SSL:50m;
ssl_stapling on;
ssl_stapling_verify on;
```

### Security Headers

```nginx
add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
add_header X-Frame-Options "DENY" always;
add_header X-Content-Type-Options "nosniff" always;
add_header X-XSS-Protection "1; mode=block" always;
add_header Referrer-Policy "strict-origin-when-cross-origin" always;
add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline';" always;
```

### Firewall Configuration

```bash
# Allow SSH, HTTP, HTTPS only
sudo ufw allow 22/tcp
sudo ufw allow 80/tcp
sudo ufw allow 443/tcp
sudo ufw enable

# Block direct access to backend
sudo ufw deny 3000/tcp
```

### Rate Limiting (Application Level)

Already configured in `config.toml`:
```toml
[rate_limiting]
enabled = true
requests_per_minute = 60
burst_size = 100
```

### Authentication Best Practices

1. **Strong Secrets**: Use 32+ character random strings
2. **Token Expiry**: Set reasonable expiry (720 hours = 30 days)
3. **HTTPS Only**: Never send tokens over HTTP
4. **Rotate Secrets**: Periodically rotate `token_secret`

---

## Troubleshooting

### Common Issues

**1. Server Won't Start**

```bash
# Check logs
docker-compose logs auxin-server

# Common causes:
# - Port 3000 already in use
# - Missing/invalid config.toml
# - Insufficient disk space
```

**2. High Memory Usage**

```bash
# Check container stats
docker stats auxin-server

# Adjust worker threads in config.toml:
[server]
workers = 2  # Reduce from 4
```

**3. Slow Repository Operations**

```bash
# Check disk I/O
iostat -x 1

# Consider:
# - Using SSD storage
# - Increasing Docker resources
# - Enabling caching in Oxen
```

**4. WebSocket Connections Failing**

```nginx
# Ensure Nginx proxy settings:
proxy_http_version 1.1;
proxy_set_header Upgrade $http_upgrade;
proxy_set_header Connection "upgrade";
proxy_read_timeout 86400;  # 24 hours
```

### Health Checks

```bash
# Server health
curl https://auxin.yourdomain.com/health

# API status
curl https://auxin.yourdomain.com/api/status

# Metrics
curl https://auxin.yourdomain.com/metrics
```

---

## Additional Resources

### Documentation
- **[API Reference](auxin-server/docs/api/README.md)** - REST API documentation
- **[OpenAPI Spec](auxin-server/docs/api/openapi.yaml)** - Complete API specification
- **[CI/CD Guide](.github/CI_CD_GUIDE.md)** - GitHub Actions workflows
- **[Configuration Reference](docs/developer/configuration.md)** - All config options

### Support
- **Issues**: [GitHub Issues](https://github.com/jbacus/auxin/issues)
- **Discussions**: [GitHub Discussions](https://github.com/jbacus/auxin/discussions)
- **Email**: support@auxin.dev (coming soon)

### Quick Links
- **User Guide**: [USER_GUIDE.md](USER_GUIDE.md)
- **Architecture Details**: [docs/developer/architecture.md](docs/developer/architecture.md)
- **Roadmap**: [ROADMAP.md](ROADMAP.md)

---

*Last Updated: 2025-11-22*
