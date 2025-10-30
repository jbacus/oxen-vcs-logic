// OxVCS-CLI-Wrapper/src/metadata_diff/diff_types.rs
//
// Types representing differences between Logic Pro project versions

use crate::logic_parser::*;
use serde::{Deserialize, Serialize};

/// Complete metadata diff between two project versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataDiff {
    /// Global project-level changes
    pub global_changes: Vec<GlobalChange>,

    /// Track-level changes
    pub track_changes: Vec<TrackChange>,

    /// Plugin-specific changes
    pub plugin_changes: Vec<PluginChange>,

    /// Automation changes
    pub automation_changes: Vec<AutomationChange>,
}

impl MetadataDiff {
    pub fn new() -> Self {
        Self {
            global_changes: Vec::new(),
            track_changes: Vec::new(),
            plugin_changes: Vec::new(),
            automation_changes: Vec::new(),
        }
    }

    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.global_changes.is_empty()
            || !self.track_changes.is_empty()
            || !self.plugin_changes.is_empty()
            || !self.automation_changes.is_empty()
    }

    /// Count total number of changes
    pub fn change_count(&self) -> usize {
        self.global_changes.len()
            + self.track_changes.len()
            + self.plugin_changes.len()
            + self.automation_changes.len()
    }
}

impl Default for MetadataDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Global project setting changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GlobalChange {
    TempoChange { from: f32, to: f32 },
    SampleRateChange { from: u32, to: u32 },
    KeySignatureChange { from: String, to: String },
    TimeSignatureChange { from: (u8, u8), to: (u8, u8) },
    BitDepthChange { from: u8, to: u8 },
}

/// Track-level changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrackChange {
    Added {
        track: Track,
    },
    Removed {
        track_name: String,
        track_id: String,
    },
    Renamed {
        track_id: String,
        old_name: String,
        new_name: String,
    },
    Reordered {
        track_name: String,
        old_position: usize,
        new_position: usize,
    },
    TypeChanged {
        track_name: String,
        old_type: TrackType,
        new_type: TrackType,
    },
    ChannelStripChanged {
        track_name: String,
        track_id: String,
        changes: ChannelStripDiff,
    },
    RegionChanged {
        track_name: String,
        region_diff: RegionDiff,
    },
    MuteChanged {
        track_name: String,
        muted: bool,
    },
    SoloChanged {
        track_name: String,
        soloed: bool,
    },
    ColorChanged {
        track_name: String,
        old_color: Option<(u8, u8, u8)>,
        new_color: Option<(u8, u8, u8)>,
    },
}

/// Channel strip differences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStripDiff {
    pub eq_changes: Vec<EQChange>,
    pub compressor_changes: Vec<CompressorChange>,
    pub reverb_changes: Vec<ReverbChange>,
    pub volume_delta: Option<f32>,
    pub pan_delta: Option<f32>,
    pub plugin_chain_changes: Vec<PluginChainChange>,
}

impl ChannelStripDiff {
    pub fn new() -> Self {
        Self {
            eq_changes: Vec::new(),
            compressor_changes: Vec::new(),
            reverb_changes: Vec::new(),
            volume_delta: None,
            pan_delta: None,
            plugin_chain_changes: Vec::new(),
        }
    }

    pub fn has_changes(&self) -> bool {
        !self.eq_changes.is_empty()
            || !self.compressor_changes.is_empty()
            || !self.reverb_changes.is_empty()
            || self.volume_delta.is_some()
            || self.pan_delta.is_some()
            || !self.plugin_chain_changes.is_empty()
    }
}

impl Default for ChannelStripDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// EQ changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EQChange {
    BandAdded { band: EQBand, position: usize },
    BandRemoved { band: EQBand, position: usize },
    BandFrequencyChanged { position: usize, from: f32, to: f32 },
    BandGainChanged { position: usize, from: f32, to: f32 },
    BandQChanged { position: usize, from: f32, to: f32 },
    BandTypeChanged { position: usize, from: EQBandType, to: EQBandType },
    BandToggled { position: usize, enabled: bool },
    BypassToggled { bypassed: bool },
}

/// Compressor changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressorChange {
    ThresholdChanged { from: f32, to: f32 },
    RatioChanged { from: f32, to: f32 },
    AttackChanged { from: f32, to: f32 },
    ReleaseChanged { from: f32, to: f32 },
    KneeChanged { from: f32, to: f32 },
    MakeupGainChanged { from: f32, to: f32 },
    BypassToggled { bypassed: bool },
}

/// Reverb changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReverbChange {
    PresetChanged { from: String, to: String },
    DecayTimeChanged { from: f32, to: f32 },
    PreDelayChanged { from: f32, to: f32 },
    MixChanged { from: f32, to: f32 },
    BypassToggled { bypassed: bool },
}

/// Plugin chain changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginChainChange {
    PluginAdded { plugin: PluginInstance },
    PluginRemoved { plugin_name: String, position: usize },
    PluginReordered { plugin_name: String, from: usize, to: usize },
    PluginBypassed { plugin_name: String, bypassed: bool },
}

/// Plugin-specific changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginChange {
    pub plugin_name: String,
    pub track_name: String,
    pub parameter_changes: Vec<ParameterChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterChange {
    pub parameter_name: String,
    pub old_value: f32,
    pub new_value: f32,
}

/// Region changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegionDiff {
    Added { region: Region },
    Removed { region_name: String },
    Moved { region_name: String, old_start: f64, new_start: f64 },
    Resized { region_name: String, old_duration: f64, new_duration: f64 },
    MuteToggled { region_name: String, muted: bool },
    LoopToggled { region_name: String, looped: bool },
    FadeChanged { region_name: String, fade_type: FadeType, old_value: f32, new_value: f32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FadeType {
    FadeIn,
    FadeOut,
}

/// Automation changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationChange {
    Added {
        track_name: String,
        parameter: String,
        point_count: usize,
    },
    Removed {
        track_name: String,
        parameter: String,
    },
    Modified {
        track_name: String,
        parameter: String,
        significant_changes: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_diff_empty() {
        let diff = MetadataDiff::new();
        assert!(!diff.has_changes());
        assert_eq!(diff.change_count(), 0);
    }

    #[test]
    fn test_metadata_diff_with_changes() {
        let mut diff = MetadataDiff::new();
        diff.global_changes.push(GlobalChange::TempoChange {
            from: 120.0,
            to: 128.0,
        });

        assert!(diff.has_changes());
        assert_eq!(diff.change_count(), 1);
    }

    #[test]
    fn test_channel_strip_diff_empty() {
        let diff = ChannelStripDiff::new();
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_channel_strip_diff_with_volume() {
        let mut diff = ChannelStripDiff::new();
        diff.volume_delta = Some(2.5);

        assert!(diff.has_changes());
    }
}
