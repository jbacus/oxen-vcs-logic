// Auxin-specific extensions to Oxen
// This module contains Logic Pro metadata support, distributed locking, activity logging, etc.

pub mod activity;
pub mod locks;
pub mod metadata;

pub use activity::{get_activities, log_activity, Activity, ActivityLog, ActivityType};
pub use locks::FileLock;
pub use metadata::LogicProMetadata;
