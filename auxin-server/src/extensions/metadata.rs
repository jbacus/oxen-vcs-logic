use serde::{Deserialize, Serialize};

/// Logic Pro specific metadata for commits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicProMetadata {
    pub bpm: Option<f64>,
    pub sample_rate: Option<i32>,
    pub key_signature: Option<String>,
    pub tags: Vec<String>,
}

impl LogicProMetadata {
    pub fn new() -> Self {
        Self {
            bpm: None,
            sample_rate: None,
            key_signature: None,
            tags: Vec::new(),
        }
    }

    pub fn with_bpm(mut self, bpm: f64) -> Self {
        self.bpm = Some(bpm);
        self
    }

    pub fn with_sample_rate(mut self, sample_rate: i32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    pub fn with_key_signature(mut self, key: impl Into<String>) -> Self {
        self.key_signature = Some(key.into());
        self
    }

    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

impl Default for LogicProMetadata {
    fn default() -> Self {
        Self::new()
    }
}
