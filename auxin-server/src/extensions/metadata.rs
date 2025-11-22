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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_metadata() {
        let metadata = LogicProMetadata::new();

        assert!(metadata.bpm.is_none());
        assert!(metadata.sample_rate.is_none());
        assert!(metadata.key_signature.is_none());
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let metadata = LogicProMetadata::new()
            .with_bpm(120.0)
            .with_sample_rate(48000)
            .with_key_signature("C major")
            .with_tags(vec!["intro".to_string(), "chorus".to_string()]);

        assert_eq!(metadata.bpm, Some(120.0));
        assert_eq!(metadata.sample_rate, Some(48000));
        assert_eq!(metadata.key_signature, Some("C major".to_string()));
        assert_eq!(metadata.tags.len(), 2);
        assert_eq!(metadata.tags[0], "intro");
        assert_eq!(metadata.tags[1], "chorus");
    }

    #[test]
    fn test_default_equals_new() {
        let metadata1 = LogicProMetadata::default();
        let metadata2 = LogicProMetadata::new();

        assert_eq!(metadata1.bpm, metadata2.bpm);
        assert_eq!(metadata1.sample_rate, metadata2.sample_rate);
        assert_eq!(metadata1.key_signature, metadata2.key_signature);
        assert_eq!(metadata1.tags, metadata2.tags);
    }

    #[test]
    fn test_serialization() {
        let metadata = LogicProMetadata::new()
            .with_bpm(140.5)
            .with_sample_rate(44100)
            .with_key_signature("A minor");

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: LogicProMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.bpm, deserialized.bpm);
        assert_eq!(metadata.sample_rate, deserialized.sample_rate);
        assert_eq!(metadata.key_signature, deserialized.key_signature);
        assert_eq!(metadata.tags, deserialized.tags);
    }

    #[test]
    fn test_partial_metadata() {
        let metadata = LogicProMetadata::new().with_bpm(110.0);

        assert_eq!(metadata.bpm, Some(110.0));
        assert!(metadata.sample_rate.is_none());
        assert!(metadata.key_signature.is_none());
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_multiple_tags() {
        let tags = vec![
            "verse".to_string(),
            "bridge".to_string(),
            "outro".to_string(),
        ];
        let metadata = LogicProMetadata::new().with_tags(tags.clone());

        assert_eq!(metadata.tags, tags);
        assert_eq!(metadata.tags.len(), 3);
    }

    #[test]
    fn test_empty_tags() {
        let metadata = LogicProMetadata::new().with_tags(vec![]);

        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_common_sample_rates() {
        let metadata_44_1 = LogicProMetadata::new().with_sample_rate(44100);
        let metadata_48 = LogicProMetadata::new().with_sample_rate(48000);
        let metadata_96 = LogicProMetadata::new().with_sample_rate(96000);

        assert_eq!(metadata_44_1.sample_rate, Some(44100));
        assert_eq!(metadata_48.sample_rate, Some(48000));
        assert_eq!(metadata_96.sample_rate, Some(96000));
    }

    #[test]
    fn test_bpm_precision() {
        let metadata = LogicProMetadata::new().with_bpm(123.456);

        assert_eq!(metadata.bpm, Some(123.456));
    }
}
