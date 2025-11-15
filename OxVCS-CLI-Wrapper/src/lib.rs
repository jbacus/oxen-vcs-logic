// STUB: Using local liboxen stub until official Rust bindings are published
pub mod liboxen_stub;
// Alias stub as liboxen for compatibility
#[allow(unused_imports)] // Used by other modules via the alias
use liboxen_stub as liboxen;

pub mod commit_metadata;
pub mod console;
pub mod daemon_client;
pub mod draft_manager;
pub mod hooks;
pub mod ignore_template;
pub mod logger;
pub mod logic_parser;
pub mod logic_project;
pub mod metadata_diff;
pub mod oxen_ops;
pub mod oxen_subprocess;
pub mod progress;
pub mod search;

pub use commit_metadata::CommitMetadata;
pub use draft_manager::{DraftManager, DraftStats};
pub use ignore_template::generate_oxenignore;
pub use logic_parser::{LogicParser, LogicProjectData};
pub use logic_project::LogicProject;
pub use metadata_diff::{MetadataDiff, MetadataDiffer, ReportGenerator};
pub use oxen_ops::OxenRepository;
pub use oxen_subprocess::{
    BranchInfo, CommitInfo as SubprocessCommitInfo, OxenSubprocess, StatusInfo,
};
