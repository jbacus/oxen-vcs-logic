use serde::{Deserialize, Serialize};

/// Structured commit metadata for Logic Pro projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    /// User-provided commit message
    pub message: String,

    /// Beats per minute (optional)
    pub bpm: Option<f32>,

    /// Sample rate in Hz (e.g., 44100, 48000, 96000)
    pub sample_rate: Option<u32>,

    /// Musical key signature (e.g., "C Major", "A Minor")
    pub key_signature: Option<String>,

    /// Optional tags for categorization
    pub tags: Vec<String>,

    /// Timestamp (will be auto-set)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl CommitMetadata {
    /// Creates a new CommitMetadata with just a message
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            bpm: None,
            sample_rate: None,
            key_signature: None,
            tags: Vec::new(),
            timestamp: None,
        }
    }

    /// Sets the BPM
    pub fn with_bpm(mut self, bpm: f32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    /// Sets the sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Sets the key signature
    pub fn with_key_signature(mut self, key: impl Into<String>) -> Self {
        self.key_signature = Some(key.into());
        self
    }

    /// Adds a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Formats the commit metadata as a structured message for Oxen
    ///
    /// Format:
    /// ```
    /// <message>
    ///
    /// BPM: <bpm>
    /// Sample Rate: <sample_rate> Hz
    /// Key: <key_signature>
    /// Tags: <tags>
    /// ```
    pub fn format_commit_message(&self) -> String {
        let mut msg = self.message.clone();

        let mut metadata_lines = Vec::new();

        if let Some(bpm) = self.bpm {
            metadata_lines.push(format!("BPM: {}", bpm));
        }

        if let Some(sr) = self.sample_rate {
            metadata_lines.push(format!("Sample Rate: {} Hz", sr));
        }

        if let Some(ref key) = self.key_signature {
            metadata_lines.push(format!("Key: {}", key));
        }

        if !self.tags.is_empty() {
            metadata_lines.push(format!("Tags: {}", self.tags.join(", ")));
        }

        if !metadata_lines.is_empty() {
            msg.push_str("\n\n");
            msg.push_str(&metadata_lines.join("\n"));
        }

        msg
    }

    /// Parses metadata from a commit message
    pub fn parse_commit_message(message: &str) -> Self {
        let lines: Vec<&str> = message.lines().collect();

        let mut metadata = CommitMetadata::new("");
        let mut main_message = String::new();
        let mut in_metadata = false;

        for line in lines {
            if line.starts_with("BPM:") {
                in_metadata = true;
                if let Some(bpm_str) = line.strip_prefix("BPM:") {
                    if let Ok(bpm) = bpm_str.trim().parse::<f32>() {
                        metadata.bpm = Some(bpm);
                    }
                }
            } else if line.starts_with("Sample Rate:") {
                in_metadata = true;
                if let Some(sr_str) = line.strip_prefix("Sample Rate:") {
                    let sr_clean = sr_str.trim().replace(" Hz", "");
                    if let Ok(sr) = sr_clean.parse::<u32>() {
                        metadata.sample_rate = Some(sr);
                    }
                }
            } else if line.starts_with("Key:") {
                in_metadata = true;
                if let Some(key) = line.strip_prefix("Key:") {
                    metadata.key_signature = Some(key.trim().to_string());
                }
            } else if line.starts_with("Tags:") {
                in_metadata = true;
                if let Some(tags_str) = line.strip_prefix("Tags:") {
                    metadata.tags = tags_str
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                }
            } else if !in_metadata && !line.trim().is_empty() {
                if !main_message.is_empty() {
                    main_message.push('\n');
                }
                main_message.push_str(line);
            }
        }

        metadata.message = main_message;
        metadata
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_commit_message() {
        let metadata = CommitMetadata::new("Initial mix")
            .with_bpm(120.0)
            .with_sample_rate(48000)
            .with_key_signature("C Major");

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("Initial mix"));
        assert!(formatted.contains("BPM: 120"));
        assert!(formatted.contains("Sample Rate: 48000 Hz"));
        assert!(formatted.contains("Key: C Major"));
    }

    #[test]
    fn test_parse_commit_message() {
        let msg = "Initial mix\n\nBPM: 120\nSample Rate: 48000 Hz\nKey: C Major";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Initial mix");
        assert_eq!(metadata.bpm, Some(120.0));
        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.key_signature, Some("C Major".to_string()));
    }

    #[test]
    fn test_with_tags() {
        let metadata = CommitMetadata::new("Test")
            .with_tag("draft")
            .with_tag("wip");

        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"draft".to_string()));
    }
}
