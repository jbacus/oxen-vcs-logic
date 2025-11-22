use serde::{Deserialize, Serialize};

/// Structured metadata for Blender project commits.
///
/// Enhances standard commit messages with Blender-specific metadata including
/// scene information, object counts, render settings, and animation data.
/// This enables rich searching, filtering, and context when browsing project history.
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
/// Scenes: <scene_count>
/// Objects: <object_count>
/// Materials: <material_count>
/// Render Engine: <engine>
/// Resolution: <width>x<height>
/// Blender Version: <version>
/// Tags: <tag1>, <tag2>, ...
/// ```
///
/// # Examples
///
/// ```
/// use auxin::BlenderMetadata;
///
/// // Create milestone commit with full metadata
/// let commit = BlenderMetadata::new("Character rigging complete")
///     .with_scene_count(3)
///     .with_object_count(1247)
///     .with_material_count(45)
///     .with_render_engine("CYCLES")
///     .with_resolution(1920, 1080)
///     .with_tag("milestone")
///     .with_tag("rigging");
///
/// let formatted = commit.format_commit_message();
/// assert!(formatted.contains("Render Engine: CYCLES"));
/// assert!(formatted.contains("Objects: 1247"));
///
/// // Parse it back
/// let parsed = BlenderMetadata::parse_commit_message(&formatted);
/// assert_eq!(parsed.render_engine, Some("CYCLES".to_string()));
/// ```
///
/// # Serialization
///
/// Supports JSON serialization via Serde for storage and IPC.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderMetadata {
    /// User-provided commit message (primary description)
    pub message: String,

    /// Number of scenes in the file
    pub scene_count: Option<u32>,

    /// Active scene name
    pub active_scene: Option<String>,

    /// Number of mesh objects
    pub mesh_count: Option<u32>,

    /// Number of light objects
    pub light_count: Option<u32>,

    /// Number of camera objects
    pub camera_count: Option<u32>,

    /// Number of materials
    pub material_count: Option<u32>,

    /// Total number of objects (all types)
    pub object_count: Option<u32>,

    /// Render engine (e.g., "CYCLES", "EEVEE", "WORKBENCH")
    pub render_engine: Option<String>,

    /// Render resolution (width, height)
    pub resolution: Option<(u32, u32)>,

    /// Render samples (for Cycles/Eevee)
    pub samples: Option<u32>,

    /// Animation frame start
    pub frame_start: Option<u32>,

    /// Animation frame end
    pub frame_end: Option<u32>,

    /// Frames per second
    pub fps: Option<u32>,

    /// Blender version that created the file
    pub blender_version: Option<String>,

    /// File size in bytes (useful for tracking bloat)
    pub file_size_bytes: Option<u64>,

    /// Optional tags for categorization (e.g., "modeling", "animation", "rendering")
    pub tags: Vec<String>,

    /// Unix timestamp (auto-set by daemon, not user-provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
}

impl BlenderMetadata {
    /// Creates a new BlenderMetadata with just a message.
    ///
    /// This is the primary constructor. Use builder methods to add optional metadata.
    ///
    /// # Arguments
    ///
    /// * `message` - Commit message (can be String, &str, or any Into<String>)
    ///
    /// # Returns
    ///
    /// BlenderMetadata with all optional fields set to None/empty
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            scene_count: None,
            active_scene: None,
            mesh_count: None,
            light_count: None,
            camera_count: None,
            material_count: None,
            object_count: None,
            render_engine: None,
            resolution: None,
            samples: None,
            frame_start: None,
            frame_end: None,
            fps: None,
            blender_version: None,
            file_size_bytes: None,
            tags: Vec::new(),
            timestamp: None,
        }
    }

    /// Sets the scene count.
    pub fn with_scene_count(mut self, scene_count: u32) -> Self {
        self.scene_count = Some(scene_count);
        self
    }

    /// Sets the active scene name.
    pub fn with_active_scene(mut self, active_scene: impl Into<String>) -> Self {
        self.active_scene = Some(active_scene.into());
        self
    }

    /// Sets the mesh count.
    pub fn with_mesh_count(mut self, mesh_count: u32) -> Self {
        self.mesh_count = Some(mesh_count);
        self
    }

    /// Sets the light count.
    pub fn with_light_count(mut self, light_count: u32) -> Self {
        self.light_count = Some(light_count);
        self
    }

    /// Sets the camera count.
    pub fn with_camera_count(mut self, camera_count: u32) -> Self {
        self.camera_count = Some(camera_count);
        self
    }

    /// Sets the material count.
    pub fn with_material_count(mut self, material_count: u32) -> Self {
        self.material_count = Some(material_count);
        self
    }

    /// Sets the total object count.
    pub fn with_object_count(mut self, object_count: u32) -> Self {
        self.object_count = Some(object_count);
        self
    }

    /// Sets the render engine.
    pub fn with_render_engine(mut self, render_engine: impl Into<String>) -> Self {
        self.render_engine = Some(render_engine.into());
        self
    }

    /// Sets the render resolution.
    pub fn with_resolution(mut self, width: u32, height: u32) -> Self {
        self.resolution = Some((width, height));
        self
    }

    /// Sets the render samples.
    pub fn with_samples(mut self, samples: u32) -> Self {
        self.samples = Some(samples);
        self
    }

    /// Sets the animation frame range.
    pub fn with_frame_range(mut self, start: u32, end: u32) -> Self {
        self.frame_start = Some(start);
        self.frame_end = Some(end);
        self
    }

    /// Sets the frames per second.
    pub fn with_fps(mut self, fps: u32) -> Self {
        self.fps = Some(fps);
        self
    }

    /// Sets the Blender version.
    pub fn with_blender_version(mut self, version: impl Into<String>) -> Self {
        self.blender_version = Some(version.into());
        self
    }

    /// Sets the file size.
    pub fn with_file_size(mut self, file_size_bytes: u64) -> Self {
        self.file_size_bytes = Some(file_size_bytes);
        self
    }

    /// Adds a tag for categorization.
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Formats the metadata as a structured commit message for version control.
    ///
    /// Generates a multi-line string with the message followed by metadata fields.
    /// Only includes fields that have been set (omits None values).
    pub fn format_commit_message(&self) -> String {
        let mut msg = self.message.clone();

        let mut metadata_lines = Vec::new();

        if let Some(count) = self.scene_count {
            metadata_lines.push(format!("Scenes: {}", count));
        }

        if let Some(ref scene) = self.active_scene {
            metadata_lines.push(format!("Active Scene: {}", scene));
        }

        if let Some(count) = self.object_count {
            metadata_lines.push(format!("Objects: {}", count));
        }

        if let Some(count) = self.mesh_count {
            metadata_lines.push(format!("Meshes: {}", count));
        }

        if let Some(count) = self.light_count {
            metadata_lines.push(format!("Lights: {}", count));
        }

        if let Some(count) = self.camera_count {
            metadata_lines.push(format!("Cameras: {}", count));
        }

        if let Some(count) = self.material_count {
            metadata_lines.push(format!("Materials: {}", count));
        }

        if let Some(ref engine) = self.render_engine {
            metadata_lines.push(format!("Render Engine: {}", engine));
        }

        if let Some((width, height)) = self.resolution {
            metadata_lines.push(format!("Resolution: {}x{}", width, height));
        }

        if let Some(samples) = self.samples {
            metadata_lines.push(format!("Samples: {}", samples));
        }

        if let (Some(start), Some(end)) = (self.frame_start, self.frame_end) {
            metadata_lines.push(format!("Frame Range: {}-{}", start, end));
        }

        if let Some(fps) = self.fps {
            metadata_lines.push(format!("FPS: {}", fps));
        }

        if let Some(ref version) = self.blender_version {
            metadata_lines.push(format!("Blender Version: {}", version));
        }

        if let Some(size) = self.file_size_bytes {
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
    pub fn parse_commit_message(message: &str) -> Self {
        let lines: Vec<&str> = message.lines().collect();

        let mut metadata = BlenderMetadata::new("");
        let mut main_message = String::new();
        let mut in_metadata = false;

        for line in lines {
            if line.starts_with("Scenes:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Scenes:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.scene_count = Some(count);
                    }
                }
            } else if line.starts_with("Active Scene:") {
                in_metadata = true;
                if let Some(scene) = line.strip_prefix("Active Scene:") {
                    metadata.active_scene = Some(scene.trim().to_string());
                }
            } else if line.starts_with("Objects:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Objects:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.object_count = Some(count);
                    }
                }
            } else if line.starts_with("Meshes:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Meshes:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.mesh_count = Some(count);
                    }
                }
            } else if line.starts_with("Lights:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Lights:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.light_count = Some(count);
                    }
                }
            } else if line.starts_with("Cameras:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Cameras:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.camera_count = Some(count);
                    }
                }
            } else if line.starts_with("Materials:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Materials:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.material_count = Some(count);
                    }
                }
            } else if line.starts_with("Render Engine:") {
                in_metadata = true;
                if let Some(engine) = line.strip_prefix("Render Engine:") {
                    metadata.render_engine = Some(engine.trim().to_string());
                }
            } else if line.starts_with("Resolution:") {
                in_metadata = true;
                if let Some(res_str) = line.strip_prefix("Resolution:") {
                    let parts: Vec<&str> = res_str.trim().split('x').collect();
                    if parts.len() == 2 {
                        if let (Ok(width), Ok(height)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                        {
                            metadata.resolution = Some((width, height));
                        }
                    }
                }
            } else if line.starts_with("Samples:") {
                in_metadata = true;
                if let Some(count_str) = line.strip_prefix("Samples:") {
                    if let Ok(count) = count_str.trim().parse::<u32>() {
                        metadata.samples = Some(count);
                    }
                }
            } else if line.starts_with("Frame Range:") {
                in_metadata = true;
                if let Some(range_str) = line.strip_prefix("Frame Range:") {
                    let parts: Vec<&str> = range_str.trim().split('-').collect();
                    if parts.len() == 2 {
                        if let (Ok(start), Ok(end)) =
                            (parts[0].parse::<u32>(), parts[1].parse::<u32>())
                        {
                            metadata.frame_start = Some(start);
                            metadata.frame_end = Some(end);
                        }
                    }
                }
            } else if line.starts_with("FPS:") {
                in_metadata = true;
                if let Some(fps_str) = line.strip_prefix("FPS:") {
                    if let Ok(fps) = fps_str.trim().parse::<u32>() {
                        metadata.fps = Some(fps);
                    }
                }
            } else if line.starts_with("Blender Version:") {
                in_metadata = true;
                if let Some(version) = line.strip_prefix("Blender Version:") {
                    metadata.blender_version = Some(version.trim().to_string());
                }
            } else if line.starts_with("File Size:") {
                in_metadata = true;
                if let Some(size_str) = line.strip_prefix("File Size:") {
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
        let metadata = BlenderMetadata::new("Test commit");
        assert_eq!(metadata.message, "Test commit");
        assert_eq!(metadata.scene_count, None);
        assert_eq!(metadata.object_count, None);
        assert!(metadata.tags.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let metadata = BlenderMetadata::new("Test")
            .with_scene_count(3)
            .with_object_count(1247)
            .with_material_count(45)
            .with_render_engine("CYCLES")
            .with_resolution(1920, 1080)
            .with_tag("modeling")
            .with_tag("final");

        assert_eq!(metadata.scene_count, Some(3));
        assert_eq!(metadata.object_count, Some(1247));
        assert_eq!(metadata.material_count, Some(45));
        assert_eq!(metadata.render_engine, Some("CYCLES".to_string()));
        assert_eq!(metadata.resolution, Some((1920, 1080)));
        assert_eq!(metadata.tags.len(), 2);
    }

    #[test]
    fn test_format_commit_message_complete() {
        let metadata = BlenderMetadata::new("Character rig complete")
            .with_scene_count(3)
            .with_object_count(1247)
            .with_material_count(45)
            .with_render_engine("CYCLES")
            .with_resolution(1920, 1080);

        let formatted = metadata.format_commit_message();

        assert!(formatted.contains("Character rig complete"));
        assert!(formatted.contains("Scenes: 3"));
        assert!(formatted.contains("Objects: 1247"));
        assert!(formatted.contains("Materials: 45"));
        assert!(formatted.contains("Render Engine: CYCLES"));
        assert!(formatted.contains("Resolution: 1920x1080"));
    }

    #[test]
    fn test_format_commit_message_no_metadata() {
        let metadata = BlenderMetadata::new("Simple commit");
        let formatted = metadata.format_commit_message();

        assert_eq!(formatted, "Simple commit");
        assert!(!formatted.contains("\n\n"));
    }

    #[test]
    fn test_parse_commit_message_complete() {
        let msg = "Final render\n\nScenes: 3\nObjects: 1247\nMaterials: 45\nRender Engine: CYCLES\nResolution: 1920x1080";
        let metadata = BlenderMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Final render");
        assert_eq!(metadata.scene_count, Some(3));
        assert_eq!(metadata.object_count, Some(1247));
        assert_eq!(metadata.material_count, Some(45));
        assert_eq!(metadata.render_engine, Some("CYCLES".to_string()));
        assert_eq!(metadata.resolution, Some((1920, 1080)));
    }

    #[test]
    fn test_parse_commit_message_no_metadata() {
        let msg = "Just a message";
        let metadata = BlenderMetadata::parse_commit_message(msg);

        assert_eq!(metadata.message, "Just a message");
        assert_eq!(metadata.scene_count, None);
        assert_eq!(metadata.object_count, None);
    }

    #[test]
    fn test_round_trip() {
        let original = BlenderMetadata::new("Round trip test")
            .with_scene_count(2)
            .with_object_count(500)
            .with_render_engine("EEVEE")
            .with_tag("test");

        let formatted = original.format_commit_message();
        let parsed = BlenderMetadata::parse_commit_message(&formatted);

        assert_eq!(parsed.message, original.message);
        assert_eq!(parsed.scene_count, original.scene_count);
        assert_eq!(parsed.object_count, original.object_count);
        assert_eq!(parsed.render_engine, original.render_engine);
        assert_eq!(parsed.tags, original.tags);
    }

    #[test]
    fn test_file_size_formatting() {
        let metadata = BlenderMetadata::new("Large file").with_file_size(104857600); // 100 MB

        let formatted = metadata.format_commit_message();
        assert!(formatted.contains("File Size: 100.00 MB"));
    }

    #[test]
    fn test_frame_range() {
        let metadata = BlenderMetadata::new("Animation")
            .with_frame_range(1, 250)
            .with_fps(24);

        let formatted = metadata.format_commit_message();
        assert!(formatted.contains("Frame Range: 1-250"));
        assert!(formatted.contains("FPS: 24"));
    }

    #[test]
    fn test_serde_serialization() {
        let metadata = BlenderMetadata::new("Test")
            .with_object_count(100)
            .with_render_engine("CYCLES");

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(json.contains("\"message\":\"Test\""));
        assert!(json.contains("\"render_engine\":\"CYCLES\""));
    }
}
