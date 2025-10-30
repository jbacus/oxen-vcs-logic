use serde::{Deserialize, Serialize};

/// Structured metadata for Logic Pro project commits.
///
/// Enhances standard commit messages with DAW-specific metadata including tempo,
/// sample rate, and musical key. This enables rich searching, filtering, and
/// context when browsing project history.
///
/// Metadata is embedded in commit messages in a structured format and can be
/// parsed back for display in UIs and reporting tools.
///
/// # Format
///
/// Commits are formatted as:
/// ```text
/// <message>
///
/// BPM: <tempo>
/// Sample Rate: <rate> Hz
/// Key: <key_signature>
/// Tags: <tag1>, <tag2>, ...
/// ```
///
/// # Examples
///
/// ```
/// use oxenvcs_cli::CommitMetadata;
///
/// // Create milestone commit with full metadata
/// let commit = CommitMetadata::new("Final mix - ready for mastering")
///     .with_bpm(128.0)
///     .with_sample_rate(48000)
///     .with_key_signature("A Minor")
///     .with_tag("milestone")
///     .with_tag("mix-v3");
///
/// let formatted = commit.format_commit_message();
/// assert!(formatted.contains("BPM: 128"));
/// assert!(formatted.contains("A Minor"));
///
/// // Parse it back
/// let parsed = CommitMetadata::parse_commit_message(&formatted);
/// assert_eq!(parsed.bpm, Some(128.0));
/// ```
///
/// # Serialization
///
/// Supports JSON serialization via Serde for storage and IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitMetadata {
    /// User-provided commit message (primary description)
    pub message: String,

    /// Beats per minute (tempo). Supports decimal values (e.g., 120.5, 128.0)
    pub bpm: Option<f32>,

    /// Sample rate in Hz (e.g., 44100, 48000, 96000, 192000)
    pub sample_rate: Option<u32>,

    /// Musical key signature (e.g., "C Major", "A Minor", "F# Major")
    pub key_signature: Option<String>,

    /// Optional tags for categorization (e.g., "draft", "mix", "mastered")
    pub tags: Vec<String>,

    /// Unix timestamp (auto-set by daemon, not user-provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl CommitMetadata {
    /// Creates a new CommitMetadata with just a message.
    ///
    /// This is the primary constructor. Use builder methods to add optional metadata.
    ///
    /// # Arguments
    ///
    /// * `message` - Commit message (can be String, &str, or any Into<String>)
    ///
    /// # Returns
    ///
    /// CommitMetadata with all optional fields set to None/empty
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// // From &str
    /// let commit = CommitMetadata::new("Initial version");
    ///
    /// // From String
    /// let message = String::from("Working draft");
    /// let commit = CommitMetadata::new(message);
    /// ```
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

    /// Sets the BPM (beats per minute).
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `bpm` - Tempo in beats per minute (supports decimals like 120.5)
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let commit = CommitMetadata::new("Uptempo mix")
    ///     .with_bpm(140.0);
    /// assert_eq!(commit.bpm, Some(140.0));
    /// ```
    pub fn with_bpm(mut self, bpm: f32) -> Self {
        self.bpm = Some(bpm);
        self
    }

    /// Sets the sample rate in Hz.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `sample_rate` - Sample rate in Hz (typical: 44100, 48000, 96000, 192000)
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let commit = CommitMetadata::new("High-res recording")
    ///     .with_sample_rate(96000);
    /// assert_eq!(commit.sample_rate, Some(96000));
    /// ```
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = Some(sample_rate);
        self
    }

    /// Sets the musical key signature.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `key` - Musical key (e.g., "C Major", "A Minor", "F# Major")
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let commit = CommitMetadata::new("Melody draft")
    ///     .with_key_signature("D Minor");
    /// assert_eq!(commit.key_signature, Some("D Minor".to_string()));
    /// ```
    pub fn with_key_signature(mut self, key: impl Into<String>) -> Self {
        self.key_signature = Some(key.into());
        self
    }

    /// Adds a tag for categorization.
    ///
    /// Builder pattern method that consumes and returns self. Can be called
    /// multiple times to add multiple tags.
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag string (e.g., "draft", "mix", "mastered", "milestone")
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let commit = CommitMetadata::new("Pre-master version")
    ///     .with_tag("mix")
    ///     .with_tag("review")
    ///     .with_tag("v3");
    ///
    /// assert_eq!(commit.tags.len(), 3);
    /// assert!(commit.tags.contains(&"mix".to_string()));
    /// ```
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Formats the metadata as a structured commit message for version control.
    ///
    /// Generates a multi-line string with the message followed by metadata fields.
    /// Only includes fields that have been set (omits None values).
    ///
    /// # Format
    ///
    /// ```text
    /// <message>
    ///
    /// BPM: <bpm>
    /// Sample Rate: <sample_rate> Hz
    /// Key: <key_signature>
    /// Tags: <tag1>, <tag2>, ...
    /// ```
    ///
    /// If no metadata fields are set, returns just the message (no extra newlines).
    ///
    /// # Returns
    ///
    /// Formatted String ready for commit
    ///
    /// # Field Order
    ///
    /// Metadata always appears in this order: BPM, Sample Rate, Key, Tags
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// // With full metadata
    /// let commit = CommitMetadata::new("Final mix")
    ///     .with_bpm(120.0)
    ///     .with_sample_rate(48000)
    ///     .with_key_signature("C Major");
    ///
    /// let formatted = commit.format_commit_message();
    /// assert!(formatted.contains("Final mix\n\nBPM: 120"));
    ///
    /// // With no metadata (just message)
    /// let simple = CommitMetadata::new("Quick save");
    /// assert_eq!(simple.format_commit_message(), "Quick save");
    /// ```
    ///
    /// # Round-Trip Compatibility
    ///
    /// Output is guaranteed to be parseable by `parse_commit_message()`:
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let original = CommitMetadata::new("Test").with_bpm(128.0);
    /// let formatted = original.format_commit_message();
    /// let parsed = CommitMetadata::parse_commit_message(&formatted);
    ///
    /// assert_eq!(parsed.bpm, original.bpm);
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

    /// Parses structured metadata from a commit message string.
    ///
    /// Extracts BPM, sample rate, key signature, and tags from a formatted commit
    /// message. Handles messages created by `format_commit_message()` and also
    /// plain text messages (returning metadata with no optional fields).
    ///
    /// # Parsing Rules
    ///
    /// - Lines starting with `BPM:` are parsed as tempo (float)
    /// - Lines starting with `Sample Rate:` are parsed as Hz (u32, "Hz" suffix optional)
    /// - Lines starting with `Key:` are parsed as key signature (string)
    /// - Lines starting with `Tags:` are parsed as comma-separated list
    /// - All other lines (before metadata section) are treated as the message
    /// - Parsing is lenient: invalid values result in None, not errors
    ///
    /// # Arguments
    ///
    /// * `message` - Formatted commit message string
    ///
    /// # Returns
    ///
    /// CommitMetadata with parsed fields (None for unparseable values)
    ///
    /// # Examples
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// // Parse formatted message
    /// let msg = "Mix v3\n\nBPM: 120\nSample Rate: 48000 Hz\nKey: A Minor";
    /// let parsed = CommitMetadata::parse_commit_message(msg);
    ///
    /// assert_eq!(parsed.message, "Mix v3");
    /// assert_eq!(parsed.bpm, Some(120.0));
    /// assert_eq!(parsed.sample_rate, Some(48000));
    /// assert_eq!(parsed.key_signature, Some("A Minor".to_string()));
    ///
    /// // Parse plain message (no metadata)
    /// let plain = "Just a commit message";
    /// let parsed = CommitMetadata::parse_commit_message(plain);
    /// assert_eq!(parsed.message, "Just a commit message");
    /// assert_eq!(parsed.bpm, None);
    /// ```
    ///
    /// # Error Handling
    ///
    /// Invalid metadata values are silently ignored (set to None):
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let msg = "Test\n\nBPM: not_a_number\nSample Rate: invalid";
    /// let parsed = CommitMetadata::parse_commit_message(msg);
    ///
    /// assert_eq!(parsed.message, "Test");
    /// assert_eq!(parsed.bpm, None); // Invalid, not an error
    /// assert_eq!(parsed.sample_rate, None);
    /// ```
    ///
    /// # Multiline Messages
    ///
    /// Preserves newlines in the message portion:
    ///
    /// ```
    /// use oxenvcs_cli::CommitMetadata;
    ///
    /// let msg = "Line 1\nLine 2\nLine 3\n\nBPM: 130";
    /// let parsed = CommitMetadata::parse_commit_message(msg);
    /// assert_eq!(parsed.message, "Line 1\nLine 2\nLine 3");
    /// ```
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
    fn test_new_basic() {
        let metadata = CommitMetadata::new("Test commit");
        assert_eq!(metadata.message, "Test commit");
        assert_eq!(metadata.bpm, None);
        assert_eq!(metadata.sample_rate, None);
        assert_eq!(metadata.key_signature, None);
        assert!(metadata.tags.is_empty());
        assert_eq!(metadata.timestamp, None);
    }

    #[test]
    fn test_new_empty_message() {
        let metadata = CommitMetadata::new("");
        assert_eq!(metadata.message, "");
    }

    #[test]
    fn test_builder_pattern() {
        let metadata = CommitMetadata::new("Test")
            .with_bpm(140.5)
            .with_sample_rate(96000)
            .with_key_signature("A Minor")
            .with_tag("mix")
            .with_tag("final");

        assert_eq!(metadata.bpm, Some(140.5));
        assert_eq!(metadata.sample_rate, Some(96000));
        assert_eq!(metadata.key_signature, Some("A Minor".to_string()));
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_format_commit_message_complete() {
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
    fn test_format_commit_message_partial() {
        let metadata = CommitMetadata::new("Work in progress").with_bpm(128.0);

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("Work in progress"));
        assert!(formatted.contains("BPM: 128"));
        assert!(!formatted.contains("Sample Rate"));
        assert!(!formatted.contains("Key:"));
    }

    #[test]
    fn test_format_commit_message_no_metadata() {
        let metadata = CommitMetadata::new("Simple commit");
        let formatted = metadata.format_commit_message();

        assert_eq!(formatted, "Simple commit");
        // Should not have extra newlines when no metadata
        assert!(!formatted.contains("\n\n"));
    }

    #[test]
    fn test_format_with_tags() {
        let metadata = CommitMetadata::new("Tagged commit")
            .with_tag("draft")
            .with_tag("review")
            .with_tag("important");

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("Tags: draft, review, important"));
    }

    #[test]
    fn test_format_multiline_message() {
        let metadata = CommitMetadata::new("First line\nSecond line\nThird line").with_bpm(120.0);

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("First line\nSecond line\nThird line"));
        assert!(formatted.contains("BPM: 120"));
    }

    #[test]
    fn test_parse_commit_message_complete() {
        let msg = "Initial mix\n\nBPM: 120\nSample Rate: 48000 Hz\nKey: C Major";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Initial mix");
        assert_eq!(metadata.bpm, Some(120.0));
        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.key_signature, Some("C Major".to_string()));
    }

    #[test]
    fn test_parse_commit_message_partial() {
        let msg = "Quick save\n\nBPM: 140";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Quick save");
        assert_eq!(metadata.bpm, Some(140.0));
        assert_eq!(metadata.sample_rate, None);
        assert_eq!(metadata.key_signature, None);
    }

    #[test]
    fn test_parse_commit_message_no_metadata() {
        let msg = "Just a message";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Just a message");
        assert_eq!(metadata.bpm, None);
        assert_eq!(metadata.sample_rate, None);
    }

    #[test]
    fn test_parse_with_tags() {
        let msg = "Commit\n\nTags: draft, wip, milestone";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.tags.len(), 3);
        assert!(metadata.tags.contains(&"draft".to_string()));
        assert!(metadata.tags.contains(&"wip".to_string()));
        assert!(metadata.tags.contains(&"milestone".to_string()));
    }

    #[test]
    fn test_parse_tags_with_spaces() {
        let msg = "Commit\n\nTags:  draft  ,  review  ,  final  ";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.tags.len(), 3);
        // Should be trimmed
        assert!(metadata.tags.contains(&"draft".to_string()));
        assert!(metadata.tags.contains(&"review".to_string()));
    }

    #[test]
    fn test_parse_multiline_message() {
        let msg = "Line 1\nLine 2\nLine 3\n\nBPM: 130";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Line 1\nLine 2\nLine 3");
        assert_eq!(metadata.bpm, Some(130.0));
    }

    #[test]
    fn test_parse_invalid_bpm() {
        let msg = "Commit\n\nBPM: invalid";
        let metadata = CommitMetadata::parse_commit_message(msg);

        // Should be None when parse fails
        assert_eq!(metadata.bpm, None);
    }

    #[test]
    fn test_parse_invalid_sample_rate() {
        let msg = "Commit\n\nSample Rate: not_a_number Hz";
        let metadata = CommitMetadata::parse_commit_message(msg);

        assert_eq!(metadata.sample_rate, None);
    }

    #[test]
    fn test_parse_sample_rate_without_hz() {
        let msg = "Commit\n\nSample Rate: 44100";
        let metadata = CommitMetadata::parse_commit_message(msg);

        // Should still parse correctly
        assert_eq!(metadata.sample_rate, Some(44100));
    }

    #[test]
    fn test_round_trip() {
        // Create metadata, format it, parse it back
        let original = CommitMetadata::new("Round trip test")
            .with_bpm(125.5)
            .with_sample_rate(96000)
            .with_key_signature("D Major")
            .with_tag("test")
            .with_tag("round-trip");

        let formatted = original.format_commit_message();
        let parsed = CommitMetadata::parse_commit_message(&formatted);

        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.bpm, original.bpm);
        assert_eq!(parsed.sample_rate, original.sample_rate);
        assert_eq!(parsed.key_signature, original.key_signature);
        assert_eq!(parsed.tags, original.tags);
    }

    #[test]
    fn test_with_tags() {
        let metadata = CommitMetadata::new("Test")
            .with_tag("draft")
            .with_tag("wip");

        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"draft".to_string()));
        assert!(metadata.tags.contains(&"wip".to_string()));
    }

    #[test]
    fn test_with_empty_tag() {
        let metadata = CommitMetadata::new("Test").with_tag("").with_tag("valid");

        assert_eq!(metadata.tags.len(), 2);
        assert!(metadata.tags.contains(&"".to_string()));
    }

    #[test]
    fn test_bpm_decimal_values() {
        let metadata = CommitMetadata::new("Test").with_bpm(120.5);

        let formatted = metadata.format_commit_message();
        assert!(formatted.contains("BPM: 120.5"));

        let parsed = CommitMetadata::parse_commit_message(&formatted);
        assert_eq!(parsed.bpm, Some(120.5));
    }

    #[test]
    fn test_various_sample_rates() {
        let rates = vec![44100, 48000, 88200, 96000, 192000];

        for rate in rates {
            let metadata = CommitMetadata::new("Test").with_sample_rate(rate);

            let formatted = metadata.format_commit_message();
            assert!(formatted.contains(&format!("Sample Rate: {} Hz", rate)));
        }
    }

    #[test]
    fn test_key_signature_variations() {
        let keys = vec!["C Major", "A Minor", "F# Major", "Bb Minor", "Db Major"];

        for key in keys {
            let metadata = CommitMetadata::new("Test").with_key_signature(key);

            let formatted = metadata.format_commit_message();
            assert!(formatted.contains(&format!("Key: {}", key)));

            let parsed = CommitMetadata::parse_commit_message(&formatted);
            assert_eq!(parsed.key_signature, Some(key.to_string()));
        }
    }

    #[test]
    fn test_metadata_order_in_output() {
        let metadata = CommitMetadata::new("Test")
            .with_key_signature("C Major")
            .with_sample_rate(48000)
            .with_bpm(120.0)
            .with_tag("test");

        let formatted = metadata.format_commit_message();

        // Check that metadata appears in expected order
        let bpm_pos = formatted.find("BPM:").unwrap();
        let sr_pos = formatted.find("Sample Rate:").unwrap();
        let key_pos = formatted.find("Key:").unwrap();
        let tag_pos = formatted.find("Tags:").unwrap();

        // BPM should come before Sample Rate
        assert!(bpm_pos < sr_pos);
        // Sample Rate should come before Key
        assert!(sr_pos < key_pos);
        // Key should come before Tags
        assert!(key_pos < tag_pos);
    }

    #[test]
    fn test_parse_empty_tags() {
        let msg = "Commit\n\nTags: ";
        let metadata = CommitMetadata::parse_commit_message(msg);

        // Should result in empty tags vec
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_serde_serialization() {
        let metadata = CommitMetadata::new("Test")
            .with_bpm(120.0)
            .with_sample_rate(48000);

        // Test that serialization works
        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"message\":\"Test\""));
        assert!(json.contains("\"bpm\":120"));
    }

    #[test]
    fn test_serde_deserialization() {
        let json = r#"{"message":"Test","bpm":120.0,"sample_rate":48000,"key_signature":"C Major","tags":["test"]}"#;
        let metadata: CommitMetadata = serde_json::from_str(json).unwrap();

        assert_eq!(metadata.message, "Test");
        assert_eq!(metadata.bpm, Some(120.0));
        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.key_signature, Some("C Major".to_string()));
        assert_eq!(metadata.tags.len(), 1);
    }
}
