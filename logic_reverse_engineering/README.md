# Logic Pro Binary Format Reverse Engineering Workspace

**Status**: Ready to Start
**Logic Pro Version**: 11.2.2 ✅
**Goal**: Parse ProjectData binary to extract metadata

---

## Quick Start

### 1. Create Your First Test Project

```bash
# Open Logic Pro
open -a "Logic Pro"

# In Logic Pro:
# 1. File → New
# 2. Empty Project
# 3. Add 1 Software Instrument track
# 4. Set Tempo to 120 BPM (top center)
# 5. File → Save As...
#    - Name: tempo_120
#    - Location: This directory (logic_reverse_engineering/projects/)
# 6. Close Logic Pro
```

### 2. Duplicate and Modify

```bash
cd projects

# Duplicate the project
cp -R tempo_120.logicx tempo_128.logicx

# Open Logic Pro with the new project
open tempo_128.logicx

# In Logic Pro:
# 1. Change ONLY the tempo to 128 BPM
# 2. Save (Cmd+S)
# 3. Close Logic Pro
```

### 3. Extract and Compare

```bash
cd ../scripts

# Extract both projects
./extract_project_data.sh ../projects/tempo_120.logicx tempo_120
./extract_project_data.sh ../projects/tempo_128.logicx tempo_128

# Compare them
./compare_pair.sh tempo_120 tempo_128

# Look at the diff
less ../findings/tempo_120_vs_tempo_128_diff.txt
```

### 4. Analyze Changed Bytes

Find a changed offset from the diff (example: 0x18B), then:

```bash
# Analyze specific offset in both files
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 0x18B
./analyze_bytes.py analyze ../binary_samples/tempo_128.bin 0x18B

# Or scan for the tempo value
./analyze_bytes.py scan ../binary_samples/tempo_120.bin 120
./analyze_bytes.py scan ../binary_samples/tempo_128.bin 128
```

---

## Workspace Structure

```
logic_reverse_engineering/
├── README.md                   ← You are here
├── projects/                   ← Your Logic Pro test projects
│   ├── tempo_120.logicx       ← Create these
│   ├── tempo_128.logicx
│   └── ...
├── binary_samples/             ← Extracted ProjectData files
│   ├── tempo_120.bin          ← Auto-generated
│   ├── tempo_128.bin
│   └── ...
├── hex_dumps/                  ← Human-readable hex dumps
│   ├── tempo_120.hex
│   ├── tempo_128.hex
│   └── ...
├── findings/                   ← Your research notes and diffs
│   ├── tempo_120_vs_tempo_128_diff.txt
│   └── tempo.md               ← Document your findings here
└── scripts/                    ← Analysis tools (ready to use)
    ├── extract_project_data.sh
    ├── compare_pair.sh
    └── analyze_bytes.py
```

---

## Available Scripts

### `extract_project_data.sh`

Extract and analyze a Logic Pro project.

```bash
./extract_project_data.sh <project.logicx> <output_name>

# Example:
./extract_project_data.sh ../projects/tempo_120.logicx tempo_120
```

**Output**:
- `binary_samples/tempo_120.bin` - Raw ProjectData binary
- `hex_dumps/tempo_120.hex` - Hex dump for viewing

### `compare_pair.sh`

Compare two extracted binaries and show differences.

```bash
./compare_pair.sh <name1> <name2>

# Example:
./compare_pair.sh tempo_120 tempo_128
```

**Output**:
- `findings/tempo_120_vs_tempo_128_diff.txt` - Full diff report
- Console output showing first 20 differences

### `analyze_bytes.py`

Swiss-army knife for binary analysis.

**Analyze specific offset**:
```bash
./analyze_bytes.py analyze <file> <offset> [--bytes N]

# Examples:
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 0x18B
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 395  # Decimal offset
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 0x10 --bytes 16
```

**Scan for a value**:
```bash
./analyze_bytes.py scan <file> <value> [--type float|int]

# Examples:
./analyze_bytes.py scan ../binary_samples/tempo_120.bin 120 --type float
./analyze_bytes.py scan ../binary_samples/sr_48000.bin 48000 --type int
```

---

## Recommended Test Projects

Create these projects in order:

### Week 1: Global Parameters

| Project Name | Parameter | Value |
|--------------|-----------|-------|
| `tempo_120` | Tempo | 120 BPM (baseline) |
| `tempo_128` | Tempo | 128 BPM |
| `tempo_60` | Tempo | 60 BPM |
| `tempo_180` | Tempo | 180 BPM |
| `sr_44100` | Sample Rate | 44100 Hz |
| `sr_48000` | Sample Rate | 48000 Hz |
| `sr_96000` | Sample Rate | 96000 Hz |
| `key_c_major` | Key | C Major |
| `key_d_major` | Key | D Major |
| `key_a_minor` | Key | A Minor |
| `time_44` | Time Sig | 4/4 |
| `time_34` | Time Sig | 3/4 |
| `time_68` | Time Sig | 6/8 |

### Week 2: Track-Level

| Project Name | Change |
|--------------|--------|
| `0_tracks` | Empty project |
| `1_track` | 1 MIDI track |
| `2_tracks` | 2 MIDI tracks |
| `track_original` | Track named "Original" |
| `track_renamed` | Same track renamed to "Renamed" |

### Week 3: Channel Strip

| Project Name | Change |
|--------------|--------|
| `volume_0db` | Track volume at 0 dB |
| `volume_plus3db` | Track volume at +3 dB |
| `volume_minus6db` | Track volume at -6 dB |
| `pan_center` | Pan at center |
| `pan_left50` | Pan 50% left |
| `pan_right50` | Pan 50% right |

---

## Workflow Example

Let's reverse-engineer the **tempo** parameter:

```bash
cd scripts

# 1. Extract baseline project
./extract_project_data.sh ../projects/tempo_120.logicx tempo_120

# 2. Extract modified project
./extract_project_data.sh ../projects/tempo_128.logicx tempo_128

# 3. Compare them
./compare_pair.sh tempo_120 tempo_128

# Output shows:
# --- tempo_120.hex
# +++ tempo_128.hex
# @@ -50,7 +50,7 @@
# -00000310  ... 00 00 f0 42 00 00 ...
# +00000310  ... 00 00 00 43 00 00 ...

# 4. Analyze the changed bytes at offset 0x312
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 0x312

# Output:
# Float (LE):     120.0  ← This looks right!

./analyze_bytes.py analyze ../binary_samples/tempo_128.bin 0x312

# Output:
# Float (LE):     128.0  ← Confirmed!

# 5. Validate with more projects
for tempo in 60 90 140 180; do
    ./extract_project_data.sh ../projects/tempo_${tempo}.logicx tempo_${tempo}
    ./analyze_bytes.py analyze ../binary_samples/tempo_${tempo}.bin 0x312
done

# All should show the correct tempo values

# 6. Document findings
cat > ../findings/tempo.md << 'EOF'
# Tempo Parameter

**Offset**: 0x312 (786 decimal)
**Type**: f32 (4-byte float, little-endian)
**Range**: 40.0 - 300.0 BPM (typical)

## Validation
- ✅ tempo_60.logicx:  60.0
- ✅ tempo_90.logicx:  90.0
- ✅ tempo_120.logicx: 120.0
- ✅ tempo_128.logicx: 128.0
- ✅ tempo_140.logicx: 140.0
- ✅ tempo_180.logicx: 180.0

**Status**: CONFIRMED
EOF

# 7. Update parser
# Edit: ../../Auxin-CLI-Wrapper/src/logic_parser/binary_parser.rs
# Replace: parse_tempo_placeholder()
# With:    const TEMPO_OFFSET: usize = 0x312;
```

---

## Tips & Tricks

### Quickly View Hex at Offset

```bash
# Show 4 bytes at offset 0x312
hexdump -s 0x312 -n 4 -C ../binary_samples/tempo_120.bin
```

### Extract String

```bash
# Extract 32 bytes starting at offset 0x100 as string
hexdump -s 0x100 -n 32 -e '32/1 "%_p" "\n"' ../binary_samples/tempo_120.bin
```

### Binary Diff (Visual)

```bash
# Install if needed
brew install vbindiff

# Visual comparison
vbindiff ../binary_samples/tempo_120.bin ../binary_samples/tempo_128.bin
```

### Batch Extract All Projects

```bash
cd scripts
for project in ../projects/*.logicx; do
    name=$(basename "$project" .logicx)
    ./extract_project_data.sh "$project" "$name"
done
```

---

## Next Steps

1. **Create test projects** (see recommended list above)
2. **Extract binaries** for all projects
3. **Compare pairs** to find changed bytes
4. **Analyze offsets** to identify data types
5. **Validate** with multiple projects
6. **Document** findings in `findings/*.md`
7. **Update parser** in `../../Auxin-CLI-Wrapper/src/logic_parser/binary_parser.rs`
8. **Write tests** in `../../Auxin-CLI-Wrapper/tests/`

---

## Resources

- **Setup Guide**: `../docs/REVERSE_ENGINEERING_SETUP.md`
- **Parser Code**: `../Auxin-CLI-Wrapper/src/logic_parser/binary_parser.rs`
- **Data Structures**: `../Auxin-CLI-Wrapper/src/logic_parser/project_data.rs`

---

**Ready to start?** Create your first test project pair (tempo_120 and tempo_128)!
