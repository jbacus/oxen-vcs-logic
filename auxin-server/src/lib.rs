// Library exports for auxin-server
// Allows integration testing

pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod extensions;

// Conditionally use real or mock Oxen implementation
#[cfg(feature = "full-oxen")]
#[path = "repo_full.rs"]
pub mod repo;

#[cfg(feature = "mock-oxen")]
#[path = "repo_mock.rs"]
pub mod repo;
