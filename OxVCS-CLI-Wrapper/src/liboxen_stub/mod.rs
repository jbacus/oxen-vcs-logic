// STUB IMPLEMENTATION
// This is a temporary stub for liboxen until Oxen.ai publishes official Rust bindings
// Replace this with the real liboxen crate when available

pub mod api;
pub mod branches;
pub mod command;
pub mod model;
pub mod opts;

// Re-exports
pub use branches::Branch;
pub use model::{Commit, LocalRepository, StagedData, StagedEntry};
pub use opts::AddOpts;
