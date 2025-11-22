//! Auxin Oxen Integration
//!
//! Shared library for Oxen VCS integration used by both Auxin CLI and Auxin Server.
//!
//! This crate provides a subprocess-based wrapper around the `oxen` CLI tool,
//! offering a Rust-friendly API with comprehensive error handling, retry logic,
//! and output caching.
//!
//! # Architecture Decision
//!
//! This crate uses subprocess execution rather than direct liboxen FFI bindings because:
//! - liboxen 0.38+ requires async/await throughout the call chain
//! - The subprocess approach is battle-tested in production (88% test coverage)
//! - Process overhead is negligible compared to network operations
//! - Works with any Oxen CLI version without version lock-in
//!
//! # Requirements
//!
//! - `oxen` CLI must be installed: `pip install oxen-ai`
//! - The `oxen` binary must be available in PATH
//!
//! # Example
//!
//! ```rust,no_run
//! use auxin_oxen::OxenSubprocess;
//! use std::path::Path;
//!
//! let oxen = OxenSubprocess::new();
//!
//! // Initialize a repository
//! oxen.init(Path::new("./my-project"))?;
//!
//! // Clone a repository
//! oxen.clone("https://hub.oxen.ai/user/repo", Path::new("./repo"))?;
//! # Ok::<(), anyhow::Error>(())
//! ```

// Re-export the logger module
pub mod logger;

// Re-export the main subprocess module
mod oxen_subprocess;
pub use oxen_subprocess::*;
