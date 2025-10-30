# User Guide: Metadata Diff

**Feature**: Logic Pro Project Metadata Comparison
**Status**: Phase 1 Implementation
**Version**: 0.1.0

---

## Overview

The metadata diff feature allows you to compare two Logic Pro project versions and see exactly what changed in a human-readable format. Instead of manually opening both projects and checking every track, plugin, and setting, you get an instant report showing:

- **Global changes** (tempo, sample rate, key signature)
- **Track modifications** (added, removed, renamed, reordered)
- **EQ changes** (frequency, gain, Q factor adjustments)
- **Compressor changes** (threshold, ratio, attack, release)
- **Volume and pan adjustments**
- **Automation changes**
- **Plugin parameter changes**

---

## Quick Start

### Basic Usage

Compare two Logic Pro projects:

```bash
oxenvcs-cli metadata-diff MyProject_v1.logicx MyProject_v2.logicx
```

### Output Example

```
METADATA DIFF REPORT
====================

SUMMARY
  • Total changes: 5
  • Global: 1
  • Tracks: 4

GLOBAL CHANGES
--------------
  • Tempo: 120 BPM → 128 BPM (+6.7%)

TRACK CHANGES
-------------

Track "Lead Synth":
    ✓ Volume: +2.5 dB
    ✓ EQ Changes:
      • Band 3 (HighShelf): Added at 8000 Hz, +3.0 dB gain
    ✓ Compressor Changes:
      • Threshold: -18.0 dB → -12.0 dB (+6.0 dB)

[+] NEW TRACK: "Strings" (MIDI)

[-] REMOVED TRACK: "Scratch Vocal"
```

---

## Command Options

### Output Formats

**Text (default):**
```bash
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx
```

**JSON (for programmatic use):**
```bash
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --output json
```

**JSON Output Example:**
```json
{
  "global_changes": [
    {
      "TempoChange": {
        "from": 120.0,
        "to": 128.0
      }
    }
  ],
  "track_changes": [
    {
      "ChannelStripChanged": {
        "track_name": "Lead Synth",
        "track_id": "track_001",
        "changes": {
          "eq_changes": [
            {
              "BandGainChanged": {
                "position": 2,
                "from": 0.0,
                "to": 3.0
              }
            }
          ],
          "volume_delta": 2.5,
          "pan_delta": null
        }
      }
    }
  ]
}
```

### Color Output

**Enable colors (useful when piping to less):**
```bash
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --color | less -R
```

**Auto-detect (default):**
- Colors enabled when outputting to terminal
- Colors disabled when piping to file or other commands

### Verbose Mode

**Show technical details:**
```bash
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --verbose
```

Verbose mode includes:
- Internal IDs and offsets
- Precise floating-point values
- Parser warnings
- Debug information

---

## Understanding the Report

### Global Changes

Changes that affect the entire project:

```
GLOBAL CHANGES
--------------
  • Tempo: 120 BPM → 128 BPM (+6.7%)
  • Key Signature: C Major → D Major
  • Sample Rate: 44100 Hz → 48000 Hz
```

### Track Changes

#### Track Added

```
[+] NEW TRACK: "Strings" (MIDI)
```

A new track was created in version B.

#### Track Removed

```
[-] REMOVED TRACK: "Scratch Vocal"
```

A track from version A was deleted in version B.

#### Channel Strip Changes

```
Track "Lead Synth":
    ✓ Volume: +2.5 dB
    ✓ Pan: 15.0% right
    ✓ EQ Changes:
      • Band 3 (HighShelf): Added at 8000 Hz, +3.0 dB gain
      • Band 1 frequency: 250 Hz → 300 Hz
    ✓ Compressor Changes:
      • Threshold: -18.0 dB → -12.0 dB (+6.0 dB)
      • Ratio: 4.0:1 → 3.0:1
```

**Breakdown:**

- **Volume**: Track volume changed by +2.5 dB (got louder)
- **Pan**: Panned 15% to the right
- **EQ**:
  - Band 3: New high shelf boost at 8 kHz (+3 dB)
  - Band 1: Frequency adjusted from 250 Hz to 300 Hz
- **Compressor**:
  - Threshold raised (less compression)
  - Ratio lowered (gentler compression)

### Automation Changes

```
AUTOMATION CHANGES
------------------
  • New Volume automation on track "Vocals" (24 points)
  • Removed Pan automation from track "Bass"
```

---

## Common Workflows

### Workflow 1: Review Changes Before Committing

```bash
# Compare working version to last commit
oxenvcs-cli metadata-diff MyProject_last.logicx MyProject_current.logicx

# Review changes
# If satisfied, commit
oxenvcs-cli add --all
oxenvcs-cli commit -m "Adjusted EQ on lead synth"
```

### Workflow 2: Compare Two Historical Versions

```bash
# Find commit IDs
oxenvcs-cli log --limit 10

# Extract specific versions (hypothetical future feature)
# oxenvcs-cli export <commit-id> output.logicx

# Compare
oxenvcs-cli metadata-diff version_2024_01_15.logicx version_2024_01_20.logicx
```

### Workflow 3: Debugging Mix Changes

You made changes but can't remember what:

```bash
# Compare before/after
oxenvcs-cli metadata-diff Mix_before.logicx Mix_after.logicx > changes.txt

# Review changes
cat changes.txt
```

The report shows:
```
Track "Drums":
    ✓ EQ Changes:
      • Band 2 gain: +0.0 dB → -3.0 dB (-3.0 dB)
```

**Ah-ha!** You accidentally cut 3 dB at that frequency, making the drums sound thin.

### Workflow 4: Learning from Others

```bash
# Compare your mix to a reference
oxenvcs-cli metadata-diff MyMix.logicx ReferenceMix.logicx

# See what they did differently
```

Output shows:
```
Track "Vocals":
    ✓ Compressor Changes:
      • Threshold: -12.0 dB → -18.0 dB (more compression)
      • Attack: 10.0 ms → 5.0 ms (faster)
```

Learn mixing techniques by comparing projects.

---

## Tips & Best Practices

### 1. Use Descriptive Project Names

**Good:**
```
MyProject_2024_01_15_rough_mix.logicx
MyProject_2024_01_16_after_eq_pass.logicx
```

**Bad:**
```
MyProject copy 1.logicx
MyProject copy 2.logicx
```

Descriptive names make it easier to identify which versions you want to compare.

### 2. Save Frequent Snapshots

Create dated copies of your project before major changes:

```bash
# Before EQ pass
cp -r MyProject.logicx MyProject_before_eq_$(date +%Y%m%d).logicx

# Make changes in Logic Pro

# After EQ pass
oxenvcs-cli metadata-diff \
    MyProject_before_eq_20250130.logicx \
    MyProject.logicx
```

### 3. Use JSON for Automation

Save diffs as JSON for later analysis:

```bash
# Generate JSON report
oxenvcs-cli metadata-diff \
    project_a.logicx \
    project_b.logicx \
    --output json > diff.json

# Process with jq
cat diff.json | jq '.global_changes[] | select(.TempoChange != null)'
```

### 4. Compare Incrementally

When working on a project over multiple sessions:

```bash
# Session 1: Drums
oxenvcs-cli metadata-diff before.logicx after_drums.logicx > session1.txt

# Session 2: Bass
oxenvcs-cli metadata-diff after_drums.logicx after_bass.logicx > session2.txt

# Session 3: Mix
oxenvcs-cli metadata-diff after_bass.logicx final.logicx > session3.txt
```

Track your progress session-by-session.

---

## Interpreting Specific Changes

### EQ Changes

```
• Band 3 gain: +0.0 dB → +3.0 dB (+3.0 dB)
```

**Meaning**: Band 3 (usually high-mids) was boosted by 3 dB.

**Common Interpretation**: Added brightness/presence.

```
• Band 1 frequency: 80 Hz → 120 Hz
```

**Meaning**: Low-frequency band moved from 80 Hz to 120 Hz.

**Common Interpretation**: Shifted low-end focus higher (less sub, more warmth).

### Compressor Changes

```
• Threshold: -18.0 dB → -12.0 dB (+6.0 dB)
```

**Meaning**: Threshold raised by 6 dB.

**Common Interpretation**: Less compression (fewer peaks above threshold).

```
• Ratio: 4.0:1 → 8.0:1
```

**Meaning**: Ratio doubled.

**Common Interpretation**: More aggressive compression (peaks squashed harder).

```
• Attack: 30.0 ms → 10.0 ms
```

**Meaning**: Attack time shortened by 20 ms.

**Common Interpretation**: Compressor reacts faster (more transient control, but may sound less natural).

### Volume Changes

```
• Volume: +2.5 dB
```

**Meaning**: Track volume increased by 2.5 dB.

**Common Interpretation**:
- Small increase (perceived as 10-15% louder)
- May need to check if this causes clipping in the mix bus

**Rule of thumb**:
- ±1 dB: Subtle adjustment
- ±3 dB: Noticeable change (perceived as ~2x louder/quieter)
- ±6 dB: Significant change (perceived as ~2x louder/quieter)
- ±10 dB: Dramatic change (perceived as ~3x louder/quieter)

### Pan Changes

```
• Pan: 25.0% left
```

**Meaning**: Track panned 25% to the left.

**Common Interpretation**: Widening stereo image, creating space in the mix.

---

## Troubleshooting

### Error: "Invalid Logic Pro project"

**Cause**: Path doesn't point to a valid `.logicx` directory.

**Solution**:
```bash
# Check path exists
ls -la MyProject.logicx

# Verify it's a directory (not a file)
file MyProject.logicx

# Check for required files
ls MyProject.logicx/Alternatives/001/ProjectData
```

### Error: "Failed to parse project"

**Cause**: Binary parser couldn't read the project file.

**Possible Reasons**:
1. Corrupted project file
2. Unsupported Logic Pro version
3. Incomplete implementation (Phase 1 limitation)

**Solutions**:
```bash
# Try opening in Logic Pro first (validates project)
open MyProject.logicx

# Check Logic Pro version
oxenvcs-cli metadata-diff --verbose project.logicx project.logicx
# Look for "Detected Logic Pro version: X.X.X"

# Report issue with project details
```

### Warning: "Parsing incomplete - some data not available"

**Cause**: Binary parser couldn't read all metadata (Phase 1 limitation).

**What This Means**:
- Basic metadata (tempo, tracks) parsed successfully
- Advanced features (plugins, automation) not yet supported
- Diff still useful for high-level changes

**Solution**: Wait for Phase 2 improvements, or contribute to the parser!

### Output Not Colored

**Cause**: Terminal doesn't support colors or output is being piped.

**Solution**:
```bash
# Force color output
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --color

# Pipe with color preservation
oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --color | less -R
```

---

## Limitations (Phase 1)

### Current Limitations

1. **Binary Parsing Incomplete**: Not all metadata can be parsed yet
   - ✅ Project-level settings (tempo, sample rate)
   - ✅ Basic track structure
   - 🚧 Channel strip details (partial)
   - ❌ Full plugin parameters
   - ❌ Automation data
   - ❌ MIDI events

2. **Third-Party Plugins**: Limited support for non-Apple plugins

3. **No Visual Diff**: Text-only output (no waveform/spectrogram comparison)

4. **Manual Export Required**: Cannot diff directly from commit history (yet)

### Workarounds

**For complete diffs**: Wait for Phase 2 audio analysis
**For plugin details**: Use Logic Pro's built-in comparison features
**For automation**: Manually inspect in Logic Pro

---

## What's Next?

### Phase 2 (Planned)

- **Audio Content Analysis**: Compare actual audio files
- **Perceptual Hashing**: Fast pre-filtering for large projects
- **Feature Extraction**: MFCC, Chroma, Spectral Contrast

### Phase 3 (Planned)

- **Temporal Alignment**: Handle tempo changes and time-stretching
- **Semantic Translation**: "Muddy" vs "Bright" instead of technical jargon

### Phase 4 (Future)

- **Interactive Visualization**: Spectrograms, A/B comparison
- **Merge Suggestions**: Intelligent conflict resolution

---

## Feedback & Bug Reports

### Found a Bug?

1. **Check if it's a known limitation** (see above)
2. **Create a minimal test case**:
   - Two Logic Pro projects that reproduce the issue
   - Run with `--verbose` to get debug info
3. **Report on GitHub**: [github.com/your-repo/issues](https://github.com/your-repo/issues)

### Feature Request?

We'd love to hear your ideas! Open an issue with:
- **Use case**: What problem are you trying to solve?
- **Proposed solution**: How should it work?
- **Alternatives**: What do you do now?

---

## Advanced Usage

### Scripting with JSON Output

```bash
#!/bin/bash
# Script: check_tempo_change.sh
# Alerts if tempo changed by more than 5 BPM

DIFF_JSON=$(oxenvcs-cli metadata-diff project_a.logicx project_b.logicx --output json)

TEMPO_CHANGE=$(echo "$DIFF_JSON" | jq -r '
  .global_changes[] |
  select(.TempoChange != null) |
  .TempoChange |
  (.to - .from) | fabs
')

if (( $(echo "$TEMPO_CHANGE > 5" | bc -l) )); then
    echo "⚠️  WARNING: Tempo changed by $TEMPO_CHANGE BPM!"
    exit 1
fi

echo "✓ Tempo change within acceptable range"
```

### Integration with Version Control Hooks

```bash
# .git/hooks/pre-commit
#!/bin/bash
# Auto-generate diff report before commit

CURRENT_PROJECT="MyProject.logicx"
LAST_VERSION="MyProject_last_commit.logicx"

if [ -d "$LAST_VERSION" ]; then
    oxenvcs-cli metadata-diff \
        "$LAST_VERSION" \
        "$CURRENT_PROJECT" \
        > .metadata_diff.txt

    echo "Metadata diff saved to .metadata_diff.txt"
fi
```

---

## FAQ

**Q: Why use this instead of just opening both projects?**

A: Manually comparing projects is time-consuming and error-prone. This tool gives you an instant, comprehensive report.

**Q: Does this work with other DAWs (Ableton, Pro Tools)?**

A: Not yet. Phase 1 focuses on Logic Pro. Future versions may support other DAWs.

**Q: Can I diff audio content, not just metadata?**

A: Not in Phase 1. Phase 2 will add audio content analysis (MFCC, spectral diff).

**Q: Will this slow down my workflow?**

A: No. Typical diff takes <5 seconds for projects with <100 tracks.

**Q: Does this modify my Logic Pro projects?**

A: No. This tool only reads projects, never writes.

**Q: Can I use this in CI/CD?**

A: Yes! JSON output is perfect for automated testing and validation.

---

**Last Updated**: 2025-10-30
**Version**: 0.1.0 (Phase 1)
**License**: MIT
