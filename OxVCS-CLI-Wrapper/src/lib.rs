// STUB: Using local liboxen stub until official Rust bindings are published
pub mod liboxen_stub;
// Alias stub as liboxen for compatibility
use liboxen_stub as liboxen;

pub mod logic_project;
pub mod oxen_ops;
pub mod ignore_template;
pub mod commit_metadata;
pub mod draft_manager;

pub use logic_project::LogicProject;
pub use oxen_ops::OxenRepository;
pub use ignore_template::generate_oxenignore;
pub use commit_metadata::CommitMetadata;
pub use draft_manager::{DraftManager, DraftStats};
