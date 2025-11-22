# Auxin Server API Documentation

## Overview

The Auxin Server provides a RESTful API for managing version-controlled creative projects. Built on Oxen.ai infrastructure, it extends standard VCS operations with features specific to binary file collaboration.

## Documentation

### OpenAPI Specification

The complete API is documented in OpenAPI 3.0 format:
- **[openapi.yaml](./openapi.yaml)** - Machine-readable API specification

You can use this specification with tools like:
- **Swagger UI** - Interactive API documentation
- **Postman** - Import for testing
- **Code generators** - Generate client libraries in any language

### Quick Start

1. **View Interactive Documentation**:
   ```bash
   # Using Swagger UI
   docker run -p 8080:8080 -e SWAGGER_JSON=/docs/openapi.yaml \
     -v $(pwd)/docs/api:/docs swaggerapi/swagger-ui
   ```

2. **Test with cURL**:
   ```bash
   # Register a user
   curl -X POST http://localhost:3000/api/auth/register \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser","email":"test@example.com","password":"password123"}'

   # Login (get token)
   TOKEN=$(curl -X POST http://localhost:3000/api/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username":"testuser","password":"password123"}' \
     | jq -r '.token')

   # Create a repository
   curl -X POST http://localhost:3000/api/repos/testuser/myproject \
     -H "Authorization: Bearer $TOKEN" \
     -H "Content-Type: application/json" \
     -d '{"description":"My first project"}'
   ```

## Authentication

All authenticated endpoints require a Bearer token in the Authorization header:

```http
Authorization: Bearer <token>
```

### Getting a Token

**Method 1: Register** (new users)
```bash
POST /api/auth/register
{
  "username": "yourname",
  "email": "you@example.com",
  "password": "securePassword123"
}

# Response includes token
{
  "id": "uuid",
  "username": "yourname",
  "email": "you@example.com",
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

**Method 2: Login** (existing users)
```bash
POST /api/auth/login
{
  "username": "yourname",
  "password": "securePassword123"
}

# Response
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": { ... }
}
```

### Token Expiration

- Default expiration: 24 hours (configurable via `AUXIN_SERVER_AUTH_TOKEN_EXPIRY_HOURS`)
- No automatic refresh - login again when expired
- Logout invalidates token immediately

## Core Concepts

### Repositories

Repositories are organized by namespace (username or organization):

```
{namespace}/{repository-name}
```

Example: `musicproducer/my-album`

### Locks

Auxin uses **pessimistic locking** to prevent merge conflicts with binary files:

1. **Acquire lock** before editing
2. Make changes
3. Commit and push
4. **Release lock** when done

Locks automatically expire after a timeout (default: 24 hours) and can be renewed via heartbeat.

### Metadata

Application-specific metadata (BPM, sample rate, key, etc.) is stored alongside commits:

- **Store**: `POST /api/repos/{namespace}/{name}/metadata/{commit}`
- **Retrieve**: `GET /api/repos/{namespace}/{name}/metadata/{commit}`

This metadata is indexed and searchable via the CLI.

### Activity Feed

All repository operations are logged to an activity feed:

```bash
GET /api/repos/{namespace}/{name}/activity?limit=50
```

Activity types:
- `commit` - Commits made
- `push`/`pull` - Sync operations
- `lock_acquired`/`lock_released` - Lock events
- `restore` - Rollbacks

## WebSocket Support

Real-time updates are available via WebSocket:

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
  // Subscribe to a repository
  ws.send(JSON.stringify({
    type: 'subscribe',
    channel: 'musicproducer/my-album'
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);
  console.log('Activity:', msg);
  // { activity_type: 'push', user: 'user1', message: '...', timestamp: '...' }
};
```

### Message Types

**Lock Acquired**:
```json
{
  "type": "lock_acquired",
  "user": "musicproducer",
  "lock_id": "abc123",
  "timestamp": "2025-11-22T12:00:00Z"
}
```

**Lock Released**:
```json
{
  "type": "lock_released",
  "lock_id": "abc123",
  "timestamp": "2025-11-22T14:30:00Z"
}
```

**Commit/Push**:
```json
{
  "activity_type": "push",
  "user": "musicproducer",
  "message": "Pushed to origin (branch: main)",
  "timestamp": "2025-11-22T15:45:00Z"
}
```

## Error Handling

### HTTP Status Codes

- `200` - Success
- `201` - Created
- `400` - Bad Request (validation error)
- `401` - Unauthorized (missing/invalid token)
- `403` - Forbidden (no permission)
- `404` - Not Found
- `409` - Conflict (resource already exists, lock held by another user)
- `500` - Internal Server Error

### Error Response Format

```json
{
  "error": "Unauthorized",
  "message": "No authorization token provided"
}
```

## Rate Limiting

Currently **no rate limiting** is enforced. In production deployments, consider adding rate limiting via:
- Nginx `limit_req` module
- API Gateway
- Application-level middleware

## CORS

CORS is configured to allow all origins in development. For production:

```rust
// Update src/main.rs
.wrap(
    actix_cors::Cors::default()
        .allowed_origin("https://yourdomain.com")
        .allowed_methods(vec!["GET", "POST", "DELETE", "PUT"])
        .allowed_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
)
```

## Examples

### Complete Workflow

```bash
# 1. Register/Login
TOKEN=$(curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"producer","email":"p@example.com","password":"pass123"}' \
  | jq -r '.token')

# 2. Create repository
curl -X POST http://localhost:3000/api/repos/producer/beats \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"description":"My beats collection"}'

# 3. Acquire lock
LOCK_ID=$(curl -X POST http://localhost:3000/api/repos/producer/beats/locks/acquire \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"user":"producer","machine_id":"laptop-1"}' \
  | jq -r '.lock_id')

# 4. Make changes locally, commit, push
curl -X POST http://localhost:3000/api/repos/producer/beats/push \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"remote":"origin","branch":"main"}'

# 5. Release lock
curl -X POST http://localhost:3000/api/repos/producer/beats/locks/release \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"lock_id":"'$LOCK_ID'"}'
```

## Client Libraries

### Official CLI

The Auxin CLI provides a complete client implementation:

```bash
# Install
cargo install auxin

# Configure server URL
auxin config set server.url http://localhost:3000

# Login
auxin auth login

# Use
auxin clone musicproducer/my-album
auxin lock acquire
# ... make changes ...
auxin push
auxin lock release
```

### Custom Clients

Generate client libraries from the OpenAPI spec using [openapi-generator](https://openapi-generator.tech/):

```bash
# Python
openapi-generator-cli generate -i openapi.yaml -g python -o python-client/

# JavaScript/TypeScript
openapi-generator-cli generate -i openapi.yaml -g typescript-axios -o ts-client/

# Go
openapi-generator-cli generate -i openapi.yaml -g go -o go-client/
```

## See Also

- [CONFIGURATION.md](../../CONFIGURATION.md) - Server configuration
- [Deployment Guide](../deployment/PRODUCTION.md) - Production deployment
- [WebSocket Protocol](./websocket.md) - Detailed WebSocket documentation
