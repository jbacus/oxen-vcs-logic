# Collaboration Workflows Guide

**For**: Musicians, Producers, and 3D Designers using Auxin
**Last Updated**: 2025-11-23

This guide describes how teams use Auxin to collaborate on creative projects, and how our tests ensure these workflows work reliably.

---

## Table of Contents

1. [Core Collaboration Patterns](#core-collaboration-patterns)
2. [Workflow 1: Sequential Handoff](#workflow-1-sequential-handoff)
3. [Workflow 2: Parallel Work with Coordination](#workflow-2-parallel-work-with-coordination)
4. [Workflow 3: Review and Feedback](#workflow-3-review-and-feedback)
5. [Workflow 4: Team Discovery](#workflow-4-team-discovery)
6. [Best Practices](#best-practices)
7. [Test Coverage](#test-coverage)

---

## Core Collaboration Patterns

Auxin supports three main collaboration patterns for creative teams:

### Pattern A: Sequential Handoff
**When to use**: Linear workflows where work passes from one person to the next
**Example**: Recording → Mixing → Mastering

### Pattern B: Parallel Work
**When to use**: Multiple people working simultaneously on different aspects
**Example**: Different musicians recording their parts in different studios

### Pattern C: Review & Iterate
**When to use**: Collaborative review and refinement
**Example**: Client feedback, producer notes, team critique

---

## Workflow 1: Sequential Handoff

### Scenario: Music Production Pipeline

**Team**:
- Alice (Producer/Recording Engineer) - Colorado
- Bob (Mixing Engineer) - London
- Charlie (Mastering Engineer) - Tokyo

**Project**: Summer Album 2025

### Step-by-Step Workflow

#### Phase 1: Alice Records (Colorado - Morning)

```bash
# Alice starts her day
cd /Users/alice/Projects/summer-album.logicx

# Get latest version
oxen pull origin main

# Check if anyone has the lock
auxin lock status
# Output: "No active lock"

# Acquire lock for 8-hour session
auxin lock acquire --timeout 8
# Output: "Lock acquired. Expires in 8 hours."

# Alice opens Logic Pro and records guitar tracks
# (Logic Pro auto-saves, daemon auto-commits to draft branch)

# After recording session
auxin add --all
auxin commit -m "Recorded guitar tracks - verses and chorus" \
  --bpm 120 \
  --key "A Minor" \
  --tags "recording,guitar,draft"

# Release lock
auxin lock release
# Output: "Lock released"

# Push to remote
oxen push origin main
# Output: "Pushed 12 files, 450MB"
```

**Tests Validating This**:
- ✅ `test_push_to_local_remote` - Verifies push succeeds
- ✅ `test_sequential_collaboration_handoff` - Full workflow test
- ✅ `test_lock_coordination_prevents_conflicts` - Lock mechanism

---

#### Phase 2: Bob Mixes (London - Evening)

```bash
# Bob starts his evening session
cd /Users/bob/Projects/summer-album.logicx

# Get Alice's latest work
oxen pull origin main
# Output: "Pulled 12 files, 450MB"

# Check what Alice did
auxin activity --limit 10
# Output shows:
# ● Alice: "Recorded guitar tracks - verses and chorus"
#   BPM: 120, Key: A Minor, Tags: recording,guitar,draft
#   2 hours ago

# Check lock status
auxin lock status
# Output: "No active lock"

# Acquire lock
auxin lock acquire --timeout 6

# Bob opens Logic Pro and mixes the guitar
# (Adjusts levels, adds EQ, compression)

# After mixing
auxin add --all
auxin commit -m "Mixed guitar - added EQ and compression" \
  --bpm 120 \
  --key "A Minor" \
  --tags "mixing,guitar"

# Add a comment for Alice
auxin comment add HEAD "Great guitar tone! Boosted 3kHz for clarity."

# Release and push
auxin lock release
oxen add .oxen/comments/
oxen commit -m "Add mixing notes"
oxen push origin main
```

**Tests Validating This**:
- ✅ `test_pull_from_local_remote` - Verifies pull succeeds
- ✅ `test_push_pull_roundtrip` - Full sync cycle
- ✅ `test_add_comment_to_commit` - Comment system
- ✅ `test_comment_sync_via_push_pull` - Comment syncing

---

#### Phase 3: Alice Reviews Bob's Mix (Colorado - Next Morning)

```bash
# Alice starts next day
cd /Users/alice/Projects/summer-album.logicx

# Get latest updates
oxen pull origin main

# Check team activity
auxin activity --limit 5
# Output shows:
# ● Bob: "Mixed guitar - added EQ and compression"
#   8 hours ago
# 💬 Bob: "Great guitar tone! Boosted 3kHz for clarity."

# View Bob's comment
auxin comment list HEAD
# Shows Bob's feedback

# Reply to Bob's comment
auxin comment add HEAD "Thanks! The mix sounds perfect. Ready for mastering."

# Push comment reply
oxen add .oxen/comments/
oxen commit -m "Reply to mixing feedback"
oxen push origin main
```

**Tests Validating This**:
- ✅ `test_activity_feed_visibility_across_users` - Activity visibility
- ✅ `test_cross_user_comment_visibility` - Comment visibility
- ✅ `test_comment_thread_on_commit` - Comment threading

---

#### Phase 4: Charlie Masters (Tokyo - Afternoon)

```bash
# Charlie joins the project
cd /Users/charlie/Projects/summer-album.logicx

# Clone the project (first time)
oxen clone https://hub.oxen.ai/alice/summer-album summer-album.logicx
cd summer-album.logicx

# See who's been working on this
auxin team
# Output:
# Team Members (3):
# ─────────────────────────────────────────
# alice@studio     : 15 commits (45%) ████████████████
# bob@home        :  8 commits (24%) ████████
# charlie@mobile  : 10 commits (30%) ██████████
#
# Last active: alice@studio (2 hours ago)

# Check activity
auxin activity --limit 20
# Shows full project history

# Acquire lock and master
auxin lock acquire --timeout 4
# ... mastering work in Logic Pro ...
auxin commit -m "Final master - optimized for streaming" \
  --bpm 120 \
  --tags "mastering,final"

auxin lock release
oxen push origin main
```

**Tests Validating This**:
- ✅ `test_discover_team_members_from_commits` - Team discovery
- ✅ `test_team_contribution_statistics` - Stats calculation
- ✅ `test_activity_feed_from_commits` - Activity feed

---

## Workflow 2: Parallel Work with Coordination

### Scenario: Band Recording Remotely

**Team**:
- Dave (Drums) - Seattle
- Emma (Bass) - Nashville
- Frank (Guitar) - Austin

**Project**: Live Sessions EP

### Time-Coordinated Workflow

#### Morning Slot: Dave Records Drums (9 AM - 12 PM PST)

```bash
# Dave's morning session
auxin lock acquire --timeout 4

# Records drums for 3 hours
# Auto-commits happen via daemon

auxin commit -m "Drums - Take 5 (keeper)" \
  --bpm 128 \
  --tags "drums,live,keeper"

auxin lock release
oxen push origin main
```

**Notification to Team**:
```
🔓 Dave released lock for "Live Sessions EP"
● Dave committed: "Drums - Take 5 (keeper)"
   BPM: 128, Tags: drums,live,keeper
```

---

#### Afternoon Slot: Emma Records Bass (12 PM - 3 PM PST)

```bash
# Emma checks when she can start
auxin lock status
# Output: "No active lock. Last held by Dave, released 5 minutes ago."

# Pull Dave's drums
oxen pull origin main

# Listen to Dave's drums in Logic Pro
auxin activity --user dave@seattle
# Shows Dave's recent work

# Acquire lock and record
auxin lock acquire --timeout 4

# Records bass parts
auxin commit -m "Bass - locked in with drums" \
  --bpm 128 \
  --tags "bass,live"

# Add comment for Dave
auxin comment add HEAD "Great pocket on the snare! Locked in perfectly."

auxin lock release
oxen add .oxen/comments/
oxen commit -m "Add bass recording notes"
oxen push origin main
```

---

#### Evening Slot: Frank Records Guitar (3 PM - 6 PM PST)

```bash
# Frank checks status
auxin lock status
# Output: "No active lock"

# Pull latest (Dave's drums + Emma's bass)
oxen pull origin main

# Review what's been done
auxin activity --limit 10
# Shows both Dave and Emma's work

auxin team
# Shows all 3 band members

# Acquire and record
auxin lock acquire --timeout 4
# ... records guitar ...
auxin commit -m "Guitar - doubled chorus riff" \
  --bpm 128 \
  --tags "guitar,live"

auxin lock release
oxen push origin main
```

**Tests Validating This**:
- ✅ `test_lock_coordination_prevents_conflicts` - No overlapping edits
- ✅ `test_activity_feed_includes_lock_events` - Lock timeline
- ✅ `test_metadata_consistency_across_users` - BPM stays consistent

---

## Workflow 3: Review and Feedback

### Scenario: Producer-Client Review Cycle

**Team**:
- Grace (Producer)
- Henry (Client/Artist)

### Review Workflow

#### Grace Submits Mix for Review

```bash
# Grace creates reviewable version
auxin commit -m "Mix v1 - ready for client review" \
  --bpm 120 \
  --key "G Major" \
  --tags "mix,review,v1" \
  --bounce "/Users/grace/Bounces/mix_v1.mp3"

oxen push origin main

# Notify client (via external channel)
# "Mix v1 ready for review - check the activity feed"
```

---

#### Henry Reviews and Comments

```bash
# Henry pulls latest
oxen pull origin main

# Check what's new
auxin activity --limit 5
# Shows: Grace's "Mix v1 - ready for client review"

# Listen to the mix
# (Opens bounce file or Logic Pro project)

# Add detailed feedback
auxin comment add HEAD "Overall great! A few notes:
- Vocals too quiet in verse
- Love the guitar tone
- Can we add more reverb to drums?"

# Push feedback
oxen add .oxen/comments/
oxen commit -m "Client feedback on Mix v1"
oxen push origin main
```

---

#### Grace Addresses Feedback

```bash
# Grace pulls feedback
oxen pull origin main

# Read Henry's comments
auxin comment list HEAD
# Shows Henry's detailed feedback

# Make changes
auxin lock acquire --timeout 2
# ... adjusts vocals, adds reverb ...

auxin commit -m "Mix v2 - addressed client feedback" \
  --bpm 120 \
  --tags "mix,review,v2" \
  --bounce "/Users/grace/Bounces/mix_v2.mp3"

# Reply to feedback
auxin comment add HEAD "Changes made:
✓ Boosted vocals by 3dB
✓ Added plate reverb to drums
✓ Guitar tone unchanged (glad you liked it!)"

auxin lock release
oxen add .oxen/comments/
oxen commit -m "Mix v2 with feedback addressed"
oxen push origin main
```

**Tests Validating This**:
- ✅ `test_comment_thread_on_commit` - Multi-message threads
- ✅ `test_activity_feed_filtering` - Find specific versions
- ✅ `test_extract_metadata_from_activity` - Track mix versions

---

## Workflow 4: Team Discovery

### Scenario: Joining an Existing Project

#### New Team Member Onboarding

```bash
# Isla joins established project
oxen clone https://hub.oxen.ai/team/big-project big-project.logicx
cd big-project.logicx

# Who's on this team?
auxin team
# Output:
# Team Members (5):
# ─────────────────────────────────────────
# alice@studio    : 45 commits (35%) ███████████████
# bob@home        : 30 commits (23%) ███████████
# charlie@mobile  : 25 commits (19%) █████████
# dave@seattle    : 20 commits (15%) ██████
# emma@nashville  : 10 commits (8%)  ████
#
# Total commits: 130
# Active contributors (last 7 days): 4/5

# What's been happening?
auxin activity --limit 20
# Shows full timeline

# What's the project state?
auxin log --limit 5
# Shows recent commits with metadata

# Filter by workflow stage
auxin log --tag mixing
# Shows all mixing-related commits

# Filter by BPM (find uptempo tracks)
auxin log --bpm 140
# Shows commits at 140 BPM
```

**Tests Validating This**:
- ✅ `test_discover_team_members_from_commits` - Team roster
- ✅ `test_team_contribution_statistics` - Contribution breakdown
- ✅ `test_activity_feed_pagination` - Browse history
- ✅ `test_activity_feed_filtering` - Find specific work

---

## Best Practices

### 1. Always Pull Before Starting Work

```bash
# ❌ WRONG: Start working immediately
auxin lock acquire
# ... may conflict with others' changes

# ✅ RIGHT: Pull first
oxen pull origin main
auxin activity --limit 5  # See what changed
auxin lock acquire
# ... work with latest version
```

**Why**: Ensures you're working with the latest version and can see if anyone else is working.

**Test**: `test_pull_from_local_remote`

---

### 2. Use Locks for All Editing Sessions

```bash
# ❌ WRONG: Edit without lock
# Opens Logic Pro
# Makes changes
# Commits
# Push fails - someone else pushed!

# ✅ RIGHT: Always use locks
auxin lock acquire --timeout 4
# Opens Logic Pro
# Makes changes
auxin commit -m "Changes"
auxin lock release
oxen push origin main  # Guaranteed no conflicts
```

**Why**: Prevents merge conflicts (impossible to merge binary Logic files).

**Test**: `test_lock_coordination_prevents_conflicts`

---

### 3. Always Release Locks When Done

```bash
# ❌ WRONG: Forget to release
auxin lock acquire
# ... work session ...
auxin commit -m "Done"
# Closes laptop, goes home
# Lock holds for hours, blocking team!

# ✅ RIGHT: Always release
auxin lock acquire --timeout 4
# ... work session ...
auxin commit -m "Done"
auxin lock release  # ← Don't forget!
oxen push origin main
```

**Why**: Blocked locks prevent teammates from working.

**Test**: `test_lock_release_notification`

---

### 4. Use Meaningful Commit Messages with Metadata

```bash
# ❌ WRONG: Vague message
auxin commit -m "Updated project"

# ✅ RIGHT: Descriptive with metadata
auxin commit -m "Recorded vocals for chorus - 3 takes, kept take 2" \
  --bpm 120 \
  --key "A Minor" \
  --tags "vocals,recording,keeper"
```

**Why**: Helps team understand what changed and find specific versions.

**Test**: `test_extract_metadata_from_activity`

---

### 5. Use Comments for Important Decisions

```bash
# After a significant commit
auxin commit -m "Final master v3"

# Document why this version is final
auxin comment add HEAD "This is the final master because:
- Client approved the vocal level
- Streaming loudness target met (-14 LUFS)
- No clipping on any platform
- All stems archived"

oxen add .oxen/comments/
oxen commit -m "Document final master decision"
oxen push origin main
```

**Why**: Creates audit trail of creative decisions.

**Test**: `test_comment_thread_on_commit`

---

### 6. Check Activity Feed Regularly

```bash
# Start of each work session
oxen pull origin main
auxin activity --limit 10

# Before acquiring lock
auxin lock status

# Check specific teammate's work
auxin activity --user bob@home
```

**Why**: Stay coordinated with team, avoid duplicating work.

**Test**: `test_activity_feed_visibility_across_users`

---

## Test Coverage

All the workflows above are validated by our comprehensive test suite:

### Push/Pull Operations (13 tests)
- ✅ Basic push/pull
- ✅ Multi-user sync
- ✅ Large files (450MB+ audio)
- ✅ Metadata preservation
- ✅ Lock coordination

### Multi-User Workflows (7 tests)
- ✅ Sequential handoff (Producer → Mixer → Mastering)
- ✅ Parallel work coordination
- ✅ Activity visibility across users
- ✅ Metadata consistency

### Team & Comments (7 tests)
- ✅ Team discovery
- ✅ Contribution statistics
- ✅ Comment threads
- ✅ Cross-user visibility
- ✅ Comment syncing

### Activity Feed (8 tests)
- ✅ Feed generation
- ✅ Filtering (by user, BPM, tags, date)
- ✅ Pagination
- ✅ Performance (500+ commits)
- ✅ Real-time updates

**Total**: 35+ integration tests covering real-world workflows

---

## Common Scenarios

### "Someone has the lock and I need to work NOW"

**Options**:
1. **Wait**: Check `auxin lock status` to see when it expires
2. **Contact them**: Use team chat to coordinate
3. **Emergency**: Admin can force-break lock (⚠️ only if absolutely necessary)

```bash
# Admin only - use with caution!
auxin lock break --force
```

**Test**: `test_lock_force_break`

---

### "I forgot to release the lock before closing my laptop"

**What happens**:
- Lock automatically expires after timeout period
- Teammates can see lock is stale
- After expiration, anyone can acquire new lock

**Prevention**:
```bash
# Add to your workflow checklist:
# 1. Commit changes
# 2. Release lock ← Don't forget!
# 3. Push to remote
# 4. Close laptop
```

**Test**: `test_lock_expiration`

---

### "How do I find that version with the 140 BPM drums?"

```bash
# Filter by BPM
auxin log --bpm 140

# Filter by tags
auxin log --tag drums

# Combine filters
auxin log --bpm 140 --tag drums --limit 20

# Find by date
auxin log --since "2025-01-01"
```

**Test**: `test_activity_feed_filtering`

---

### "Who worked on this project last week?"

```bash
# See all team members
auxin team

# Recent activity
auxin activity --limit 50

# Specific user's work
auxin activity --user alice@studio
```

**Test**: `test_discover_team_members_from_commits`

---

## Performance Expectations

Based on our test suite:

| Operation | Size | Expected Time | Test |
|-----------|------|---------------|------|
| **Push** | 10MB | < 10 seconds | `test_push_to_local_remote` |
| **Push** | 450MB | < 60 seconds | `test_push_large_audio_file` |
| **Pull** | 10MB | < 10 seconds | `test_pull_from_local_remote` |
| **Pull** | 450MB | < 60 seconds | `test_push_large_audio_file` |
| **Lock acquire** | - | < 2 seconds | `test_lock_acquisition_notification` |
| **Lock release** | - | < 2 seconds | `test_lock_release_notification` |
| **Activity feed** | 100 commits | < 1 second | `test_activity_feed_pagination` |
| **Activity feed** | 500 commits | < 5 seconds | `test_activity_feed_performance` |

---

## Summary

Auxin collaboration workflows are:

✅ **Tested**: 35+ integration tests covering all workflows
✅ **Reliable**: Lock coordination prevents conflicts
✅ **Visible**: Activity feed shows team progress
✅ **Communicative**: Comments enable feedback loops
✅ **Discoverable**: Team discovery shows contributors
✅ **Fast**: Large file operations complete in < 60 seconds

**Next Steps**:
1. Review your team's workflow patterns
2. Choose the pattern that fits (Sequential, Parallel, or Review)
3. Follow the best practices
4. Check activity feed regularly
5. Use locks consistently

For detailed technical documentation, see `COLLABORATION_TEST_COVERAGE.md`.

---

*Last Updated: 2025-11-23*
*Test Coverage: 85% of collaboration features*
