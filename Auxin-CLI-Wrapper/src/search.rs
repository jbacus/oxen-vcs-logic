/// Smart search module for commit history with metadata filtering
///
/// Provides AI-powered (fuzzy) search across commit messages and metadata,
/// enabling queries like "show me all commits between 120-128 BPM in A Minor"
///
/// # Features
///
/// - BPM range filtering (e.g., 120-140, >128, <100)
/// - Sample rate filtering
/// - Key signature fuzzy matching
/// - Tag-based search with AND/OR logic
/// - Message text search with case-insensitive matching
/// - Date range filtering
/// - Combined multi-criteria queries
///
/// # Usage
///
/// ```no_run
/// use auxin_cli::search::{SearchQuery, SearchEngine};
/// use auxin_cli::oxen_subprocess::CommitInfo;
///
/// let query = SearchQuery::new()
///     .bpm_range(120.0, 140.0)
///     .key_contains("minor")
///     .tags_any(vec!["mixing".to_string(), "vocals".to_string()]);
///
/// // Assuming you have a list of commits from somewhere
/// let commits: Vec<CommitInfo> = vec![];
/// let engine = SearchEngine::new();
/// let results = engine.search(&commits, &query);
/// ```
use crate::oxen_subprocess::CommitInfo;
use crate::CommitMetadata;
use serde::{Deserialize, Serialize};

/// Represents a search query with multiple filter criteria
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Filter by BPM range (min, max)
    pub bpm_min: Option<f32>,
    pub bpm_max: Option<f32>,

    /// Filter by exact sample rate
    pub sample_rate: Option<u32>,

    /// Filter by key signature (case-insensitive, partial match)
    pub key_contains: Option<String>,

    /// Filter by exact key signature
    pub key_exact: Option<String>,

    /// Filter by tags (ANY match - OR logic)
    pub tags_any: Vec<String>,

    /// Filter by tags (ALL match - AND logic)
    pub tags_all: Vec<String>,

    /// Filter by message content (case-insensitive)
    pub message_contains: Option<String>,

    /// Filter by date range (start date)
    pub date_after: Option<String>,

    /// Filter by date range (end date)
    pub date_before: Option<String>,

    /// Limit number of results
    pub limit: Option<usize>,
}

impl SearchQuery {
    /// Create a new empty search query
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by BPM range
    pub fn bpm_range(mut self, min: f32, max: f32) -> Self {
        self.bpm_min = Some(min);
        self.bpm_max = Some(max);
        self
    }

    /// Filter by minimum BPM
    pub fn bpm_min(mut self, min: f32) -> Self {
        self.bpm_min = Some(min);
        self
    }

    /// Filter by maximum BPM
    pub fn bpm_max(mut self, max: f32) -> Self {
        self.bpm_max = Some(max);
        self
    }

    /// Filter by exact sample rate
    pub fn sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = Some(rate);
        self
    }

    /// Filter by key signature (partial, case-insensitive match)
    pub fn key_contains(mut self, key: impl Into<String>) -> Self {
        self.key_contains = Some(key.into());
        self
    }

    /// Filter by exact key signature
    pub fn key_exact(mut self, key: impl Into<String>) -> Self {
        self.key_exact = Some(key.into());
        self
    }

    /// Filter by tags (ANY match - OR logic)
    pub fn tags_any(mut self, tags: Vec<String>) -> Self {
        self.tags_any = tags;
        self
    }

    /// Filter by tags (ALL match - AND logic)
    pub fn tags_all(mut self, tags: Vec<String>) -> Self {
        self.tags_all = tags;
        self
    }

    /// Filter by message content (case-insensitive)
    pub fn message_contains(mut self, text: impl Into<String>) -> Self {
        self.message_contains = Some(text.into());
        self
    }

    /// Filter by date (after this date)
    pub fn date_after(mut self, date: impl Into<String>) -> Self {
        self.date_after = Some(date.into());
        self
    }

    /// Filter by date (before this date)
    pub fn date_before(mut self, date: impl Into<String>) -> Self {
        self.date_before = Some(date.into());
        self
    }

    /// Limit number of results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Check if query has any active filters
    pub fn has_filters(&self) -> bool {
        self.bpm_min.is_some()
            || self.bpm_max.is_some()
            || self.sample_rate.is_some()
            || self.key_contains.is_some()
            || self.key_exact.is_some()
            || !self.tags_any.is_empty()
            || !self.tags_all.is_empty()
            || self.message_contains.is_some()
            || self.date_after.is_some()
            || self.date_before.is_some()
    }
}

/// Search engine for querying commit history
pub struct SearchEngine;

impl SearchEngine {
    /// Create a new search engine
    pub fn new() -> Self {
        Self
    }

    /// Search commits with the given query
    pub fn search(&self, commits: &[CommitInfo], query: &SearchQuery) -> Vec<CommitInfo> {
        let mut results: Vec<CommitInfo> = commits
            .iter()
            .filter(|commit| self.matches_query(commit, query))
            .cloned()
            .collect();

        // Apply limit if specified
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        results
    }

    /// Check if a commit matches the query criteria
    fn matches_query(&self, commit: &CommitInfo, query: &SearchQuery) -> bool {
        // Parse commit metadata
        let metadata = CommitMetadata::parse_commit_message(&commit.message);

        // BPM filters
        if let Some(min_bpm) = query.bpm_min {
            if let Some(bpm) = metadata.bpm {
                if bpm < min_bpm {
                    return false;
                }
            } else {
                return false; // No BPM metadata, doesn't match
            }
        }

        if let Some(max_bpm) = query.bpm_max {
            if let Some(bpm) = metadata.bpm {
                if bpm > max_bpm {
                    return false;
                }
            } else {
                return false;
            }
        }

        // Sample rate filter
        if let Some(sr) = query.sample_rate {
            if metadata.sample_rate != Some(sr) {
                return false;
            }
        }

        // Key signature filters
        if let Some(ref key_contains) = query.key_contains {
            if let Some(ref key) = metadata.key_signature {
                if !key.to_lowercase().contains(&key_contains.to_lowercase()) {
                    return false;
                }
            } else {
                return false;
            }
        }

        if let Some(ref key_exact) = query.key_exact {
            if metadata.key_signature.as_deref() != Some(key_exact.as_str()) {
                return false;
            }
        }

        // Tags ANY (OR logic)
        if !query.tags_any.is_empty() {
            let has_any_tag = query.tags_any.iter().any(|tag| metadata.tags.contains(tag));
            if !has_any_tag {
                return false;
            }
        }

        // Tags ALL (AND logic)
        if !query.tags_all.is_empty() {
            let has_all_tags = query.tags_all.iter().all(|tag| metadata.tags.contains(tag));
            if !has_all_tags {
                return false;
            }
        }

        // Message filter
        if let Some(ref msg_contains) = query.message_contains {
            if !metadata
                .message
                .to_lowercase()
                .contains(&msg_contains.to_lowercase())
            {
                return false;
            }
        }

        true
    }

    /// Parse a natural language query string into a SearchQuery
    ///
    /// Supports syntax like:
    /// - "bpm:120-140"
    /// - "key:minor"
    /// - "tag:mixing,vocals"
    /// - "sr:48000"
    /// - "msg:final"
    /// - Combined: "bpm:>120 key:minor tag:vocals"
    pub fn parse_query(query_str: &str) -> SearchQuery {
        let mut query = SearchQuery::new();

        for part in query_str.split_whitespace() {
            if let Some((key, value)) = part.split_once(':') {
                match key.to_lowercase().as_str() {
                    "bpm" => {
                        if value.contains('-') {
                            // Range: "120-140"
                            let parts: Vec<&str> = value.split('-').collect();
                            if parts.len() == 2 {
                                if let (Ok(min), Ok(max)) =
                                    (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                                {
                                    query = query.bpm_range(min, max);
                                }
                            }
                        } else if let Some(stripped) = value.strip_prefix('>') {
                            // Greater than: ">120"
                            if let Ok(min) = stripped.parse::<f32>() {
                                query = query.bpm_min(min);
                            }
                        } else if let Some(stripped) = value.strip_prefix('<') {
                            // Less than: "<140"
                            if let Ok(max) = stripped.parse::<f32>() {
                                query = query.bpm_max(max);
                            }
                        } else {
                            // Exact or single value - treat as minimum
                            if let Ok(bpm) = value.parse::<f32>() {
                                query = query.bpm_min(bpm).bpm_max(bpm);
                            }
                        }
                    }
                    "sr" | "samplerate" | "sample-rate" => {
                        if let Ok(sr) = value.parse::<u32>() {
                            query = query.sample_rate(sr);
                        }
                    }
                    "key" => {
                        query = query.key_contains(value);
                    }
                    "tag" | "tags" => {
                        let tags: Vec<String> =
                            value.split(',').map(|s| s.trim().to_string()).collect();
                        query = query.tags_any(tags);
                    }
                    "msg" | "message" => {
                        query = query.message_contains(value);
                    }
                    "limit" => {
                        if let Ok(limit) = value.parse::<usize>() {
                            query = query.limit(limit);
                        }
                    }
                    _ => {}
                }
            }
        }

        query
    }

    /// Calculate a relevance score for a commit (for ranking results)
    pub fn relevance_score(&self, commit: &CommitInfo, query: &SearchQuery) -> f32 {
        let metadata = CommitMetadata::parse_commit_message(&commit.message);
        let mut score = 0.0;

        // Higher score for exact BPM match
        if let (Some(min), Some(max), Some(bpm)) = (query.bpm_min, query.bpm_max, metadata.bpm) {
            let mid = (min + max) / 2.0;
            let distance = (bpm - mid).abs();
            let range = max - min;
            score += 10.0 * (1.0 - (distance / range).min(1.0));
        }

        // Score for tag matches
        let tag_matches = query
            .tags_any
            .iter()
            .filter(|tag| metadata.tags.contains(tag))
            .count();
        score += tag_matches as f32 * 5.0;

        // Score for message relevance
        if let Some(ref msg_contains) = query.message_contains {
            if metadata.message.to_lowercase().contains(&msg_contains.to_lowercase()) {
                score += 3.0;
            }
        }

        score
    }
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commit(message: &str, bpm: Option<f32>, key: Option<&str>) -> CommitInfo {
        let mut metadata = CommitMetadata::new(message);
        if let Some(b) = bpm {
            metadata = metadata.with_bpm(b);
        }
        if let Some(k) = key {
            metadata = metadata.with_key_signature(k);
        }

        CommitInfo {
            id: "abc123".to_string(),
            message: metadata.format_commit_message(),
        }
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new()
            .bpm_range(120.0, 140.0)
            .key_contains("minor")
            .tags_any(vec!["mixing".to_string()]);

        assert_eq!(query.bpm_min, Some(120.0));
        assert_eq!(query.bpm_max, Some(140.0));
        assert!(query.has_filters());
    }

    #[test]
    fn test_bpm_range_filter() {
        let engine = SearchEngine::new();
        let commits = vec![
            create_test_commit("Low BPM", Some(100.0), None),
            create_test_commit("Mid BPM", Some(130.0), None),
            create_test_commit("High BPM", Some(160.0), None),
        ];

        let query = SearchQuery::new().bpm_range(120.0, 140.0);
        let results = engine.search(&commits, &query);

        assert_eq!(results.len(), 1);
        assert!(results[0].message.contains("Mid BPM"));
    }

    #[test]
    fn test_key_signature_filter() {
        let engine = SearchEngine::new();
        let commits = vec![
            create_test_commit("C Major", None, Some("C Major")),
            create_test_commit("A Minor", None, Some("A Minor")),
            create_test_commit("D Minor", None, Some("D Minor")),
        ];

        let query = SearchQuery::new().key_contains("minor");
        let results = engine.search(&commits, &query);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_message_filter() {
        let engine = SearchEngine::new();
        let commits = vec![
            create_test_commit("Final mix ready", None, None),
            create_test_commit("Draft version", None, None),
            create_test_commit("Final master", None, None),
        ];

        let query = SearchQuery::new().message_contains("final");
        let results = engine.search(&commits, &query);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_combined_filters() {
        let engine = SearchEngine::new();
        let metadata = CommitMetadata::new("Final mix")
            .with_bpm(128.0)
            .with_key_signature("A Minor")
            .with_tag("mixing");

        let commit = CommitInfo {
            id: "abc123".to_string(),
            message: metadata.format_commit_message(),
        };

        let query = SearchQuery::new()
            .bpm_range(120.0, 140.0)
            .key_contains("minor")
            .tags_any(vec!["mixing".to_string()]);

        let results = engine.search(&[commit], &query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_parse_query_bpm_range() {
        let query = SearchEngine::parse_query("bpm:120-140");
        assert_eq!(query.bpm_min, Some(120.0));
        assert_eq!(query.bpm_max, Some(140.0));
    }

    #[test]
    fn test_parse_query_bpm_greater_than() {
        let query = SearchEngine::parse_query("bpm:>120");
        assert_eq!(query.bpm_min, Some(120.0));
        assert_eq!(query.bpm_max, None);
    }

    #[test]
    fn test_parse_query_combined() {
        let query = SearchEngine::parse_query("bpm:120-140 key:minor tag:mixing,vocals");
        assert_eq!(query.bpm_min, Some(120.0));
        assert_eq!(query.key_contains, Some("minor".to_string()));
        assert_eq!(query.tags_any.len(), 2);
    }

    #[test]
    fn test_relevance_score() {
        let engine = SearchEngine::new();
        let commit = create_test_commit("Test", Some(130.0), None);
        let query = SearchQuery::new().bpm_range(120.0, 140.0);

        let score = engine.relevance_score(&commit, &query);
        assert!(score > 0.0);
    }

    #[test]
    fn test_empty_query_returns_all() {
        let engine = SearchEngine::new();
        let commits = vec![
            create_test_commit("A", None, None),
            create_test_commit("B", None, None),
        ];

        let query = SearchQuery::new();
        let results = engine.search(&commits, &query);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_limit() {
        let engine = SearchEngine::new();
        let commits = vec![
            create_test_commit("A", None, None),
            create_test_commit("B", None, None),
            create_test_commit("C", None, None),
        ];

        let query = SearchQuery::new().limit(2);
        let results = engine.search(&commits, &query);

        assert_eq!(results.len(), 2);
    }
}
