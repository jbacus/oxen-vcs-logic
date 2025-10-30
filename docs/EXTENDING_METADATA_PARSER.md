# Extending the Metadata Parser

**Status**: Phase 1 - In Progress
**Last Updated**: 2025-10-30

This document provides guidance on extending the Logic Pro metadata parser to support additional project elements and improve parsing accuracy.

---

## Overview

The metadata parser consists of three main components:

1. **Binary Parser** (`logic_parser/binary_parser.rs`) - Low-level binary parsing
2. **Data Structures** (`logic_parser/project_data.rs`) - Type definitions
3. **Diff Engine** (`metadata_diff/diff_engine.rs`) - Comparison algorithms

---

## Current Implementation Status

### ✅ Implemented

- Core data structures (tracks, channel strips, EQ, compressor, reverb)
- Project validation and package structure parsing
- Basic metadata extraction (tempo, sample rate placeholder)
- Complete diff engine with all comparison algorithms
- Human-readable report generation

### 🚧 Partially Implemented

- Binary format parsing (requires reverse engineering)
- Logic Pro version detection (placeholder)
- Track and region extraction (skeleton)

### ❌ Not Yet Implemented

- Plugin parameter parsing (beyond channel strip)
- Automation curve data extraction
- MIDI event parsing
- Sample-accurate region timing
- Third-party plugin support

---

## Adding Support for New Data Types

### Step 1: Define Data Structures

Add new types to `logic_parser/project_data.rs`:

```rust
// Example: Adding delay plugin support
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DelaySettings {
    pub bypassed: bool,
    pub delay_time: f32,     // milliseconds
    pub feedback: f32,       // 0.0 to 1.0
    pub mix: f32,            // 0.0 to 1.0
    pub sync_tempo: bool,
}
```

Add the field to `ChannelStrip`:

```rust
pub struct ChannelStrip {
    // ... existing fields
    pub delay: Option<DelaySettings>,
}
```

### Step 2: Implement Binary Parsing

Add parsing logic to `logic_parser/binary_parser.rs`:

```rust
fn parse_delay_settings(binary: &[u8], offset: usize) -> Result<DelaySettings> {
    // TODO: Reverse engineer binary format
    // This requires analyzing actual Logic Pro project files

    // Example structure (hypothetical):
    // Offset +0: u8 bypassed (0 or 1)
    // Offset +1: padding (3 bytes)
    // Offset +4: f32 delay_time
    // Offset +8: f32 feedback
    // Offset +12: f32 mix
    // Offset +16: u8 sync_tempo

    let bypassed = binary[offset] != 0;
    let delay_time = parse_f32_at_offset(binary, offset + 4)?;
    let feedback = parse_f32_at_offset(binary, offset + 8)?;
    let mix = parse_f32_at_offset(binary, offset + 12)?;
    let sync_tempo = binary[offset + 16] != 0;

    Ok(DelaySettings {
        bypassed,
        delay_time,
        feedback,
        mix,
        sync_tempo,
    })
}
```

### Step 3: Add Diff Support

Add diff types to `metadata_diff/diff_types.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DelayChange {
    DelayTimeChanged { from: f32, to: f32 },
    FeedbackChanged { from: f32, to: f32 },
    MixChanged { from: f32, to: f32 },
    SyncToggled { sync_tempo: bool },
    BypassToggled { bypassed: bool },
}
```

Update `ChannelStripDiff`:

```rust
pub struct ChannelStripDiff {
    // ... existing fields
    pub delay_changes: Vec<DelayChange>,
}
```

### Step 4: Implement Diff Logic

Add comparison logic to `metadata_diff/diff_engine.rs`:

```rust
fn diff_delay(
    delay_a: Option<&DelaySettings>,
    delay_b: Option<&DelaySettings>,
) -> Option<Vec<DelayChange>> {
    let mut changes = Vec::new();

    match (delay_a, delay_b) {
        (None, Some(_)) => {
            changes.push(DelayChange::BypassToggled { bypassed: false });
        }
        (Some(_), None) => {
            changes.push(DelayChange::BypassToggled { bypassed: true });
        }
        (Some(delay_a), Some(delay_b)) => {
            if delay_a.bypassed != delay_b.bypassed {
                changes.push(DelayChange::BypassToggled {
                    bypassed: delay_b.bypassed,
                });
            }

            if (delay_a.delay_time - delay_b.delay_time).abs() > 1.0 {
                changes.push(DelayChange::DelayTimeChanged {
                    from: delay_a.delay_time,
                    to: delay_b.delay_time,
                });
            }

            // ... other parameters
        }
        (None, None) => {}
    }

    if changes.is_empty() {
        None
    } else {
        Some(changes)
    }
}
```

### Step 5: Add Report Formatting

Add formatting logic to `metadata_diff/report_generator.rs`:

```rust
fn format_delay_change(&self, change: &DelayChange) -> String {
    match change {
        DelayChange::DelayTimeChanged { from, to } => {
            format!("      • Delay Time: {:.1} ms → {:.1} ms\n", from, to)
        }
        DelayChange::FeedbackChanged { from, to } => {
            format!("      • Feedback: {:.0}% → {:.0}%\n", from * 100.0, to * 100.0)
        }
        DelayChange::MixChanged { from, to } => {
            format!("      • Mix: {:.0}% → {:.0}%\n", from * 100.0, to * 100.0)
        }
        DelayChange::SyncToggled { sync_tempo } => {
            format!("      • Tempo Sync: {}\n", if *sync_tempo { "ON" } else { "OFF" })
        }
        DelayChange::BypassToggled { bypassed } => {
            format!("      • Delay {}\n", if *bypassed { "bypassed" } else { "enabled" })
        }
    }
}
```

### Step 6: Write Tests

Add tests for the new feature:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_time_change() {
        let mut cs_a = ChannelStrip::default();
        cs_a.delay = Some(DelaySettings {
            bypassed: false,
            delay_time: 250.0,
            feedback: 0.5,
            mix: 0.3,
            sync_tempo: false,
        });

        let mut cs_b = ChannelStrip::default();
        cs_b.delay = Some(DelaySettings {
            bypassed: false,
            delay_time: 500.0,
            feedback: 0.5,
            mix: 0.3,
            sync_tempo: false,
        });

        let diff = diff_channel_strip(&cs_a, &cs_b).unwrap();
        assert!(!diff.delay_changes.is_empty());
    }
}
```

---

## Reverse Engineering Logic Pro Binary Format

### Tools Needed

1. **Hex Editor**: HexFiend (macOS), 010 Editor, or ImHex
2. **Binary Analysis**: Kaitai Struct for format definition
3. **Test Projects**: Multiple Logic Pro projects with known changes

### Methodology

#### 1. Create Test Projects

Create pairs of Logic Pro projects with single, known changes:

- Project A: Tempo = 120 BPM
- Project B: Tempo = 128 BPM (only change)

This isolates the binary bytes that changed.

#### 2. Binary Diff Analysis

```bash
# Extract ProjectData files
cd ProjectA.logicx/Alternatives/001
cp ProjectData /tmp/projectdata_a

cd ../../../ProjectB.logicx/Alternatives/001
cp ProjectData /tmp/projectdata_b

# Use hexdump to compare
hexdump -C /tmp/projectdata_a > /tmp/a.hex
hexdump -C /tmp/projectdata_b > /tmp/b.hex

# Diff the hex dumps
diff /tmp/a.hex /tmp/b.hex
```

#### 3. Identify Patterns

Look for:

- **4-byte floats**: Tempo, volume, pan, frequencies
- **4-byte ints**: Sample rate, counts
- **Strings**: Track names, plugin names (UTF-8 or UTF-16)
- **Sequences**: Arrays of data (tracks, regions)

#### 4. Validate Hypotheses

Implement parsing for the identified offset:

```rust
// Test hypothesis: Tempo at offset 0x120
let tempo = parse_f32_at_offset(binary, 0x120)?;
println!("Parsed tempo: {}", tempo);
```

Create multiple test projects to confirm the offset is consistent.

#### 5. Document Format

Create format documentation:

```
Logic Pro ProjectData Format (v11.0.0)
=======================================

Header (bytes 0-15):
  0x00-0x03: Magic number (0x4C47434D)
  0x04-0x07: Format version (u32 little-endian)
  0x08-0x0B: File size (u32 little-endian)
  0x0C-0x0F: Reserved

Global Settings (bytes 16-127):
  0x10-0x13: Tempo (f32 BPM)
  0x14-0x17: Sample rate (u32 Hz)
  0x18: Key signature index (u8)
  0x19: Time signature numerator (u8)
  0x1A: Time signature denominator (u8)
  ...
```

### Known Challenges

1. **Format Changes**: Logic Pro versions may have different formats
2. **Compressed Data**: Some sections may use compression
3. **Proprietary Encoding**: Apple-specific data structures
4. **Plugin Data**: Third-party plugins store custom data

### Fallback Strategy

If full binary parsing proves too difficult:

1. **FCP XML Export**: Use Logic Pro's Final Cut Pro XML export feature
2. **Partial Parsing**: Focus on high-value parameters (EQ, compressor, volume)
3. **Hybrid Approach**: Parse what's accessible, note what's not

---

## Testing Strategy

### Unit Tests

Test individual components in isolation:

```rust
#[test]
fn test_parse_eq_band() {
    let data = create_test_eq_data();
    let eq = parse_eq_settings(&data, 0).unwrap();

    assert_eq!(eq.bands.len(), 4);
    assert_eq!(eq.bands[0].frequency, 8000.0);
    assert_eq!(eq.bands[0].gain, 3.0);
}
```

### Integration Tests

Test with real Logic Pro projects:

```rust
#[test]
fn test_parse_real_project() {
    let project_path = Path::new("tests/fixtures/Demo_Project.logicx");
    let data = LogicParser::parse(project_path).unwrap();

    assert!(data.tempo > 0.0);
    assert!(!data.tracks.is_empty());
}
```

### Regression Tests

Ensure changes don't break existing functionality:

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Check coverage threshold
if [ "$(cargo tarpaulin --out Stdout | grep '%')" -lt 80 ]; then
    echo "Coverage below 80%"
    exit 1
fi
```

---

## Performance Considerations

### Lazy Parsing

Don't parse everything upfront:

```rust
pub struct LogicProjectData {
    // Eagerly loaded
    pub tempo: f32,
    pub sample_rate: u32,

    // Lazily loaded
    tracks: OnceCell<Vec<Track>>,
    automation: OnceCell<Vec<AutomationCurve>>,
}

impl LogicProjectData {
    pub fn tracks(&mut self) -> &Vec<Track> {
        self.tracks.get_or_init(|| {
            // Parse tracks on first access
            parse_tracks(&self.binary_data)
        })
    }
}
```

### Caching

Cache parsed results:

```rust
use std::collections::HashMap;

pub struct ParserCache {
    projects: HashMap<PathBuf, LogicProjectData>,
}

impl ParserCache {
    pub fn get_or_parse(&mut self, path: &Path) -> Result<&LogicProjectData> {
        if !self.projects.contains_key(path) {
            let data = LogicParser::parse(path)?;
            self.projects.insert(path.to_path_buf(), data);
        }
        Ok(self.projects.get(path).unwrap())
    }
}
```

### Parallel Processing

Parse multiple projects concurrently:

```rust
use rayon::prelude::*;

pub fn parse_multiple_projects(paths: &[PathBuf]) -> Vec<Result<LogicProjectData>> {
    paths.par_iter()
        .map(|path| LogicParser::parse(path))
        .collect()
}
```

---

## Error Handling

### Graceful Degradation

Don't fail completely if some data can't be parsed:

```rust
pub struct LogicProjectData {
    pub tempo: Option<f32>,          // None if parsing failed
    pub sample_rate: Option<u32>,
    pub tracks: Vec<Track>,
    pub parse_warnings: Vec<String>,
}

impl LogicParser {
    pub fn parse(path: &Path) -> Result<LogicProjectData> {
        let mut data = LogicProjectData::default();
        let mut warnings = Vec::new();

        // Try to parse tempo
        match parse_tempo(&binary) {
            Ok(tempo) => data.tempo = Some(tempo),
            Err(e) => warnings.push(format!("Failed to parse tempo: {}", e)),
        }

        // Continue with other fields...
        data.parse_warnings = warnings;
        Ok(data)
    }
}
```

### Clear Error Messages

Provide actionable error messages:

```rust
if !project_path.exists() {
    return Err(anyhow::anyhow!(
        "Logic Pro project not found: {}\n\
         Hint: Make sure the path points to a .logicx directory",
        project_path.display()
    ));
}
```

---

## Contributing

### Submission Checklist

Before submitting changes:

- [ ] Tests pass: `cargo test`
- [ ] Code formatted: `cargo fmt`
- [ ] No lints: `cargo clippy`
- [ ] Documentation updated
- [ ] Examples added (if new feature)
- [ ] Changelog entry

### Code Review

Focus areas:

1. **Correctness**: Does the parser produce accurate results?
2. **Performance**: Is it efficient for large projects?
3. **Robustness**: Does it handle edge cases gracefully?
4. **Documentation**: Is it clear how to use and extend?

---

## Resources

### External Documentation

- [Logic Pro Project Format - Library of Congress](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml)
- [Robert Heaton - Reverse Engineering Logic Pro](https://robertheaton.com/2017/07/17/reverse-engineering-logic-pro-synth-files/)
- [Kaitai Struct](https://kaitai.io/) - Binary format definition

### Internal Documentation

- [Phase 1 Work Plan](PHASE_1_WORK_PLAN.md) - Overall implementation plan
- [Architecture](ARCHITECTURE.md) - System design
- [User Guide](USER_GUIDE.md) - End-user documentation

---

## FAQ

**Q: Why is binary parsing so difficult?**

A: Logic Pro uses a proprietary, undocumented binary format. Without official API/SDK, we must reverse-engineer it through trial and error.

**Q: Can we use FCP XML instead?**

A: FCP XML export is an option, but it's lossy (doesn't include all parameters) and requires manual export from Logic Pro, which breaks automation.

**Q: What if Logic Pro changes the format?**

A: We implement version detection and maintain separate parsers for each format version. Graceful degradation ensures older parsers still work.

**Q: How accurate is the metadata diff?**

A: Currently, the diff engine is 100% accurate for parsed data. The limitation is in binary parsing coverage (what % of project data we can parse).

---

**Last Updated**: 2025-10-30
**Status**: Phase 1 In Progress
**Next Review**: After Week 6 (Go/No-Go decision)
