// Auxin-specific extensions to Oxen
// This module contains Logic Pro metadata support, distributed locking, etc.

pub mod metadata;
pub mod locks;

pub use metadata::LogicProMetadata;
pub use locks::FileLock;
