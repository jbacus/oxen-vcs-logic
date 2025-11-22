# Authentication and Authorization

This document describes the authentication and authorization system for the Auxin server.

## Overview

The Auxin server implements a user-based authentication system with project ownership and access control, similar to GitHub.

## Features

- **User Accounts**: Register and login with email and password
- **Token-based Authentication**: Bearer tokens for API access
- **Project Ownership**: Each repository has an owner
- **Collaborators**: Owners can add/remove collaborators
- **Public/Private Repositories**: Control visibility and access
- **GitHub-like Access Model**: Familiar permission structure

## User Management

### Registration

```bash
POST /api/auth/register
Content-Type: application/json

{
  "username": "alice",
  "email": "alice@example.com",
  "password": "securepassword123"
}
```

**Response:**
```json
{
  "token": "auxin_<uuid>",
  "user": {
    "id": "<uuid>",
    "username": "alice",
    "email": "alice@example.com",
    "created_at": "2025-11-22T10:00:00Z"
  }
}
```

### Login

```bash
POST /api/auth/login
Content-Type: application/json

{
  "email": "alice@example.com",
  "password": "securepassword123"
}
```

**Response:**
```json
{
  "token": "auxin_<uuid>",
  "user": {
    "id": "<uuid>",
    "username": "alice",
    "email": "alice@example.com",
    "created_at": "2025-11-22T10:00:00Z"
  }
}
```

### Get Current User

```bash
GET /api/auth/me
Authorization: Bearer auxin_<uuid>
```

**Response:**
```json
{
  "id": "<uuid>",
  "username": "alice",
  "email": "alice@example.com",
  "created_at": "2025-11-22T10:00:00Z"
}
```

### Logout

```bash
POST /api/auth/logout
Authorization: Bearer auxin_<uuid>
```

## Project Access Control

### Creating a Repository

Repositories now require authentication and automatically set the creator as owner:

```bash
POST /api/repos/namespace/myproject
Authorization: Bearer auxin_<uuid>
Content-Type: application/json

{
  "description": "My project",
  "visibility": "public"  // or "private", defaults to "public"
}
```

**Response:**
```json
{
  "namespace": "namespace",
  "name": "myproject",
  "path": "/var/oxen/data/namespace/myproject",
  "description": "My project",
  "owner": "alice",
  "visibility": "public"
}
```

### Access Control Model

#### Public Repositories
- **Read**: Anyone (including unauthenticated users)
- **Write**: Owner and collaborators only

#### Private Repositories
- **Read**: Owner and collaborators only
- **Write**: Owner and collaborators only

### Collaborator Management

#### List Collaborators

```bash
GET /api/repos/namespace/myproject/collaborators
Authorization: Bearer auxin_<uuid>  # Required for private repos
```

**Response:**
```json
{
  "owner_id": "<uuid>",
  "owner_username": "alice",
  "collaborators": [
    {
      "user_id": "<uuid>"
    }
  ]
}
```

#### Add Collaborator

Only the repository owner can add collaborators:

```bash
POST /api/repos/namespace/myproject/collaborators
Authorization: Bearer auxin_<uuid>
Content-Type: application/json

{
  "user_id": "<collaborator-uuid>"
}
```

**Response:**
```json
{
  "status": "success",
  "message": "Collaborator added",
  "user_id": "<collaborator-uuid>"
}
```

#### Remove Collaborator

Only the repository owner can remove collaborators:

```bash
DELETE /api/repos/namespace/myproject/collaborators/<user_id>
Authorization: Bearer auxin_<uuid>
```

**Response:**
```json
{
  "status": "success",
  "message": "Collaborator removed"
}
```

### Change Visibility

Only the repository owner can change visibility:

```bash
PUT /api/repos/namespace/myproject/visibility
Authorization: Bearer auxin_<uuid>
Content-Type: application/json

{
  "visibility": "private"  // or "public"
}
```

**Response:**
```json
{
  "status": "success",
  "visibility": "private"
}
```

## API Endpoint Authorization

### Public Endpoints (No Auth Required)

- `POST /api/auth/register`
- `POST /api/auth/login`
- `GET /api/repos` - Lists only public repos if not authenticated
- `GET /api/repos/{namespace}/{name}` - Only for public repos

### Authenticated Endpoints (Require Login)

#### Read Operations (Public repos allow unauthenticated, Private require auth)

- `GET /api/repos/{namespace}/{name}/commits`
- `GET /api/repos/{namespace}/{name}/status`
- `GET /api/repos/{namespace}/{name}/branches`
- `GET /api/repos/{namespace}/{name}/metadata/{commit}`
- `GET /api/repos/{namespace}/{name}/locks/status`
- `GET /api/repos/{namespace}/{name}/activity`
- `GET /api/repos/{namespace}/{name}/bounces`
- `GET /api/repos/{namespace}/{name}/collaborators`

#### Write Operations (Require Owner or Collaborator)

- `POST /api/repos/{namespace}/{name}` - Create (sets you as owner)
- `POST /api/repos/{namespace}/{name}/clone` - Clone (sets you as owner)
- `POST /api/repos/{namespace}/{name}/push`
- `POST /api/repos/{namespace}/{name}/pull`
- `POST /api/repos/{namespace}/{name}/fetch`
- `POST /api/repos/{namespace}/{name}/branches` - Create branch
- `DELETE /api/repos/{namespace}/{name}/branches/{branch}`
- `POST /api/repos/{namespace}/{name}/commits/{commit}/restore`
- `POST /api/repos/{namespace}/{name}/metadata/{commit}`
- `POST /api/repos/{namespace}/{name}/locks/acquire`
- `POST /api/repos/{namespace}/{name}/locks/release`
- `POST /api/repos/{namespace}/{name}/locks/heartbeat`
- `POST /api/repos/{namespace}/{name}/bounces/{commit}`
- `DELETE /api/repos/{namespace}/{name}/bounces/{commit}`

#### Owner-Only Operations

- `POST /api/repos/{namespace}/{name}/collaborators` - Add collaborator
- `DELETE /api/repos/{namespace}/{name}/collaborators/{user_id}` - Remove collaborator
- `PUT /api/repos/{namespace}/{name}/visibility` - Change visibility

## Storage

### User Data

Users are stored in `{SYNC_DIR}/.auxin/users.json`:

```json
[
  {
    "id": "<uuid>",
    "username": "alice",
    "email": "alice@example.com",
    "password_hash": "$2b$12$...",
    "created_at": "2025-11-22T10:00:00Z"
  }
]
```

### Project Metadata

Each repository has a `.oxen/project.json` file:

```json
{
  "owner_id": "<uuid>",
  "owner_username": "alice",
  "visibility": "public",
  "collaborators": ["<uuid1>", "<uuid2>"],
  "created_at": "2025-11-22T10:00:00Z",
  "updated_at": "2025-11-22T12:00:00Z"
}
```

## Security Features

- **Password Hashing**: bcrypt with default cost factor
- **Token Expiration**: Configurable via `AUTH_TOKEN_EXPIRY_HOURS` (default: 24 hours)
- **Token Cleanup**: Automatic removal of expired tokens
- **Path Traversal Protection**: Validation of namespace and repository names
- **HTTPS Recommended**: Use HTTPS in production for secure token transmission

## Configuration

Environment variables:

```bash
# Authentication
AUTH_TOKEN_SECRET=change_this_in_production
AUTH_TOKEN_EXPIRY_HOURS=24

# Storage
SYNC_DIR=/var/oxen/data
```

## Backward Compatibility

Repositories created before the authentication system was added will:
- Allow access if no `project.json` file exists
- Can be claimed by creating metadata for them

## Error Codes

- `401 Unauthorized`: Missing or invalid authentication token
- `403 Forbidden`: User doesn't have permission to perform the action
- `404 Not Found`: Repository or resource not found

## Examples

### Complete Workflow

1. **Register a user:**
```bash
curl -X POST http://localhost:3000/api/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"securepass123"}'
```

2. **Create a private repository:**
```bash
curl -X POST http://localhost:3000/api/repos/alice/myproject \
  -H "Authorization: Bearer auxin_<token>" \
  -H "Content-Type: application/json" \
  -d '{"visibility":"private"}'
```

3. **Add a collaborator:**
```bash
curl -X POST http://localhost:3000/api/repos/alice/myproject/collaborators \
  -H "Authorization: Bearer auxin_<token>" \
  -H "Content-Type: application/json" \
  -d '{"user_id":"<bob-uuid>"}'
```

4. **List repositories (shows only accessible ones):**
```bash
curl http://localhost:3000/api/repos \
  -H "Authorization: Bearer auxin_<token>"
```
