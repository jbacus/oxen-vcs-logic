/// Oxen Backend Abstraction Layer
///
/// This module provides a trait-based abstraction over Oxen operations,
/// allowing for multiple backend implementations:
///
/// 1. `OxenSubprocess` - Current implementation using CLI subprocess (production)
/// 2. `OxenFFI` - Future implementation using direct liboxen FFI (10-100x faster)
///
/// # Migration Strategy
///
/// When liboxen crate is published:
/// 1. Implement `OxenBackend` trait for `OxenFFI`
/// 2. Feature-flag to select implementation
/// 3. Run both in parallel for validation
/// 4. Deprecate subprocess after confirming FFI stability
///
/// # Example
///
/// ```no_run
/// use auxin_cli::oxen_backend::{OxenBackend, BackendType, create_backend};
///
/// // Create backend (defaults to subprocess)
/// let backend = create_backend(BackendType::Subprocess)?;
///
/// // Use backend-agnostic API
/// backend.init(path)?;
/// backend.add(path, &files)?;
/// backend.commit(path, "Message")?;
/// ```

use anyhow::Result;
use std::path::Path;

// Re-export common types
pub use crate::oxen_subprocess::{BranchInfo, CommitInfo, StatusInfo};

/// Backend implementation type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendType {
    /// Use subprocess calls to oxen CLI (current, production-ready)
    Subprocess,
    /// Use direct FFI bindings to liboxen (future, 10-100x faster)
    #[allow(dead_code)]
    FFI,
}

impl Default for BackendType {
    fn default() -> Self {
        // Default to subprocess until FFI is ready
        BackendType::Subprocess
    }
}

/// Trait defining all Oxen VCS operations
///
/// This trait abstracts the underlying Oxen implementation, allowing
/// both subprocess and FFI backends to be used interchangeably.
///
/// # Performance Characteristics
///
/// | Operation | Subprocess | FFI (Expected) |
/// |-----------|------------|----------------|
/// | init      | ~100ms     | <10ms          |
/// | add       | ~50ms      | <1ms           |
/// | commit    | ~500ms     | <50ms          |
/// | log       | ~200ms     | <20ms          |
/// | status    | ~100ms     | <10ms          |
pub trait OxenBackend: Send + Sync {
    /// Check if the backend is available
    fn is_available(&self) -> bool;

    /// Get backend version string
    fn version(&self) -> Result<String>;

    /// Verify backend version compatibility
    fn verify_version(&self) -> Result<()>;

    /// Initialize a new Oxen repository
    fn init(&self, path: &Path) -> Result<()>;

    /// Add files to staging area
    fn add(&self, repo_path: &Path, files: &[&Path]) -> Result<()>;

    /// Add all files to staging area
    fn add_all(&self, repo_path: &Path) -> Result<()>;

    /// Create a commit
    fn commit(&self, repo_path: &Path, message: &str) -> Result<CommitInfo>;

    /// Get commit history
    fn log(&self, repo_path: &Path, limit: Option<usize>) -> Result<Vec<CommitInfo>>;

    /// Get repository status
    fn status(&self, repo_path: &Path) -> Result<StatusInfo>;

    /// Checkout a commit or branch
    fn checkout(&self, repo_path: &Path, target: &str) -> Result<()>;

    /// Create a new branch
    fn create_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()>;

    /// List all branches
    fn list_branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>>;

    /// Get current branch name
    fn current_branch(&self, repo_path: &Path) -> Result<String>;

    /// Delete a branch
    fn delete_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()>;

    /// Push to remote
    fn push(
        &self,
        repo_path: &Path,
        remote: Option<&str>,
        branch: Option<&str>,
    ) -> Result<()>;

    /// Pull from remote
    fn pull(&self, repo_path: &Path) -> Result<()>;

    /// Get backend type
    fn backend_type(&self) -> BackendType;

    /// Get backend name for logging
    fn name(&self) -> &'static str;
}

/// Create a backend instance based on type
///
/// # Arguments
///
/// * `backend_type` - The type of backend to create
///
/// # Returns
///
/// A boxed trait object implementing `OxenBackend`
///
/// # Example
///
/// ```no_run
/// use auxin_cli::oxen_backend::{create_backend, BackendType};
///
/// let backend = create_backend(BackendType::Subprocess)?;
/// println!("Using backend: {}", backend.name());
/// ```
pub fn create_backend(backend_type: BackendType) -> Result<Box<dyn OxenBackend>> {
    match backend_type {
        BackendType::Subprocess => {
            let subprocess = crate::oxen_subprocess::OxenSubprocess::new();
            Ok(Box::new(SubprocessBackend::new(subprocess)))
        }
        BackendType::FFI => {
            // liboxen 0.38.4 is available via the 'ffi' feature flag
            // Build with: cargo build --features ffi
            // Note: Requires chrono 0.4.29 to avoid arrow-arith conflict
            Err(anyhow::anyhow!(
                "FFI backend implementation pending. liboxen v0.38.4 compiles successfully \
                 with chrono 0.4.29 workaround. Implement FFIBackend trait methods to enable."
            ))
        }
    }
}

/// Create the default backend (subprocess)
pub fn create_default_backend() -> Result<Box<dyn OxenBackend>> {
    create_backend(BackendType::default())
}

// ========== Subprocess Backend Implementation ==========

/// Wrapper that implements OxenBackend for OxenSubprocess
pub struct SubprocessBackend {
    inner: crate::oxen_subprocess::OxenSubprocess,
}

impl SubprocessBackend {
    /// Create a new subprocess backend
    pub fn new(subprocess: crate::oxen_subprocess::OxenSubprocess) -> Self {
        Self { inner: subprocess }
    }

    /// Create with default settings
    pub fn default() -> Self {
        Self::new(crate::oxen_subprocess::OxenSubprocess::new())
    }
}

impl OxenBackend for SubprocessBackend {
    fn is_available(&self) -> bool {
        self.inner.is_available()
    }

    fn version(&self) -> Result<String> {
        self.inner.version()
    }

    fn verify_version(&self) -> Result<()> {
        self.inner.verify_version()
    }

    fn init(&self, path: &Path) -> Result<()> {
        self.inner.init(path)
    }

    fn add(&self, repo_path: &Path, files: &[&Path]) -> Result<()> {
        self.inner.add(repo_path, files)
    }

    fn add_all(&self, repo_path: &Path) -> Result<()> {
        self.inner.add_all(repo_path)
    }

    fn commit(&self, repo_path: &Path, message: &str) -> Result<CommitInfo> {
        self.inner.commit(repo_path, message)
    }

    fn log(&self, repo_path: &Path, limit: Option<usize>) -> Result<Vec<CommitInfo>> {
        self.inner.log(repo_path, limit)
    }

    fn status(&self, repo_path: &Path) -> Result<StatusInfo> {
        self.inner.status(repo_path)
    }

    fn checkout(&self, repo_path: &Path, target: &str) -> Result<()> {
        self.inner.checkout(repo_path, target)
    }

    fn create_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        self.inner.create_branch(repo_path, branch_name)
    }

    fn list_branches(&self, repo_path: &Path) -> Result<Vec<BranchInfo>> {
        self.inner.list_branches(repo_path)
    }

    fn current_branch(&self, repo_path: &Path) -> Result<String> {
        self.inner.current_branch(repo_path)
    }

    fn delete_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        self.inner.delete_branch(repo_path, branch_name)
    }

    fn push(
        &self,
        repo_path: &Path,
        remote: Option<&str>,
        branch: Option<&str>,
    ) -> Result<()> {
        self.inner.push(repo_path, remote, branch)
    }

    fn pull(&self, repo_path: &Path) -> Result<()> {
        self.inner.pull(repo_path)
    }

    fn backend_type(&self) -> BackendType {
        BackendType::Subprocess
    }

    fn name(&self) -> &'static str {
        "OxenSubprocess"
    }
}

// ========== FFI Backend Stub ==========

/// FFI backend using direct liboxen bindings
///
/// This is a placeholder that will be implemented when liboxen is published.
/// Expected benefits:
/// - 10-100x performance improvement
/// - Type-safe operations (no string parsing)
/// - No command injection risk
/// - Better error messages with stack traces
///
/// # Implementation Notes
///
/// When liboxen is available:
/// 1. Add `liboxen` to Cargo.toml dependencies
/// 2. Implement all trait methods using liboxen API
/// 3. Handle error conversion from liboxen errors to anyhow
/// 4. Add connection pooling for remote operations
#[allow(dead_code)]
pub struct FFIBackend {
    // When liboxen is available:
    // repo: Option<liboxen::Repository>,
    // config: FFIConfig,
}

#[allow(dead_code)]
impl FFIBackend {
    /// Create a new FFI backend
    ///
    /// # Note
    ///
    /// This is not yet implemented. When liboxen is published:
    /// ```ignore
    /// pub fn new() -> Result<Self> {
    ///     Ok(Self {
    ///         repo: None,
    ///         config: FFIConfig::default(),
    ///     })
    /// }
    /// ```
    pub fn new() -> Result<Self> {
        Err(anyhow::anyhow!("FFI backend not yet implemented"))
    }
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_type_default() {
        assert_eq!(BackendType::default(), BackendType::Subprocess);
    }

    #[test]
    fn test_create_subprocess_backend() {
        let backend = create_backend(BackendType::Subprocess);
        assert!(backend.is_ok());

        let backend = backend.unwrap();
        assert_eq!(backend.backend_type(), BackendType::Subprocess);
        assert_eq!(backend.name(), "OxenSubprocess");
    }

    #[test]
    fn test_create_ffi_backend_not_available() {
        let backend = create_backend(BackendType::FFI);
        assert!(backend.is_err());

        let err = backend.err().unwrap().to_string();
        assert!(err.contains("not yet available"));
    }

    #[test]
    fn test_create_default_backend() {
        let backend = create_default_backend();
        assert!(backend.is_ok());
        assert_eq!(backend.unwrap().backend_type(), BackendType::Subprocess);
    }

    #[test]
    fn test_subprocess_backend_wrapper() {
        let backend = SubprocessBackend::default();
        assert_eq!(backend.backend_type(), BackendType::Subprocess);
        assert_eq!(backend.name(), "OxenSubprocess");
    }
}
