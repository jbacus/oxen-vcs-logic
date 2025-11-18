# Auxin Server Deployment Scripts

Local deployment scripts for setting up and managing auxin-server on macOS.

## Quick Start

```bash
# 1. Install and setup
./setup.sh

# 2. Start the server
./start.sh

# 3. Check status
./status.sh

# 4. Test the server
curl http://localhost:3000/health
curl http://localhost:3000/api/repos
```

## Scripts Overview

### `setup.sh` - Initial Installation

Performs complete server installation:

- ✅ Checks for macOS and Rust
- ✅ Creates data directories (`/var/oxen/data`)
- ✅ Creates config directory (`~/.config/auxin-server`)
- ✅ Generates environment configuration with secure secrets
- ✅ Builds release binary
- ✅ Installs binary to `/usr/local/bin/auxin-server`
- ✅ Creates LaunchAgent plist for automatic service management

**Usage:**
```bash
./setup.sh
```

**Requirements:**
- macOS (tested on macOS 14.0+)
- Rust toolchain (`rustup`)
- sudo access (for `/var/oxen` and `/usr/local/bin`)

### `start.sh` - Start Server

Loads the LaunchAgent and starts the server.

**Usage:**
```bash
./start.sh
```

**What it does:**
- Loads the LaunchAgent plist
- Waits for server to initialize
- Tests health endpoint
- Reports status

**Logs:**
- `~/Library/Logs/auxin-server.log` - stdout
- `~/Library/Logs/auxin-server-error.log` - stderr

### `stop.sh` - Stop Server

Gracefully stops the server by unloading the LaunchAgent.

**Usage:**
```bash
./stop.sh
```

### `restart.sh` - Restart Server

Convenience script that stops then starts the server.

**Usage:**
```bash
./restart.sh
```

### `status.sh` - Check Status

Comprehensive status report including:

- ✅ Installation check
- ✅ Service running status
- ✅ Process ID and memory usage
- ✅ HTTP endpoint test
- ✅ Repository count
- ✅ Configuration summary
- ✅ Log file status and recent errors

**Usage:**
```bash
./status.sh
```

**Example output:**
```
===== Auxin Server Status =====

[✓] Binary installed: /usr/local/bin/auxin-server
[✓] LaunchAgent configured
[✓] Service is running
  PID: 12345
  Memory: 45.2 MB

[INFO] Testing HTTP endpoint...
[✓] Server responding at http://localhost:3000
  Repositories: 0

[INFO] Configuration:
[✓] Config file: /Users/john/.config/auxin-server/.env
  Port: 3000
  Data Dir: /var/oxen/data

[INFO] Logs:
[✓] Output log: /Users/john/Library/Logs/auxin-server.log (2.3K, 45 lines)
[✓] Error log: (no errors)
```

### `uninstall.sh` - Complete Removal

Interactive uninstallation script with safety prompts.

**Usage:**
```bash
./uninstall.sh
```

**What it removes:**
1. Running service (stops if active)
2. LaunchAgent plist
3. Binary from `/usr/local/bin`
4. Configuration (asks for confirmation)
5. Data directory (asks for double confirmation)
6. Log files (asks for confirmation)

**Safety features:**
- Requires confirmation before proceeding
- Asks separately for config, data, and logs
- Data directory requires typing "yes" (not just "y")

## Configuration

### Location

Configuration file: `~/.config/auxin-server/.env`

### Default Settings

```bash
# Server
SYNC_DIR=/var/oxen/data          # Repository storage location
OXEN_SERVER_PORT=3000             # HTTP server port
OXEN_SERVER_HOST=127.0.0.1        # Bind to localhost only (secure)

# Authentication
AUTH_TOKEN_SECRET=<random-32-byte-hex>  # Auto-generated
AUTH_TOKEN_EXPIRY_HOURS=24

# Logging
RUST_LOG=info,auxin_server=debug

# Optional Features (disabled by default)
ENABLE_REDIS_LOCKS=false
ENABLE_WEB_UI=false
```

### Editing Configuration

```bash
# Edit config
nano ~/.config/auxin-server/.env

# Restart to apply changes
./restart.sh
```

### Enabling Optional Features

**Redis Locks (for distributed locking):**
```bash
# 1. Install Redis
brew install redis
brew services start redis

# 2. Enable in config
ENABLE_REDIS_LOCKS=true
REDIS_URL=redis://localhost:6379

# 3. Rebuild with feature
cd .. && cargo build --release --features redis-locks
sudo cp target/release/auxin-server /usr/local/bin/
./restart.sh
```

**Web UI (requires PostgreSQL):**
```bash
# 1. Install PostgreSQL
brew install postgresql@15
brew services start postgresql@15
createdb auxin

# 2. Enable in config
ENABLE_WEB_UI=true
DATABASE_URL=postgres://yourusername@localhost:5432/auxin

# 3. Rebuild with feature
cd .. && cargo build --release --features web-ui
sudo cp target/release/auxin-server /usr/local/bin/
./restart.sh
```

## Directory Structure

```
~/.config/auxin-server/           # Configuration
  └── .env                         # Environment config

/var/oxen/data/                    # Repository storage
  └── {namespace}/
      └── {repo_name}/
          └── .oxen/

/usr/local/bin/                    # Binary
  └── auxin-server

~/Library/LaunchAgents/            # Service management
  └── com.auxin.server.plist

~/Library/Logs/                    # Logs
  ├── auxin-server.log
  └── auxin-server-error.log
```

## Common Tasks

### Viewing Logs

```bash
# Real-time log watching
tail -f ~/Library/Logs/auxin-server.log

# Error log
tail -f ~/Library/Logs/auxin-server-error.log

# Last 50 lines
tail -50 ~/Library/Logs/auxin-server.log

# Search logs
grep "ERROR" ~/Library/Logs/auxin-server.log
```

### Testing API Endpoints

```bash
# Health check
curl http://localhost:3000/health

# List repositories
curl http://localhost:3000/api/repos

# Create a repository
curl -X POST http://localhost:3000/api/repos/myuser/myrepo \
  -H "Content-Type: application/json" \
  -d '{"description": "My first repository"}'

# Get repository info
curl http://localhost:3000/api/repos/myuser/myrepo
```

### Updating the Server

```bash
# 1. Stop the server
./stop.sh

# 2. Pull latest changes
cd .. && git pull

# 3. Rebuild
cargo build --release

# 4. Reinstall binary
sudo cp target/release/auxin-server /usr/local/bin/

# 5. Start the server
./start.sh
```

### Troubleshooting

**Server won't start:**
```bash
# Check if port 3000 is in use
lsof -i :3000

# Check error log
cat ~/Library/Logs/auxin-server-error.log

# Check LaunchAgent status
launchctl list | grep com.auxin.server

# Check permissions on data directory
ls -la /var/oxen/data
```

**High memory usage:**
```bash
# Check process stats
./status.sh

# Check for large repositories
du -sh /var/oxen/data/*

# Restart to clear cache
./restart.sh
```

**Cannot access from other machines:**

By default, the server binds to `127.0.0.1` (localhost only) for security.

To allow network access:
```bash
# Edit config
nano ~/.config/auxin-server/.env

# Change host to 0.0.0.0 (WARNING: ensure firewall is configured!)
OXEN_SERVER_HOST=0.0.0.0

# Restart
./restart.sh
```

## LaunchAgent Details

The LaunchAgent plist configures macOS to manage the server process:

- **Label:** `com.auxin.server`
- **RunAtLoad:** Service starts automatically on login
- **KeepAlive:** Service restarts if it crashes
- **ProcessType:** Interactive (normal priority)
- **Logs:** Redirect to `~/Library/Logs/`

To manually manage:
```bash
# Load (start)
launchctl load ~/Library/LaunchAgents/com.auxin.server.plist

# Unload (stop)
launchctl unload ~/Library/LaunchAgents/com.auxin.server.plist

# Check status
launchctl list | grep com.auxin.server
```

## Development Mode

For development, you may want to run the server directly instead of via LaunchAgent:

```bash
# Stop LaunchAgent version
./stop.sh

# Run directly with debug logging
cd .. && RUST_LOG=debug cargo run

# Or run release binary directly
cd .. && RUST_LOG=debug ./target/release/auxin-server
```

## Security Considerations

1. **Default binding:** Server binds to `127.0.0.1` (localhost only)
2. **Auth secret:** Randomly generated 32-byte secret on installation
3. **File permissions:** Config file set to `600` (owner read/write only)
4. **Data directory:** Owned by current user, not world-readable
5. **Logs:** May contain sensitive information, stored in user's home directory

**For production deployment:**
- Use HTTPS reverse proxy (nginx/caddy)
- Set strong `AUTH_TOKEN_SECRET`
- Configure firewall rules
- Enable Redis locks for distributed scenarios
- Regular backups of `/var/oxen/data`

## Contributing

When modifying the scripts:
- Follow existing style (colors, logging functions)
- Test on clean macOS installation
- Update this README
- Ensure scripts remain idempotent where possible

## License

MIT (see ../README.md)
