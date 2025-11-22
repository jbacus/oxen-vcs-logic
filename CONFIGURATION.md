# Auxin Configuration

Auxin uses a unified configuration system across its CLI and server components, managed primarily through `config.toml` files. This system allows for flexible configuration at different levels, with a clear precedence order.

## Configuration Precedence

Settings are applied in the following order, with higher numbers overriding lower ones:

1.  **Environment Variables**: Highest priority. Useful for CI/CD pipelines, Docker deployments, or temporary overrides without modifying files.
2.  **Project Configuration (`.auxin/config.toml`)**: Located in the root of an Auxin project directory. These settings apply specifically to that project and override user-level defaults. They can be committed to version control to ensure consistent project behavior across teams.
3.  **User Configuration (`~/.auxin/config.toml`)**: Your personal default settings for all Auxin projects. This file is located in your home directory under `.auxin/config.toml` (e.g., `/Users/youruser/.auxin/config.toml` on macOS, or `/home/youruser/.config/auxin/config.toml` on Linux).
4.  **Built-in Defaults**: Lowest priority. These are the default values compiled into the Auxin binaries.

## Configuration File Structure (`config.toml`)

The `config.toml` file is structured into several sections, each managing a different aspect of Auxin's behavior. A comprehensive example with all available options and their default values can be found in `config.toml.example` in the project root.

### `[defaults]`

General settings applicable to both CLI and server.

*   `verbose`: (boolean) Enable verbose output. Can be overridden by the `--verbose` or `-v` CLI flag.
    *   Environment Variable: `AUXIN_VERBOSE`
*   `color`: (string) Control colored output. Options: `"auto"`, `"always"`, `"never"`.
    *   Environment Variable: `AUXIN_COLOR`

### `[lock]`

Settings related to lock management for collaborative workflows.

*   `timeout_hours`: (integer) Default duration (in hours) for which a lock is held before automatic expiration.
    *   Environment Variable: `AUXIN_LOCK_TIMEOUT`
*   `auto_renew`: (boolean) Whether locks should be automatically renewed by a background daemon. (Currently not fully implemented)
*   `renew_before_minutes`: (integer) How many minutes before expiration to attempt auto-renewal.

### `[network]`

Settings for network operations, affecting resilience and behavior with remote servers.

*   `max_retries`: (integer) Maximum number of retry attempts for failed network operations.
    *   Environment Variable: `AUXIN_MAX_RETRIES`
*   `initial_backoff_ms`: (integer) Initial delay in milliseconds before the first retry (uses exponential backoff).
*   `max_backoff_ms`: (integer) Maximum delay in milliseconds for exponential backoff.
*   `connectivity_check_interval_s`: (integer) How often (in seconds) to check network connectivity.
*   `connectivity_check_timeout_s`: (integer) How long (in seconds) to wait for a connectivity check to succeed before declaring the network unreachable.

### `[queue]`

Settings for the offline operation queue (CLI only).

*   `auto_sync`: (boolean) Automatically synchronize pending operations when network connectivity is restored.
    *   Environment Variable: `AUXIN_AUTO_SYNC`
*   `queue_dir`: (string) Filesystem path to store queued operations. Supports `~` for home directory expansion.
    *   Environment Variable: `AUXIN_QUEUE_DIR`
*   `max_entries`: (integer) Maximum number of completed queue entries to retain. Oldest entries are removed when the limit is reached.
*   `cleanup_after_days`: (integer) Automatically remove completed queue entries older than this many days. Set to `0` to disable.

### `[ui]`

User interface settings for the CLI.

*   `progress`: (boolean) Display progress bars and spinners for long-running operations.
*   `emoji`: (boolean) Use emoji characters in CLI output (e.g., `✓`, `✗`, `⚠️`). Disable if your terminal does not support them.
*   `terminal_width`: (integer) Specifies the terminal width for wrapping output. Set to `0` for auto-detection.

### `[project]`

Default project-related settings (CLI only).

*   `project_type`: (string) Default type for new Auxin projects. Options: `"auto"`, `"logicpro"`, `"sketchup"`, `"blender"`.
    *   Environment Variable: `AUXIN_PROJECT_TYPE`

### `[cli]`

Settings specific to how the CLI connects and interacts with an Auxin server.

*   `url`: (string) The base URL of the Auxin server (e.g., `http://localhost:3000`).
    *   Environment Variable: `AUXIN_SERVER_URL`
*   `token`: (string, optional) Authentication token for the Auxin server.
    *   Environment Variable: `AUXIN_SERVER_TOKEN`
*   `timeout_secs`: (integer) Request timeout in seconds for server API calls.
    *   Environment Variable: `AUXIN_SERVER_TIMEOUT_SECS`
*   `use_server_locks`: (boolean) If `true`, the CLI will use the Auxin server for distributed lock management. If `false`, local Git-based locking will be used.
    *   Environment Variable: `AUXIN_USE_SERVER_LOCKS`
*   `use_server_metadata`: (boolean) If `true`, the CLI will store and retrieve project metadata (BPM, key, etc.) from the Auxin server. If `false`, metadata will be stored locally within the `.oxen/metadata` directory.
    *   Environment Variable: `AUXIN_USE_SERVER_METADATA`
*   `default_namespace`: (string, optional) The default namespace to use when creating or interacting with repositories on the Auxin server.
    *   Environment Variable: `AUXIN_DEFAULT_NAMESPACE`

### `[server]`

Settings specific to the operation of the `auxin-server` component.

*   `sync_dir`: (string) The absolute path to the directory where the Auxin server will store all repository data.
    *   Environment Variable: `AUXIN_SERVER_SYNC_DIR`
*   `host`: (string) The host address the Auxin server will bind to (e.g., `0.0.0.0` for all interfaces, `127.0.0.1` for localhost only).
    *   Environment Variable: `AUXIN_SERVER_HOST`
*   `port`: (integer) The port number the Auxin server will listen on.
    *   Environment Variable: `AUXIN_SERVER_PORT`
*   `auth_token_secret`: (string) A secret key used for signing authentication tokens. **IMPORTANT: Change this to a strong, unique value in production environments!**
    *   Environment Variable: `AUXIN_SERVER_AUTH_TOKEN_SECRET`
*   `auth_token_expiry_hours`: (integer) The duration (in hours) before authentication tokens expire.
    *   Environment Variable: `AUXIN_SERVER_AUTH_TOKEN_EXPIRY_HOURS`
*   `enable_redis_locks`: (boolean) If `true`, enables Redis for distributed lock management across multiple server instances. Requires `redis_url` to be configured.
    *   Environment Variable: `AUXIN_SERVER_ENABLE_REDIS_LOCKS`
*   `enable_web_ui`: (boolean) If `true`, enables serving the web-based user interface (frontend) from the server. Requires frontend assets to be built.
    *   Environment Variable: `AUXIN_SERVER_ENABLE_WEB_UI`
*   `redis_url`: (string, optional) The connection URL for a Redis server (e.g., `redis://127.0.0.1/`). Required if `enable_redis_locks` is `true`.
    *   Environment Variable: `AUXIN_SERVER_REDIS_URL`
*   `database_url`: (string, optional) The connection URL for the database (e.g., `sqlite://data.db` for a local SQLite file, or a PostgreSQL connection string). Required if `enable_web_ui` is `true` and project CRUD operations are desired.
    *   Environment Variable: `AUXIN_SERVER_DATABASE_URL`

## Example Usage

To configure Auxin, you can create a `config.toml` file in your user configuration directory (`~/.auxin/config.toml`) or within a specific project (`.auxin/config.toml`).

```toml
# ~/.auxin/config.toml
[defaults]
verbose = true

[cli]
url = "https://auxin.mycompany.com"
default_namespace = "myteam"

[server]
sync_dir = "/mnt/auxin_data"
port = 8080
auth_token_secret = "super_secret_production_key_123"
enable_redis_locks = true
redis_url = "redis://my-redis-instance:6379/"
```

## Docker Deployment

When deploying the Auxin server in Docker, you have three flexible options for configuration:

### Option 1: Use Default Configuration

The Docker image includes a default `config.docker.toml` file with sensible defaults for containerized deployments. To use it as-is:

```bash
docker-compose up
```

The default configuration uses:
- Data directory: `/var/oxen/data` (mounted volume)
- Host: `0.0.0.0` (all interfaces)
- Port: `3000`
- Auth secret: `dev_secret_change_in_production` ⚠️ **Change this in production!**

### Option 2: Mount Custom Configuration File

For production deployments, create your own `config.toml` and mount it into the container:

1. Create your custom configuration:

```bash
cd auxin-server
cp config.docker.toml config.toml
# Edit config.toml with your production settings
```

2. Update `docker-compose.yml` to mount your config:

```yaml
services:
  auxin-server:
    volumes:
      - auxin-data:/var/oxen/data
      - ./config.toml:/app/config.toml:ro  # Mount custom config
```

3. Start the container:

```bash
docker-compose up
```

### Option 3: Override with Environment Variables

You can override any configuration value using environment variables with the `AUXIN_` prefix. This is ideal for CI/CD pipelines or when you want to keep secrets out of config files.

Update your `docker-compose.yml`:

```yaml
services:
  auxin-server:
    environment:
      # Logging (not part of unified config)
      - RUST_LOG=info,auxin_server=debug

      # Override server settings
      - AUXIN_SERVER_PORT=3000
      - AUXIN_SERVER_HOST=0.0.0.0
      - AUXIN_SERVER_AUTH_TOKEN_SECRET=your_secret_here

      # Enable optional features
      - AUXIN_SERVER_ENABLE_REDIS_LOCKS=true
      - AUXIN_SERVER_REDIS_URL=redis://redis:6379
```

Environment variables take precedence over `config.toml` values, allowing you to:
- Keep base configuration in `config.toml`
- Override sensitive values (secrets) via environment variables
- Use different settings per environment (dev/staging/prod)

### Complete Docker Compose Example

Here's a production-ready `docker-compose.yml` example:

```yaml
version: '3.8'

services:
  auxin-server:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: auxin-server
    ports:
      - "3000:3000"
    volumes:
      - auxin-data:/var/oxen/data
      - ./config.toml:/app/config.toml:ro  # Optional: custom config
    environment:
      - RUST_LOG=info,auxin_server=debug
      - AUXIN_SERVER_AUTH_TOKEN_SECRET=${AUXIN_AUTH_SECRET}  # From .env file
      - AUXIN_SERVER_ENABLE_REDIS_LOCKS=true
      - AUXIN_SERVER_REDIS_URL=redis://redis:6379
    restart: unless-stopped
    depends_on:
      - redis

  redis:
    image: redis:7-alpine
    container_name: auxin-redis
    ports:
      - "6379:6379"
    volumes:
      - redis-data:/data
    restart: unless-stopped

volumes:
  auxin-data:
    driver: local
  redis-data:
    driver: local
```

### Migration from .env Files (Deprecated)

**Note:** The `.env` file approach is deprecated. If you're migrating from an older deployment:

| Old Variable (`.env`)       | New Variable (`AUXIN_*` prefix)           |
|-----------------------------|-------------------------------------------|
| `SYNC_DIR`                  | `AUXIN_SERVER_SYNC_DIR`                   |
| `OXEN_SERVER_HOST`          | `AUXIN_SERVER_HOST`                       |
| `OXEN_SERVER_PORT`          | `AUXIN_SERVER_PORT`                       |
| `AUTH_TOKEN_SECRET`         | `AUXIN_SERVER_AUTH_TOKEN_SECRET`          |
| `AUTH_TOKEN_EXPIRY_HOURS`   | `AUXIN_SERVER_AUTH_TOKEN_EXPIRY_HOURS`    |
| `ENABLE_REDIS_LOCKS`        | `AUXIN_SERVER_ENABLE_REDIS_LOCKS`         |
| `REDIS_URL`                 | `AUXIN_SERVER_REDIS_URL`                  |
| `ENABLE_WEB_UI`             | `AUXIN_SERVER_ENABLE_WEB_UI`              |
| `DATABASE_URL`              | `AUXIN_SERVER_DATABASE_URL`               |

Instead of a `.env` file, use either:
1. A `config.toml` file (recommended for persistent settings)
2. Environment variables with `AUXIN_SERVER_*` prefix (recommended for secrets)

### Security Best Practices for Docker

1. **Never use default secrets in production!** Always override `auth_token_secret`:
   ```bash
   AUXIN_SERVER_AUTH_TOKEN_SECRET=$(openssl rand -base64 32)
   ```

2. **Use Docker secrets** for sensitive values in production:
   ```yaml
   secrets:
     auth_token:
       external: true

   services:
     auxin-server:
       secrets:
         - auth_token
       environment:
         - AUXIN_SERVER_AUTH_TOKEN_SECRET=/run/secrets/auth_token
   ```

3. **Mount config files as read-only** (`:ro` flag) to prevent container modifications

4. **Use environment-specific configs**: Maintain separate `config.toml` files for development, staging, and production

For the complete Docker configuration reference, see `auxin-server/config.docker.toml` in the repository.