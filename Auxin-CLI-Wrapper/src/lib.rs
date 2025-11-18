// STUB: Using local liboxen stub until official Rust bindings are published
pub mod liboxen_stub;
// Alias stub as liboxen for compatibility
#[allow(unused_imports)] // Used by other modules via the alias
use liboxen_stub as liboxen;

pub mod auth;
pub mod backup_recovery;
pub mod collaboration;
pub mod commit_metadata;
pub mod config;
pub mod conflict_detection;
pub mod console;
pub mod daemon_client;
pub mod draft_manager;
pub mod hooks;
pub mod ignore_template;
pub mod lock_integration;
pub mod logger;
pub mod logic_parser;
pub mod logic_project;
pub mod metadata_diff;
pub mod sketchup_project;
pub mod sketchup_metadata;
pub mod blender_project;
pub mod blender_metadata;
pub mod network_resilience;
pub mod offline_queue;
pub mod operation_history;
pub mod oxen_ops;
pub mod oxen_subprocess;
pub mod progress;
pub mod remote_lock;
pub mod search;
pub mod workflow_automation;

pub use auth::{AuthManager, Credentials};
pub use backup_recovery::{
    BackupRecoveryManager, RecoveryHelper, Snapshot, SnapshotType,
};
pub use collaboration::{
    Activity, ActivityFeed, ActivityType, Comment, CommentManager, TeamManager, TeamMember,
};
pub use commit_metadata::CommitMetadata;
pub use config::{Config, ProjectType};
pub use conflict_detection::{ConflictCheckResult, ConflictDetector, ConflictRecommendation};
pub use draft_manager::{DraftManager, DraftStats};
pub use ignore_template::{generate_blender_oxenignore, generate_oxenignore, generate_sketchup_oxenignore};
pub use logic_parser::{LogicParser, LogicProjectData};
pub use logic_project::LogicProject;
pub use metadata_diff::{MetadataDiff, MetadataDiffer, ReportGenerator};
pub use sketchup_metadata::SketchUpMetadata;
pub use sketchup_project::SketchUpProject;
pub use blender_metadata::BlenderMetadata;
pub use blender_project::BlenderProject;
pub use network_resilience::{
    check_network_availability, is_transient_error,
    NetworkResilienceManager, OperationData, OperationType, QueuedOperation,
};
pub use offline_queue::{
    OfflineQueue, QueueEntry, QueueStats, QueuedOperation as OfflineQueuedOperation, SyncReport,
};
pub use operation_history::{
    HistoryOperation, OperationHistoryEntry, OperationHistoryManager, OperationResult,
    OperationStats,
};
pub use oxen_ops::OxenRepository;
pub use oxen_subprocess::{
    BranchInfo, CommitInfo as SubprocessCommitInfo, OxenSubprocess, StatusInfo,
};
pub use remote_lock::{RemoteLock, RemoteLockManager};
pub use workflow_automation::{WorkflowAutomation, WorkflowConfig};
