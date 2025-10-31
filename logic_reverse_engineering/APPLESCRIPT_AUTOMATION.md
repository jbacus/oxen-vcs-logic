# AppleScript Automation for Logic Pro Test Projects

**Purpose**: Automate creation of Logic Pro test projects for binary reverse engineering
**Status**: Ready to use
**Time Saved**: ~2 hours of manual work

---

## Quick Start

### One-Time Setup (5 minutes)

**1. Enable Accessibility Permissions**

```bash
# Open System Preferences
open "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
```

Then:
- Click the lock icon to make changes
- Click the "+" button
- Add **Terminal** (or Script Editor if using that)
- Check the box next to Terminal

**2. Make Scripts Executable**

```bash
cd logic_reverse_engineering/scripts
chmod +x *.sh *.applescript
```

**3. Create Template Project (Manual - 2 minutes)**

```bash
# This will guide you through creating the template
./automated_tempo_workflow.sh
```

Or manually:
1. Open Logic Pro
2. File → New → Empty Project
3. Add 1 Software Instrument track (optional)
4. Set tempo to 120 BPM
5. Save As: `template_120.logicx` in `projects/` directory
6. Close Logic Pro

---

## Automated Workflows

### Option 1: Fully Automated (Recommended)

Creates and configures all tempo test projects automatically:

```bash
cd scripts
./automated_tempo_workflow.sh
```

**What it does**:
1. Duplicates template for each tempo variant
2. Opens each project in Logic Pro
3. Sets the tempo automatically
4. Saves and closes
5. Repeats for all projects

**Time**: ~5 minutes (mostly waiting for Logic Pro)

**Projects created**:
- tempo_60.logicx (60 BPM)
- tempo_90.logicx (90 BPM)
- tempo_120.logicx (120 BPM)
- tempo_128.logicx (128 BPM)
- tempo_140.logicx (140 BPM)
- tempo_180.logicx (180 BPM)

### Option 2: Semi-Automated

Duplicates projects, you set tempo manually:

```bash
cd scripts
./batch_create_projects.sh
```

**What it does**:
1. Duplicates template for all test cases
2. Prompts you to open each one
3. You manually set the tempo
4. You save and close
5. Repeat

**Time**: ~10 minutes
**Benefit**: More control, less reliance on UI automation

### Option 3: Manual with Helpers

Use individual scripts as needed:

```bash
# Set tempo in currently open project
osascript set_tempo_in_project.applescript 128

# Or use direct AppleScript
osascript -e 'tell application "Logic Pro" to tell front document to set tempo to 128'
```

---

## After Project Creation

### Extract All Binaries

```bash
cd scripts
./extract_all.sh
```

This extracts ProjectData from all .logicx projects and creates hex dumps.

### Compare Two Projects

```bash
./compare_pair.sh tempo_120 tempo_128
```

Shows differences between two projects.

---

## Script Reference

### `automated_tempo_workflow.sh` ⭐ Recommended

**Purpose**: Fully automated test project creation
**Usage**: `./automated_tempo_workflow.sh`
**Requires**: Template project, accessibility permissions
**Output**: 6 tempo test projects, all configured

### `batch_create_projects.sh`

**Purpose**: Duplicate template for manual configuration
**Usage**: `./batch_create_projects.sh`
**Requires**: Template project
**Output**: Duplicated projects (you set parameters manually)

### `set_tempo_in_project.applescript`

**Purpose**: Set tempo in currently open Logic Pro project
**Usage**: `osascript set_tempo_in_project.applescript <tempo>`
**Example**: `osascript set_tempo_in_project.applescript 128`
**Requires**: Project open in Logic Pro

### `extract_all.sh`

**Purpose**: Batch extract ProjectData from all projects
**Usage**: `./extract_all.sh`
**Output**: binary_samples/*.bin and hex_dumps/*.hex

### `create_test_project.applescript`

**Purpose**: Create project from scratch (experimental)
**Usage**: `osascript create_test_project.applescript <name> <tempo> <sr> <key> <time>`
**Status**: May not work reliably - use template approach instead

---

## Creating Different Test Projects

### Tempo Variations (Automated)

Use `automated_tempo_workflow.sh` - already configured for:
- 60, 90, 120, 128, 140, 180 BPM

### Sample Rate Variations (Manual)

Sample rate must be set before project creation:

```bash
# 1. Set Logic Pro sample rate:
#    Logic Pro > Preferences > Audio > General > Sample Rate

# 2. Create project with that sample rate
#    File > New > Empty Project

# 3. Save as sr_44100.logicx (or sr_48000, sr_96000)

# 4. Change sample rate in preferences

# 5. Repeat for each rate
```

**Sample rates to test**:
- 44100 Hz
- 48000 Hz
- 96000 Hz

### Key Signature Variations (Manual)

```bash
# 1. Duplicate template
cp -R projects/template_120.logicx projects/key_c_major.logicx

# 2. Open in Logic Pro
open projects/key_c_major.logicx

# 3. Set key signature:
#    - Press 'K' to open Key Signature dialog
#    - Select key
#    - Click OK

# 4. Save and close
```

**Keys to test**:
- C Major
- D Major
- A Minor
- F# Minor

### Time Signature Variations (Manual)

```bash
# 1. Duplicate template
cp -R projects/template_120.logicx projects/time_34.logicx

# 2. Open in Logic Pro
open projects/time_34.logicx

# 3. Set time signature:
#    - Option+T to open Time Signature dialog
#    - Set to 3/4
#    - Click OK

# 4. Save and close
```

**Time signatures to test**:
- 4/4
- 3/4
- 6/8
- 5/4

---

## Troubleshooting

### "Operation not permitted" Error

**Cause**: Accessibility permissions not granted

**Solution**:
1. System Preferences → Security & Privacy → Privacy tab
2. Select "Accessibility" from left sidebar
3. Click lock icon to make changes
4. Click "+" and add Terminal
5. Restart Terminal

### Scripts Don't Open Logic Pro

**Check**:
```bash
# Verify Logic Pro is installed
ls -la /Applications/Logic\ Pro.app

# Try opening manually
open -a "Logic Pro"
```

### Tempo Not Setting Correctly

**Try manual fallback**:
1. Open project in Logic Pro
2. Click tempo display (top center)
3. Type new tempo
4. Press Enter
5. Save

### Projects Not Saving to Correct Location

**Verify paths**:
```bash
cd scripts
pwd  # Should be .../logic_reverse_engineering/scripts

ls ../projects/  # Should show your projects
```

### Logic Pro Dialogs Interfere

**Disable startup dialogs**:
- Logic Pro → Preferences → General
- Uncheck "Show startup dialog"
- Uncheck "Show tips on startup"

### AppleScript Timing Issues

Scripts include `delay` commands, but Logic Pro might be slower on your machine.

**Adjust delays** in `automated_tempo_workflow.sh`:
```bash
# Change this:
sleep 5

# To this:
sleep 8  # Increase if Logic Pro opens slowly
```

---

## Advanced Usage

### Create Custom Tempo List

Edit `automated_tempo_workflow.sh`:

```bash
# Find this section:
declare -A TEMPO_PROJECTS=(
    ["tempo_60"]=60
    ["tempo_90"]=90
    ["tempo_120"]=120
    ["tempo_128"]=128
    ["tempo_140"]=140
    ["tempo_180"]=180
)

# Add your own:
declare -A TEMPO_PROJECTS=(
    ["tempo_80"]=80
    ["tempo_100"]=100
    ["tempo_120"]=120
    ["tempo_150"]=150
)
```

### Batch Set Parameter in Existing Projects

```bash
# Loop through projects and set tempo
cd scripts
for project in ../projects/tempo_*.logicx; do
    name=$(basename "$project" .logicx)
    tempo=${name#tempo_}  # Extract tempo from name

    echo "Setting $name to ${tempo} BPM..."

    open "$project"
    sleep 5

    osascript set_tempo_in_project.applescript "$tempo"
    sleep 2

    # Save and close
    osascript -e 'tell application "Logic Pro" to tell application "System Events" to tell process "Logic Pro" to keystroke "s" using command down'
    sleep 1
    osascript -e 'tell application "Logic Pro" to tell application "System Events" to tell process "Logic Pro" to keystroke "w" using command down'
    sleep 2
done
```

### Verify Project Parameters

Check what tempo is actually set:

```applescript
osascript -e 'tell application "Logic Pro" to tell front document to get tempo'
```

---

## Integration with Reverse Engineering Workflow

### Complete Automated Workflow

```bash
cd logic_reverse_engineering/scripts

# 1. Create all test projects (5 min)
./automated_tempo_workflow.sh

# 2. Extract all binaries (30 sec)
./extract_all.sh

# 3. Compare first pair
./compare_pair.sh tempo_120 tempo_128

# 4. Analyze changed offsets
# Look at the diff output for changed bytes
# Example: if offset 0x312 changed
./analyze_bytes.py analyze ../binary_samples/tempo_120.bin 0x312
./analyze_bytes.py analyze ../binary_samples/tempo_128.bin 0x312

# 5. Validate with more projects
for tempo in 60 90 140 180; do
    ./analyze_bytes.py analyze ../binary_samples/tempo_${tempo}.bin 0x312
done

# 6. Document findings
cat > ../findings/tempo.md << 'EOF'
# Tempo Parameter

**Offset**: 0x312
**Type**: f32 (little-endian)
**Validated**: ✅ Yes

## Test Results
- tempo_60: 60.0 ✓
- tempo_90: 90.0 ✓
- tempo_120: 120.0 ✓
- tempo_128: 128.0 ✓
- tempo_140: 140.0 ✓
- tempo_180: 180.0 ✓
EOF
```

---

## Tips for Success

### 1. Test with Template First

Before running full automation:
```bash
# Create just one project manually
cp -R projects/template_120.logicx projects/test.logicx
open projects/test.logicx

# Try setting tempo
osascript scripts/set_tempo_in_project.applescript 128

# Verify it worked
osascript -e 'tell application "Logic Pro" to tell front document to get tempo'
```

### 2. Run in Small Batches

Instead of all 6 tempos at once, do 2-3 at a time:
```bash
# Edit automated_tempo_workflow.sh to only include:
declare -A TEMPO_PROJECTS=(
    ["tempo_120"]=120
    ["tempo_128"]=128
)
```

### 3. Keep Logic Pro Clean

- Close all other projects before running automation
- Disable plugins that slow startup
- Clear recent files list if it's long

### 4. Monitor Progress

Watch Logic Pro as automation runs:
- Projects should open automatically
- Tempo should change in the display
- Projects should save and close

If anything hangs, press Cmd+. to stop the script.

---

## Time Savings Comparison

### Manual Approach
- Create 6 tempo projects: ~20 minutes
- Create sample rate variants: ~15 minutes
- Create key signature variants: ~10 minutes
- Create time signature variants: ~10 minutes
- **Total**: ~55 minutes

### Automated Approach
- Initial setup: ~5 minutes (one time)
- Create 6 tempo projects: ~5 minutes
- Sample/key/time sig: ~30 minutes (semi-automated)
- **Total**: ~35 minutes
- **Savings**: ~20 minutes (36%)

### Multiplied Over Testing Cycles
- 5 testing cycles: **Save 100 minutes** (1.5 hours)
- 10 testing cycles: **Save 200 minutes** (3+ hours)

---

## Related Documentation

- **Main Workflow**: `README.md` - Overall reverse engineering process
- **Analysis Tools**: See `compare_pair.sh`, `analyze_bytes.py`
- **Setup Guide**: `../docs/REVERSE_ENGINEERING_SETUP.md`

---

## Success Stories

After automation is working, you should be able to:

1. **Create 6 tempo projects**: 5 minutes ✓
2. **Extract all binaries**: 30 seconds ✓
3. **Find tempo offset**: 10 minutes ✓
4. **Validate with all projects**: 5 minutes ✓

**Total time to discover tempo parameter**: ~20 minutes

Compare to **manual approach**: ~2 hours

---

**Ready to automate?** Start with `./automated_tempo_workflow.sh`!
