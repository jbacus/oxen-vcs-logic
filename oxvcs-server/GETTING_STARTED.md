# Getting Started with OxVCS Server Development

This guide will walk you through setting up your development environment and starting Phase 1 implementation.

## Prerequisites Checklist

- [ ] Rust 1.70+ installed (`rustup install stable`)
- [ ] Docker installed and running
- [ ] Docker Compose installed
- [ ] Git installed
- [ ] Your favorite code editor (VS Code, RustRover, etc.)

## Step 1: Verify Installation

```bash
# Check Rust
rustc --version
cargo --version

# Check Docker
docker --version
docker-compose --version
```

## Step 2: Initial Setup

```bash
# Navigate to project
cd oxvcs-server

# Copy environment configuration
cp .env.example .env

# Start infrastructure services
docker-compose up -d

# Verify services are running
docker-compose ps
# Should show postgres, redis, and minio running
```

## Step 3: Build the Project

```bash
# Build (this will download dependencies)
cargo build

# This might take 5-10 minutes on first run
# Subsequent builds will be much faster
```

## Step 4: Run the Server

```bash
# Run in development mode
cargo run

# You should see output like:
# [INFO] Starting OxVCS Server...
# [INFO] Configuration loaded
# [INFO] Database connection established
# [INFO] Database migrations complete
# [INFO] Server listening on 0.0.0.0:8080
```

## Step 5: Test the Server

Open a new terminal:

```bash
# Health check
curl http://localhost:8080/health
# Expected: OK

# Register a test user
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "securepass123"
  }'

# Expected: JSON response with user info and JWT token
```

## Step 6: Verify Database

```bash
# Connect to PostgreSQL
docker exec -it oxvcs-postgres psql -U oxvcs -d oxvcs_server

# List tables
\dt

# Should see: users, api_keys, repositories, etc.

# Check user was created
SELECT username, email, created_at FROM users;

# Exit
\q
```

## Step 7: Explore the Code

Open the project in your editor and review:

1. **`src/main.rs`** - Server entry point, see how routes are registered
2. **`src/api/auth.rs`** - Registration and login implementation
3. **`src/models.rs`** - Database models
4. **`migrations/*.sql`** - Database schema

## Development Workflow

### Making Changes

```bash
# 1. Create a feature branch
git checkout -b feature/my-feature

# 2. Make your changes in src/

# 3. Format code
cargo fmt

# 4. Check for issues
cargo clippy

# 5. Run tests
cargo test

# 6. Run server to test
cargo run
```

### Adding a New Endpoint

Example: Add a `/api/v1/users/me` endpoint to get current user:

1. **Add route in `src/main.rs`**:
   ```rust
   .route("/api/v1/users/me", get(api::users::get_current_user))
   ```

2. **Create `src/api/users.rs`**:
   ```rust
   use axum::{extract::State, Json};
   use crate::{error::AppResult, models::User};

   pub async fn get_current_user(
       State(state): State<AppState>,
       // TODO: Add auth middleware to extract user
   ) -> AppResult<Json<User>> {
       // Implementation
   }
   ```

3. **Test**:
   ```bash
   cargo run
   curl -H "Authorization: Bearer <token>" \
        http://localhost:8080/api/v1/users/me
   ```

### Adding a Database Migration

```bash
# Create new migration file
touch migrations/006_add_user_preferences.sql

# Edit the file with your SQL:
# CREATE TABLE user_preferences (...);

# Restart server (migrations run automatically)
cargo run
```

### Debugging

```bash
# Run with debug logging
RUST_LOG=debug cargo run

# View SQL queries
RUST_LOG=sqlx=debug cargo run

# Use Rust debugger (VS Code)
# Set breakpoints and press F5
```

## Common Issues

### "Connection refused" to database

```bash
# Check if postgres is running
docker-compose ps

# Restart services
docker-compose down
docker-compose up -d
```

### "JWT_SECRET must be set"

```bash
# Make sure .env file exists and has JWT_SECRET
cat .env | grep JWT_SECRET

# If missing, copy from .env.example
cp .env.example .env
```

### Compilation errors

```bash
# Clean and rebuild
cargo clean
cargo build
```

## Next Steps: Phase 1 Implementation

Now that your environment is set up, follow the Phase 1 tasks from `OXVCS_SERVER_PLAN.md`:

### Week 1: Core Setup ✅ (Done!)
- [x] Project structure created
- [x] Database schema defined
- [x] Basic auth endpoints working

### Week 2: Complete Authentication
- [ ] Add JWT middleware for protected routes
- [ ] Implement API key generation
- [ ] Add password reset flow
- [ ] Write authentication tests

**Start here**:
```bash
# Create auth middleware
touch src/auth/middleware.rs

# Follow implementation plan in OXVCS_SERVER_PLAN.md
# See "Phase 1: Foundation" section
```

### Week 3: Repository Management
- [ ] Implement repository CRUD endpoints
- [ ] Add authorization checks
- [ ] Repository listing with pagination
- [ ] Write repository tests

### Week 4: Testing & Documentation
- [ ] Write comprehensive tests
- [ ] Set up CI/CD pipeline
- [ ] API documentation
- [ ] Code review

## Resources

- **Full Plan**: See `OXVCS_SERVER_PLAN.md` for complete 24-week roadmap
- **Rust Book**: https://doc.rust-lang.org/book/
- **Axum Docs**: https://docs.rs/axum/latest/axum/
- **SQLx Docs**: https://docs.rs/sqlx/latest/sqlx/
- **PostgreSQL Docs**: https://www.postgresql.org/docs/

## Getting Help

If you're stuck:

1. Check the error message carefully
2. Search the issue in the Rust/Axum docs
3. Ask in project discussions
4. Check `OXVCS_SERVER_PLAN.md` for architecture guidance

## Quick Reference

```bash
# Start services
docker-compose up -d

# Stop services
docker-compose down

# View logs
docker-compose logs -f postgres

# Build project
cargo build

# Run server
cargo run

# Run tests
cargo test

# Format code
cargo fmt

# Check code
cargo clippy

# Clean build
cargo clean
```

---

**You're ready to start development! 🚀**

Begin with Week 2 tasks and refer to `OXVCS_SERVER_PLAN.md` for detailed implementation guidance.
