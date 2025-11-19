use serde::{Deserialize, Serialize};

/// Structured metadata for SketchUp project commits.
///
/// Enhances standard commit messages with SketchUp-specific metadata including
/// model units, layer information, and component counts. This enables rich searching,
/// filtering, and context when browsing project history.
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
/// Units: <units>
/// Layers: <layer_count>
/// Components: <component_count>
/// Tags: <tag1>, <tag2>, ...
/// ```
///
/// # Examples
///
/// ```
/// use auxin_cli::SketchUpMetadata;
///
/// // Create milestone commit with full metadata
/// let commit = SketchUpMetadata::new("Final presentation model")
///     .with_units("Inches")
///     .with_layer_count(15)
///     .with_component_count(234)
///     .with_tag("milestone")
///     .with_tag("presentation");
///
/// let formatted = commit.format_commit_message();
/// assert!(formatted.contains("Units: Inches"));
/// assert!(formatted.contains("Components: 234"));
///
/// // Parse it back
/// let parsed = SketchUpMetadata::parse_commit_message(&formatted);
/// assert_eq!(parsed.units, Some("Inches".to_string()));
/// ```
///
/// # Serialization
///
/// Supports JSON serialization via Serde for storage and IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchUpMetadata {
    /// User-provided commit message (primary description)
    pub message: String,

    /// Model units (e.g., "Inches", "Feet", "Meters", "Millimeters")
    pub units: Option<String>,

    /// Number of layers/tags in the model
    pub layer_count: Option<u32>,

    /// Number of component instances in the model
    pub component_count: Option<u32>,

    /// Number of groups in the model
    pub group_count: Option<u32>,

    /// Model file size in bytes (useful for tracking bloat)
    pub file_size_bytes: Option<u64>,

    /// Optional tags for categorization (e.g., "draft", "presentation", "construction-docs")
    pub tags: Vec<String>,

    /// Unix timestamp (auto-set by daemon, not user-provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl SketchUpMetadata {
    /// Creates a new SketchUpMetadata with just a message.
    ///
    /// This is the primary constructor. Use builder methods to add optional metadata.
    ///
    /// # Arguments
    ///
    /// * `message` - Commit message (can be String, &str, or any Into<String>)
    ///
    /// # Returns
    ///
    /// SketchUpMetadata with all optional fields set to None/empty
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// // From &str
    /// let commit = SketchUpMetadata::new("Initial version");
    ///
    /// // From String
    /// let message = String::from("Working draft");
    /// let commit = SketchUpMetadata::new(message);
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            units: None,
            layer_count: None,
            component_count: None,
            group_count: None,
            file_size_bytes: None,
            tags: Vec::new(),
            timestamp: None,
        }
    }

    /// Sets the model units.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `units` - Model units (e.g., "Inches", "Feet", "Meters", "Millimeters")
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Architectural model")
    ///     .with_units("Inches");
    /// assert_eq!(commit.units, Some("Inches".to_string()));
    /// ```
    pub fn with_units(mut self, units: impl Into<String>) -> Self {
        self.units = Some(units.into());
        self
    }

    /// Sets the layer count.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `layer_count` - Number of layers/tags in the model
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Organized model")
    ///     .with_layer_count(10);
    /// assert_eq!(commit.layer_count, Some(10));
    /// ```
    pub fn with_layer_count(mut self, layer_count: u32) -> Self {
        self.layer_count = Some(layer_count);
        self
    }

    /// Sets the component count.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `component_count` - Number of component instances
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Component-heavy model")
    ///     .with_component_count(500);
    /// assert_eq!(commit.component_count, Some(500));
    /// ```
    pub fn with_component_count(mut self, component_count: u32) -> Self {
        self.component_count = Some(component_count);
        self
    }

    /// Sets the group count.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `group_count` - Number of groups in the model
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Model with groups")
    ///     .with_group_count(25);
    /// assert_eq!(commit.group_count, Some(25));
    /// ```
    pub fn with_group_count(mut self, group_count: u32) -> Self {
        self.group_count = Some(group_count);
        self
    }

    /// Sets the file size.
    ///
    /// Builder pattern method that consumes and returns self.
    ///
    /// # Arguments
    ///
    /// * `file_size_bytes` - File size in bytes
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Large model")
    ///     .with_file_size(1024 * 1024 * 50); // 50 MB
    /// assert_eq!(commit.file_size_bytes, Some(52428800));
    /// ```
    pub fn with_file_size(mut self, file_size_bytes: u64) -> Self {
        self.file_size_bytes = Some(file_size_bytes);
        self
    }

    /// Adds a tag for categorization.
    ///
    /// Builder pattern method that consumes and returns self. Can be called
    /// multiple times to add multiple tags.
    ///
    /// # Arguments
    ///
    /// * `tag` - Tag string (e.g., "draft", "presentation", "construction-docs")
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// let commit = SketchUpMetadata::new("Client presentation")
    ///     .with_tag("presentation")
    ///     .with_tag("client-review")
    ///     .with_tag("v3");
    ///
    /// assert_eq!(commit.tags.len(), 3);
    /// assert!(commit.tags.contains(&"presentation".to_string()));
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
    /// Units: <units>
    /// Layers: <layer_count>
    /// Components: <component_count>
    /// Groups: <group_count>
    /// File Size: <size> MB
    /// Tags: <tag1>, <tag2>, ...
    /// ```
    ///
    /// If no metadata fields are set, returns just the message (no extra newlines).
    ///
    /// # Returns
    ///
    /// Formatted String ready for commit
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// // With full metadata
    /// let commit = SketchUpMetadata::new("Final model")
    ///     .with_units("Meters")
    ///     .with_layer_count(10)
    ///     .with_component_count(150);
    ///
    /// let formatted = commit.format_commit_message();
    /// assert!(formatted.contains("Final model\n\nUnits: Meters"));
    ///
    /// // With no metadata (just message)
    /// let simple = SketchUpMetadata::new("Quick save");
    /// assert_eq!(simple.format_commit_message(), "Quick save");
    /// ```
    pub fn format_commit_message(&self) -> String {
        let mut msg = self.message.clone();

        let mut metadata_lines = Vec::new();

        if let Some(ref units) = self.units {
            metadata_lines.push(format!("Units: {}", units));
        }

        if let Some(layers) = self.layer_count {
            metadata_lines.push(format!("Layers: {}", layers));
        }

        if let Some(components) = self.component_count {
            metadata_lines.push(format!("Components: {}", components));
        }

        if let Some(groups) = self.group_count {
            metadata_lines.push(format!("Groups: {}", groups));
        }

        if let Some(size) = self.file_size_bytes {
            // Convert to human-readable format
            let size_mb = size as f64 / (1024.0 * 1024.0);
            metadata_lines.push(format!("File Size: {:.2} MB", size_mb));
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
    /// Extracts SketchUp-specific metadata from a formatted commit message.
    /// Handles messages created by `format_commit_message()` and also
    /// plain text messages (returning metadata with no optional fields).
    ///
    /// # Parsing Rules
    ///
    /// - Lines starting with `Units:` are parsed as units string
    /// - Lines starting with `Layers:` are parsed as layer count (u32)
    /// - Lines starting with `Components:` are parsed as component count (u32)
    /// - Lines starting with `Groups:` are parsed as group count (u32)
    /// - Lines starting with `File Size:` are parsed as bytes (converting from MB)
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
    /// SketchUpMetadata with parsed fields (None for unparseable values)
    ///
    /// # Examples
    ///
    /// ```
    /// use auxin_cli::SketchUpMetadata;
    ///
    /// // Parse formatted message
    /// let msg = "Model v3\n\nUnits: Meters\nLayers: 10\nComponents: 150";
    /// let parsed = SketchUpMetadata::parse_commit_message(msg);
    ///
    /// assert_eq!(parsed.message, "Model v3");
    /// assert_eq!(parsed.units, Some("Meters".to_string()));
    /// assert_eq!(parsed.layer_count, Some(10));
    /// assert_eq!(parsed.component_count, Some(150));
    ///
    /// // Parse plain message (no metadata)
    /// let plain = "Just a commit message";
    /// let parsed = SketchUpMetadata::parse_commit_message(plain);
    /// assert_eq!(parsed.message, "Just a commit message");
    /// assert_eq!(parsed.units, None);
    /// ```
    pub fn parse_commit_message(message: &str) -> Self {
        let lines: Vec<&str> = message.lines().collect();

        let mut metadata = SketchUpMetadata::new("");
        let mut main_message = String::new();
        let mut in_metadata = false;

        for line in lines {
            if line.starts_with("Units:") {
                in_metadata = true;
                if let Some(units_str) = line.strip_prefix("Units:") {
                    metadata.units = Some(units_str.trim().to_string());
                }
            } else if line.starts_with("Layers:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Layers:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.layer_count = Some(count);
                    }
                }
            } else if line.starts_with("Components:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Components:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.component_count = Some(count);
                    }
                }
            } else if line.starts_with("Groups:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Groups:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.group_count = Some(count);
                    }
                }
            } else if line.starts_with("File Size:") {
                in_metadata = true;
                if let Some(size_str) = line.strip_prefix("File Size:") {
                    // Parse "X.XX MB" format
                    let size_clean = size_str.trim().replace(" MB", "");
                    if let Ok(size_mb) = size_clean.parse::<f64>() {
                        metadata.file_size_bytes = Some((size_mb * 1024.0 * 1024.0) as u64);
                    }
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
        let metadata = SketchUpMetadata::new("Test commit");
        assert_eq!(metadata.message, "Test commit");
        assert_eq!(metadata.units, None);
        assert_eq!(metadata.layer_count, None);
        assert_eq!(metadata.component_count, None);
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let metadata = SketchUpMetadata::new("Test")
            .with_units("Meters")
            .with_layer_count(10)
            .with_component_count(150)
            .with_group_count(25)
            .with_tag("draft")
            .with_tag("v1");

        assert_eq!(metadata.units, Some("Meters".to_string()));
        assert_eq!(metadata.layer_count, Some(10));
        assert_eq!(metadata.component_count, Some(150));
        assert_eq!(metadata.group_count, Some(25));
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_format_commit_message_complete() {
        let metadata = SketchUpMetadata::new("Final model")
            .with_units("Inches")
            .with_layer_count(15)
            .with_component_count(200);

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("Final model"));
        assert!(formatted.contains("Units: Inches"));
        assert!(formatted.contains("Layers: 15"));
        assert!(formatted.contains("Components: 200"));
    }

    #[test]
    fn test_format_commit_message_no_metadata() {
        let metadata = SketchUpMetadata::new("Simple commit");
        let formatted = metadata.format_commit_message();

        assert_eq!(formatted, "Simple commit");
        assert!(!formatted.contains("\n\n"));
    }

    #[test]
    fn test_parse_commit_message_complete() {
        let msg = "Final model\n\nUnits: Meters\nLayers: 10\nComponents: 150";
        let metadata = SketchUpMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Final model");
        assert_eq!(metadata.units, Some("Meters".to_string()));
        assert_eq!(metadata.layer_count, Some(10));
        assert_eq!(metadata.component_count, Some(150));
    }

    #[test]
    fn test_parse_commit_message_no_metadata() {
        let msg = "Just a message";
        let metadata = SketchUpMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Just a message");
        assert_eq!(metadata.units, None);
        assert_eq!(metadata.layer_count, None);
    }

    #[test]
    fn test_round_trip() {
        let original = SketchUpMetadata::new("Round trip test")
            .with_units("Feet")
            .with_layer_count(12)
            .with_component_count(300)
            .with_tag("test");

        let formatted = original.format_commit_message();
        let parsed = SketchUpMetadata::parse_commit_message(&formatted);

        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.units, original.units);
        assert_eq!(parsed.layer_count, original.layer_count);
        assert_eq!(parsed.component_count, original.component_count);
        assert_eq!(parsed.tags, original.tags);
    }

    #[test]
    fn test_file_size_formatting() {
        let metadata = SketchUpMetadata::new("Large model").with_file_size(52428800); // 50 MB

        let formatted = metadata.format_commit_message();
        assert!(formatted.contains("File Size: 50.00 MB"));
    }

    #[test]
    fn test_parse_invalid_layer_count() {
        let msg = "Commit\n\nLayers: invalid";
        let metadata = SketchUpMetadata::parse_commit_message(msg);

        assert_eq!(metadata.layer_count, None);
    }

    #[test]
    fn test_serde_serialization() {
        let metadata = SketchUpMetadata::new("Test")
            .with_units("Meters")
            .with_layer_count(10);

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"message\":\"Test\""));
        assert!(json.contains("\"units\":\"Meters\""));
    }
}
