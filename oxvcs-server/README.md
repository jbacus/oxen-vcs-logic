# OxVCS Server

Self-hosted repository server for OxVCS (Logic Pro version control).

## Quick Start

### Prerequisites

- Rust 1.70+ (`rustup install stable`)
- Docker & Docker Compose
- PostgreSQL 16+ (or use Docker Compose)
- Redis 7+ (or use Docker Compose)
- MinIO or AWS S3 (or use Docker Compose for MinIO)

### Development Setup

1. **Clone and navigate**:
   ```bash
   cd oxvcs-server
   ```

2. **Start infrastructure** (Postgres, Redis, MinIO):
   ```bash
   docker-compose up -d postgres redis minio
   ```

3. **Configure environment**:
   ```bash
   cp .env.example .env
   # Edit .env with your settings
   ```

4. **Run migrations**:
   ```bash
   # Migrations run automatically on server start
   # Or manually: sqlx migrate run
   ```

5. **Build and run**:
   ```bash
   cargo build
   cargo run
   ```

6. **Test the server**:
   ```bash
   curl http://localhost:8080/health
   # Should return: OK
   ```

### Register a User

```bash
curl -X POST http://localhost:8080/api/v1/auth/register \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "email": "test@example.com",
    "password": "password123",
    "full_name": "Test User"
  }'
```

### Login

```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "username": "testuser",
    "password": "password123"
  }'
```

## Project Structure

```
oxvcs-server/
├── migrations/          # SQL migrations
├── src/
│   ├── main.rs         # Server entry point
│   ├── config.rs       # Configuration
│   ├── db.rs           # Database connection
│   ├── error.rs        # Error types
│   ├── models.rs       # Data models
│   ├── api/            # API endpoints
│   ├── auth/           # Authentication
│   ├── storage/        # S3 & deduplication
│   ├── locks/          # Distributed locking
│   └── websocket/      # Real-time events
└── tests/              # Integration tests
```

## Development

### Running Tests

```bash
# Unit tests
cargo test

# Integration tests (requires running database)
docker-compose up -d
cargo test --test integration
```

### Database Migrations

```bash
# Create new migration
sqlx migrate add create_new_table

# Run migrations
sqlx migrate run

# Revert last migration
sqlx migrate revert
```

### Debugging

```bash
# Run with verbose logging
RUST_LOG=debug cargo run

# View postgres logs
docker-compose logs -f postgres

# View all logs
docker-compose logs -f
```

## API Documentation

See `OXVCS_SERVER_PLAN.md` for complete API documentation.

### Available Endpoints

- `GET /health` - Health check
- `POST /api/v1/auth/register` - Register new user
- `POST /api/v1/auth/login` - Login

*More endpoints coming in future phases.*

## Configuration

All configuration is done via environment variables. See `.env.example` for all available options.

### Required Variables

- `DATABASE_URL` - PostgreSQL connection string
- `REDIS_URL` - Redis connection string
- `S3_ENDPOINT` - S3 endpoint URL
- `S3_ACCESS_KEY` - S3 access key
- `S3_SECRET_KEY` - S3 secret key
- `S3_BUCKET` - S3 bucket name
- `JWT_SECRET` - Secret for JWT signing

## Deployment

See `OXVCS_SERVER_PLAN.md` for complete deployment guide including:
- Docker production builds
- Kubernetes manifests
- Infrastructure requirements
- Monitoring setup

## Contributing

1. Create feature branch
2. Make changes
3. Add tests
4. Run `cargo fmt` and `cargo clippy`
5. Submit pull request

## License

MIT

## Support

- Issues: https://github.com/jbacus/oxen-vcs-logic/issues
- Documentation: See `OXVCS_SERVER_PLAN.md`
