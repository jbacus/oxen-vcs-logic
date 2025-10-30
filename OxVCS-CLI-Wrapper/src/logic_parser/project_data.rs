// OxVCS-CLI-Wrapper/src/logic_parser/project_data.rs
//
// Data structures representing Logic Pro project metadata.
// These structures are populated by parsing the .logicx binary format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Complete Logic Pro project metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogicProjectData {
    /// Project tempo in BPM
    pub tempo: f32,

    /// Sample rate in Hz (e.g., 44100, 48000)
    pub sample_rate: u32,

    /// Key signature (e.g., "C Major", "A Minor")
    pub key_signature: String,

    /// Time signature (numerator, denominator)
    pub time_signature: (u8, u8),

    /// Bit depth (16, 24, 32)
    pub bit_depth: u8,

    /// All tracks in the project
    pub tracks: Vec<Track>,

    /// Automation curves
    pub automation: Vec<AutomationCurve>,

    /// Plugin instances
    pub plugins: Vec<PluginInstance>,

    /// Logic Pro version that created this project
    pub logic_version: String,
}

impl LogicProjectData {
    /// Find a track by its unique ID
    pub fn find_track(&self, track_id: &str) -> Option<&Track> {
        self.tracks.iter().find(|t| t.id == track_id)
    }

    /// Check if a track exists by ID
    pub fn has_track(&self, track_id: &str) -> bool {
        self.tracks.iter().any(|t| t.id == track_id)
    }

    /// Get tracks by type
    pub fn tracks_by_type(&self, track_type: TrackType) -> Vec<&Track> {
        self.tracks.iter().filter(|t| t.track_type == track_type).collect()
    }
}

/// A single track in the project
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Track {
    /// Unique identifier (persists across versions)
    pub id: String,

    /// User-visible track name
    pub name: String,

    /// Type of track
    pub track_type: TrackType,

    /// Track order in project (0-indexed)
    pub track_number: usize,

    /// Channel strip (effects, volume, pan)
    pub channel_strip: ChannelStrip,

    /// Regions on this track
    pub regions: Vec<Region>,

    /// Track color (RGB)
    pub color: Option<(u8, u8, u8)>,

    /// Whether track is muted
    pub muted: bool,

    /// Whether track is soloed
    pub soloed: bool,
}

/// Type of track
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrackType {
    Audio,
    MIDI,
    Aux,
    Bus,
    Master,
}

/// Channel strip containing effects and routing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChannelStrip {
    /// EQ settings (if enabled)
    pub eq: Option<EQSettings>,

    /// Compressor settings (if enabled)
    pub compressor: Option<CompressorSettings>,

    /// Reverb settings (if enabled)
    pub reverb: Option<ReverbSettings>,

    /// Other plugins in order
    pub plugin_chain: Vec<PluginInstance>,

    /// Volume in dB
    pub volume: f32,

    /// Pan (-1.0 = full left, 0.0 = center, 1.0 = full right)
    pub pan: f32,
}

impl Default for ChannelStrip {
    fn default() -> Self {
        Self {
            eq: None,
            compressor: None,
            reverb: None,
            plugin_chain: Vec::new(),
            volume: 0.0,
            pan: 0.0,
        }
    }
}

/// EQ settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EQSettings {
    /// Whether EQ is bypassed
    pub bypassed: bool,

    /// EQ bands
    pub bands: Vec<EQBand>,
}

/// A single EQ band
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EQBand {
    /// Band type
    pub band_type: EQBandType,

    /// Frequency in Hz
    pub frequency: f32,

    /// Gain in dB
    pub gain: f32,

    /// Q factor (bandwidth)
    pub q: f32,

    /// Whether this band is enabled
    pub enabled: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EQBandType {
    LowCut,
    LowShelf,
    Parametric,
    HighShelf,
    HighCut,
}

/// Compressor settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressorSettings {
    /// Whether compressor is bypassed
    pub bypassed: bool,

    /// Threshold in dB
    pub threshold: f32,

    /// Ratio (e.g., 4.0 for 4:1)
    pub ratio: f32,

    /// Attack time in milliseconds
    pub attack: f32,

    /// Release time in milliseconds
    pub release: f32,

    /// Knee width (0 = hard knee)
    pub knee: f32,

    /// Makeup gain in dB
    pub makeup_gain: f32,
}

/// Reverb settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReverbSettings {
    /// Whether reverb is bypassed
    pub bypassed: bool,

    /// Reverb type/preset name
    pub preset: String,

    /// Decay time in seconds
    pub decay_time: f32,

    /// Pre-delay in milliseconds
    pub pre_delay: f32,

    /// Dry/wet mix (0.0 = fully dry, 1.0 = fully wet)
    pub mix: f32,
}

/// A plugin instance
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PluginInstance {
    /// Unique ID
    pub id: String,

    /// Plugin name (e.g., "Channel EQ", "Compressor")
    pub name: String,

    /// Plugin type (AU, VST, etc.)
    pub plugin_type: String,

    /// Track this plugin belongs to
    pub track_id: String,

    /// Position in plugin chain
    pub chain_position: usize,

    /// Whether plugin is bypassed
    pub bypassed: bool,

    /// Plugin parameters (name -> value)
    pub parameters: HashMap<String, f32>,
}

/// A region on a track
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Region {
    /// Region name
    pub name: String,

    /// Start time in seconds
    pub start_time: f64,

    /// End time in seconds
    pub end_time: f64,

    /// Region type
    pub region_type: RegionType,

    /// Whether region is muted
    pub muted: bool,

    /// Loop enabled
    pub looped: bool,

    /// Fade in/out
    pub fade_in: f32,
    pub fade_out: f32,
}

impl Region {
    pub fn duration(&self) -> f64 {
        self.end_time - self.start_time
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegionType {
    Audio,
    MIDI,
}

/// Automation curve
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationCurve {
    /// Track this automation belongs to
    pub track_id: String,

    /// Parameter being automated (e.g., "Volume", "Pan")
    pub parameter: String,

    /// Automation points (time in seconds, value)
    pub points: Vec<AutomationPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutomationPoint {
    pub time: f64,
    pub value: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_track_lookup() {
        let project = LogicProjectData {
            tempo: 120.0,
            sample_rate: 48000,
            key_signature: "C Major".to_string(),
            time_signature: (4, 4),
            bit_depth: 24,
            tracks: vec![
                Track {
                    id: "track1".to_string(),
                    name: "Lead Synth".to_string(),
                    track_type: TrackType::MIDI,
                    track_number: 0,
                    channel_strip: ChannelStrip::default(),
                    regions: vec![],
                    color: Some((255, 0, 0)),
                    muted: false,
                    soloed: false,
                },
            ],
            automation: vec![],
            plugins: vec![],
            logic_version: "11.0.0".to_string(),
        };

        assert!(project.has_track("track1"));
        assert!(!project.has_track("track2"));
        assert_eq!(project.find_track("track1").unwrap().name, "Lead Synth");
    }

    #[test]
    fn test_region_duration() {
        let region = Region {
            name: "Verse".to_string(),
            start_time: 10.0,
            end_time: 25.5,
            region_type: RegionType::Audio,
            muted: false,
            looped: false,
            fade_in: 0.0,
            fade_out: 0.0,
        };

        assert_eq!(region.duration(), 15.5);
    }
}
