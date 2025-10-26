use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalRepository {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub id: String,
    pub message: String,
    pub author: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedEntry {
    pub filename: String,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StagedData {
    pub staged_files: Vec<StagedEntry>,
    pub staged_dirs: Vec<PathBuf>,
    pub untracked_files: Vec<PathBuf>,
    pub untracked_dirs: Vec<PathBuf>,
    pub modified_files: Vec<StagedEntry>,
}

impl StagedData {
    pub fn empty() -> Self {
        Self {
            staged_files: vec![],
            staged_dirs: vec![],
            untracked_files: vec![],
            untracked_dirs: vec![],
            modified_files: vec![],
        }
    }
}
