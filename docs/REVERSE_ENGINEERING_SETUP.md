# Reverse Engineering Setup Guide

**Goal**: Parse Logic Pro `.logicx` binary format to extract project metadata
**Timeline**: 3-6 weeks
**Priority**: CRITICAL - Blocks Phase 1 completion

---

## Prerequisites Checklist

### ✅ Already Have
- [x] Parser framework implemented (`binary_parser.rs`)
- [x] Data structures defined (`project_data.rs`)
- [x] Diff engine ready to use parsed data
- [x] Test suite ready for validation

### 🔴 Required to Start

#### 1. **macOS Environment**
**Required**: macOS 14.0+ (Sonoma or later)
**Why**: Logic Pro only runs on macOS

**Current Status**: Available (you're on macOS)

#### 2. **Logic Pro 11.x**
**Required**: Logic Pro 11.0 or later
**Why**: Need to create and modify test projects

**Check if installed**:
```bash
# Check if Logic Pro is installed
ls -la /Applications/Logic\ Pro.app

# Check version
defaults read /Applications/Logic\ Pro.app/Contents/Info.plist CFBundleShortVersionString
```

**Cost**: $199 one-time purchase from Mac App Store
**Alternative**: 90-day free trial (if available)

#### 3. **Hex Editor**
**Recommended**: HexFiend (free, macOS-native)

**Install**:
```bash
# Using Homebrew
brew install --cask hexfiend

# Or download from:
# https://hexfiend.com/
```

**Alternatives**:
- 010 Editor ($50, powerful templates)
- ImHex (free, open source)
- Standard `hexdump` command (built-in)

---

## Test Project Creation Strategy

### Phase 1: Minimal Test Projects

Create pairs of Logic Pro projects with **single, known changes**. This isolates the binary bytes that represent each parameter.

#### Test Set 1: Global Parameters

**1a. Tempo Change**
```
Project: tempo_120.logicx
- Tempo: 120 BPM
- Sample Rate: 48000 Hz
- Key: C Major
- Time Signature: 4/4
- 1 empty MIDI track

Project: tempo_128.logicx
- Tempo: 128 BPM (ONLY CHANGE)
- Everything else identical
```

**1b. Sample Rate Change**
```
Project: sr_44100.logicx
- Sample Rate: 44100 Hz
- Tempo: 120 BPM
- Everything else identical

Project: sr_48000.logicx
- Sample Rate: 48000 Hz (ONLY CHANGE)
```

**1c. Key Signature Change**
```
Project: key_c_major.logicx
- Key: C Major

Project: key_d_major.logicx
- Key: D Major (ONLY CHANGE)
```

**1d. Time Signature Change**
```
Project: time_44.logicx
- Time Signature: 4/4

Project: time_34.logicx
- Time Signature: 3/4 (ONLY CHANGE)
```

#### Test Set 2: Track-Level Changes

**2a. Track Added**
```
Project: 0_tracks.logicx
- No tracks (empty project)

Project: 1_track.logicx
- 1 MIDI track named "Test Track"
```

**2b. Track Renamed**
```
Project: track_original.logicx
- Track name: "Original Name"

Project: track_renamed.logicx
- Track name: "New Name"
```

#### Test Set 3: Channel Strip Changes

**3a. Volume Change**
```
Project: volume_0db.logicx
- Track volume: 0.0 dB

Project: volume_plus3db.logicx
- Track volume: +3.0 dB
```

**3b. Pan Change**
```
Project: pan_center.logicx
- Track pan: Center (0.0)

Project: pan_left.logicx
- Track pan: 50% Left (-0.5)
```

**3c. EQ Change**
```
Project: eq_flat.logicx
- Channel EQ: All bands at 0 dB

Project: eq_8khz_boost.logicx
- Channel EQ: Band 3 (8 kHz) +3 dB
```

### Creating Test Projects

**Step-by-step**:

1. **Open Logic Pro**
2. **Create New Project**:
   - Empty Project
   - No tracks initially (or 1 MIDI track)
   - Set sample rate: 48000 Hz
   - Set tempo: 120 BPM
3. **Save As** `tempo_120.logicx`
4. **Close Logic Pro** (important!)
5. **Duplicate the project**:
   ```bash
   cp -R tempo_120.logicx tempo_128.logicx
   ```
6. **Open** `tempo_128.logicx` in Logic Pro
7. **Change ONLY the tempo** to 128 BPM
8. **Save and close**

Repeat for each test pair.

---

## Reverse Engineering Workflow

### Step 1: Binary Extraction

Extract the ProjectData binary from each project:

```bash
#!/bin/bash
# extract_project_data.sh

PROJECT_NAME=$1
OUTPUT_NAME=$2

# Extract ProjectData file
cp "${PROJECT_NAME}.logicx/Alternatives/001/ProjectData" \
   "binary_samples/${OUTPUT_NAME}.bin"

echo "Extracted: ${OUTPUT_NAME}.bin"
```

**Usage**:
```bash
mkdir binary_samples

./extract_project_data.sh tempo_120 tempo_120
./extract_project_data.sh tempo_128 tempo_128

# Now you have:
# - binary_samples/tempo_120.bin
# - binary_samples/tempo_128.bin
```

### Step 2: Binary Comparison

Use multiple methods to identify differences:

#### Method A: Hex Dump Diff

```bash
#!/bin/bash
# hex_diff.sh

hexdump -C binary_samples/tempo_120.bin > tempo_120.hex
hexdump -C binary_samples/tempo_128.bin > tempo_128.hex

# Diff with context
diff -u tempo_120.hex tempo_128.hex > tempo_diff.txt

# View differences
cat tempo_diff.txt
```

**Example Output**:
```
--- tempo_120.hex
+++ tempo_128.hex
@@ -25,7 +25,7 @@
 00000180  00 00 00 00 00 00 00 00  00 00 f0 42 00 00 00 00
-                                            ^^^^
+00000180  00 00 00 00 00 00 00 00  00 00 00 43 00 00 00 00
+                                            ^^^^
```

The changed bytes at offset 0x18B: `f0 42` → `00 43`

#### Method B: Visual Hex Comparison

Open both files in HexFiend:
```bash
open -a HexFiend binary_samples/tempo_120.bin
open -a HexFiend binary_samples/tempo_128.bin
```

Use HexFiend's "Compare" feature (File → Compare).

#### Method C: Binary Diff Tool

```bash
# Install vbindiff (visual binary diff)
brew install vbindiff

# Compare files
vbindiff binary_samples/tempo_120.bin binary_samples/tempo_128.bin
```

### Step 3: Hypothesis Formation

**Analyze the changed bytes**:

```python
#!/usr/bin/env python3
# analyze_float.py

import struct
import sys

def bytes_to_float(hex_string):
    """Convert hex bytes to float (little-endian)"""
    bytes_data = bytes.fromhex(hex_string.replace(' ', ''))
    return struct.unpack('<f', bytes_data)[0]

def bytes_to_uint(hex_string):
    """Convert hex bytes to unsigned int (little-endian)"""
    bytes_data = bytes.fromhex(hex_string.replace(' ', ''))
    return struct.unpack('<I', bytes_data)[0]

# Example: Changed bytes at offset 0x18B
before = "f0 42 00 00"  # From tempo_120.bin
after = "00 43 00 00"   # From tempo_128.bin

print("Float interpretation:")
print(f"  Before: {bytes_to_float(before)}")  # Should be ~120.0
print(f"  After: {bytes_to_float(after)}")    # Should be ~128.0

print("\nUint interpretation:")
print(f"  Before: {bytes_to_uint(before)}")
print(f"  After: {bytes_to_uint(after)}")
```

**Run**:
```bash
python3 analyze_float.py

# Output:
# Float interpretation:
#   Before: 120.0
#   After: 128.0
```

**Hypothesis Confirmed**: Offset 0x18B contains tempo as 4-byte float (little-endian).

### Step 4: Validation

Test the hypothesis with **multiple projects**:

```bash
# Create more tempo variations
tempo_60.logicx   # Tempo: 60 BPM
tempo_90.logicx   # Tempo: 90 BPM
tempo_140.logicx  # Tempo: 140 BPM
tempo_180.logicx  # Tempo: 180 BPM

# Extract and verify
for tempo in 60 90 120 128 140 180; do
    ./extract_project_data.sh tempo_${tempo} tempo_${tempo}

    # Read 4 bytes at offset 0x18B
    hexdump -s 0x18B -n 4 -e '4/1 "%02x " "\n"' \
        binary_samples/tempo_${tempo}.bin | \
        python3 -c "import sys, struct; print(struct.unpack('<f', bytes.fromhex(sys.stdin.read().strip()))[0])"
done

# Expected output:
# 60.0
# 90.0
# 120.0
# 128.0
# 140.0
# 180.0
```

If all match, offset is **confirmed**.

### Step 5: Documentation

**Format Specification**:

```markdown
# Logic Pro ProjectData Binary Format

## Header (bytes 0-15)
- 0x00-0x03: Magic number (unknown)
- 0x04-0x07: Format version (u32)
- 0x08-0x0B: File size (u32)
- 0x0C-0x0F: Reserved

## Global Settings (bytes 16-255)
- 0x10: Unknown
- ...
- 0x18B-0x18E: Tempo (f32, little-endian) ✅ CONFIRMED
- 0x18F-0x192: Sample rate (u32, little-endian) [HYPOTHESIS]
- ...
```

### Step 6: Implementation

Update `binary_parser.rs`:

```rust
// Replace placeholder with real implementation
fn parse_tempo_placeholder(binary: &[u8]) -> Result<f32> {
    // CONFIRMED: Tempo at offset 0x18B (395 decimal)
    const TEMPO_OFFSET: usize = 0x18B;

    parse_f32_at_offset(binary, TEMPO_OFFSET)
}

fn parse_sample_rate_placeholder(binary: &[u8]) -> Result<u32> {
    // HYPOTHESIS: Sample rate at offset 0x18F (399 decimal)
    const SAMPLE_RATE_OFFSET: usize = 0x18F;

    parse_u32_at_offset(binary, SAMPLE_RATE_OFFSET)
}
```

**Test**:
```rust
#[test]
fn test_parse_real_tempo() {
    let binary = std::fs::read("tests/fixtures/tempo_120.logicx/Alternatives/001/ProjectData").unwrap();
    let tempo = parse_tempo_placeholder(&binary).unwrap();
    assert_eq!(tempo, 120.0);
}
```

---

## Tools & Scripts

### Complete Toolkit Setup

```bash
# Create working directory
mkdir -p logic_reverse_engineering/{projects,binary_samples,hex_dumps,scripts,findings}
cd logic_reverse_engineering

# Install tools
brew install hexfiend vbindiff

# Create analysis scripts
cat > scripts/extract_all.sh << 'EOF'
#!/bin/bash
for project in projects/*.logicx; do
    name=$(basename "$project" .logicx)
    cp "$project/Alternatives/001/ProjectData" "binary_samples/${name}.bin"
    hexdump -C "binary_samples/${name}.bin" > "hex_dumps/${name}.hex"
    echo "Extracted: $name"
done
EOF

chmod +x scripts/extract_all.sh

# Create Python analysis tools
cat > scripts/analyze_bytes.py << 'EOF'
#!/usr/bin/env python3
import struct
import sys
import argparse

def analyze_offset(filename, offset, byte_count=4):
    with open(filename, 'rb') as f:
        f.seek(offset)
        data = f.read(byte_count)

    print(f"Bytes at offset 0x{offset:X} ({offset}):")
    print(f"  Hex: {' '.join(f'{b:02x}' for b in data)}")
    print(f"  Float (LE): {struct.unpack('<f', data)[0] if byte_count == 4 else 'N/A'}")
    print(f"  Float (BE): {struct.unpack('>f', data)[0] if byte_count == 4 else 'N/A'}")
    print(f"  Uint (LE): {struct.unpack('<I', data)[0] if byte_count == 4 else 'N/A'}")
    print(f"  Uint (BE): {struct.unpack('>I', data)[0] if byte_count == 4 else 'N/A'}")

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('file', help='Binary file to analyze')
    parser.add_argument('offset', type=lambda x: int(x, 0), help='Offset (decimal or hex)')
    parser.add_argument('--bytes', type=int, default=4, help='Number of bytes to read')
    args = parser.parse_args()

    analyze_offset(args.file, args.offset, args.bytes)
EOF

chmod +x scripts/analyze_bytes.py
```

---

## Workspace Organization

```
logic_reverse_engineering/
├── projects/               # Logic Pro test projects
│   ├── tempo_120.logicx
│   ├── tempo_128.logicx
│   ├── sr_44100.logicx
│   ├── sr_48000.logicx
│   └── ...
├── binary_samples/         # Extracted ProjectData files
│   ├── tempo_120.bin
│   ├── tempo_128.bin
│   └── ...
├── hex_dumps/              # Human-readable hex dumps
│   ├── tempo_120.hex
│   ├── tempo_128.hex
│   └── ...
├── scripts/                # Analysis scripts
│   ├── extract_all.sh
│   ├── analyze_bytes.py
│   └── compare_pair.sh
├── findings/               # Research notes
│   ├── tempo.md            # Tempo analysis results
│   ├── sample_rate.md      # Sample rate findings
│   ├── tracks.md           # Track structure notes
│   └── format_spec.md      # Running format specification
└── README.md               # Workspace overview
```

---

## Quick Start Checklist

### Day 1: Setup

- [ ] Install Logic Pro 11.x
- [ ] Install HexFiend: `brew install --cask hexfiend`
- [ ] Install vbindiff: `brew install vbindiff`
- [ ] Create workspace directory structure
- [ ] Copy analysis scripts from this document

### Day 2: Initial Projects

- [ ] Create `tempo_120.logicx` (baseline project)
- [ ] Create `tempo_128.logicx` (tempo change only)
- [ ] Extract both ProjectData files
- [ ] Run hex diff to find changed bytes
- [ ] Analyze changed bytes as float/int

### Day 3: Validation

- [ ] Create 4 more tempo variations (60, 90, 140, 180 BPM)
- [ ] Verify tempo offset is consistent
- [ ] Document findings in `findings/tempo.md`
- [ ] Update `binary_parser.rs` with real offset

### Week 1 Goals

- [ ] Confirm tempo offset
- [ ] Confirm sample rate offset
- [ ] Confirm key signature location
- [ ] Confirm time signature location
- [ ] Update parser with real implementations
- [ ] Write integration tests

---

## Expected Challenges

### 1. **Variable-Length Structures**

Some data (like track lists) may be variable-length:
```
Header:
  Track count: 3

Track array:
  Track 0: [data...]
  Track 1: [data...]
  Track 2: [data...]
```

**Solution**: Look for count fields, then iterate.

### 2. **Compressed or Encoded Data**

Some sections might be:
- Compressed (zlib, lz4)
- Encoded (Base64, custom)
- Encrypted (unlikely, but possible)

**Solution**: Look for magic numbers (e.g., `78 9C` = zlib).

### 3. **Version Differences**

Logic Pro 10.8 vs 11.0 might have different formats.

**Solution**: Detect version in header, handle accordingly.

### 4. **Proprietary Structures**

Apple may use custom data structures.

**Solution**: Document what we can parse, note what we can't.

---

## Success Metrics

### Week 1
- [ ] Parse tempo with 100% accuracy
- [ ] Parse sample rate with 100% accuracy
- [ ] Parse key signature (at least detect changes)
- [ ] Parse time signature

### Week 2-3
- [ ] Parse track count
- [ ] Parse track names
- [ ] Parse track types
- [ ] Parse basic channel strip settings

### Week 4-6
- [ ] Parse EQ settings (all bands)
- [ ] Parse compressor settings
- [ ] Parse volume/pan
- [ ] Parse region information

### Go/No-Go Criteria (Week 6)
**GO if**: Can parse 80%+ of high-value metadata
- ✅ Tempo, sample rate, key, time signature
- ✅ Track list, names, types
- ✅ Volume, pan
- ✅ At least basic EQ/compressor

**NO-GO if**: Cannot reliably parse even basic parameters
- ❌ Tempo/sample rate incorrect or unstable
- ❌ Cannot identify track structure
- ❌ Format too complex/obfuscated

**Pivot to**: FCP XML export approach

---

## Resources

### Tools
- **HexFiend**: https://hexfiend.com/
- **010 Editor**: https://www.sweetscape.com/010editor/
- **Kaitai Struct**: https://kaitai.io/ (binary format definition)

### References
- **Robert Heaton's Blog**: https://robertheaton.com/2017/07/17/reverse-engineering-logic-pro-synth-files/
- **Logic Pro Format Spec** (Library of Congress): https://www.loc.gov/preservation/digital/formats/fdd/fdd000640.shtml

### Community
- **Logic Pro User Forums**: https://discussions.apple.com/community/pro_applications/logic_pro
- **Reddit r/Logic_Studio**: Potential for collaboration

---

## Next Steps

**Ready to start?**

1. ✅ You have macOS (Darwin 25.0.0)
2. ❓ **Check if Logic Pro is installed**:
   ```bash
   ls -la /Applications/Logic\ Pro.app
   ```
3. ❓ **Install HexFiend** (if not already):
   ```bash
   brew install --cask hexfiend
   ```
4. ✅ Parser framework ready
5. ✅ Scripts documented above

**Let me know**:
- Do you have Logic Pro installed?
- Should I generate the analysis scripts as actual files?
- Want to start with a specific parameter (tempo, sample rate)?

I can walk through the first reverse engineering session step-by-step once you confirm you have Logic Pro.
