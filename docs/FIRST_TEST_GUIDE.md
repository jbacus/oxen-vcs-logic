# First Real Test Guide

## Goal
Validate the entire system with a real or simulated Logic Pro workflow.

## Prerequisites
- ✅ Environment setup complete (macOS, Xcode, oxen CLI)
- ✅ Subprocess integration complete
- ✅ Integration tests passing

## Test Scenarios

### Scenario 1: Simple Workflow (30 minutes)

**Create test project:**
```bash
# Create realistic Logic Pro structure
mkdir -p ~/Desktop/TestSong.logicx/Alternatives/001
mkdir -p ~/Desktop/TestSong.logicx/Resources
mkdir -p ~/Desktop/TestSong.logicx/Media

# Create ProjectData
cat > ~/Desktop/TestSong.logicx/Alternatives/001/ProjectData <<'EOF'
<?xml version="1.0" encoding="UTF-8"?>
<project>
  <tempo>120</tempo>
  <sampleRate>48000</sampleRate>
  <key>C Major</key>
</project>
EOF

# Create some fake audio files
dd if=/dev/zero of=~/Desktop/TestSong.logicx/Media/audio1.wav bs=1M count=10
dd if=/dev/zero of=~/Desktop/TestSong.logicx/Media/audio2.wav bs=1M count=5
```

**Initialize and test:**
```bash
cd ~/Desktop/TestSong.logicx

# Initialize
oxenvcs-cli init --logic .

# Verify .oxenignore created
cat .oxenignore

# Check status
oxenvcs-cli status .

# Add all files
oxenvcs-cli add --all .

# Initial commit
oxenvcs-cli commit -m "Initial version" --bpm 120 --sample-rate 48000 --key "C Major" .

# View history
oxenvcs-cli log .
```

**Simulate changes:**
```bash
# "Edit" the project
echo "modified" >> Alternatives/001/ProjectData

# Add new audio file
dd if=/dev/zero of=Media/audio3.wav bs=1M count=3

# Check what changed
oxenvcs-cli status .

# Stage and commit
oxenvcs-cli add --all .
oxenvcs-cli commit -m "Added third track" --bpm 120 .

# View history
oxenvcs-cli log .

# Test rollback
oxenvcs-cli checkout <first-commit-id> .

# Verify files reverted
cat Alternatives/001/ProjectData  # Should not have "modified"
ls Media/  # Should only show audio1 and audio2
```

**Expected outcome:**
- ✅ Repository initialized with .oxenignore
- ✅ Files tracked correctly
- ✅ Commits created with metadata
- ✅ Rollback works
- ✅ History is accurate

### Scenario 2: Large File Test (30 minutes)

**Test with realistic audio file sizes:**
```bash
cd ~/Desktop/TestSong.logicx

# Create large audio file (100MB)
dd if=/dev/zero of=Media/large_audio.wav bs=1M count=100

# Add and commit
oxenvcs-cli add Media/large_audio.wav .
time oxenvcs-cli commit -m "Large audio file test" .

# Measure performance
# Should complete in <30 seconds
```

**Metrics to collect:**
- Time to add 100MB file
- Time to commit
- Memory usage during commit
- Disk space used in .oxen/

### Scenario 3: .oxenignore Validation (15 minutes)

**Create files that should be ignored:**
```bash
cd ~/Desktop/TestSong.logicx

# Create ignored directories
mkdir -p Bounces
mkdir -p "Freeze Files"
mkdir -p Autosave

# Create files in ignored locations
dd if=/dev/zero of=Bounces/export.wav bs=1M count=50
dd if=/dev/zero of="Freeze Files/freeze.wav" bs=1M count=20
echo "temp" > Autosave/temp.logicx

# Create macOS system files
touch .DS_Store
touch ._metadata

# Check status - should NOT show ignored files
oxenvcs-cli status .

# Verify
oxen status  # Use raw oxen to double-check
```

**Expected:**
- ✅ Bounces/ not tracked
- ✅ Freeze Files/ not tracked
- ✅ Autosave/ not tracked
- ✅ .DS_Store not tracked
- ✅ ._ files not tracked

### Scenario 4: Real Logic Pro (if available) (1 hour)

**Test with actual Logic Pro:**
```bash
# Open a real Logic Pro project
# Or create new one: File → New

# Initialize version control
cd ~/Music/Logic/YourProject.logicx
oxenvcs-cli init --logic .

# Check what Logic created
oxenvcs-cli status .

# Commit initial state
oxenvcs-cli add --all .
oxenvcs-cli commit -m "Initial Logic Pro project" --bpm 128 --sample-rate 44100 .

# Make changes in Logic Pro:
# - Add a track
# - Record some audio
# - Change tempo
# - Save (Cmd+S)

# Check status after Logic Pro save
oxenvcs-cli status .

# Commit changes
oxenvcs-cli add --all .
oxenvcs-cli commit -m "Added drum track" --bpm 128 .

# View history
oxenvcs-cli log .

# Test rollback
# First, note current state
ls -R Alternatives/

# Rollback to first commit
oxenvcs-cli checkout <first-commit-id> .

# Open in Logic Pro - should be initial state
# Then checkout back to latest
oxenvcs-cli checkout main .  # or use commit hash
```

**Things to verify:**
- ✅ Logic Pro projectData detected correctly
- ✅ Audio files tracked
- ✅ Changes detected after Logic Pro save
- ✅ Rollback restores exact state
- ✅ Logic Pro can open rolled-back project
- ✅ No corruption or data loss

## Troubleshooting

### Logic Pro won't open project after rollback
**Likely cause:** File permissions changed

**Solution:**
```bash
chmod -R 755 ~/Music/Logic/YourProject.logicx
```

### Oxen CLI errors
**Check oxen directly:**
```bash
cd ~/Desktop/TestSong.logicx
oxen status
oxen log
```

### Files not being tracked
**Check .oxenignore patterns:**
```bash
cat .oxenignore
# Verify file not matching ignored patterns
```

### Slow performance
**Check file sizes:**
```bash
du -sh ~/Desktop/TestSong.logicx
du -sh ~/Desktop/TestSong.logicx/.oxen
```

## Success Criteria

After completing all scenarios:

- [ ] Can initialize Logic Pro projects
- [ ] Can add and commit files
- [ ] .oxenignore patterns work correctly
- [ ] Large files (100MB+) handled
- [ ] Rollback restores exact state
- [ ] Performance acceptable (<30s for commit)
- [ ] No data loss or corruption
- [ ] Real Logic Pro projects work

## Document Findings

Create `FIRST_TEST_RESULTS.md`:

```markdown
# First Test Results

**Date:** YYYY-MM-DD
**Tester:** Your Name
**Environment:** macOS X.Y.Z, Logic Pro X.Y

## Test Summary

| Scenario | Result | Notes |
|----------|--------|-------|
| Simple Workflow | ✅/❌ | ... |
| Large Files | ✅/❌ | ... |
| .oxenignore | ✅/❌ | ... |
| Real Logic Pro | ✅/❌ | ... |

## Issues Found

1. **Issue description**
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - Severity: Critical/High/Medium/Low

## Performance Metrics

- Init time: Xs
- Add time (100MB): Xs
- Commit time: Xs
- Status check time: Xs

## Recommendations

- What worked well
- What needs improvement
- Blockers discovered
- Next steps
```

## Next Steps After Testing

Based on test results:
1. File GitHub issues for any bugs found
2. Update integration tests to cover edge cases
3. Begin Swift component testing (LaunchAgent)
4. Start working through TESTING_ROADMAP.md Phase 2
