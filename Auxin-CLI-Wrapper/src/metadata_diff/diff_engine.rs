// Auxin-CLI-Wrapper/src/metadata_diff/diff_engine.rs
//
// Core diff engine for comparing Logic Pro project metadata

use crate::logic_parser::*;
use super::diff_types::*;

/// Threshold for considering a float value as "changed" (in dB for volume)
const VOLUME_THRESHOLD: f32 = 0.1;

/// Threshold for considering pan as "changed" (0.05 = 5%)
const PAN_THRESHOLD: f32 = 0.05;

/// Threshold for EQ gain changes (in dB)
const EQ_GAIN_THRESHOLD: f32 = 0.5;

/// Threshold for frequency changes (in Hz)
const FREQUENCY_THRESHOLD: f32 = 10.0;

/// Compare two Logic Pro project versions and generate a diff
pub fn diff_metadata(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
) -> MetadataDiff {
    let mut diff = MetadataDiff::new();

    // Compare global settings
    diff_global_settings(version_a, version_b, &mut diff);

    // Compare tracks
    diff_tracks(version_a, version_b, &mut diff);

    // Compare plugins (beyond channel strip)
    diff_plugins(version_a, version_b, &mut diff);

    // Compare automation
    diff_automation(version_a, version_b, &mut diff);

    diff
}

fn diff_global_settings(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
    diff: &mut MetadataDiff,
) {
    // Tempo
    if (version_a.tempo - version_b.tempo).abs() > 0.1 {
        diff.global_changes.push(GlobalChange::TempoChange {
            from: version_a.tempo,
            to: version_b.tempo,
        });
    }

    // Sample rate
    if version_a.sample_rate != version_b.sample_rate {
        diff.global_changes.push(GlobalChange::SampleRateChange {
            from: version_a.sample_rate,
            to: version_b.sample_rate,
        });
    }

    // Key signature
    if version_a.key_signature != version_b.key_signature {
        diff.global_changes.push(GlobalChange::KeySignatureChange {
            from: version_a.key_signature.clone(),
            to: version_b.key_signature.clone(),
        });
    }

    // Time signature
    if version_a.time_signature != version_b.time_signature {
        diff.global_changes.push(GlobalChange::TimeSignatureChange {
            from: version_a.time_signature,
            to: version_b.time_signature,
        });
    }

    // Bit depth
    if version_a.bit_depth != version_b.bit_depth {
        diff.global_changes.push(GlobalChange::BitDepthChange {
            from: version_a.bit_depth,
            to: version_b.bit_depth,
        });
    }
}

fn diff_tracks(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
    diff: &mut MetadataDiff,
) {
    // Check for new tracks in B
    for track_b in &version_b.tracks {
        if !version_a.has_track(&track_b.id) {
            diff.track_changes.push(TrackChange::Added {
                track: track_b.clone(),
            });
        }
    }

    // Check for removed tracks (in A but not B)
    for track_a in &version_a.tracks {
        if !version_b.has_track(&track_a.id) {
            diff.track_changes.push(TrackChange::Removed {
                track_name: track_a.name.clone(),
                track_id: track_a.id.clone(),
            });
        }
    }

    // Check for modified tracks (in both A and B)
    for track_b in &version_b.tracks {
        if let Some(track_a) = version_a.find_track(&track_b.id) {
            diff_track_changes(track_a, track_b, diff);
        }
    }
}

fn diff_track_changes(track_a: &Track, track_b: &Track, diff: &mut MetadataDiff) {
    // Name change
    if track_a.name != track_b.name {
        diff.track_changes.push(TrackChange::Renamed {
            track_id: track_b.id.clone(),
            old_name: track_a.name.clone(),
            new_name: track_b.name.clone(),
        });
    }

    // Track number (reordering)
    if track_a.track_number != track_b.track_number {
        diff.track_changes.push(TrackChange::Reordered {
            track_name: track_b.name.clone(),
            old_position: track_a.track_number,
            new_position: track_b.track_number,
        });
    }

    // Track type
    if track_a.track_type != track_b.track_type {
        diff.track_changes.push(TrackChange::TypeChanged {
            track_name: track_b.name.clone(),
            old_type: track_a.track_type,
            new_type: track_b.track_type,
        });
    }

    // Mute state
    if track_a.muted != track_b.muted {
        diff.track_changes.push(TrackChange::MuteChanged {
            track_name: track_b.name.clone(),
            muted: track_b.muted,
        });
    }

    // Solo state
    if track_a.soloed != track_b.soloed {
        diff.track_changes.push(TrackChange::SoloChanged {
            track_name: track_b.name.clone(),
            soloed: track_b.soloed,
        });
    }

    // Color
    if track_a.color != track_b.color {
        diff.track_changes.push(TrackChange::ColorChanged {
            track_name: track_b.name.clone(),
            old_color: track_a.color,
            new_color: track_b.color,
        });
    }

    // Channel strip changes
    if let Some(cs_diff) = diff_channel_strip(&track_a.channel_strip, &track_b.channel_strip) {
        diff.track_changes.push(TrackChange::ChannelStripChanged {
            track_name: track_b.name.clone(),
            track_id: track_b.id.clone(),
            changes: cs_diff,
        });
    }

    // Region changes
    diff_regions(&track_a.regions, &track_b.regions, &track_b.name, diff);
}

pub fn diff_channel_strip(
    cs_a: &ChannelStrip,
    cs_b: &ChannelStrip,
) -> Option<ChannelStripDiff> {
    let mut changes = ChannelStripDiff::new();

    // EQ diff
    if let Some(eq_changes) = diff_eq(cs_a.eq.as_ref(), cs_b.eq.as_ref()) {
        changes.eq_changes = eq_changes;
    }

    // Compressor diff
    if let Some(comp_changes) = diff_compressor(cs_a.compressor.as_ref(), cs_b.compressor.as_ref()) {
        changes.compressor_changes = comp_changes;
    }

    // Reverb diff
    if let Some(reverb_changes) = diff_reverb(cs_a.reverb.as_ref(), cs_b.reverb.as_ref()) {
        changes.reverb_changes = reverb_changes;
    }

    // Volume diff
    let volume_delta = cs_b.volume - cs_a.volume;
    if volume_delta.abs() > VOLUME_THRESHOLD {
        changes.volume_delta = Some(volume_delta);
    }

    // Pan diff
    let pan_delta = cs_b.pan - cs_a.pan;
    if pan_delta.abs() > PAN_THRESHOLD {
        changes.pan_delta = Some(pan_delta);
    }

    // Plugin chain diff
    changes.plugin_chain_changes = diff_plugin_chain(&cs_a.plugin_chain, &cs_b.plugin_chain);

    if changes.has_changes() {
        Some(changes)
    } else {
        None
    }
}

fn diff_eq(eq_a: Option<&EQSettings>, eq_b: Option<&EQSettings>) -> Option<Vec<EQChange>> {
    let mut changes = Vec::new();

    match (eq_a, eq_b) {
        (None, Some(eq)) => {
            // EQ was added
            changes.push(EQChange::BypassToggled { bypassed: false });
            for (i, band) in eq.bands.iter().enumerate() {
                if band.enabled {
                    changes.push(EQChange::BandAdded {
                        band: band.clone(),
                        position: i,
                    });
                }
            }
        }
        (Some(_eq), None) => {
            // EQ was removed
            changes.push(EQChange::BypassToggled { bypassed: true });
        }
        (Some(eq_a), Some(eq_b)) => {
            // EQ exists in both, check for changes

            // Bypass state
            if eq_a.bypassed != eq_b.bypassed {
                changes.push(EQChange::BypassToggled {
                    bypassed: eq_b.bypassed,
                });
            }

            // Compare bands
            for i in 0..eq_a.bands.len().max(eq_b.bands.len()) {
                let band_a = eq_a.bands.get(i);
                let band_b = eq_b.bands.get(i);

                match (band_a, band_b) {
                    (None, Some(band)) => {
                        changes.push(EQChange::BandAdded {
                            band: band.clone(),
                            position: i,
                        });
                    }
                    (Some(band), None) => {
                        changes.push(EQChange::BandRemoved {
                            band: band.clone(),
                            position: i,
                        });
                    }
                    (Some(band_a), Some(band_b)) => {
                        // Check individual parameters

                        // Frequency
                        if (band_a.frequency - band_b.frequency).abs() > FREQUENCY_THRESHOLD {
                            changes.push(EQChange::BandFrequencyChanged {
                                position: i,
                                from: band_a.frequency,
                                to: band_b.frequency,
                            });
                        }

                        // Gain
                        if (band_a.gain - band_b.gain).abs() > EQ_GAIN_THRESHOLD {
                            changes.push(EQChange::BandGainChanged {
                                position: i,
                                from: band_a.gain,
                                to: band_b.gain,
                            });
                        }

                        // Q factor
                        if (band_a.q - band_b.q).abs() > 0.1 {
                            changes.push(EQChange::BandQChanged {
                                position: i,
                                from: band_a.q,
                                to: band_b.q,
                            });
                        }

                        // Band type
                        if band_a.band_type != band_b.band_type {
                            changes.push(EQChange::BandTypeChanged {
                                position: i,
                                from: band_a.band_type,
                                to: band_b.band_type,
                            });
                        }

                        // Enabled state
                        if band_a.enabled != band_b.enabled {
                            changes.push(EQChange::BandToggled {
                                position: i,
                                enabled: band_b.enabled,
                            });
                        }
                    }
                    (None, None) => unreachable!(),
                }
            }
        }
        (None, None) => {}
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_compressor(
    comp_a: Option<&CompressorSettings>,
    comp_b: Option<&CompressorSettings>,
) -> Option<Vec<CompressorChange>> {
    let mut changes = Vec::new();

    match (comp_a, comp_b) {
        (None, Some(_)) => {
            changes.push(CompressorChange::BypassToggled { bypassed: false });
        }
        (Some(_), None) => {
            changes.push(CompressorChange::BypassToggled { bypassed: true });
        }
        (Some(comp_a), Some(comp_b)) => {
            // Bypass state
            if comp_a.bypassed != comp_b.bypassed {
                changes.push(CompressorChange::BypassToggled {
                    bypassed: comp_b.bypassed,
                });
            }

            // Threshold
            if (comp_a.threshold - comp_b.threshold).abs() > 0.5 {
                changes.push(CompressorChange::ThresholdChanged {
                    from: comp_a.threshold,
                    to: comp_b.threshold,
                });
            }

            // Ratio
            if (comp_a.ratio - comp_b.ratio).abs() > 0.1 {
                changes.push(CompressorChange::RatioChanged {
                    from: comp_a.ratio,
                    to: comp_b.ratio,
                });
            }

            // Attack
            if (comp_a.attack - comp_b.attack).abs() > 1.0 {
                changes.push(CompressorChange::AttackChanged {
                    from: comp_a.attack,
                    to: comp_b.attack,
                });
            }

            // Release
            if (comp_a.release - comp_b.release).abs() > 1.0 {
                changes.push(CompressorChange::ReleaseChanged {
                    from: comp_a.release,
                    to: comp_b.release,
                });
            }

            // Knee
            if (comp_a.knee - comp_b.knee).abs() > 0.5 {
                changes.push(CompressorChange::KneeChanged {
                    from: comp_a.knee,
                    to: comp_b.knee,
                });
            }

            // Makeup gain
            if (comp_a.makeup_gain - comp_b.makeup_gain).abs() > 0.5 {
                changes.push(CompressorChange::MakeupGainChanged {
                    from: comp_a.makeup_gain,
                    to: comp_b.makeup_gain,
                });
            }
        }
        (None, None) => {}
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_reverb(
    reverb_a: Option<&ReverbSettings>,
    reverb_b: Option<&ReverbSettings>,
) -> Option<Vec<ReverbChange>> {
    let mut changes = Vec::new();

    match (reverb_a, reverb_b) {
        (None, Some(_)) => {
            changes.push(ReverbChange::BypassToggled { bypassed: false });
        }
        (Some(_), None) => {
            changes.push(ReverbChange::BypassToggled { bypassed: true });
        }
        (Some(rev_a), Some(rev_b)) => {
            // Bypass state
            if rev_a.bypassed != rev_b.bypassed {
                changes.push(ReverbChange::BypassToggled {
                    bypassed: rev_b.bypassed,
                });
            }

            // Preset
            if rev_a.preset != rev_b.preset {
                changes.push(ReverbChange::PresetChanged {
                    from: rev_a.preset.clone(),
                    to: rev_b.preset.clone(),
                });
            }

            // Decay time
            if (rev_a.decay_time - rev_b.decay_time).abs() > 0.1 {
                changes.push(ReverbChange::DecayTimeChanged {
                    from: rev_a.decay_time,
                    to: rev_b.decay_time,
                });
            }

            // Pre-delay
            if (rev_a.pre_delay - rev_b.pre_delay).abs() > 1.0 {
                changes.push(ReverbChange::PreDelayChanged {
                    from: rev_a.pre_delay,
                    to: rev_b.pre_delay,
                });
            }

            // Mix
            if (rev_a.mix - rev_b.mix).abs() > 0.05 {
                changes.push(ReverbChange::MixChanged {
                    from: rev_a.mix,
                    to: rev_b.mix,
                });
            }
        }
        (None, None) => {}
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}

fn diff_plugin_chain(
    chain_a: &[PluginInstance],
    chain_b: &[PluginInstance],
) -> Vec<PluginChainChange> {
    let mut changes = Vec::new();

    // Simple implementation: check for added/removed plugins
    // TODO: Implement more sophisticated matching (by ID, handle reordering)

    for (i, plugin_b) in chain_b.iter().enumerate() {
        if chain_a.len() <= i {
            changes.push(PluginChainChange::PluginAdded {
                plugin: plugin_b.clone(),
            });
        }
    }

    if chain_b.len() < chain_a.len() {
        for (i, plugin) in chain_a.iter().enumerate().skip(chain_b.len()) {
            changes.push(PluginChainChange::PluginRemoved {
                plugin_name: plugin.name.clone(),
                position: i,
            });
        }
    }

    changes
}

fn diff_regions(
    regions_a: &[Region],
    regions_b: &[Region],
    track_name: &str,
    diff: &mut MetadataDiff,
) {
    // Simple implementation: detect added/removed regions
    // TODO: Implement region matching by name and time range

    for region_b in regions_b {
        let found = regions_a.iter().any(|r| {
            r.name == region_b.name && (r.start_time - region_b.start_time).abs() < 0.001
        });

        if !found {
            diff.track_changes.push(TrackChange::RegionChanged {
                track_name: track_name.to_string(),
                region_diff: RegionDiff::Added {
                    region: region_b.clone(),
                },
            });
        }
    }

    for region_a in regions_a {
        let found = regions_b.iter().any(|r| {
            r.name == region_a.name && (r.start_time - region_a.start_time).abs() < 0.001
        });

        if !found {
            diff.track_changes.push(TrackChange::RegionChanged {
                track_name: track_name.to_string(),
                region_diff: RegionDiff::Removed {
                    region_name: region_a.name.clone(),
                },
            });
        }
    }
}

fn diff_plugins(
    _version_a: &LogicProjectData,
    _version_b: &LogicProjectData,
    _diff: &mut MetadataDiff,
) {
    // TODO: Implement plugin parameter diffing
    // This is handled partially in channel strip, but could be extended
}

fn diff_automation(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
    diff: &mut MetadataDiff,
) {
    // Check for new automation curves
    for auto_b in &version_b.automation {
        let found = version_a.automation.iter().any(|a| {
            a.track_id == auto_b.track_id && a.parameter == auto_b.parameter
        });

        if !found {
            diff.automation_changes.push(AutomationChange::Added {
                track_name: auto_b.track_id.clone(), // TODO: Resolve track name
                parameter: auto_b.parameter.clone(),
                point_count: auto_b.points.len(),
            });
        }
    }

    // Check for removed automation curves
    for auto_a in &version_a.automation {
        let found = version_b.automation.iter().any(|a| {
            a.track_id == auto_a.track_id && a.parameter == auto_a.parameter
        });

        if !found {
            diff.automation_changes.push(AutomationChange::Removed {
                track_name: auto_a.track_id.clone(), // TODO: Resolve track name
                parameter: auto_a.parameter.clone(),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_project(tempo: f32) -> LogicProjectData {
        LogicProjectData {
            tempo,
            sample_rate: 48000,
            key_signature: "C Major".to_string(),
            time_signature: (4, 4),
            bit_depth: 24,
            tracks: vec![],
            automation: vec![],
            plugins: vec![],
            logic_version: "11.0.0".to_string(),
        }
    }

    #[test]
    fn test_no_changes() {
        let project_a = create_test_project(120.0);
        let project_b = create_test_project(120.0);

        let diff = diff_metadata(&project_a, &project_b);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_tempo_change() {
        let project_a = create_test_project(120.0);
        let project_b = create_test_project(128.0);

        let diff = diff_metadata(&project_a, &project_b);
        assert!(diff.has_changes());
        assert_eq!(diff.global_changes.len(), 1);

        match &diff.global_changes[0] {
            GlobalChange::TempoChange { from, to } => {
                assert_eq!(*from, 120.0);
                assert_eq!(*to, 128.0);
            }
            _ => panic!("Expected TempoChange"),
        }
    }

    #[test]
    fn test_volume_change() {
        let mut cs_a = ChannelStrip::default();
        cs_a.volume = 0.0;

        let mut cs_b = ChannelStrip::default();
        cs_b.volume = 2.5;

        let diff = diff_channel_strip(&cs_a, &cs_b).unwrap();
        assert_eq!(diff.volume_delta, Some(2.5));
    }

    #[test]
    fn test_volume_no_change_below_threshold() {
        let mut cs_a = ChannelStrip::default();
        cs_a.volume = 0.0;

        let mut cs_b = ChannelStrip::default();
        cs_b.volume = 0.05; // Below threshold

        let diff = diff_channel_strip(&cs_a, &cs_b);
        assert!(diff.is_none());
    }
}
