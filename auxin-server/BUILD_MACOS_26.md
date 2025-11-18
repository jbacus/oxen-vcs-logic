# Building auxin-server on macOS 26.x

## Problem

The original `liboxen 0.10` dependency includes DuckDB, which has C++ compilation issues on macOS 26.x (development/beta versions) due to SDK incompatibilities.

## Solution Implemented

We've implemented **feature flags** to make liboxen optional, allowing the server to build without the full Oxen functionality while maintaining all server infrastructure and Auxin-specific features (locks, metadata).

### Feature Flags

```toml
[features]
default = ["mock-oxen"]  # Use mock by default until liboxen 0.38 API is updated
full-oxen = ["liboxen"]  # Full Oxen functionality (TODO: update for liboxen 0.38 API changes)
mock-oxen = []           # Mock implementation for development/testing
```

### What Works in Mock Mode

✅ **Fully Functional:**
- HTTP server (Actix Web)
- API endpoints (/health, /api/repos)
- Authentication & token management
- Repository discovery (scans .oxen directories)
- **Auxin extensions** (Logic Pro metadata, distributed locks)
- LaunchAgent service management
- Configuration management
- All deployment scripts

❌ **Not Implemented (returns 501 Not Implemented):**
- VCS operations (add, commit, push, pull, clone)
- Branch management (create, checkout, merge)
- Commit history

### Building

#### Option 1: Mock Implementation (Default - Works on macOS 26.x)

```bash
# Build with mock Oxen (no liboxen dependency)
cargo build --release

# Binary location
./target/release/auxin-server
```

This builds successfully on **macOS 26.x** without any C++ compilation issues.

#### Option 2: Full Oxen (Requires liboxen 0.38 API Update)

```bash
# Build with full Oxen functionality (currently broken - API changes)
cargo build --release --no-default-features --features full-oxen
```

**Note:** liboxen 0.38 has significant API changes from 0.10. The `api::local::` module structure has been reorganized. Full-oxen feature requires updating the API calls in `src/repo_full.rs`.

### Testing

```bash
# Run tests
cargo test

# Start server (needs /var/oxen/data directory)
sudo mkdir -p /var/oxen/data
sudo chown $USER /var/oxen/data
./target/release/auxin-server
```

Test endpoints:
```bash
curl http://localhost:3000/health
# Expected: OK

curl http://localhost:3000/api/repos
# Expected: []
```

### Deployment

The deployment scripts work perfectly with the mock implementation:

```bash
cd scripts
./setup.sh    # Builds and installs
./start.sh    # Starts via LaunchAgent
./status.sh   # Check status
./stop.sh     # Stops server
```

All server infrastructure, monitoring, logging, and service management work identically whether using mock or full Oxen.

## Architecture

### File Structure

```
src/
├── repo_full.rs    # Full Oxen implementation (uses liboxen 0.38 - needs API update)
├── repo_mock.rs    # Mock implementation (VCS ops return NotImplemented)
└── lib.rs          # Conditionally exports repo module based on feature

// In lib.rs:
#[cfg(feature = "full-oxen")]
#[path = "repo_full.rs"]
pub mod repo;

#[cfg(feature = "mock-oxen")]
#[path = "repo_mock.rs"]
pub mod repo;
```

### Mock Implementation Details

The mock (`repo_mock.rs`) provides:

1. **Repository initialization** - Creates minimal `.oxen` directory structure
2. **Repository discovery** - Opens existing `.oxen` directories
3. **Auxin extensions** - Full support for locks and Logic Pro metadata
4. **HTTP API** - All endpoints respond correctly (VCS ops return 501)

Example mock initialization:
```rust
pub fn init(repo_path: impl AsRef<Path>) -> AppResult<Self> {
    let oxen_dir = repo_path.join(".oxen");
    std::fs::create_dir_all(&oxen_dir)?;

    // Create HEAD
    std::fs::write(oxen_dir.join("HEAD"), "refs/heads/main\n")?;

    // Create refs structure
    std::fs::create_dir_all(oxen_dir.join("refs/heads"))?;

    // Auxin extensions (fully functional)
    std::fs::create_dir_all(oxen_dir.join("metadata"))?;
    std::fs::create_dir_all(oxen_dir.join("locks"))?;

    Ok(Self { repo_path })
}
```

## Updating to Full Oxen (liboxen 0.38)

When ready to implement full Oxen functionality:

### 1. Update API Calls in repo_full.rs

The liboxen 0.38 API has changed:

**Old (0.10):**
```rust
use liboxen::api;

api::local::repositories::init(&path)?;
api::local::staging::add(&repo, &file)?;
api::local::commits::commit(&repo, message)?;
```

**New (0.38):** *(TODO: determine exact API)*
```rust
use liboxen::repositories;
use liboxen::core::v_latest::branches;

// API structure has changed - needs investigation
```

### 2. Update Default Feature

Once `repo_full.rs` is updated for liboxen 0.38:

```toml
[features]
default = ["full-oxen"]  # Switch back to full Oxen
```

### 3. Test Full Integration

```bash
cargo build --release --features full-oxen
cargo test --features full-oxen
```

## Benefits of This Solution

1. ✅ **Builds on macOS 26.x** - No DuckDB compilation issues
2. ✅ **Server infrastructure testable** - Can test deployment, HTTP API, auth, etc.
3. ✅ **Auxin features work** - Locks and metadata fully functional
4. ✅ **Clear separation** - Mock vs full implementation cleanly separated
5. ✅ **Future-proof** - Easy to switch when liboxen API is updated
6. ✅ **Development-friendly** - Fast builds without C++ compilation

## Current Status

- **Mock Implementation:** ✅ Complete and tested
- **Deployment Scripts:** ✅ Working perfectly
- **Server Infrastructure:** ✅ Fully functional
- **Auxin Extensions:** ✅ Locks and metadata working
- **Full Oxen Implementation:** 🟡 Needs update for liboxen 0.38 API

## Next Steps

1. **For Testing/Development:** Use mock implementation (default) - works perfectly
2. **For Production:** Update `repo_full.rs` to liboxen 0.38 API
3. **Alternative:** Wait for liboxen 0.39+ which may have better macOS 26.x compatibility

## Conclusion

**You can now build and run auxin-server on macOS 26.x!**

The mock implementation provides full server functionality except for VCS operations, which is perfect for:
- Testing deployment infrastructure
- Developing Auxin-specific features (locks, metadata)
- CI/CD pipeline testing
- Documentation and script development

When full VCS functionality is needed, update the liboxen API integration in `repo_full.rs`.
