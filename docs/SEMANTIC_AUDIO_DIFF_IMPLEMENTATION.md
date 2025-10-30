# Semantic Audio Diff Implementation Plan

**Based on**: "Semantic Audio Diff for Music Production" architecture document
**Status**: Design Phase
**Complexity**: High - Multi-year research project
**Priority**: Phase 3 (post-MVP)

---

## Executive Summary

The semantic audio diff system transforms OxVCS from a simple version control tool into an **intelligent audio production assistant** that:

1. **Understands** what changed (not just that something changed)
2. **Explains** changes in producer-friendly language ("Harshness increased" vs "spectral delta at 5kHz")
3. **Enables** intelligent audio merging (not just file-level conflicts)
4. **Validates** changes with objective quality metrics

**Key Insight**: Bifurcated strategy separating **metadata** (Logic Pro project settings) from **audio content** (WAV/AIFF files).

---

## Architecture Overview

### The Five-Layer Stack

```
┌─────────────────────────────────────────────────────────┐
│  Layer 5: User Interface & Visualization               │
│  - Comparative spectrograms                            │
│  - DTW alignment path display                          │
│  - Semantic annotations on timeline                    │
│  - A/B listening tools                                 │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 4: Semantic Translation                         │
│  - Feature → Semantic mapping (Table 1)                │
│  - ML-based classification                             │
│  - Production-centric lexicon                          │
│  - Integrated causal reporting                         │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Temporal Alignment (Audio Diff Engine)       │
│  - Dynamic Time Warping (DTW)                          │
│  - Segmental chunking                                  │
│  - Difference localization                             │
│  - Merge scaffolding                                   │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 2: Feature Extraction & Hashing                 │
│  - Perceptual audio hashing (pre-filter)               │
│  - MFCC, Chroma, Spectral features                     │
│  - Multi-dimensional feature vectors                   │
│  - Indexing (ANN/LSH)                                  │
└─────────────────────────────────────────────────────────┘
                            ↓
┌─────────────────────────────────────────────────────────┐
│  Layer 1: Metadata Parsing (Logic Pro Reverse Eng.)    │
│  - Binary format parsing                               │
│  - MIDI event extraction                               │
│  - Plugin parameter diffing                            │
│  - Automation curve comparison                         │
└─────────────────────────────────────────────────────────┘
```

---

## Phase-Based Implementation

### Phase 0: Foundation (MVP - Current)
**Timeline**: Weeks 1-4
**Goal**: Basic VCS functionality without semantic analysis

- ✓ Project initialization
- ✓ Binary file tracking (Oxen)
- ✓ Commit/rollback
- ✓ .oxenignore patterns
- ⚠ **No semantic diff** - just track changes

**Status**: Nearly complete (existing OxVCS architecture)

---

### Phase 1: Metadata Layer (3-6 months)
**Timeline**: Months 1-6 post-MVP
**Goal**: Understand WHAT changed in Logic Pro project

#### 1.1 Logic Pro Binary Parsing

**Challenge**: Proprietary `.logicx` format

**Approach**:
```rust
// OxVCS-CLI-Wrapper/src/logic_parser.rs

use std::path::Path;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LogicProjectData {
    pub tempo: f32,
    pub sample_rate: u32,
    pub key_signature: String,
    pub tracks: Vec<Track>,
    pub automation: Vec<AutomationCurve>,
    pub plugins: Vec<PluginInstance>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Track {
    pub id: String,
    pub name: String,
    pub track_type: TrackType,
    pub channel_strip: ChannelStrip,
    pub regions: Vec<Region>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChannelStrip {
    pub eq: Option<EQSettings>,
    pub compressor: Option<CompressorSettings>,
    pub reverb: Option<ReverbSettings>,
    pub volume: f32,
    pub pan: f32,
}

pub fn parse_logic_project(path: &Path) -> Result<LogicProjectData> {
    // Read projectData binary file
    let binary = std::fs::read(path.join("Alternatives/001/ProjectData"))?;

    // Parse binary format (reverse-engineered)
    // This is the hard part - requires extensive research
    let project = parse_binary_format(&binary)?;

    Ok(project)
}
```

**Research Required**:
- Reverse-engineer Logic Pro binary format
- Study existing work: [robertheaton.com/reverse-engineering-logic-pro](https://robertheaton.com/2017/07/17/reverse-engineering-logic-pro-synth-files/)
- Analyze `.logicx` package structure
- Extract MIDI, automation, plugin states

**Tools**:
- Hex editor (HexFiend, 010 Editor)
- Binary analysis (Kaitai Struct)
- Logic Pro SDK (if available)

#### 1.2 Structured Metadata Diff

```rust
// OxVCS-CLI-Wrapper/src/metadata_diff.rs

pub struct MetadataDiff {
    pub track_changes: Vec<TrackChange>,
    pub plugin_changes: Vec<PluginChange>,
    pub automation_changes: Vec<AutomationChange>,
    pub structural_changes: Vec<StructuralChange>,
}

#[derive(Debug)]
pub enum TrackChange {
    Added { track: Track },
    Removed { track_id: String },
    Renamed { old_name: String, new_name: String },
    ChannelStripChanged { track_id: String, changes: ChannelStripDiff },
}

#[derive(Debug)]
pub struct ChannelStripDiff {
    pub eq_changes: Vec<EQChange>,
    pub compressor_changes: Vec<CompressorChange>,
    pub volume_delta: Option<f32>,
    pub pan_delta: Option<f32>,
}

pub fn diff_metadata(
    version_a: &LogicProjectData,
    version_b: &LogicProjectData,
) -> MetadataDiff {
    let mut diff = MetadataDiff::default();

    // Compare tracks
    for track_b in &version_b.tracks {
        if let Some(track_a) = version_a.find_track(&track_b.id) {
            // Track exists in both - check for changes
            if let Some(changes) = diff_channel_strip(&track_a.channel_strip, &track_b.channel_strip) {
                diff.track_changes.push(TrackChange::ChannelStripChanged {
                    track_id: track_b.id.clone(),
                    changes,
                });
            }
        } else {
            // New track in B
            diff.track_changes.push(TrackChange::Added { track: track_b.clone() });
        }
    }

    // Compare plugins
    diff.plugin_changes = diff_plugins(&version_a.plugins, &version_b.plugins);

    // Compare automation
    diff.automation_changes = diff_automation(&version_a.automation, &version_b.automation);

    diff
}
```

**Output Example**:
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

---

### Phase 2: Audio Feature Extraction (6-12 months)
**Timeline**: Months 7-12 post-MVP
**Goal**: Analyze WHAT changed in audio files

#### 2.1 Perceptual Hashing (Pre-Filter)

```rust
// OxVCS-CLI-Wrapper/src/audio_hash.rs

use chromaprint::Fingerprinter;

pub struct AudioHash {
    pub hash: Vec<u32>,
    pub duration: f32,
}

pub fn compute_audio_hash(audio_path: &Path) -> Result<AudioHash> {
    // Use Chromaprint (AcoustID) for robust perceptual hashing
    let mut fingerprinter = Fingerprinter::new();

    // Load audio (use libsndfile or similar)
    let audio_data = load_audio_file(audio_path)?;

    // Compute fingerprint
    fingerprinter.start(audio_data.sample_rate, audio_data.channels)?;
    fingerprinter.feed(&audio_data.samples)?;
    fingerprinter.finish()?;

    let hash = fingerprinter.get_raw_fingerprint()?;

    Ok(AudioHash {
        hash,
        duration: audio_data.duration,
    })
}

pub fn hash_distance(hash_a: &AudioHash, hash_b: &AudioHash) -> f32 {
    // Hamming distance on hash bits
    let mut distance = 0;
    for (a, b) in hash_a.hash.iter().zip(&hash_b.hash) {
        distance += (a ^ b).count_ones();
    }

    // Normalize by hash length
    distance as f32 / (hash_a.hash.len() * 32) as f32
}

pub fn should_skip_diff(hash_a: &AudioHash, hash_b: &AudioHash) -> bool {
    // If perceptual distance < threshold, files are perceptually identical
    const THRESHOLD: f32 = 0.05; // 5% difference
    hash_distance(hash_a, hash_b) < THRESHOLD
}
```

**Dependencies**:
- `chromaprint` or `acoustid` (perceptual hashing)
- `symphonia` or `libsndfile` (audio loading)

#### 2.2 Feature Extraction

```python
# audio-diff-engine/feature_extraction.py

import librosa
import numpy as np

class AudioFeatureExtractor:
    """Extract multi-dimensional features for semantic comparison."""

    def __init__(self, sr=22050, hop_length=512):
        self.sr = sr
        self.hop_length = hop_length

    def extract_features(self, audio_path):
        """Extract comprehensive feature set from audio file."""

        # Load audio
        y, sr = librosa.load(audio_path, sr=self.sr)

        features = {
            # Timbral features
            'mfcc': librosa.feature.mfcc(y=y, sr=sr, n_mfcc=13),

            # Harmonic features
            'chroma': librosa.feature.chroma_cqt(y=y, sr=sr),

            # Spectral features
            'spectral_contrast': librosa.feature.spectral_contrast(y=y, sr=sr),
            'spectral_centroid': librosa.feature.spectral_centroid(y=y, sr=sr),
            'spectral_bandwidth': librosa.feature.spectral_bandwidth(y=y, sr=sr),

            # Temporal features
            'onset_strength': librosa.onset.onset_strength(y=y, sr=sr),
            'rms_energy': librosa.feature.rms(y=y),

            # Beat-synchronous features
            'tempo': librosa.beat.tempo(y=y, sr=sr)[0],
            'beat_frames': librosa.beat.beat_track(y=y, sr=sr)[1],
        }

        return features

    def extract_chunks(self, audio_path, chunk_duration=30, overlap=5):
        """
        Split audio into semantic chunks.

        Args:
            audio_path: Path to audio file
            chunk_duration: Duration of each chunk in seconds
            overlap: Overlap between chunks in seconds

        Returns:
            List of (start_time, end_time, features) tuples
        """
        y, sr = librosa.load(audio_path, sr=self.sr)

        # Detect beat grid for semantic chunking
        tempo, beat_frames = librosa.beat.beat_track(y=y, sr=sr)
        beat_times = librosa.frames_to_time(beat_frames, sr=sr)

        chunks = []
        hop = chunk_duration - overlap

        for start_time in np.arange(0, len(y) / sr, hop):
            end_time = start_time + chunk_duration

            # Extract chunk
            start_sample = int(start_time * sr)
            end_sample = int(end_time * sr)
            chunk_y = y[start_sample:end_sample]

            # Extract features for chunk
            chunk_features = self.extract_features_from_samples(chunk_y, sr)

            chunks.append((start_time, end_time, chunk_features))

        return chunks
```

**Dependencies**:
- `librosa` (audio analysis)
- `numpy`, `scipy` (numerical processing)
- `essentia` (alternative, more comprehensive)

---

### Phase 3: Temporal Alignment (12-18 months)
**Timeline**: Months 13-18 post-MVP
**Goal**: Handle time-warping, tempo changes, time-stretching

#### 3.1 Dynamic Time Warping

```python
# audio-diff-engine/dtw_alignment.py

import numpy as np
from dtw import accelerated_dtw
import fastdtw

class AudioAligner:
    """Align two audio versions using optimized DTW."""

    def align_features(self, features_a, features_b, metric='euclidean'):
        """
        Align two feature sequences using FastDTW.

        Returns:
            - alignment_path: List of (index_a, index_b) tuples
            - distance_map: Local alignment costs at each point
            - total_distance: Overall difference score
        """

        # Use FastDTW for speed (O(n) vs O(n²))
        distance, path = fastdtw.fastdtw(
            features_a,
            features_b,
            dist=metric
        )

        # Compute local distance map
        distance_map = self._compute_distance_map(features_a, features_b, path)

        return {
            'path': path,
            'distance_map': distance_map,
            'total_distance': distance,
        }

    def localize_differences(self, alignment_result, threshold=0.7):
        """
        Identify segments where audio diverged significantly.

        Returns:
            List of (start_time, end_time, difference_score) tuples
        """
        distance_map = alignment_result['distance_map']

        # Find regions where local distance > threshold
        divergent_regions = []
        in_region = False
        region_start = None

        for i, dist in enumerate(distance_map):
            if dist > threshold and not in_region:
                # Start of divergent region
                region_start = i
                in_region = True
            elif dist <= threshold and in_region:
                # End of divergent region
                divergent_regions.append((
                    region_start,
                    i,
                    np.mean(distance_map[region_start:i])
                ))
                in_region = False

        return divergent_regions

    def segment_based_alignment(self, audio_a, audio_b, segment_duration=10):
        """
        Align audio in segments for scalability.

        This breaks large files into manageable chunks for DTW.
        """
        # Extract features for both versions
        features_a = self.feature_extractor.extract_chunks(audio_a, segment_duration)
        features_b = self.feature_extractor.extract_chunks(audio_b, segment_duration)

        # Align each segment pair
        segment_alignments = []
        for (start_a, end_a, feat_a), (start_b, end_b, feat_b) in zip(features_a, features_b):
            alignment = self.align_features(feat_a, feat_b)
            segment_alignments.append({
                'time_range_a': (start_a, end_a),
                'time_range_b': (start_b, end_b),
                'alignment': alignment,
            })

        return segment_alignments
```

**Dependencies**:
- `dtw-python` or `fastdtw` (optimized DTW)
- `tslearn` (time series ML, includes DTW)

---

### Phase 4: Semantic Translation (18-24 months)
**Timeline**: Months 19-24 post-MVP
**Goal**: Translate features into producer-friendly language

#### 4.1 Feature-to-Semantic Mapping

```python
# audio-diff-engine/semantic_mapper.py

class SemanticMapper:
    """Map acoustic features to semantic descriptors."""

    # Table 1 from document
    FEATURE_TO_SEMANTIC = {
        'high_mid_boost': {
            'feature': 'spectral_contrast',
            'frequency_range': (2000, 6000),
            'threshold': 3.0,  # dB
            'semantic': 'Harshness, Abrasiveness',
            'likely_cause': 'Excessive high-mid EQ boost or resonant filter',
        },
        'mfcc_delta': {
            'feature': 'mfcc',
            'threshold': 0.5,  # normalized distance
            'semantic': 'Timbre Change, Instrumentation Switch',
            'likely_cause': 'Swapped synth preset, heavy distortion/saturation',
        },
        'low_mid_boost': {
            'feature': 'spectral_centroid',
            'frequency_range': (180, 300),
            'threshold': 2.5,  # dB
            'semantic': 'Muddy, Lack of Clarity, Uncontrolled Low End',
            'likely_cause': 'Low-mid frequency build-up, inadequate HPF',
        },
        'chroma_shift': {
            'feature': 'chroma',
            'threshold': 0.3,
            'semantic': 'Tonal/Harmonic Shift',
            'likely_cause': 'Pitch-shifting, chord progression change',
        },
        'rms_decrease': {
            'feature': 'rms_energy',
            'threshold': -3.0,  # dB
            'semantic': 'Volume/Dynamic Reduction',
            'likely_cause': 'Compression, limiting, volume automation',
        },
    }

    def analyze_difference(self, features_a, features_b, metadata_diff=None):
        """
        Analyze feature differences and generate semantic report.

        Args:
            features_a, features_b: Extracted audio features
            metadata_diff: Optional metadata changes from Layer 1

        Returns:
            Semantic report with causal attribution
        """
        report = []

        for rule_name, rule in self.FEATURE_TO_SEMANTIC.items():
            # Compute delta for this feature
            delta = self._compute_feature_delta(
                features_a[rule['feature']],
                features_b[rule['feature']],
                rule
            )

            if abs(delta) > rule['threshold']:
                # Significant change detected
                change = {
                    'type': rule['semantic'],
                    'magnitude': delta,
                    'likely_cause': rule['likely_cause'],
                    'confidence': 'high' if metadata_diff else 'medium',
                }

                # Check if metadata explains this
                if metadata_diff:
                    attribution = self._attribute_to_metadata(
                        rule_name,
                        delta,
                        metadata_diff
                    )
                    if attribution:
                        change['confirmed_cause'] = attribution
                        change['confidence'] = 'very_high'

                report.append(change)

        return report

    def _attribute_to_metadata(self, feature_change, delta, metadata_diff):
        """
        Establish causal link between audio change and metadata change.

        This is the key insight from the document: correlating
        audio analysis with known parameter changes.
        """
        # Example: If we detect high-mid boost in audio...
        if feature_change == 'high_mid_boost':
            # ...check if metadata shows EQ change in that range
            for change in metadata_diff.plugin_changes:
                if change.plugin_type == 'EQ':
                    for band in change.eq_changes:
                        if 2000 <= band.frequency <= 6000 and band.gain_delta > 2.0:
                            return f"Caused by +{band.gain_delta:.1f} dB EQ boost at {band.frequency} Hz on track '{change.track_name}'"

        # ... similar logic for other changes

        return None

    def generate_human_report(self, semantic_analysis):
        """Generate natural language report for producer."""

        report_lines = []

        for change in semantic_analysis:
            if change['confidence'] == 'very_high':
                # We know exactly what caused it
                report_lines.append(
                    f"⚠️  {change['type']} detected: {change['confirmed_cause']}"
                )
            elif change['confidence'] == 'high':
                report_lines.append(
                    f"ℹ️  {change['type']} detected (likely: {change['likely_cause']})"
                )
            else:
                report_lines.append(
                    f"❓ {change['type']} detected (unattributed change)"
                )

        return "\n".join(report_lines)
```

#### 4.2 Machine Learning for Complex Changes

```python
# audio-diff-engine/ml_classifier.py

import tensorflow as tf
from transformers import AutoFeatureExtractor, AutoModelForAudioClassification

class MLAudioClassifier:
    """Use pre-trained models for high-level change detection."""

    def __init__(self):
        # Load pre-trained audio classification model
        # (e.g., from Hugging Face)
        self.model_name = "MIT/ast-finetuned-audioset-10-10-0.4593"
        self.feature_extractor = AutoFeatureExtractor.from_pretrained(self.model_name)
        self.model = AutoModelForAudioClassification.from_pretrained(self.model_name)

    def classify_instrumentation_change(self, audio_a, audio_b):
        """
        Detect high-level instrumentation changes using transfer learning.

        Example output: "Transitioned from acoustic piano to synthesizer leads"
        """
        # Extract embeddings from both versions
        embedding_a = self._get_embedding(audio_a)
        embedding_b = self._get_embedding(audio_b)

        # Compute embedding distance
        distance = np.linalg.norm(embedding_a - embedding_b)

        if distance > threshold:
            # Classify both versions
            label_a = self._classify(audio_a)
            label_b = self._classify(audio_b)

            if label_a != label_b:
                return f"Instrumentation change: {label_a} → {label_b}"

        return None

    def infer_production_effects(self, audio_a, audio_b):
        """
        Infer signal processing applied (reverb, delay, compression).

        Uses semantic music production ML models.
        """
        # This requires specialized models trained on production parameters
        # Research area: Semantic Music Production

        # Analyze spatial/temporal characteristics
        spatial_change = self._analyze_spatial_features(audio_a, audio_b)
        temporal_change = self._analyze_temporal_features(audio_a, audio_b)

        effects = []

        if spatial_change['reverb_increase'] > threshold:
            effects.append({
                'type': 'reverb',
                'parameter': 'decay_time',
                'change': f"+{spatial_change['reverb_increase']:.1f}s",
                'confidence': 0.8,
            })

        return effects
```

**Dependencies**:
- `transformers` (Hugging Face models)
- `tensorflow` or `pytorch` (deep learning)
- Pre-trained models: AST, Wav2Vec2, CLAP

---

### Phase 5: Visualization & UX (24-30 months)
**Timeline**: Months 25-30 post-MVP
**Goal**: Producer-friendly interface

#### 5.1 Spectrogram Diff Viewer

```swift
// OxVCS-App/Sources/Views/SemanticDiffView.swift

import SwiftUI
import Accelerate

struct SemanticDiffView: View {
    let versionA: AudioFile
    let versionB: AudioFile
    let diffResult: SemanticDiffResult

    @State private var playbackPosition: TimeInterval = 0
    @State private var selectedSegment: DiffSegment?

    var body: some View {
        VStack(spacing: 0) {
            // Comparative spectrogram
            ComparativeSpectrogramView(
                versionA: versionA,
                versionB: versionB,
                differenceMap: diffResult.differenceMap,
                playbackPosition: $playbackPosition
            )
            .frame(height: 300)

            // DTW alignment path
            DTWAlignmentView(
                alignmentPath: diffResult.alignmentPath,
                differenceScores: diffResult.localDistances
            )
            .frame(height: 100)

            // Semantic annotations timeline
            SemanticTimelineView(
                semanticChanges: diffResult.semanticReport,
                playbackPosition: $playbackPosition,
                selectedSegment: $selectedSegment
            )
            .frame(height: 150)

            // A/B comparison controls
            ABComparisonControls(
                versionA: versionA,
                versionB: versionB,
                playbackPosition: $playbackPosition,
                selectedSegment: selectedSegment
            )
        }
    }
}

struct ComparativeSpectrogramView: View {
    let versionA: AudioFile
    let versionB: AudioFile
    let differenceMap: [[Float]]
    @Binding var playbackPosition: TimeInterval

    var body: some View {
        Canvas { context, size in
            // Draw Version A spectrogram
            drawSpectrogram(context: context, audio: versionA, rect: CGRect(x: 0, y: 0, width: size.width, height: size.height / 3))

            // Draw Version B spectrogram
            drawSpectrogram(context: context, audio: versionB, rect: CGRect(x: 0, y: size.height / 3, width: size.width, height: size.height / 3))

            // Draw difference map (color-coded overlay)
            drawDifferenceOverlay(context: context, map: differenceMap, rect: CGRect(x: 0, y: 2 * size.height / 3, width: size.width, height: size.height / 3))

            // Draw playback cursor
            drawPlaybackCursor(context: context, position: playbackPosition, size: size)
        }
    }

    private func drawDifferenceOverlay(context: GraphicsContext, map: [[Float]], rect: CGRect) {
        // Color-code differences:
        // Blue = content only in A
        // Red = content only in B
        // Green = similar content
        // Yellow/Orange = spectral shifts

        for (timeIdx, timeSlice) in map.enumerated() {
            for (freqIdx, delta) in timeSlice.enumerated() {
                let x = rect.minX + CGFloat(timeIdx) / CGFloat(map.count) * rect.width
                let y = rect.minY + CGFloat(freqIdx) / CGFloat(timeSlice.count) * rect.height

                let color = deltaToColor(delta)
                let rect = CGRect(x: x, y: y, width: 2, height: 2)
                context.fill(Path(rect), with: .color(color))
            }
        }
    }

    private func deltaToColor(_ delta: Float) -> Color {
        if delta > 0.5 {
            return .red.opacity(Double(delta))  // Added in B
        } else if delta < -0.5 {
            return .blue.opacity(Double(abs(delta)))  // Removed from B
        } else if abs(delta) > 0.2 {
            return .yellow.opacity(Double(abs(delta)))  // Changed
        } else {
            return .green.opacity(0.3)  // Similar
        }
    }
}

struct SemanticTimelineView: View {
    let semanticChanges: [SemanticChange]
    @Binding var playbackPosition: TimeInterval
    @Binding var selectedSegment: DiffSegment?

    var body: some View {
        GeometryReader { geometry in
            ZStack(alignment: .topLeading) {
                // Timeline background
                Rectangle()
                    .fill(Color.gray.opacity(0.1))

                // Semantic annotations
                ForEach(semanticChanges) { change in
                    SemanticAnnotationBox(
                        change: change,
                        totalDuration: maxDuration,
                        width: geometry.size.width,
                        isSelected: selectedSegment?.id == change.segment.id
                    )
                    .onTapGesture {
                        selectedSegment = change.segment
                    }
                }
            }
        }
    }
}

struct SemanticAnnotationBox: View {
    let change: SemanticChange
    let totalDuration: TimeInterval
    let width: CGFloat
    let isSelected: Bool

    var body: some View {
        let x = CGFloat(change.startTime / totalDuration) * width
        let w = CGFloat((change.endTime - change.startTime) / totalDuration) * width

        VStack(alignment: .leading, spacing: 2) {
            Text(change.type)
                .font(.caption.bold())
                .foregroundColor(.white)

            if let cause = change.confirmedCause {
                Text(cause)
                    .font(.caption2)
                    .foregroundColor(.white.opacity(0.9))
            }
        }
        .padding(4)
        .background(changeColor.opacity(isSelected ? 0.9 : 0.7))
        .cornerRadius(4)
        .border(isSelected ? Color.white : Color.clear, width: 2)
        .frame(width: w)
        .position(x: x + w/2, y: 40)
    }

    var changeColor: Color {
        switch change.confidence {
        case .veryHigh: return .red
        case .high: return .orange
        case .medium: return .yellow
        case .low: return .gray
        }
    }
}

struct ABComparisonControls: View {
    let versionA: AudioFile
    let versionB: AudioFile
    @Binding var playbackPosition: TimeInterval
    let selectedSegment: DiffSegment?

    @State private var currentVersion: Version = .A
    @State private var isPlaying: Bool = false

    enum Version {
        case A, B
    }

    var body: some View {
        HStack(spacing: 20) {
            // A/B Toggle
            Button(action: toggleVersion) {
                HStack {
                    Text("Version:")
                    Text(currentVersion == .A ? "A" : "B")
                        .font(.title2.bold())
                        .foregroundColor(currentVersion == .A ? .blue : .red)
                }
                .padding()
                .background(Color.gray.opacity(0.2))
                .cornerRadius(8)
            }
            .keyboardShortcut("t", modifiers: [])

            // Play/Pause
            Button(action: togglePlayback) {
                Image(systemName: isPlaying ? "pause.fill" : "play.fill")
                    .font(.title)
            }
            .keyboardShortcut(.space, modifiers: [])

            // Play selected segment only
            if let segment = selectedSegment {
                Button("Play Segment Only") {
                    playSegment(segment)
                }
            }

            Spacer()

            // PESQ/PEASS quality score
            if let qualityScore = diffResult.qualityScore {
                VStack(alignment: .trailing) {
                    Text("Quality Score")
                        .font(.caption)
                    Text(String(format: "%.2f", qualityScore))
                        .font(.title2.bold())
                        .foregroundColor(qualityScoreColor(qualityScore))
                }
            }
        }
        .padding()
    }

    private func toggleVersion() {
        currentVersion = currentVersion == .A ? .B : .A
        // Switch audio output
        audioPlayer.switchTo(currentVersion == .A ? versionA : versionB)
    }
}
```

---

## Implementation Dependencies

### Core Libraries

**Rust/Swift Components**:
```toml
# OxVCS-CLI-Wrapper/Cargo.toml

[dependencies]
# Existing
oxen = "0.x.x"
serde = { version = "1.0", features = ["derive"] }
anyhow = "1.0"

# New for semantic diff
symphonia = "0.5"  # Audio decoding
hound = "3.5"  # WAV reading/writing
chromaprint = "0.x"  # Perceptual hashing
```

**Python Audio Engine**:
```python
# audio-diff-engine/requirements.txt

# Audio analysis
librosa==0.10.0
essentia==2.1b6
numpy==1.24.0
scipy==1.11.0

# DTW
dtw-python==1.3.0
fastdtw==0.3.4
tslearn==0.6.0

# ML models
transformers==4.30.0
torch==2.0.0
tensorflow==2.13.0

# Visualization
matplotlib==3.7.0
plotly==5.14.0

# Quality metrics
pypesq==1.2.4
```

---

## Challenges & Risks

### Technical Challenges

1. **Logic Pro Format Reverse Engineering** (HIGH RISK)
   - Proprietary binary format
   - No official SDK/API
   - Format may change with Logic updates
   - **Mitigation**: Focus on XML-based metadata where possible, community research

2. **DTW Computational Cost** (MEDIUM RISK)
   - O(n²) complexity for standard DTW
   - Large audio files (10+ minutes) may timeout
   - **Mitigation**: Use FastDTW, Segmental DTW, aggressive chunking

3. **Feature-to-Semantic Mapping Accuracy** (MEDIUM RISK)
   - Subjective mapping (what is "muddy"?)
   - Producer preferences vary
   - **Mitigation**: Extensive user testing, customizable lexicon

4. **ML Model Training Data** (MEDIUM RISK)
   - Need labeled dataset of production changes
   - Expensive to create
   - **Mitigation**: Transfer learning from pre-trained models, active learning

### Resource Requirements

**Development Time**: 24-30 months (2-2.5 years)

**Team Size**: 3-5 developers
- 1 × Audio DSP engineer (Rust/Python)
- 1 × ML engineer (Python/TensorFlow)
- 1 × Swift UI developer
- 1 × Research engineer (reverse engineering)
- 1 × Project lead

**Compute Resources**:
- GPU for ML training/inference
- Storage for audio feature database
- ~10-20 TB for training data

---

## Phased Rollout Strategy

### MVP (Phase 0) - Ship First
**Features**: Basic VCS without semantic analysis
**Timeline**: Weeks 1-4
**Value**: Solves immediate version control need

### Enhanced Metadata (Phase 1) - Quick Win
**Features**: Parse & diff Logic Pro metadata
**Timeline**: Months 1-6
**Value**: "EQ changed +3 dB at 8kHz" reports

### Audio Hashing (Phase 2A) - Performance Gate
**Features**: Perceptual hashing pre-filter
**Timeline**: Months 7-9
**Value**: Fast "no change" detection

### Basic Feature Diff (Phase 2B) - Research Prototype
**Features**: MFCC/Chroma extraction, basic diff
**Timeline**: Months 10-12
**Value**: Proof of concept for audio analysis

### DTW Alignment (Phase 3) - Core Innovation
**Features**: Temporal alignment, segment localization
**Timeline**: Months 13-18
**Value**: Enables audio merging

### Semantic Translation (Phase 4) - Production Ready
**Features**: Full feature-to-semantic mapping, ML
**Timeline**: Months 19-24
**Value**: Producer-friendly reports

### Visualization (Phase 5) - Polish
**Features**: Interactive spectrograms, A/B comparison
**Timeline**: Months 25-30
**Value**: Professional-grade UX

---

## Success Metrics

### Technical Metrics
- **Accuracy**: >90% of metadata changes correctly detected
- **Precision**: <5% false positive rate on semantic changes
- **Performance**: <10s processing time for 5-minute audio
- **Quality**: PESQ/PEASS scores correlate >0.8 with human ratings

### User Metrics
- **Adoption**: 70%+ of users enable semantic diff
- **Satisfaction**: 8+/10 rating on usefulness
- **Time Saved**: 50%+ reduction in manual A/B comparison time
- **Confidence**: 80%+ trust in merge suggestions

---

## Alternatives & Trade-offs

### Simplified Approach (Faster, Less Accurate)
**Skip**: ML models, complex DTW
**Keep**: Metadata diff, basic feature extraction
**Timeline**: 6-12 months
**Trade-off**: Less accurate, no complex merging

### Cloud-Based Processing (Offload Compute)
**Architecture**: Local client + cloud API
**Pros**: Unlimited compute, easier ML updates
**Cons**: Privacy concerns, requires network
**Timeline**: Similar, but different skills needed

### Plugin-Based (Extend Existing Tools)
**Integrate**: Build as Logic Pro plugin or third-party DAW tool
**Pros**: Leverage existing ecosystems
**Cons**: Limited by plugin APIs, no VCS integration

---

## Conclusion

The semantic audio diff system is **ambitious but feasible**. It transforms OxVCS from a basic file tracker into an intelligent production assistant.

**Recommendation**:
1. **Ship MVP (Phase 0)** first - solve basic VCS need
2. **Prototype Phase 1** (metadata diff) - validate architecture
3. **Evaluate** after 6 months - decide if full semantic system is justified
4. **Iterate** based on user feedback and research progress

This is a **multi-year research project** that could define the future of audio version control. The document provides a solid architectural foundation - now it needs careful, incremental implementation with continuous validation.

---

**Next Steps**:
1. Review this plan with stakeholders
2. Prioritize which phases align with product vision
3. Allocate resources for Phase 1 prototype
4. Begin Logic Pro binary format research
5. Build proof-of-concept metadata parser

*Status: Design Complete, Awaiting Implementation Decision*
