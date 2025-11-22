use crate::oxen_subprocess::OxenSubprocess;
use crate::remote_lock::RemoteLockManager;
use anyhow::Result;
use std::path::Path;

/// Represents the result of a conflict check
#[derive(Debug, Clone, PartialEq)]
pub struct ConflictCheckResult {
    /// Whether there are potential conflicts
    pub has_conflicts: bool,

    /// Local commits not in remote
    pub local_ahead: usize,

    /// Remote commits not in local
    pub remote_ahead: usize,

    /// Whether the project is currently locked by another user
    pub locked_by_other: bool,

    /// Lock owner if locked
    pub lock_owner: Option<String>,

    /// Recommended action
    pub recommendation: ConflictRecommendation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConflictRecommendation {
    /// Safe to pull/push
    Safe,

    /// Should acquire lock first
    AcquireLock,

    /// Should release lock first
    ReleaseLock,

    /// Diverged - manual intervention needed
    ManualMergeRequired,

    /// Check network connection
    CheckNetwork,
}

/// Manager for detecting and preventing conflicts
pub struct ConflictDetector {
    #[allow(dead_code)]
    oxen: OxenSubprocess,
    lock_manager: RemoteLockManager,
}

impl ConflictDetector {
    pub fn new() -> Self {
        Self {
            oxen: OxenSubprocess::new(),
            lock_manager: RemoteLockManager::new(),
        }
    }

    pub fn with_oxen(oxen: OxenSubprocess) -> Self {
        Self {
            oxen,
            lock_manager: RemoteLockManager::new(),
        }
    }

    /// Check for conflicts before pulling
    pub fn check_before_pull(
        &self,
        repo_path: &Path,
        _branch: &str,
    ) -> Result<ConflictCheckResult> {
        // Note: Full divergence checking requires fetch, which needs raw command access
        // For now, we provide basic conflict detection based on lock status
        // TODO: Add public fetch method to OxenSubprocess for full conflict detection

        // Check lock status
        let lock_status = self.check_lock_status(repo_path)?;

        // Determine recommendation
        let recommendation = if lock_status.is_locked && lock_status.locked_by_other {
            ConflictRecommendation::AcquireLock
        } else {
            ConflictRecommendation::Safe
        };

        Ok(ConflictCheckResult {
            has_conflicts: false, // Conservative: assume no conflicts without fetch
            local_ahead: 0,
            remote_ahead: 0,
            locked_by_other: lock_status.locked_by_other,
            lock_owner: lock_status.owner,
            recommendation,
        })
    }

    /// Check for conflicts before pushing
    pub fn check_before_push(
        &self,
        repo_path: &Path,
        _branch: &str,
    ) -> Result<ConflictCheckResult> {
        // Note: Full divergence checking requires fetch, which needs raw command access
        // For now, we provide basic conflict detection based on lock status
        // TODO: Add public fetch method to OxenSubprocess for full conflict detection

        // Check lock status
        let lock_status = self.check_lock_status(repo_path)?;

        // Determine recommendation
        let recommendation = if !lock_status.is_locked || lock_status.locked_by_other {
            ConflictRecommendation::AcquireLock
        } else {
            ConflictRecommendation::Safe
        };

        Ok(ConflictCheckResult {
            has_conflicts: false, // Conservative: assume no conflicts without fetch
            local_ahead: 0,
            remote_ahead: 0,
            locked_by_other: lock_status.locked_by_other,
            lock_owner: lock_status.owner,
            recommendation,
        })
    }

    /// Check current lock status
    fn check_lock_status(&self, repo_path: &Path) -> Result<LockStatus> {
        match self.lock_manager.get_lock(repo_path) {
            Ok(Some(lock)) => {
                let is_owned_by_current = lock.is_owned_by_current_user();
                Ok(LockStatus {
                    is_locked: true,
                    locked_by_other: !is_owned_by_current,
                    owner: Some(lock.locked_by.clone()),
                })
            }
            Ok(None) => Ok(LockStatus {
                is_locked: false,
                locked_by_other: false,
                owner: None,
            }),
            Err(_) => Ok(LockStatus {
                is_locked: false,
                locked_by_other: false,
                owner: None,
            }),
        }
    }
}

impl Default for ConflictDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct LockStatus {
    is_locked: bool,
    locked_by_other: bool,
    owner: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conflict_detector_creation() {
        let _detector = ConflictDetector::new();
        // Just verify it compiles and creates
        assert!(true);
    }

    #[test]
    fn test_conflict_check_result_no_conflicts() {
        let result = ConflictCheckResult {
            has_conflicts: false,
            local_ahead: 0,
            remote_ahead: 0,
            locked_by_other: false,
            lock_owner: None,
            recommendation: ConflictRecommendation::Safe,
        };

        assert!(!result.has_conflicts);
        assert_eq!(result.recommendation, ConflictRecommendation::Safe);
    }

    #[test]
    fn test_conflict_check_result_with_divergence() {
        let result = ConflictCheckResult {
            has_conflicts: true,
            local_ahead: 3,
            remote_ahead: 2,
            locked_by_other: false,
            lock_owner: None,
            recommendation: ConflictRecommendation::ManualMergeRequired,
        };

        assert!(result.has_conflicts);
        assert_eq!(result.local_ahead, 3);
        assert_eq!(result.remote_ahead, 2);
        assert_eq!(
            result.recommendation,
            ConflictRecommendation::ManualMergeRequired
        );
    }

    #[test]
    fn test_conflict_check_result_locked_by_other() {
        let result = ConflictCheckResult {
            has_conflicts: false,
            local_ahead: 0,
            remote_ahead: 0,
            locked_by_other: true,
            lock_owner: Some("other@user".to_string()),
            recommendation: ConflictRecommendation::AcquireLock,
        };

        assert!(result.locked_by_other);
        assert_eq!(result.lock_owner.as_deref(), Some("other@user"));
        assert_eq!(result.recommendation, ConflictRecommendation::AcquireLock);
    }

    #[test]
    fn test_conflict_recommendation_variants() {
        assert_eq!(ConflictRecommendation::Safe, ConflictRecommendation::Safe);
        assert_ne!(
            ConflictRecommendation::Safe,
            ConflictRecommendation::AcquireLock
        );
    }
}
