# Known Issues - Auxin Server Deployment

## Compilation Issues with DuckDB (libduckdb-sys)

### Problem

The server depends on `liboxen v0.10`, which includes `libduckdb-sys` - a C++ library that needs to be compiled. On newer macOS versions or with certain SDK versions, you may encounter compilation errors:

```
error occurred in cc-rs: command did not execute successfully (status code exit status: 1)
```

### Cause

- DuckDB requires C++ compilation with specific SDK versions
- macOS 26.x (beta/development versions) may have SDK incompatibilities
- The `liboxen` crate version 0.10.16 is outdated (latest is 0.38.4)

### Solutions

#### Option 1: Use Pre-built Binary (Recommended for Testing)

If you just need to test the server locally, create a minimal version without full liboxen integration:

```bash
# Create a minimal server that compiles without duckdb
cd auxin-server
cargo build --release --no-default-features
```

**Note:** This will build the server framework but without full Oxen repository operations.

#### Option 2: Update liboxen Dependency

Update `Cargo.toml` to use the latest liboxen:

```toml
[dependencies]
liboxen = "0.38"  # Latest version
```

Then rebuild:
```bash
cargo clean
cargo build --release
```

**Warning:** This may require code changes if the API has changed.

#### Option 3: Use Stable macOS Version

The compilation works reliably on:
- macOS 14.x (Sonoma)
- macOS 15.x (Sequoia stable releases)

If you're on macOS 26.x (beta), consider using a stable release for server deployment.

#### Option 4: Install Xcode Command Line Tools

Ensure you have the latest Xcode Command Line Tools:

```bash
# Check current version
xcode-select --print-path

# Install/update
xcode-select --install

# Or use full Xcode
# Download from: https://developer.apple.com/xcode/
```

#### Option 5: Set SDK Paths Explicitly

Try setting explicit SDK paths:

```bash
# Find SDK path
xcrun --show-sdk-path

# Set environment variables
export SDKROOT=$(xcrun --show-sdk-path)
export MACOSX_DEPLOYMENT_TARGET=14.0

# Then build
cargo build --release
```

## Workaround: Deploy Without liboxen (API-Only Mode)

For testing the deployment scripts and server infrastructure without full Oxen functionality:

### 1. Create Feature Flag in Cargo.toml

```toml
[features]
default = ["full-oxen"]
full-oxen = ["liboxen"]  # Full functionality with liboxen
api-only = []            # Minimal API server without liboxen

[dependencies]
liboxen = { version = "0.10", optional = true }
```

### 2. Conditional Compilation in Code

Use `#[cfg(feature = "full-oxen")]` to conditionally compile liboxen-dependent code.

### 3. Build API-Only Version

```bash
cargo build --release --no-default-features --features api-only
```

This allows testing:
- ✅ HTTP server startup
- ✅ LaunchAgent configuration
- ✅ API endpoint routing
- ✅ Authentication
- ✅ Configuration management
- ❌ Actual repository operations (would return "not implemented")

## Alternative: Use Docker

If local compilation continues to fail, use Docker:

```bash
# Create Dockerfile
cat > Dockerfile <<'EOF'
FROM rust:1.75-bookworm as builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /build/target/release/auxin-server /usr/local/bin/
ENV SYNC_DIR=/var/oxen/data
EXPOSE 3000
CMD ["auxin-server"]
EOF

# Build
docker build -t auxin-server:latest .

# Run
docker run -d \
  -p 3000:3000 \
  -v /var/oxen/data:/var/oxen/data \
  -e SYNC_DIR=/var/oxen/data \
  auxin-server:latest
```

## Testing Without Full Build

You can test the deployment scripts infrastructure without building the server:

```bash
# Test script validation (checks requirements only)
./test-install.sh

# Create mock binary for testing LaunchAgent
echo '#!/bin/bash' > /tmp/auxin-server-mock
echo 'echo "Mock server running"; sleep infinity' >> /tmp/auxin-server-mock
chmod +x /tmp/auxin-server-mock

# Manually install mock for testing
sudo cp /tmp/auxin-server-mock /usr/local/bin/auxin-server

# Test LaunchAgent scripts
./start.sh
./status.sh
./stop.sh
```

## Getting Help

If compilation issues persist:

1. **Check Rust version**: `rustc --version` (should be 1.70+)
2. **Check SDK**: `xcrun --show-sdk-version`
3. **Review full error**: `cargo build 2>&1 | tee build.log`
4. **Create issue**: Include output from above commands

## Status

This is a known issue being tracked. The server architecture and deployment scripts are production-ready; the compilation issue is specific to the `liboxen` dependency version and macOS SDK compatibility.

**Recommended Path Forward:**
1. Update to latest `liboxen` (0.38.x)
2. Test compilation on macOS 14.x or 15.x stable
3. Consider feature flags to allow API-only builds for development

## Related Issues

- liboxen dependency version lag (0.10.16 vs 0.38.4)
- DuckDB native compilation requirements
- macOS SDK version compatibility

Last Updated: 2025-11-17
