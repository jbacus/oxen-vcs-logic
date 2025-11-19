# liboxen FFI Migration Guide

This document describes how to migrate Auxin from the subprocess-based Oxen integration to direct liboxen FFI bindings when the crate becomes available.

## Current Architecture

Auxin currently uses `OxenSubprocess` which executes `oxen` CLI commands via `std::process::Command`:

```
Auxin → OxenSubprocess → oxen CLI → Oxen operations
```

**Performance overhead:** ~50-500ms per operation due to process spawning and output parsing.

## Target Architecture

With liboxen FFI, operations will be direct function calls:

```
Auxin → OxenBackend trait → FFIBackend → liboxen → Oxen operations
```

**Expected performance:** <1-50ms per operation (10-100x improvement).

## Migration Steps

### 1. Add liboxen Dependency

When published, add to `Cargo.toml`:

```toml
[dependencies]
liboxen = "0.1"  # Use actual version when available
```

### 2. Implement FFIBackend

In `src/oxen_backend.rs`, implement the `OxenBackend` trait for `FFIBackend`:

```rust
use liboxen::{Repository, Commit};

pub struct FFIBackend {
    // Connection pool for remote operations
    pool: Option<ConnectionPool>,
}

impl FFIBackend {
    pub fn new() -> Result<Self> {
        Ok(Self { pool: None })
    }
}

impl OxenBackend for FFIBackend {
    fn is_available(&self) -> bool {
        true  // Always available when compiled with liboxen
    }

    fn version(&self) -> Result<String> {
        Ok(liboxen::version().to_string())
    }

    fn init(&self, path: &Path) -> Result<()> {
        Repository::init(path)
            .map_err(|e| anyhow::anyhow!("Failed to init: {}", e))?;
        Ok(())
    }

    fn add(&self, repo_path: &Path, files: &[&Path]) -> Result<()> {
        let repo = Repository::open(repo_path)?;
        for file in files {
            repo.add(file)?;
        }
        Ok(())
    }

    fn commit(&self, repo_path: &Path, message: &str) -> Result<CommitInfo> {
        let repo = Repository::open(repo_path)?;
        let commit = repo.commit(message)?;
        Ok(CommitInfo {
            id: commit.id().to_string(),
            message: commit.message().to_string(),
        })
    }

    // ... implement remaining methods

    fn backend_type(&self) -> BackendType {
        BackendType::FFI
    }

    fn name(&self) -> &'static str {
        "liboxen FFI"
    }
}
```

### 3. Update Backend Factory

Modify `create_backend()` in `src/oxen_backend.rs`:

```rust
pub fn create_backend(backend_type: BackendType) -> Result<Box<dyn OxenBackend>> {
    match backend_type {
        BackendType::Subprocess => {
            Ok(Box::new(SubprocessBackend::default()))
        }
        BackendType::FFI => {
            Ok(Box::new(FFIBackend::new()?))
        }
    }
}
```

### 4. Add Feature Flag (Optional)

For gradual rollout, add a feature flag:

```toml
[features]
default = ["subprocess"]
subprocess = []
ffi = ["liboxen"]
```

```rust
#[cfg(feature = "ffi")]
pub fn create_default_backend() -> Result<Box<dyn OxenBackend>> {
    create_backend(BackendType::FFI)
}

#[cfg(not(feature = "ffi"))]
pub fn create_default_backend() -> Result<Box<dyn OxenBackend>> {
    create_backend(BackendType::Subprocess)
}
```

### 5. Validation Strategy

Run both backends in parallel to validate:

```rust
pub fn validate_backends(repo_path: &Path) -> Result<()> {
    let subprocess = create_backend(BackendType::Subprocess)?;
    let ffi = create_backend(BackendType::FFI)?;

    // Compare results
    let sub_status = subprocess.status(repo_path)?;
    let ffi_status = ffi.status(repo_path)?;

    assert_eq!(sub_status.modified, ffi_status.modified);
    assert_eq!(sub_status.untracked, ffi_status.untracked);

    Ok(())
}
```

## Performance Comparison

| Operation | Subprocess | FFI (Expected) | Improvement |
|-----------|------------|----------------|-------------|
| init      | ~100ms     | <10ms          | 10x         |
| add (single) | ~50ms   | <1ms           | 50x         |
| add (batch) | ~50ms    | <5ms           | 10x         |
| commit    | ~500ms     | <50ms          | 10x         |
| log (100) | ~200ms     | <20ms          | 10x         |
| status    | ~100ms     | <10ms          | 10x         |
| checkout  | ~300ms     | <30ms          | 10x         |

## Error Handling

Convert liboxen errors to anyhow:

```rust
use liboxen::Error as OxenError;

fn convert_error(e: OxenError) -> anyhow::Error {
    match e {
        OxenError::NotFound(msg) => anyhow::anyhow!("Not found: {}", msg),
        OxenError::InvalidPath(p) => anyhow::anyhow!("Invalid path: {:?}", p),
        OxenError::Network(msg) => anyhow::anyhow!("Network error: {}", msg),
        _ => anyhow::anyhow!("Oxen error: {}", e),
    }
}
```

## Security Benefits

FFI provides additional security benefits:

1. **No command injection** - No shell execution, paths are typed
2. **Type safety** - Compile-time verification of arguments
3. **Better error messages** - Stack traces instead of stderr parsing
4. **No subprocess masking** - Errors propagate directly

## Deprecation Timeline

1. **Phase 1** (Current): Subprocess only
2. **Phase 2** (liboxen published): Both available, subprocess default
3. **Phase 3** (Validation complete): FFI default, subprocess available
4. **Phase 4** (6 months after Phase 3): Subprocess deprecated
5. **Phase 5** (12 months after Phase 3): Subprocess removed

## Testing

Add integration tests that run against both backends:

```rust
#[test]
fn test_init_both_backends() {
    for backend_type in [BackendType::Subprocess, BackendType::FFI] {
        let backend = create_backend(backend_type).unwrap();
        let temp = tempfile::TempDir::new().unwrap();

        backend.init(temp.path()).unwrap();

        // Verify .oxen directory exists
        assert!(temp.path().join(".oxen").exists());
    }
}
```

## Tracking

- liboxen repository: https://github.com/Oxen-AI/Oxen
- Auxin issue: Track in GitHub issues when migration begins
- Performance benchmarks: Add to CI when FFI is available

## Questions?

- Oxen.ai community: hello@oxen.ai
- Auxin repository: https://github.com/jbacus/auxin
