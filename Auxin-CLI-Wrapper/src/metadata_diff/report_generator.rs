// OxVCS-CLI-Wrapper/src/metadata_diff/report_generator.rs
//
// Generate human-readable reports from metadata diffs

use super::diff_types::*;
use colored::*;

pub struct ReportGenerator {
    /// Use color in output
    use_color: bool,
    /// Include technical details
    verbose: bool,
}

impl ReportGenerator {
    pub fn new() -> Self {
        Self {
            use_color: true,
            verbose: false,
        }
    }

    pub fn with_color(mut self, use_color: bool) -> Self {
        self.use_color = use_color;
        self
    }

    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Generate a formatted text report
    pub fn generate_report(&self, diff: &MetadataDiff) -> String {
        let mut report = String::new();

        // Header
        report.push_str(&self.header("METADATA DIFF REPORT"));
        report.push('\n');

        if !diff.has_changes() {
            report.push_str(&self.info("No changes detected"));
            return report;
        }

        // Summary
        report.push_str(&self.generate_summary(diff));
        report.push('\n');

        // Global changes
        if !diff.global_changes.is_empty() {
            report.push_str(&self.section_header("GLOBAL CHANGES"));
            for change in &diff.global_changes {
                report.push_str(&self.format_global_change(change));
            }
            report.push('\n');
        }

        // Track changes
        if !diff.track_changes.is_empty() {
            report.push_str(&self.section_header("TRACK CHANGES"));
            for change in &diff.track_changes {
                report.push_str(&self.format_track_change(change));
            }
            report.push('\n');
        }

        // Automation changes
        if !diff.automation_changes.is_empty() {
            report.push_str(&self.section_header("AUTOMATION CHANGES"));
            for change in &diff.automation_changes {
                report.push_str(&self.format_automation_change(change));
            }
            report.push('\n');
        }

        // Plugin changes
        if !diff.plugin_changes.is_empty() {
            report.push_str(&self.section_header("PLUGIN CHANGES"));
            for change in &diff.plugin_changes {
                report.push_str(&self.format_plugin_change(change));
            }
        }

        report
    }

    fn generate_summary(&self, diff: &MetadataDiff) -> String {
        let mut summary = String::new();
        summary.push_str(&self.subsection_header("SUMMARY"));

        summary.push_str(&format!(
            "  {} Total changes: {}\n",
            self.bullet(),
            diff.change_count()
        ));

        if !diff.global_changes.is_empty() {
            summary.push_str(&format!(
                "  {} Global: {}\n",
                self.bullet(),
                diff.global_changes.len()
            ));
        }

        if !diff.track_changes.is_empty() {
            summary.push_str(&format!(
                "  {} Tracks: {}\n",
                self.bullet(),
                diff.track_changes.len()
            ));
        }

        if !diff.automation_changes.is_empty() {
            summary.push_str(&format!(
                "  {} Automation: {}\n",
                self.bullet(),
                diff.automation_changes.len()
            ));
        }

        if !diff.plugin_changes.is_empty() {
            summary.push_str(&format!(
                "  {} Plugins: {}\n",
                self.bullet(),
                diff.plugin_changes.len()
            ));
        }

        summary
    }

    fn format_global_change(&self, change: &GlobalChange) -> String {
        let bullet = self.bullet();
        match change {
            GlobalChange::TempoChange { from, to } => {
                let percent = ((to - from) / from * 100.0).abs();
                format!(
                    "  {} Tempo: {} BPM → {} BPM ({:+.1}%)\n",
                    bullet,
                    from,
                    to,
                    if to > from { percent } else { -percent }
                )
            }
            GlobalChange::SampleRateChange { from, to } => {
                format!("  {} Sample Rate: {} Hz → {} Hz\n", bullet, from, to)
            }
            GlobalChange::KeySignatureChange { from, to } => {
                format!("  {} Key Signature: {} → {}\n", bullet, from, to)
            }
            GlobalChange::TimeSignatureChange { from, to } => {
                format!(
                    "  {} Time Signature: {}/{} → {}/{}\n",
                    bullet, from.0, from.1, to.0, to.1
                )
            }
            GlobalChange::BitDepthChange { from, to } => {
                format!("  {} Bit Depth: {} bit → {} bit\n", bullet, from, to)
            }
        }
    }

    fn format_track_change(&self, change: &TrackChange) -> String {
        match change {
            TrackChange::Added { track } => {
                format!(
                    "{} {} \"{}\" ({:?})\n",
                    self.add_marker(),
                    self.bold("NEW TRACK:"),
                    track.name,
                    track.track_type
                )
            }
            TrackChange::Removed { track_name, .. } => {
                format!(
                    "{} {} \"{}\"\n",
                    self.remove_marker(),
                    self.bold("REMOVED TRACK:"),
                    track_name
                )
            }
            TrackChange::Renamed {
                old_name, new_name, ..
            } => {
                format!(
                    "  {} Track renamed: \"{}\" → \"{}\"\n",
                    self.bullet(),
                    old_name,
                    new_name
                )
            }
            TrackChange::Reordered {
                track_name,
                old_position,
                new_position,
            } => {
                format!(
                    "  {} Track \"{}\" moved: position {} → {}\n",
                    self.bullet(),
                    track_name,
                    old_position + 1,
                    new_position + 1
                )
            }
            TrackChange::ChannelStripChanged {
                track_name, changes, ..
            } => {
                let mut output = format!("\n{} {}:\n", self.bold("Track"), self.bold(track_name));
                output.push_str(&self.format_channel_strip_diff(changes));
                output
            }
            TrackChange::MuteChanged { track_name, muted } => {
                format!(
                    "  {} Track \"{}\" {}\n",
                    self.bullet(),
                    track_name,
                    if *muted { "muted" } else { "unmuted" }
                )
            }
            TrackChange::SoloChanged { track_name, soloed } => {
                format!(
                    "  {} Track \"{}\" {}\n",
                    self.bullet(),
                    track_name,
                    if *soloed { "soloed" } else { "unsoloed" }
                )
            }
            TrackChange::RegionChanged {
                track_name,
                region_diff,
            } => self.format_region_diff(track_name, region_diff),
            TrackChange::TypeChanged {
                track_name,
                old_type,
                new_type,
            } => {
                format!(
                    "  {} Track \"{}\" type changed: {:?} → {:?}\n",
                    self.bullet(),
                    track_name,
                    old_type,
                    new_type
                )
            }
            TrackChange::ColorChanged {
                track_name,
                old_color,
                new_color,
            } => {
                format!(
                    "  {} Track \"{}\" color changed: {:?} → {:?}\n",
                    self.bullet(),
                    track_name,
                    old_color,
                    new_color
                )
            }
        }
    }

    fn format_channel_strip_diff(&self, diff: &ChannelStripDiff) -> String {
        let mut output = String::new();

        // Volume
        if let Some(delta) = diff.volume_delta {
            output.push_str(&format!(
                "    {} Volume: {:+.1} dB\n",
                self.check_mark(),
                delta
            ));
        }

        // Pan
        if let Some(delta) = diff.pan_delta {
            let direction = if delta > 0.0 { "right" } else { "left" };
            output.push_str(&format!(
                "    {} Pan: {:.1}% {}\n",
                self.check_mark(),
                (delta * 100.0).abs(),
                direction
            ));
        }

        // EQ changes
        if !diff.eq_changes.is_empty() {
            output.push_str(&format!("    {} EQ Changes:\n", self.check_mark()));
            for change in &diff.eq_changes {
                output.push_str(&self.format_eq_change(change));
            }
        }

        // Compressor changes
        if !diff.compressor_changes.is_empty() {
            output.push_str(&format!("    {} Compressor Changes:\n", self.check_mark()));
            for change in &diff.compressor_changes {
                output.push_str(&self.format_compressor_change(change));
            }
        }

        // Reverb changes
        if !diff.reverb_changes.is_empty() {
            output.push_str(&format!("    {} Reverb Changes:\n", self.check_mark()));
            for change in &diff.reverb_changes {
                output.push_str(&self.format_reverb_change(change));
            }
        }

        output
    }

    fn format_eq_change(&self, change: &EQChange) -> String {
        match change {
            EQChange::BandAdded { band, position } => {
                format!(
                    "      • Band {} ({:?}): Added at {:.0} Hz, {:+.1} dB gain\n",
                    position + 1,
                    band.band_type,
                    band.frequency,
                    band.gain
                )
            }
            EQChange::BandRemoved { position, .. } => {
                format!("      • Band {} removed\n", position + 1)
            }
            EQChange::BandFrequencyChanged { position, from, to } => {
                format!(
                    "      • Band {} frequency: {:.0} Hz → {:.0} Hz\n",
                    position + 1,
                    from,
                    to
                )
            }
            EQChange::BandGainChanged { position, from, to } => {
                format!(
                    "      • Band {} gain: {:+.1} dB → {:+.1} dB ({:+.1} dB)\n",
                    position + 1,
                    from,
                    to,
                    to - from
                )
            }
            EQChange::BandQChanged { position, from, to } => {
                format!(
                    "      • Band {} Q factor: {:.1} → {:.1}\n",
                    position + 1,
                    from,
                    to
                )
            }
            EQChange::BandTypeChanged {
                position,
                from,
                to,
            } => {
                format!(
                    "      • Band {} type: {:?} → {:?}\n",
                    position + 1,
                    from,
                    to
                )
            }
            EQChange::BandToggled { position, enabled } => {
                format!(
                    "      • Band {} {}\n",
                    position + 1,
                    if *enabled { "enabled" } else { "disabled" }
                )
            }
            EQChange::BypassToggled { bypassed } => {
                format!(
                    "      • EQ {}\n",
                    if *bypassed { "bypassed" } else { "enabled" }
                )
            }
        }
    }

    fn format_compressor_change(&self, change: &CompressorChange) -> String {
        match change {
            CompressorChange::ThresholdChanged { from, to } => {
                format!(
                    "      • Threshold: {:.1} dB → {:.1} dB ({:+.1} dB)\n",
                    from,
                    to,
                    to - from
                )
            }
            CompressorChange::RatioChanged { from, to } => {
                format!("      • Ratio: {:.1}:1 → {:.1}:1\n", from, to)
            }
            CompressorChange::AttackChanged { from, to } => {
                format!("      • Attack: {:.1} ms → {:.1} ms\n", from, to)
            }
            CompressorChange::ReleaseChanged { from, to } => {
                format!("      • Release: {:.1} ms → {:.1} ms\n", from, to)
            }
            CompressorChange::KneeChanged { from, to } => {
                format!("      • Knee: {:.1} → {:.1}\n", from, to)
            }
            CompressorChange::MakeupGainChanged { from, to } => {
                format!(
                    "      • Makeup Gain: {:+.1} dB → {:+.1} dB\n",
                    from, to
                )
            }
            CompressorChange::BypassToggled { bypassed } => {
                format!(
                    "      • Compressor {}\n",
                    if *bypassed { "bypassed" } else { "enabled" }
                )
            }
        }
    }

    fn format_reverb_change(&self, change: &ReverbChange) -> String {
        match change {
            ReverbChange::PresetChanged { from, to } => {
                format!("      • Preset: \"{}\" → \"{}\"\n", from, to)
            }
            ReverbChange::DecayTimeChanged { from, to } => {
                format!("      • Decay Time: {:.2} s → {:.2} s\n", from, to)
            }
            ReverbChange::PreDelayChanged { from, to } => {
                format!("      • Pre-Delay: {:.1} ms → {:.1} ms\n", from, to)
            }
            ReverbChange::MixChanged { from, to } => {
                format!(
                    "      • Mix: {:.0}% → {:.0}%\n",
                    from * 100.0,
                    to * 100.0
                )
            }
            ReverbChange::BypassToggled { bypassed } => {
                format!(
                    "      • Reverb {}\n",
                    if *bypassed { "bypassed" } else { "enabled" }
                )
            }
        }
    }

    fn format_region_diff(&self, track_name: &str, diff: &RegionDiff) -> String {
        match diff {
            RegionDiff::Added { region } => {
                format!(
                    "  {} Region \"{}\" added on track \"{}\": {:.3}s - {:.3}s\n",
                    self.bullet(),
                    region.name,
                    track_name,
                    region.start_time,
                    region.end_time
                )
            }
            RegionDiff::Removed { region_name } => {
                format!(
                    "  {} Region \"{}\" removed from track \"{}\"\n",
                    self.bullet(),
                    region_name,
                    track_name
                )
            }
            RegionDiff::Moved {
                region_name,
                old_start,
                new_start,
            } => {
                format!(
                    "  {} Region \"{}\" moved: {:.3}s → {:.3}s\n",
                    self.bullet(),
                    region_name,
                    old_start,
                    new_start
                )
            }
            RegionDiff::Resized {
                region_name,
                old_duration,
                new_duration,
            } => {
                format!(
                    "  {} Region \"{}\" resized: {:.3}s → {:.3}s\n",
                    self.bullet(),
                    region_name,
                    old_duration,
                    new_duration
                )
            }
            _ => format!("  {} Region change on track \"{}\"\n", self.bullet(), track_name),
        }
    }

    fn format_automation_change(&self, change: &AutomationChange) -> String {
        match change {
            AutomationChange::Added {
                track_name,
                parameter,
                point_count,
            } => {
                format!(
                    "  {} New {} automation on track \"{}\" ({} points)\n",
                    self.bullet(),
                    parameter,
                    track_name,
                    point_count
                )
            }
            AutomationChange::Removed {
                track_name,
                parameter,
            } => {
                format!(
                    "  {} Removed {} automation from track \"{}\"\n",
                    self.bullet(),
                    parameter,
                    track_name
                )
            }
            AutomationChange::Modified {
                track_name,
                parameter,
                significant_changes,
            } => {
                format!(
                    "  {} Modified {} automation on track \"{}\" ({} significant changes)\n",
                    self.bullet(),
                    parameter,
                    track_name,
                    significant_changes
                )
            }
        }
    }

    fn format_plugin_change(&self, change: &PluginChange) -> String {
        let mut output = format!(
            "  {} Plugin \"{}\" on track \"{}\":\n",
            self.bullet(),
            change.plugin_name,
            change.track_name
        );

        for param in &change.parameter_changes {
            output.push_str(&format!(
                "    • {}: {:.2} → {:.2}\n",
                param.parameter_name, param.old_value, param.new_value
            ));
        }

        output
    }

    // Formatting helpers
    fn header(&self, text: &str) -> String {
        if self.use_color {
            format!("{}\n{}\n", text.bold().cyan(), "=".repeat(text.len()))
        } else {
            format!("{}\n{}\n", text, "=".repeat(text.len()))
        }
    }

    fn section_header(&self, text: &str) -> String {
        if self.use_color {
            format!("{}\n{}\n", text.bold().yellow(), "-".repeat(text.len()))
        } else {
            format!("{}\n{}\n", text, "-".repeat(text.len()))
        }
    }

    fn subsection_header(&self, text: &str) -> String {
        if self.use_color {
            format!("{}\n", text.bold())
        } else {
            format!("{}\n", text)
        }
    }

    fn bold(&self, text: &str) -> String {
        if self.use_color {
            text.bold().to_string()
        } else {
            text.to_string()
        }
    }

    fn bullet(&self) -> String {
        if self.use_color {
            "•".blue().to_string()
        } else {
            "•".to_string()
        }
    }

    fn check_mark(&self) -> String {
        if self.use_color {
            "✓".green().to_string()
        } else {
            "✓".to_string()
        }
    }

    fn add_marker(&self) -> String {
        if self.use_color {
            "[+]".green().to_string()
        } else {
            "[+]".to_string()
        }
    }

    fn remove_marker(&self) -> String {
        if self.use_color {
            "[-]".red().to_string()
        } else {
            "[-]".to_string()
        }
    }

    fn info(&self, text: &str) -> String {
        if self.use_color {
            text.dimmed().to_string()
        } else {
            text.to_string()
        }
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_diff_report() {
        let diff = MetadataDiff::new();
        let generator = ReportGenerator::new().with_color(false);
        let report = generator.generate_report(&diff);

        assert!(report.contains("No changes detected"));
    }

    #[test]
    fn test_tempo_change_report() {
        let mut diff = MetadataDiff::new();
        diff.global_changes.push(GlobalChange::TempoChange {
            from: 120.0,
            to: 128.0,
        });

        let generator = ReportGenerator::new().with_color(false);
        let report = generator.generate_report(&diff);

        assert!(report.contains("Tempo"));
        assert!(report.contains("120"));
        assert!(report.contains("128"));
    }
}
