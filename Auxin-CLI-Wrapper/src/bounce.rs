//! Audio bounce management for commit snapshots
//!
//! Bounces are audio files that capture the sonic state of a project at each
//! major commit. They serve as audio "screenshots" for:
//! - Quick A/B comparison between versions
//! - Audio fingerprinting and semantic analysis
//! - Historical record of project evolution

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Filter criteria for searching bounces
#[derive(Debug, Default, Clone)]
pub struct BounceFilter {
    /// Filter by audio format
    pub format: Option<AudioFormat>,
    /// Filter by filename pattern (regex) - alias for filename_pattern
    pub pattern: Option<String>,
    /// Filter by filename pattern (regex)
    pub filename_pattern: Option<String>,
    /// Filter by minimum duration (seconds)
    pub min_duration: Option<f64>,
    /// Filter by maximum duration (seconds)
    pub max_duration: Option<f64>,
    /// Filter by date range (after)
    pub after: Option<DateTime<Utc>>,
    /// Filter by date range (before)
    pub before: Option<DateTime<Utc>>,
    /// Filter by user who added
    pub added_by: Option<String>,
    /// Filter by minimum file size (bytes)
    pub min_size: Option<u64>,
    /// Filter by maximum file size (bytes)
    pub max_size: Option<u64>,
}

impl BounceFilter {
    /// Get the effective pattern (pattern takes precedence over filename_pattern)
    pub fn effective_pattern(&self) -> Option<&String> {
        self.pattern.as_ref().or(self.filename_pattern.as_ref())
    }
}

/// Supported audio formats for bounces
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Aiff,
    Mp3,
    Flac,
    M4a,
}

impl AudioFormat {
    /// Detect format from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "wav" => Some(AudioFormat::Wav),
            "aif" | "aiff" => Some(AudioFormat::Aiff),
            "mp3" => Some(AudioFormat::Mp3),
            "flac" => Some(AudioFormat::Flac),
            "m4a" | "aac" => Some(AudioFormat::M4a),
            _ => None,
        }
    }

    /// Get file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Aiff => "aiff",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Flac => "flac",
            AudioFormat::M4a => "m4a",
        }
    }

    /// Get MIME type for this format
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Aiff => "audio/aiff",
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::M4a => "audio/mp4",
        }
    }
}

/// Metadata about a bounce file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceMetadata {
    /// Commit ID this bounce is associated with
    pub commit_id: String,

    /// Original filename
    pub original_filename: String,

    /// Audio format
    pub format: AudioFormat,

    /// File size in bytes
    pub size_bytes: u64,

    /// Duration in seconds (if known)
    pub duration_secs: Option<f64>,

    /// Sample rate (if known)
    pub sample_rate: Option<u32>,

    /// Bit depth (if known)
    pub bit_depth: Option<u16>,

    /// Number of channels (if known)
    pub channels: Option<u8>,

    /// When the bounce was added
    pub added_at: DateTime<Utc>,

    /// User who added the bounce
    pub added_by: String,

    /// Optional description
    pub description: Option<String>,

    /// Audio fingerprint hash (for comparison)
    pub fingerprint: Option<String>,
}

impl BounceMetadata {
    /// Create new bounce metadata
    pub fn new(commit_id: &str, original_filename: &str, format: AudioFormat, size_bytes: u64) -> Self {
        Self {
            commit_id: commit_id.to_string(),
            original_filename: original_filename.to_string(),
            format,
            size_bytes,
            duration_secs: None,
            sample_rate: None,
            bit_depth: None,
            channels: None,
            added_at: Utc::now(),
            added_by: get_current_user(),
            description: None,
            fingerprint: None,
        }
    }

    /// Set duration
    pub fn with_duration(mut self, secs: f64) -> Self {
        self.duration_secs = Some(secs);
        self
    }

    /// Set audio properties
    pub fn with_audio_info(mut self, sample_rate: u32, bit_depth: u16, channels: u8) -> Self {
        self.sample_rate = Some(sample_rate);
        self.bit_depth = Some(bit_depth);
        self.channels = Some(channels);
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Format duration for display
    pub fn format_duration(&self) -> String {
        match self.duration_secs {
            Some(secs) => {
                let mins = (secs / 60.0).floor() as u32;
                let remaining_secs = secs % 60.0;
                format!("{}:{:05.2}", mins, remaining_secs)
            }
            None => "unknown".to_string(),
        }
    }

    /// Format file size for display
    pub fn format_size(&self) -> String {
        if self.size_bytes >= 1_000_000_000 {
            format!("{:.2} GB", self.size_bytes as f64 / 1_000_000_000.0)
        } else if self.size_bytes >= 1_000_000 {
            format!("{:.2} MB", self.size_bytes as f64 / 1_000_000.0)
        } else if self.size_bytes >= 1_000 {
            format!("{:.1} KB", self.size_bytes as f64 / 1_000.0)
        } else {
            format!("{} bytes", self.size_bytes)
        }
    }
}

/// Manages bounce files for a repository
pub struct BounceManager {
    /// Root directory of the repository
    #[allow(dead_code)]
    repo_root: PathBuf,

    /// Directory where bounces are stored
    bounces_dir: PathBuf,
}

impl BounceManager {
    /// Create a new bounce manager for a repository
    pub fn new(repo_root: &Path) -> Self {
        let bounces_dir = repo_root.join(".auxin").join("bounces");
        Self {
            repo_root: repo_root.to_path_buf(),
            bounces_dir,
        }
    }

    /// Initialize bounce storage directory
    pub fn init(&self) -> Result<()> {
        if !self.bounces_dir.exists() {
            fs::create_dir_all(&self.bounces_dir)
                .context("Failed to create bounces directory")?;
        }
        Ok(())
    }

    /// Add a bounce file for a commit
    pub fn add_bounce(
        &self,
        commit_id: &str,
        source_file: &Path,
        description: Option<&str>,
    ) -> Result<BounceMetadata> {
        // Ensure bounces directory exists
        self.init()?;

        // Validate source file
        if !source_file.exists() {
            return Err(anyhow!("Source file not found: {}", source_file.display()));
        }

        // Detect format from extension
        let ext = source_file
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow!("Cannot determine file format"))?;

        let format = AudioFormat::from_extension(ext)
            .ok_or_else(|| anyhow!("Unsupported audio format: {}", ext))?;

        // Get file info
        let file_meta = fs::metadata(source_file)
            .context("Failed to read file metadata")?;
        let size_bytes = file_meta.len();

        // Generate filename: commit_id.extension
        let bounce_filename = format!("{}.{}", commit_id, format.extension());
        let dest_path = self.bounces_dir.join(&bounce_filename);

        // Copy file to bounces directory
        fs::copy(source_file, &dest_path)
            .context("Failed to copy bounce file")?;

        // Create metadata
        let original_filename = source_file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let mut metadata = BounceMetadata::new(commit_id, &original_filename, format, size_bytes);

        if let Some(desc) = description {
            metadata = metadata.with_description(desc);
        }

        // Try to extract audio info (using afinfo on macOS)
        if let Ok(info) = self.extract_audio_info(&dest_path) {
            if let Some(duration) = info.duration {
                metadata.duration_secs = Some(duration);
            }
            if let Some(sr) = info.sample_rate {
                metadata.sample_rate = Some(sr);
            }
            if let Some(bd) = info.bit_depth {
                metadata.bit_depth = Some(bd);
            }
            if let Some(ch) = info.channels {
                metadata.channels = Some(ch);
            }
        }

        // Save metadata
        self.save_metadata(&metadata)?;

        Ok(metadata)
    }

    /// Get bounce for a commit
    pub fn get_bounce(&self, commit_id: &str) -> Result<Option<BounceMetadata>> {
        let metadata_path = self.bounces_dir.join(format!("{}.json", commit_id));

        if !metadata_path.exists() {
            return Ok(None);
        }

        let contents = fs::read_to_string(&metadata_path)
            .context("Failed to read bounce metadata")?;

        let metadata: BounceMetadata = serde_json::from_str(&contents)
            .context("Failed to parse bounce metadata")?;

        Ok(Some(metadata))
    }

    /// Get path to bounce audio file
    pub fn get_bounce_path(&self, commit_id: &str) -> Result<Option<PathBuf>> {
        // Try common extensions
        for ext in &["wav", "aiff", "mp3", "flac", "m4a"] {
            let path = self.bounces_dir.join(format!("{}.{}", commit_id, ext));
            if path.exists() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    /// List all bounces
    pub fn list_bounces(&self) -> Result<Vec<BounceMetadata>> {
        if !self.bounces_dir.exists() {
            return Ok(vec![]);
        }

        let mut bounces = Vec::new();

        for entry in fs::read_dir(&self.bounces_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<BounceMetadata>(&contents) {
                        bounces.push(metadata);
                    }
                }
            }
        }

        // Sort by added date (newest first)
        bounces.sort_by(|a, b| b.added_at.cmp(&a.added_at));

        Ok(bounces)
    }

    /// Search bounces with filters
    pub fn search_bounces(&self, filter: &BounceFilter) -> Result<Vec<BounceMetadata>> {
        let all_bounces = self.list_bounces()?;

        let filtered: Vec<BounceMetadata> = all_bounces
            .into_iter()
            .filter(|bounce| {
                // Format filter
                if let Some(format) = &filter.format {
                    if bounce.format != *format {
                        return false;
                    }
                }

                // Filename pattern filter (check both pattern and filename_pattern)
                if let Some(pattern) = filter.effective_pattern() {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(&bounce.original_filename) {
                            return false;
                        }
                    }
                }

                // Duration filters
                if let Some(min_dur) = filter.min_duration {
                    if let Some(dur) = bounce.duration_secs {
                        if dur < min_dur {
                            return false;
                        }
                    } else {
                        return false; // No duration means doesn't match min
                    }
                }

                if let Some(max_dur) = filter.max_duration {
                    if let Some(dur) = bounce.duration_secs {
                        if dur > max_dur {
                            return false;
                        }
                    }
                }

                // Date range filters
                if let Some(after) = &filter.after {
                    if bounce.added_at < *after {
                        return false;
                    }
                }

                if let Some(before) = &filter.before {
                    if bounce.added_at > *before {
                        return false;
                    }
                }

                // User filter
                if let Some(user) = &filter.added_by {
                    if !bounce.added_by.to_lowercase().contains(&user.to_lowercase()) {
                        return false;
                    }
                }

                // Size filters
                if let Some(min_size) = filter.min_size {
                    if bounce.size_bytes < min_size {
                        return false;
                    }
                }

                if let Some(max_size) = filter.max_size {
                    if bounce.size_bytes > max_size {
                        return false;
                    }
                }

                true
            })
            .collect();

        Ok(filtered)
    }

    /// Compare two bounces by commit ID
    pub fn compare_bounces(&self, commit_a: &str, commit_b: &str) -> Result<BounceComparison> {
        let bounce_a = self.get_bounce(commit_a)?
            .ok_or_else(|| anyhow!("No bounce found for commit {}", commit_a))?;
        let bounce_b = self.get_bounce(commit_b)?
            .ok_or_else(|| anyhow!("No bounce found for commit {}", commit_b))?;

        Ok(BounceComparison {
            bounce_a,
            bounce_b,
        })
    }

    /// Play a bounce using the system audio player
    pub fn play_bounce(&self, commit_id: &str) -> Result<()> {
        let path = self.get_bounce_path(commit_id)?
            .ok_or_else(|| anyhow!("No bounce found for commit {}", commit_id))?;

        // Use macOS 'afplay' command
        let status = Command::new("afplay")
            .arg(&path)
            .status()
            .context("Failed to play audio (is afplay available?)")?;

        if !status.success() {
            return Err(anyhow!("Audio playback failed"));
        }

        Ok(())
    }

    /// Delete a bounce
    pub fn delete_bounce(&self, commit_id: &str) -> Result<()> {
        // Delete audio file
        if let Some(audio_path) = self.get_bounce_path(commit_id)? {
            fs::remove_file(&audio_path)
                .context("Failed to delete bounce audio file")?;
        }

        // Delete metadata
        let metadata_path = self.bounces_dir.join(format!("{}.json", commit_id));
        if metadata_path.exists() {
            fs::remove_file(&metadata_path)
                .context("Failed to delete bounce metadata")?;
        }

        Ok(())
    }

    /// Save bounce metadata to JSON file
    fn save_metadata(&self, metadata: &BounceMetadata) -> Result<()> {
        let path = self.bounces_dir.join(format!("{}.json", metadata.commit_id));
        let json = serde_json::to_string_pretty(metadata)
            .context("Failed to serialize metadata")?;
        fs::write(&path, json)
            .context("Failed to write metadata file")?;
        Ok(())
    }

    /// Extract audio information using system tools
    fn extract_audio_info(&self, path: &Path) -> Result<AudioInfo> {
        // Use macOS afinfo command
        let output = Command::new("afinfo")
            .arg(path)
            .output()
            .context("Failed to run afinfo")?;

        if !output.status.success() {
            return Err(anyhow!("afinfo failed"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut info = AudioInfo::default();

        for line in stdout.lines() {
            if let Some(value) = line.strip_prefix("estimated duration:") {
                if let Ok(secs) = value.trim().trim_end_matches(" sec").parse::<f64>() {
                    info.duration = Some(secs);
                }
            } else if let Some(value) = line.strip_prefix("sample rate:") {
                if let Ok(sr) = value.trim().parse::<u32>() {
                    info.sample_rate = Some(sr);
                }
            } else if let Some(value) = line.strip_prefix("bits per channel:") {
                if let Ok(bd) = value.trim().parse::<u16>() {
                    info.bit_depth = Some(bd);
                }
            } else if let Some(value) = line.strip_prefix("channels:") {
                if let Ok(ch) = value.trim().parse::<u8>() {
                    info.channels = Some(ch);
                }
            }
        }

        Ok(info)
    }
}

/// Audio file information
#[derive(Debug, Default)]
struct AudioInfo {
    duration: Option<f64>,
    sample_rate: Option<u32>,
    bit_depth: Option<u16>,
    channels: Option<u8>,
}

/// Comparison between two bounces
#[derive(Debug, Clone)]
pub struct BounceComparison {
    pub bounce_a: BounceMetadata,
    pub bounce_b: BounceMetadata,
}

impl BounceComparison {
    /// Get duration difference in seconds
    pub fn duration_diff(&self) -> Option<f64> {
        match (self.bounce_a.duration_secs, self.bounce_b.duration_secs) {
            (Some(a), Some(b)) => Some(b - a),
            _ => None,
        }
    }

    /// Get size difference in bytes
    pub fn size_diff(&self) -> i64 {
        self.bounce_b.size_bytes as i64 - self.bounce_a.size_bytes as i64
    }

    /// Format comparison as a report
    pub fn format_report(&self) -> String {
        let mut report = String::new();

        report.push_str(&format!("Bounce A: {} (commit {})\n",
            self.bounce_a.original_filename,
            &self.bounce_a.commit_id[..8.min(self.bounce_a.commit_id.len())]));
        report.push_str(&format!("Bounce B: {} (commit {})\n\n",
            self.bounce_b.original_filename,
            &self.bounce_b.commit_id[..8.min(self.bounce_b.commit_id.len())]));

        // Duration comparison
        report.push_str("Duration:\n");
        report.push_str(&format!("  A: {}\n", self.bounce_a.format_duration()));
        report.push_str(&format!("  B: {}\n", self.bounce_b.format_duration()));
        if let Some(diff) = self.duration_diff() {
            let sign = if diff >= 0.0 { "+" } else { "" };
            report.push_str(&format!("  Diff: {}{:.2}s\n", sign, diff));
        }

        // Size comparison
        report.push_str("\nFile Size:\n");
        report.push_str(&format!("  A: {}\n", self.bounce_a.format_size()));
        report.push_str(&format!("  B: {}\n", self.bounce_b.format_size()));
        let size_diff = self.size_diff();
        let sign = if size_diff >= 0 { "+" } else { "" };
        if size_diff.abs() >= 1_000_000 {
            report.push_str(&format!("  Diff: {}{:.2} MB\n", sign, size_diff as f64 / 1_000_000.0));
        } else if size_diff.abs() >= 1_000 {
            report.push_str(&format!("  Diff: {}{:.1} KB\n", sign, size_diff as f64 / 1_000.0));
        } else {
            report.push_str(&format!("  Diff: {} bytes\n", size_diff));
        }

        // Format comparison
        report.push_str("\nFormat:\n");
        report.push_str(&format!("  A: {:?}\n", self.bounce_a.format));
        report.push_str(&format!("  B: {:?}\n", self.bounce_b.format));

        // Sample rate comparison
        if self.bounce_a.sample_rate.is_some() || self.bounce_b.sample_rate.is_some() {
            report.push_str("\nSample Rate:\n");
            report.push_str(&format!("  A: {} Hz\n",
                self.bounce_a.sample_rate.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string())));
            report.push_str(&format!("  B: {} Hz\n",
                self.bounce_b.sample_rate.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string())));
        }

        report
    }
}

/// Get current user identifier
fn get_current_user() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("wav"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("WAV"), Some(AudioFormat::Wav));
        assert_eq!(AudioFormat::from_extension("aif"), Some(AudioFormat::Aiff));
        assert_eq!(AudioFormat::from_extension("aiff"), Some(AudioFormat::Aiff));
        assert_eq!(AudioFormat::from_extension("mp3"), Some(AudioFormat::Mp3));
        assert_eq!(AudioFormat::from_extension("flac"), Some(AudioFormat::Flac));
        assert_eq!(AudioFormat::from_extension("m4a"), Some(AudioFormat::M4a));
        assert_eq!(AudioFormat::from_extension("xyz"), None);
    }

    #[test]
    fn test_bounce_metadata_format_duration() {
        let mut meta = BounceMetadata::new("abc123", "test.wav", AudioFormat::Wav, 1000);

        assert_eq!(meta.format_duration(), "unknown");

        meta.duration_secs = Some(65.5);
        assert_eq!(meta.format_duration(), "1:05.50");

        meta.duration_secs = Some(3661.25);
        assert_eq!(meta.format_duration(), "61:01.25");
    }

    #[test]
    fn test_bounce_metadata_format_size() {
        let meta = BounceMetadata::new("abc123", "test.wav", AudioFormat::Wav, 500);
        assert_eq!(meta.format_size(), "500 bytes");

        let meta = BounceMetadata::new("abc123", "test.wav", AudioFormat::Wav, 1500);
        assert_eq!(meta.format_size(), "1.5 KB");

        let meta = BounceMetadata::new("abc123", "test.wav", AudioFormat::Wav, 1_500_000);
        assert_eq!(meta.format_size(), "1.50 MB");

        let meta = BounceMetadata::new("abc123", "test.wav", AudioFormat::Wav, 1_500_000_000);
        assert_eq!(meta.format_size(), "1.50 GB");
    }

    #[test]
    fn test_bounce_manager_creation() {
        let manager = BounceManager::new(Path::new("/tmp/test-repo"));
        assert_eq!(manager.bounces_dir, PathBuf::from("/tmp/test-repo/.auxin/bounces"));
    }

    #[test]
    fn test_audio_format_mime_types() {
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Flac.mime_type(), "audio/flac");
    }
}
