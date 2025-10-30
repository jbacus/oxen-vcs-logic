# Phase 1 Work Plan: Metadata Layer Implementation

**Created**: 2025-10-30
**Status**: Ready for Implementation
**Timeline**: 3-6 months
**Priority**: HIGH - Foundational for all subsequent phases

---

## Executive Summary

Phase 1 implements the **Metadata Layer** of the semantic audio diff system. This phase focuses on parsing Logic Pro's proprietary `.logicx` format to extract structured project metadata and generate human-readable diff reports that explain *what changed* in producer-friendly language.

**Key Deliverable**: A Rust-based metadata parser and diff engine that outputs reports like:
```
Metadata Changes (Version A → Version B):

Track "Lead Synth":
  • EQ: Added +3 dB shelf at 8 kHz
  • Compressor: Threshold changed -18 dB → -12 dB
  • Volume: Increased by 2.5 dB

Track "Drums":
  • Region "Kick" (01:15-01:20): Reversed
  • Automation: New volume automation curve on bars 10-14

New Track Added: "Strings" (MIDI)
```

**Why This Matters**:
- Establishes causal reporting (metadata changes explain audio changes)
- Provides immediate value to users (actionable diff reports)
- Validates architecture before investing in expensive audio analysis
- Required foundation for Phases 2-5

---

## Phase 1 Goals

### Primary Goals
1. ✅ Parse Logic Pro `.logicx` project files into structured data
2. ✅ Extract key metadata: tempo, sample rate, key signature, tracks, plugins, automation
3. ✅ Generate structured diffs between two project versions
4. ✅ Produce human-readable reports in producer-friendly language

### Success Metrics
- **Accuracy**: 90%+ of metadata changes correctly detected
- **Coverage**: Support 80%+ of common Logic Pro project elements
- **Performance**: Parse typical project (<100 tracks) in <5 seconds
- **Usability**: 8+/10 producer satisfaction with diff report clarity

### Non-Goals (Deferred to Later Phases)
- ❌ Audio content analysis (Phase 2)
- ❌ DTW temporal alignment (Phase 3)
- ❌ ML-based semantic translation (Phase 4)
- ❌ Interactive visualization UI (Phase 5)

---

## Technical Architecture

### Component Structure

```
OxVCS-CLI-Wrapper/src/
├── logic_parser/
│   ├── mod.rs                 # Public API
│   ├── binary_parser.rs       # Low-level binary parsing
│   ├── project_data.rs        # LogicProjectData structs
│   ├── track_parser.rs        # Track/region extraction
│   ├── plugin_parser.rs       # Plugin parameter parsing
│   └── automation_parser.rs   # Automation curve parsing
│
├── metadata_diff/
│   ├── mod.rs                 # Public API
│   ├── diff_engine.rs         # Core diff algorithms
│   ├── track_diff.rs          # Track-level comparison
│   ├── channel_strip_diff.rs  # Channel strip comparison
│   └── report_generator.rs    # Human-readable output
│
└── commands/
    └── metadata_diff.rs       # CLI command: oxvcs metadata-diff
```

### Data Structures

```rust
// Core data model
pub struct LogicProjectData {
    pub tempo: f32,
    pub sample_rate: u32,
    pub key_signature: String,
    pub time_signature: (u8, u8),
    pub tracks: Vec<Track>,
    pub automation: Vec<AutomationCurve>,
    pub plugins: Vec<PluginInstance>,
}

pub struct Track {
    pub id: String,
    pub name: String,
    pub track_type: TrackType,
    pub channel_strip: ChannelStrip,
    pub regions: Vec<Region>,
}

pub struct ChannelStrip {
    pub eq: Option<EQSettings>,
    pub compressor: Option<CompressorSettings>,
    pub reverb: Option<ReverbSettings>,
    pub volume: f32,    // dB
    pub pan: f32,       // -1.0 (left) to +1.0 (right)
}

// Diff output model
pub struct MetadataDiff {
    pub global_changes: Vec<GlobalChange>,
    pub track_changes: Vec<TrackChange>,
    pub plugin_changes: Vec<PluginChange>,
    pub automation_changes: Vec<AutomationChange>,
}
```

---

## Implementation Plan

### Week 1-2: Research & Setup

#### Tasks
1. **Study Logic Pro Binary Format**
   - Review Library of Congress technical specification
   - Analyze existing reverse engineering work (robertheaton.com)
   - Document `.logicx` package structure
   - Identify target binary files within package

2. **Set Up Development Environment**
   - Install Logic Pro 11.x for testing
   - Create sample projects covering common scenarios:
     - Basic project (4 tracks, simple EQ/compression)
     - Complex project (50+ tracks, automation, plugin-heavy)
     - Edge cases (external references, frozen tracks)
   - Set up hex editor (HexFiend) for binary analysis

3. **Define Data Model**
   - Finalize Rust structs for `LogicProjectData`
   - Design JSON schema for serialized metadata
   - Plan for extensibility (future Logic Pro versions)

#### Deliverables
- ✅ Research document: `docs/LOGIC_PRO_BINARY_FORMAT.md`
- ✅ Test fixture library: `tests/fixtures/logic-projects/`
- ✅ Data model code: `OxVCS-CLI-Wrapper/src/logic_parser/project_data.rs`

#### Risks & Mitigations
- **Risk**: Logic Pro format is undocumented
  - **Mitigation**: Focus on XML-based metadata where possible, community research
- **Risk**: Format changes between Logic versions
  - **Mitigation**: Version detection, graceful degradation, extensible parser

---

### Week 3-6: Binary Parser Implementation

#### Tasks
1. **Package Structure Parser** (Week 3)
   - Parse `.logicx` as directory bundle
   - Extract `Alternatives/001/ProjectData` (main binary)
   - Detect Logic Pro version from package metadata
   - Handle both folder-based and package-based projects

2. **Global Metadata Extraction** (Week 3-4)
   - Parse tempo, sample rate, key signature, time signature
   - Extract project-level settings (sample rate, bit depth)
   - Test with fixture projects, validate accuracy

3. **Track Information Extraction** (Week 4-5)
   - Parse track list (name, type, ID)
   - Extract track order and hierarchy (folders, submixes)
   - Parse region information (start, end, name, properties)
   - Handle different track types (audio, MIDI, aux, bus)

4. **Channel Strip Parsing** (Week 5-6)
   - Extract EQ settings (band count, frequency, gain, Q)
   - Parse compressor parameters (threshold, ratio, attack, release)
   - Extract volume/pan/mute/solo state
   - Handle plugin chain (order, bypass state)

#### Code Example
```rust
// OxVCS-CLI-Wrapper/src/logic_parser/binary_parser.rs

use std::path::Path;
use anyhow::{Context, Result};

pub fn parse_logic_project(path: &Path) -> Result<LogicProjectData> {
    // 1. Validate .logicx package structure
    if !path.exists() || !path.is_dir() {
        anyhow::bail!("Invalid .logicx package: {:?}", path);
    }

    // 2. Locate ProjectData binary
    let project_data_path = path
        .join("Alternatives")
        .join("001")
        .join("ProjectData");

    if !project_data_path.exists() {
        anyhow::bail!("ProjectData not found in package");
    }

    // 3. Read binary file
    let binary = std::fs::read(&project_data_path)
        .context("Failed to read ProjectData binary")?;

    // 4. Parse binary format (reverse-engineered)
    let project = parse_binary_format(&binary)?;

    Ok(project)
}

fn parse_binary_format(binary: &[u8]) -> Result<LogicProjectData> {
    // Binary parsing logic based on reverse engineering
    // This is the hard part - requires extensive research

    // Example: Parse header
    let header = parse_header(binary)?;

    // Parse tempo (example location - needs research)
    let tempo = parse_f32_at_offset(binary, header.tempo_offset)?;

    // Parse tracks
    let tracks = parse_tracks(binary, &header)?;

    Ok(LogicProjectData {
        tempo,
        sample_rate: header.sample_rate,
        key_signature: header.key_signature,
        time_signature: header.time_signature,
        tracks,
        automation: vec![],
        plugins: vec![],
    })
}
```

#### Deliverables
- ✅ Binary parser: `logic_parser/binary_parser.rs`
- ✅ Track parser: `logic_parser/track_parser.rs`
- ✅ Unit tests: 50+ tests covering common scenarios
- ✅ Integration tests: Parse 10+ real Logic Pro projects

#### Risks & Mitigations
- **Risk**: Binary format too complex to reverse engineer
  - **Mitigation**: Fall back to XML-based metadata (FCP XML export)
  - **Mitigation**: Focus on high-value parameters first (EQ, compressor, volume)
- **Risk**: Performance issues with large projects
  - **Mitigation**: Lazy parsing, caching, parallel processing

---

### Week 7-10: Diff Engine Implementation

#### Tasks
1. **Core Diff Algorithm** (Week 7)
   - Implement `diff_metadata(version_a, version_b) -> MetadataDiff`
   - Track matching by ID (handle renames, reordering)
   - Global changes (tempo, sample rate, key)

2. **Track-Level Diff** (Week 7-8)
   - Detect added/removed/renamed tracks
   - Compare track types and properties
   - Region-level diff (added/removed/moved/resized)

3. **Channel Strip Diff** (Week 8-9)
   - EQ diff: band-by-band comparison (frequency, gain, Q)
   - Compressor diff: parameter-by-parameter comparison
   - Volume/pan delta calculation
   - Plugin chain diff (order changes, bypass state)

4. **Automation Diff** (Week 9-10)
   - Detect new/removed automation curves
   - Compare curve shapes (sample points)
   - Identify significant changes vs. minor adjustments

#### Code Example
```rust
// OxVCS-CLI-Wrapper/src/metadata_diff/diff_engine.rs

pub fn diff_metadata(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
) -> MetadataDiff {
    let mut diff = MetadataDiff::default();

    // Global changes
    if version_a.tempo != version_b.tempo {
        diff.global_changes.push(GlobalChange::TempoChange {
            from: version_a.tempo,
            to: version_b.tempo,
        });
    }

    // Track changes
    for track_b in &version_b.tracks {
        if let Some(track_a) = version_a.find_track(&track_b.id) {
            // Track exists in both - check for changes
            if let Some(cs_diff) = diff_channel_strip(
                &track_a.channel_strip,
                &track_b.channel_strip
            ) {
                diff.track_changes.push(TrackChange::ChannelStripChanged {
                    track_name: track_b.name.clone(),
                    changes: cs_diff,
                });
            }
        } else {
            // New track in B
            diff.track_changes.push(TrackChange::Added {
                track: track_b.clone()
            });
        }
    }

    // Check for removed tracks
    for track_a in &version_a.tracks {
        if !version_b.has_track(&track_a.id) {
            diff.track_changes.push(TrackChange::Removed {
                track_name: track_a.name.clone(),
            });
        }
    }

    diff
}

fn diff_channel_strip(
    cs_a: &ChannelStrip,
    cs_b: &ChannelStrip,
) -> Option<ChannelStripDiff> {
    let mut changes = ChannelStripDiff::default();
    let mut has_changes = false;

    // EQ diff
    if let (Some(eq_a), Some(eq_b)) = (&cs_a.eq, &cs_b.eq) {
        if let Some(eq_changes) = diff_eq(eq_a, eq_b) {
            changes.eq_changes = eq_changes;
            has_changes = true;
        }
    }

    // Volume diff
    let volume_delta = cs_b.volume - cs_a.volume;
    if volume_delta.abs() > 0.1 {  // >0.1 dB threshold
        changes.volume_delta = Some(volume_delta);
        has_changes = true;
    }

    // Pan diff
    let pan_delta = cs_b.pan - cs_a.pan;
    if pan_delta.abs() > 0.05 {  // >5% threshold
        changes.pan_delta = Some(pan_delta);
        has_changes = true;
    }

    if has_changes {
        Some(changes)
    } else {
        None
    }
}
```

#### Deliverables
- ✅ Diff engine: `metadata_diff/diff_engine.rs`
- ✅ Specialized diff modules: `track_diff.rs`, `channel_strip_diff.rs`
- ✅ Unit tests: 80+ tests covering all diff scenarios
- ✅ Benchmark tests: Performance on large projects

#### Risks & Mitigations
- **Risk**: False positives (detecting changes that didn't happen)
  - **Mitigation**: Threshold tuning, significance filters
- **Risk**: False negatives (missing important changes)
  - **Mitigation**: Extensive test coverage, real-world validation

---

### Week 11-12: Integration & Testing

#### Tasks
1. **CLI Integration** (Week 11)
   - Add new command: `oxvcs metadata-diff <commit-a> <commit-b>`
   - Integrate with existing OxVCS workflow
   - Add `--output-format` flag (text, json, markdown)

2. **Comprehensive Testing** (Week 11-12)
   - Test with real Logic Pro projects from users
   - Edge case testing (corrupted files, partial data)
   - Performance testing (large projects, 100+ tracks)
   - Cross-version testing (Logic Pro 10.8, 11.0, 11.1)

3. **Error Handling & Robustness** (Week 12)
   - Graceful degradation (partial parsing on errors)
   - Clear error messages for users
   - Logging for debugging

#### CLI Example
```bash
# Compare two commits
oxvcs metadata-diff abc123 def456

# Compare current working state to last commit
oxvcs metadata-diff HEAD~1 HEAD

# Output as JSON for programmatic use
oxvcs metadata-diff abc123 def456 --output-format json > diff.json

# Verbose output with technical details
oxvcs metadata-diff abc123 def456 --verbose
```

#### Deliverables
- ✅ CLI command: `commands/metadata_diff.rs`
- ✅ Integration tests: 20+ end-to-end scenarios
- ✅ Performance benchmarks: Report on typical project sizes
- ✅ Error handling: Comprehensive error coverage

---

### Week 13-16: Reporting & Refinement

#### Tasks
1. **Human-Readable Report Generator** (Week 13-14)
   - Implement `report_generator.rs`
   - Producer-friendly language ("Harshness" not "5kHz boost")
   - Hierarchical report structure (global → track → plugin)
   - Color-coded output for terminal

2. **Visualization Preparation** (Week 14-15)
   - Export diff data in format suitable for UI (Phase 5)
   - JSON schema for semantic diff data
   - Support for timeline annotations

3. **Documentation & Polish** (Week 15-16)
   - User guide: How to interpret metadata diffs
   - Developer docs: Extending the parser
   - Architecture docs: System design decisions
   - Blog post: Announcing Phase 1 completion

#### Report Example
```
=============================================================================
METADATA DIFF: Version abc123 → def456
=============================================================================

GLOBAL CHANGES:
  • Tempo: 120 BPM → 128 BPM (+6.7%)
  • Key Signature: C Major → D Major (transposed up 2 semitones)

-----------------------------------------------------------------------------
TRACK CHANGES:
-----------------------------------------------------------------------------

Track "Lead Synth" (Track 5):
  ✓ EQ Changes:
    • Band 3 (High Shelf): Added +3.0 dB boost at 8 kHz
    • Q factor: 1.2 → 0.8 (wider)

  ✓ Compressor Changes:
    • Threshold: -18.0 dB → -12.0 dB (less compression)
    • Ratio: 4:1 → 3:1

  ✓ Volume: +2.5 dB

Track "Drums" (Track 2):
  ✓ Region Changes:
    • "Kick Pattern" (01:15.000 - 01:20.000): Reversed

  ✓ Automation:
    • New volume automation curve (bars 10-14)
    • Fade-out applied at 01:18.500

[+] NEW TRACK: "Strings" (Track 8)
    • Type: MIDI
    • Patch: "Strings > Ensemble > Full Strings"
    • 4 regions added

[-] REMOVED TRACK: "Scratch Vocal" (Track 3)

-----------------------------------------------------------------------------
SUMMARY:
-----------------------------------------------------------------------------
  • 3 tracks modified
  • 1 track added
  • 1 track removed
  • 2 EQ changes
  • 1 compressor change
  • 1 automation curve added
  • Global tempo change detected
```

#### Deliverables
- ✅ Report generator: `metadata_diff/report_generator.rs`
- ✅ User guide: `docs/USER_GUIDE_METADATA_DIFF.md`
- ✅ Developer docs: `docs/EXTENDING_METADATA_PARSER.md`
- ✅ Announcement blog post

---

## Resource Requirements

### Team
- **1 × Rust Developer** (full-time, 3-6 months)
  - Binary format reverse engineering
  - Parser implementation
  - Diff engine implementation

- **0.5 × Audio Engineer** (consulting, 10-20 hours/month)
  - Validate metadata extraction accuracy
  - Review producer-friendly language
  - Test with real-world projects

- **0.25 × Technical Writer** (consulting, 5-10 hours/month)
  - Documentation
  - User guides
  - Blog posts

### Tools & Infrastructure
- Logic Pro 11.x license ($199)
- Hex editor (HexFiend - free)
- Binary analysis tools (Kaitai Struct - free)
- CI/CD for automated testing
- macOS development machine (required for testing)

### Budget Estimate
- **Personnel**: $50k-$100k (1 FTE for 3-6 months)
- **Tools**: $1k
- **Infrastructure**: $2k
- **Total**: ~$53k-$103k

---

## Risk Management

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Logic Pro format too complex | HIGH | HIGH | Fall back to FCP XML export, focus on high-value parameters |
| Format changes in future Logic versions | MEDIUM | HIGH | Version detection, extensible parser design |
| Performance issues with large projects | MEDIUM | MEDIUM | Lazy parsing, caching, parallel processing |
| False positives/negatives in diff | MEDIUM | MEDIUM | Extensive testing, threshold tuning |

### Business Risks

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Low user adoption | MEDIUM | HIGH | User testing, clear documentation, gradual rollout |
| Apple changes Logic format frequently | LOW | HIGH | Monitor Logic Pro updates, maintain parser |
| Phase 1 doesn't provide enough value | LOW | MEDIUM | Focus on high-impact features (EQ, compression, volume) |

---

## Decision Points

### Go/No-Go Criteria (Week 6)
**Question**: Is binary parsing feasible, or should we pivot to XML-based approach?

**Evaluate**:
- Can we parse basic metadata (tempo, tracks, EQ)?
- Accuracy rate on test projects
- Time/effort required for complete implementation

**Decision**:
- ✅ **GO**: Continue binary parsing if >80% accuracy on test projects
- 🔄 **PIVOT**: Switch to FCP XML if binary parsing too complex
- ❌ **NO-GO**: Pause Phase 1 if neither approach works

### Milestone Review (Week 12)
**Question**: Should we proceed with reporting & refinement, or iterate on core parsing?

**Evaluate**:
- Test coverage (target: 80%+)
- Accuracy on real-world projects (target: 90%+)
- Performance (target: <5s for typical project)
- User feedback from alpha testing

**Decision**:
- ✅ **PROCEED**: Move to reporting if metrics met
- 🔄 **ITERATE**: Extend testing/refinement if close but not quite there
- ❌ **REASSESS**: Re-evaluate approach if far from targets

---

## Success Criteria

### Technical Success
- ✅ Parse 90%+ of Logic Pro project metadata correctly
- ✅ Generate accurate diffs for 95%+ of changes
- ✅ Parse typical project (<100 tracks) in <5 seconds
- ✅ Handle edge cases gracefully (corrupt files, unsupported features)

### User Success
- ✅ 80%+ of users find metadata diff reports useful
- ✅ 8+/10 satisfaction rating on report clarity
- ✅ 70%+ of users enable metadata diff feature
- ✅ Positive feedback from alpha/beta testers

### Business Success
- ✅ Validate architecture for Phases 2-5
- ✅ Demonstrate unique value proposition (competitive differentiation)
- ✅ Build user excitement for semantic audio diff vision
- ✅ Secure resources for Phase 2 based on Phase 1 success

---

## Next Steps After Phase 1

### Immediate (Week 17-20)
1. **User Testing**: Deploy to 10-20 alpha users, gather feedback
2. **Iteration**: Fix bugs, refine reports based on feedback
3. **Documentation**: Finalize user guides, API docs
4. **Marketing**: Blog posts, demos, social media

### Short-Term (Months 6-9)
1. **Phase 1.5**: Polish and stabilization
2. **Evaluate Phase 2**: Decide whether to proceed with audio analysis
3. **Business Review**: Assess ROI, user adoption, competitive landscape

### Phase 2 Planning (If Approved)
- Audio feature extraction (MFCC, Chroma, Spectral Contrast)
- Perceptual hashing for efficiency
- Initial DTW exploration
- Timeline: 6-12 months

---

## Appendix A: Research Resources

### Logic Pro Binary Format
- Library of Congress: [Logic Pro Project Format](https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml)
- Robert Heaton: [Reverse Engineering Logic Pro Synth Files](https://robertheaton.com/2017/07/17/reverse-engineering-logic-pro-synth-files/)
- Apple Support: [Manage Project Assets](https://support.apple.com/en-eg/guide/logicpro/lgcpce0d70e7/mac)

### Audio Metadata Analysis
- Semantic Audio Feature Extraction (SAFE): [FAST Project](https://www.semanticaudio.ac.uk/demonstrators/15-semantic-audio-feature-extraction-safe/)
- Semantic Music Production: [ResearchGate Meta-Study](https://www.researchgate.net/publication/362246579_Semantic_Music_Production_A_Meta-Study)

### Diff Algorithms
- Dynamic Time Warping: [Wikipedia](https://en.wikipedia.org/wiki/Dynamic_time_warping)
- Structural Diff Algorithms: [Git Internals](https://git-scm.com/book/en/v2/Git-Internals-Git-Objects)

---

## Appendix B: Test Scenarios

### Basic Scenarios (Must Pass)
1. ✅ Tempo change (120 → 128 BPM)
2. ✅ Key change (C Major → D Major)
3. ✅ Track added/removed
4. ✅ EQ boost/cut on single band
5. ✅ Compressor threshold change
6. ✅ Volume/pan adjustment
7. ✅ Region reversed/normalized

### Advanced Scenarios (Should Pass)
1. ✅ Multi-band EQ changes
2. ✅ Complex automation curves
3. ✅ Plugin chain reordering
4. ✅ Track renaming
5. ✅ Time signature change
6. ✅ Multiple simultaneous changes

### Edge Cases (Handle Gracefully)
1. ⚠️ Corrupted project file
2. ⚠️ Unsupported plugin (third-party)
3. ⚠️ Very large project (200+ tracks)
4. ⚠️ Mixed Logic Pro versions
5. ⚠️ External references (missing files)

---

**Status**: Ready for implementation
**Next Action**: Allocate Rust developer, begin Week 1 research phase
**Contact**: Project lead for questions and resource allocation
