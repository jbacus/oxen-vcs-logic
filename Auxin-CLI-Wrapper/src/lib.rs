// NOTE: liboxen_stub has been removed as it provided no real functionality.
// When liboxen crate is officially published, implement OxenBackend trait
// for FFIBackend in oxen_backend.rs

pub mod auth;
pub mod backup_recovery;
pub mod blender_metadata;
pub mod blender_project;
pub mod bounce;
pub mod chunked_upload;
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
pub mod network_resilience;
pub mod offline_queue;
pub mod operation_history;
pub mod oxen_backend;
pub mod oxen_ops;
pub mod oxen_subprocess;
pub mod progress;
pub mod remote_lock;
pub mod search;
pub mod server_client;
pub mod sketchup_metadata;
pub mod sketchup_project;
pub mod thumbnail;
pub mod workflow_automation;
pub mod write_ahead_log;

pub use auth::{AuthManager, Credentials};
pub use backup_recovery::{BackupRecoveryManager, RecoveryHelper, Snapshot, SnapshotType};
pub use blender_metadata::BlenderMetadata;
pub use blender_project::BlenderProject;
pub use bounce::{
    AudioFormat, BounceComparison, BounceFilter, BounceManager, BounceMetadata, NullTestResult,
};
pub use chunked_upload::{
    ChunkedUploadManager, UploadConfig, UploadProgress, UploadResult, UploadSession,
    UploadSessionInfo, UploadStatus,
};
pub use collaboration::{
    Activity, ActivityFeed, ActivityType, Comment, CommentManager, TeamManager, TeamMember,
};
pub use commit_metadata::CommitMetadata;
pub use config::ServerConnectionConfig;
pub use config::{Config, ProjectType};
pub use conflict_detection::{ConflictCheckResult, ConflictDetector, ConflictRecommendation};
pub use console::{Console, ConsoleMode, DaemonStatus, LogEntry, LogLevel, RepositoryStatus};
pub use draft_manager::{DraftManager, DraftStats};
pub use ignore_template::{
    generate_blender_oxenignore, generate_oxenignore, generate_sketchup_oxenignore,
};
pub use logic_parser::{LogicParser, LogicProjectData};
pub use logic_project::LogicProject;
pub use metadata_diff::{MetadataDiff, MetadataDiffer, ReportGenerator};
pub use network_resilience::{
    check_network_availability, check_network_health, estimate_transfer_time, is_transient_error,
    AdaptiveRetryPolicy, CircuitBreaker, CircuitBreakerStats, CircuitState, ConnectivityState,
    ErrorKind, NetworkHealth, NetworkHealthMonitor, NetworkQuality, NetworkResilienceManager,
    OperationData, OperationType, QueuedOperation, RetryPolicy, RetryableError,
};
pub use offline_queue::{
    OfflineQueue, QueueEntry, QueueStats, QueuedOperation as OfflineQueuedOperation, SyncReport,
};
pub use operation_history::{
    HistoryOperation, OperationHistoryEntry, OperationHistoryManager, OperationResult,
    OperationStats,
};
pub use oxen_backend::{
    create_backend, create_default_backend, BackendType, OxenBackend, SubprocessBackend,
};
pub use oxen_ops::OxenRepository;
pub use oxen_subprocess::{
    BranchInfo, CommitInfo as SubprocessCommitInfo, OxenConfig, OxenError, OxenSubprocess,
    StatusInfo,
};
pub use remote_lock::{RemoteLock, RemoteLockManager};
pub use server_client::{
    AuxinServerClient, LockHolder, LockInfo, LogicProMetadata as ServerMetadata, ServerConfig,
};
pub use sketchup_metadata::SketchUpMetadata;
pub use sketchup_project::SketchUpProject;
pub use thumbnail::{ThumbnailDiff, ThumbnailManager, ThumbnailMetadata};
pub use workflow_automation::{WorkflowAutomation, WorkflowConfig};
pub use write_ahead_log::{
    RecoveryReport, WalEntry, WalOperation, WalRecoveryManager, WalStats, WalStatus, WriteAheadLog,
};
