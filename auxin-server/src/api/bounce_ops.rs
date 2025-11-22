//! Bounce audio file API operations
//!
//! Handles storage and retrieval of audio bounce files for commits.

use actix_multipart::Multipart;
use actix_web::{web, HttpResponse};
use chrono::{DateTime, Utc};
use futures::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use tracing::{error, info};

use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Supported audio formats
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AudioFormat {
    Wav,
    Aiff,
    Mp3,
    Flac,
    M4a,
}

impl AudioFormat {
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

    pub fn extension(&self) -> &'static str {
        match self {
            AudioFormat::Wav => "wav",
            AudioFormat::Aiff => "aiff",
            AudioFormat::Mp3 => "mp3",
            AudioFormat::Flac => "flac",
            AudioFormat::M4a => "m4a",
        }
    }

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

/// Bounce metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BounceMetadata {
    pub commit_id: String,
    pub original_filename: String,
    pub format: AudioFormat,
    pub size_bytes: u64,
    pub duration_secs: Option<f64>,
    pub sample_rate: Option<u32>,
    pub bit_depth: Option<u16>,
    pub channels: Option<u8>,
    pub added_at: DateTime<Utc>,
    pub added_by: String,
    pub description: Option<String>,
}

/// Bounce upload request
#[derive(Debug, Deserialize)]
pub struct BounceUploadRequest {
    pub description: Option<String>,
}

/// Bounce query parameters for filtering
#[derive(Debug, Deserialize)]
pub struct BounceQuery {
    /// Filter by audio format
    pub format: Option<String>,
    /// Filter by filename pattern
    pub pattern: Option<String>,
    /// Minimum duration in seconds
    pub min_duration: Option<f64>,
    /// Maximum duration in seconds
    pub max_duration: Option<f64>,
    /// Minimum size in bytes
    pub min_size: Option<u64>,
    /// Maximum size in bytes
    pub max_size: Option<u64>,
    /// Filter by user
    pub user: Option<String>,
}

/// Get bounces directory for a repository
fn get_bounces_dir(config: &Config, namespace: &str, repo_name: &str) -> PathBuf {
    PathBuf::from(&config.sync_dir)
        .join(namespace)
        .join(repo_name)
        .join(".auxin")
        .join("bounces")
}

/// List all bounces for a repository with optional filtering
pub async fn list_bounces(
    config: web::Data<Config>,
    path: web::Path<(String, String)>,
    query: web::Query<BounceQuery>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name) = path.into_inner();
    info!("Listing bounces for {}/{}", namespace, repo_name);

    let bounces_dir = get_bounces_dir(&config, &namespace, &repo_name);

    if !bounces_dir.exists() {
        return Ok(HttpResponse::Ok().json(Vec::<BounceMetadata>::new()));
    }

    let mut bounces = Vec::new();

    if let Ok(entries) = fs::read_dir(&bounces_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                if let Ok(contents) = fs::read_to_string(&path) {
                    if let Ok(metadata) = serde_json::from_str::<BounceMetadata>(&contents) {
                        bounces.push(metadata);
                    }
                }
            }
        }
    }

    // Apply filters
    let filtered: Vec<BounceMetadata> = bounces
        .into_iter()
        .filter(|bounce| {
            // Format filter
            if let Some(fmt) = &query.format {
                let format_matches = match fmt.to_lowercase().as_str() {
                    "wav" => matches!(bounce.format, AudioFormat::Wav),
                    "aiff" | "aif" => matches!(bounce.format, AudioFormat::Aiff),
                    "mp3" => matches!(bounce.format, AudioFormat::Mp3),
                    "flac" => matches!(bounce.format, AudioFormat::Flac),
                    "m4a" => matches!(bounce.format, AudioFormat::M4a),
                    _ => true,
                };
                if !format_matches {
                    return false;
                }
            }

            // Filename pattern filter
            if let Some(pattern) = &query.pattern {
                if !bounce
                    .original_filename
                    .to_lowercase()
                    .contains(&pattern.to_lowercase())
                {
                    return false;
                }
            }

            // Duration filters
            if let Some(min_dur) = query.min_duration {
                if let Some(dur) = bounce.duration_secs {
                    if dur < min_dur {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            if let Some(max_dur) = query.max_duration {
                if let Some(dur) = bounce.duration_secs {
                    if dur > max_dur {
                        return false;
                    }
                }
            }

            // Size filters
            if let Some(min_size) = query.min_size {
                if bounce.size_bytes < min_size {
                    return false;
                }
            }

            if let Some(max_size) = query.max_size {
                if bounce.size_bytes > max_size {
                    return false;
                }
            }

            // User filter
            if let Some(user) = &query.user {
                if !bounce
                    .added_by
                    .to_lowercase()
                    .contains(&user.to_lowercase())
                {
                    return false;
                }
            }

            true
        })
        .collect();

    // Sort by added date (newest first)
    let mut result = filtered;
    result.sort_by(|a, b| b.added_at.cmp(&a.added_at));

    info!("Found {} bounces (filtered from query)", result.len());
    Ok(HttpResponse::Ok().json(result))
}

/// Get bounce metadata for a specific commit
pub async fn get_bounce(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Getting bounce for {}/{} commit {}",
        namespace, repo_name, commit_id
    );

    let bounces_dir = get_bounces_dir(&config, &namespace, &repo_name);
    let metadata_path = bounces_dir.join(format!("{}.json", commit_id));

    if !metadata_path.exists() {
        return Err(AppError::NotFound(format!(
            "No bounce found for commit {}",
            commit_id
        )));
    }

    let contents = fs::read_to_string(&metadata_path)
        .map_err(|e| AppError::Internal(format!("Failed to read bounce metadata: {}", e)))?;

    let metadata: BounceMetadata = serde_json::from_str(&contents)
        .map_err(|e| AppError::Internal(format!("Failed to parse bounce metadata: {}", e)))?;

    Ok(HttpResponse::Ok().json(metadata))
}

/// Get bounce audio file for streaming
pub async fn get_bounce_audio(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Getting bounce audio for {}/{} commit {}",
        namespace, repo_name, commit_id
    );

    let bounces_dir = get_bounces_dir(&config, &namespace, &repo_name);

    // Try to find the audio file
    let audio_path = ["wav", "aiff", "mp3", "flac", "m4a"]
        .iter()
        .map(|ext| bounces_dir.join(format!("{}.{}", commit_id, ext)))
        .find(|p| p.exists());

    let audio_path = match audio_path {
        Some(path) => path,
        None => {
            return Err(AppError::NotFound(format!(
                "No bounce audio found for commit {}",
                commit_id
            )));
        }
    };

    // Determine content type
    let ext = audio_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("wav");
    let format = AudioFormat::from_extension(ext).unwrap_or(AudioFormat::Wav);

    // Read and return file
    let data = fs::read(&audio_path)
        .map_err(|e| AppError::Internal(format!("Failed to read bounce audio: {}", e)))?;

    Ok(HttpResponse::Ok()
        .content_type(format.mime_type())
        .body(data))
}

/// Upload a bounce file for a commit
pub async fn upload_bounce(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
    mut payload: Multipart,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Uploading bounce for {}/{} commit {}",
        namespace, repo_name, commit_id
    );

    let bounces_dir = get_bounces_dir(&config, &namespace, &repo_name);

    // Create bounces directory if it doesn't exist
    fs::create_dir_all(&bounces_dir)
        .map_err(|e| AppError::Internal(format!("Failed to create bounces directory: {}", e)))?;

    let mut description: Option<String> = None;
    let mut audio_data: Option<Vec<u8>> = None;
    let mut filename: Option<String> = None;

    // Process multipart fields
    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = match field.content_disposition() {
            Some(cd) => cd,
            None => continue,
        };
        let field_name = content_disposition.get_name().unwrap_or("");

        match field_name {
            "description" => {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk.map_err(|e| {
                        AppError::Internal(format!("Failed to read description: {}", e))
                    })?;
                    data.extend_from_slice(&chunk);
                }
                description = Some(String::from_utf8_lossy(&data).to_string());
            }
            "file" => {
                filename = content_disposition.get_filename().map(|s| s.to_string());
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    let chunk = chunk
                        .map_err(|e| AppError::Internal(format!("Failed to read file: {}", e)))?;
                    data.extend_from_slice(&chunk);
                }
                audio_data = Some(data);
            }
            _ => {}
        }
    }

    // Validate we got an audio file
    let audio_data =
        audio_data.ok_or_else(|| AppError::BadRequest("No audio file provided".to_string()))?;

    let filename =
        filename.ok_or_else(|| AppError::BadRequest("No filename provided".to_string()))?;

    // Get format from filename
    let ext = std::path::Path::new(&filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| AppError::BadRequest("Cannot determine file format".to_string()))?;

    let format = AudioFormat::from_extension(ext)
        .ok_or_else(|| AppError::BadRequest(format!("Unsupported audio format: {}", ext)))?;

    // Save audio file
    let audio_path = bounces_dir.join(format!("{}.{}", commit_id, format.extension()));
    let mut file = fs::File::create(&audio_path)
        .map_err(|e| AppError::Internal(format!("Failed to create audio file: {}", e)))?;
    file.write_all(&audio_data)
        .map_err(|e| AppError::Internal(format!("Failed to write audio file: {}", e)))?;

    // Create metadata
    let metadata = BounceMetadata {
        commit_id: commit_id.clone(),
        original_filename: filename,
        format,
        size_bytes: audio_data.len() as u64,
        duration_secs: None, // Could extract with audio analysis
        sample_rate: None,
        bit_depth: None,
        channels: None,
        added_at: Utc::now(),
        added_by: "api".to_string(), // Could get from auth
        description,
    };

    // Save metadata
    let metadata_path = bounces_dir.join(format!("{}.json", commit_id));
    let json = serde_json::to_string_pretty(&metadata)
        .map_err(|e| AppError::Internal(format!("Failed to serialize metadata: {}", e)))?;
    fs::write(&metadata_path, json)
        .map_err(|e| AppError::Internal(format!("Failed to write metadata: {}", e)))?;

    info!("Bounce uploaded successfully for commit {}", commit_id);
    Ok(HttpResponse::Created().json(metadata))
}

/// Delete a bounce
pub async fn delete_bounce(
    config: web::Data<Config>,
    path: web::Path<(String, String, String)>,
) -> AppResult<HttpResponse> {
    let (namespace, repo_name, commit_id) = path.into_inner();
    info!(
        "Deleting bounce for {}/{} commit {}",
        namespace, repo_name, commit_id
    );

    let bounces_dir = get_bounces_dir(&config, &namespace, &repo_name);

    // Delete audio file
    for ext in &["wav", "aiff", "mp3", "flac", "m4a"] {
        let audio_path = bounces_dir.join(format!("{}.{}", commit_id, ext));
        if audio_path.exists() {
            fs::remove_file(&audio_path)
                .map_err(|e| AppError::Internal(format!("Failed to delete audio file: {}", e)))?;
            break;
        }
    }

    // Delete metadata
    let metadata_path = bounces_dir.join(format!("{}.json", commit_id));
    if metadata_path.exists() {
        fs::remove_file(&metadata_path)
            .map_err(|e| AppError::Internal(format!("Failed to delete metadata: {}", e)))?;
    }

    info!("Bounce deleted for commit {}", commit_id);
    Ok(HttpResponse::NoContent().finish())
}
