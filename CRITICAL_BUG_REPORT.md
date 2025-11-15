# Critical Bug Report - Restore Command

**Date Discovered**: 2025-11-14
**Severity**: 🔴 **CRITICAL**
**Component**: `oxenvcs-cli restore`
**Status**: 🔴 **UNRESOLVED - Blocker for MVP**

---

## Summary

The `restore` command silently fails when given invalid or short commit hashes, reporting success even though no files were restored.

---

## Steps to Reproduce

```bash
# 1. Create a repository with commits
cd test-project.logicx
oxenvcs-cli init --logic .
oxenvcs-cli add --all
oxenvcs-cli commit -m "Initial" --bpm 120

# 2. Modify and commit again
echo "changed" > projectData
oxenvcs-cli add --all
oxenvcs-cli commit -m "Changed" --bpm 130

# 3. Get commit ID (shows short hash)
oxenvcs-cli log
# Output: Commit: cb41da4f3b1669f449519f361918b73f

# 4. Try to restore using short hash
oxenvcs-cli restore cb41da4f
```

**Expected**: Error message saying "Revision not found" or "Short hash not supported"

**Actual**:
```
Restoring to commit: cb41da4f
Successfully restored to commit: cb41da4f
```

But the files are **NOT restored** - still show "changed" content!

---

## Root Cause Analysis

### Upstream Oxen CLI Bug

The `oxen checkout` command **returns exit code 0** even when it fails to find a revision:

```bash
$ oxen checkout invalid_hash 2>&1; echo "Exit: $?"
Revision not found: invalid_hash
...
Exit: 0    # <-- THIS IS THE BUG
```

### Why Our Code Can't Detect It

Our `handle_output` function correctly checks `output.status.success()`:

```rust
if !output.status.success() {
    return Err(anyhow!("oxen command failed: ..."));
}
```

But since Oxen returns success even on error, we can't detect the failure.

### Short Hash vs Full Hash

- Oxen CLI **requires full commit hashes** for checkout (32+ characters)
- Our `log` command displays short hashes for readability
- Users naturally copy the short hash and try to use it
- Oxen silently fails when given short hash (with exit code 0!)

---

## Impact

### User Experience

1. **Data Loss Risk**: Users think they've restored to a previous version but haven't
2. **Silent Failure**: No error message, looks like success
3. **Confusing Workflow**: Log shows short hashes, restore needs full hashes
4. **Trust Erosion**: Tool appears buggy even though it's upstream issue

### Production Readiness

This is a **blocker for MVP release** because:
- Core functionality (restore) is unreliable
- Could lead to actual data loss scenarios
- Users won't be able to use the primary rollback feature

---

## Proposed Solutions

### Option 1: Expand Short Hashes (Recommended)

**Approach**: Automatically expand short hashes to full hashes before calling `oxen checkout`

**Implementation**:
```rust
pub async fn restore(&self, commit_id: &str) -> Result<()> {
    // If short hash, expand to full hash
    let full_hash = if commit_id.len() < 32 {
        self.expand_commit_hash(commit_id).await?
    } else {
        commit_id.to_string()
    };

    self.oxen.checkout(&self.path, &full_hash)?;
    Ok(())
}

async fn expand_commit_hash(&self, short_hash: &str) -> Result<String> {
    let commits = self.get_history(None).await?;

    for commit in commits {
        if commit.id.starts_with(short_hash) {
            return Ok(commit.id);
        }
    }

    Err(anyhow!("Commit not found: {}", short_hash))
}
```

**Pros**:
- User-friendly (works with short hashes)
- Catches invalid hashes before calling oxen
- Can provide better error messages

**Cons**:
- Requires fetching full log first (slower)
- More complex implementation

**Estimated Time**: 1-2 hours

---

### Option 2: Parse Stderr for Error Messages

**Approach**: Check stderr for "Revision not found" even if exit code is 0

**Implementation**:
```rust
fn handle_output(&self, output: Output, args: &[&str]) -> Result<String> {
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check for known error messages in stderr (even if exit code is 0)
    if stderr.contains("Revision not found")
        || stderr.contains("not found")
        || stderr.contains("error:") {
        return Err(anyhow!("oxen command failed: {}", stderr));
    }

    if !output.status.success() {
        return Err(anyhow!("oxen command failed: {}", stderr));
    }

    Ok(stdout)
}
```

**Pros**:
- Simple implementation
- Works around Oxen's broken exit codes
- Catches other silent failures too

**Cons**:
- Brittle (depends on error message format)
- Doesn't solve short hash problem
- Could break if Oxen changes messages

**Estimated Time**: 30 minutes

---

### Option 3: Require Full Hashes + Document Limitation

**Approach**: Validate hash length and show error if too short

**Implementation**:
```rust
pub async fn restore(&self, commit_id: &str) -> Result<()> {
    if commit_id.len() < 32 {
        return Err(anyhow!(
            "Full commit hash required (minimum 32 characters).\n\
             Run 'oxenvcs-cli log' to see full hashes."
        ));
    }

    // Also parse stderr for "Revision not found"
    self.oxen.checkout(&self.path, commit_id)?;
    Ok(())
}
```

**Pros**:
- Simple and explicit
- Avoids ambiguity with short hashes
- Forces users to be specific

**Cons**:
- Poor UX (users must copy long hashes)
- Doesn't match Git conventions (short hashes work there)
- Doesn't fully solve silent failure issue

**Estimated Time**: 15 minutes

---

## Recommended Fix: Hybrid Approach

Combine **Option 1** and **Option 2**:

1. **Expand short hashes automatically** for UX
2. **Parse stderr** to catch Oxen's broken exit codes
3. **Add integration tests** for both scenarios

**Implementation Plan**:

```rust
// In oxen_subprocess.rs
fn handle_output(&self, output: Output, args: &[&str]) -> Result<String> {
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Check stderr for errors (catches Oxen's broken exit codes)
    if stderr.contains("Revision not found")
        || stderr.contains("error:")
        || stderr.contains("fatal:") {
        return Err(anyhow!("{}", stderr.trim()));
    }

    // Also check exit code (for well-behaved commands)
    if !output.status.success() {
        return Err(anyhow!("Command failed: {}", stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// In oxen_ops.rs
pub async fn restore(&self, commit_id: &str) -> Result<()> {
    // Expand short hashes
    let full_hash = if commit_id.len() < 32 {
        vlog!("Expanding short hash: {}", commit_id);
        self.find_commit_by_prefix(commit_id).await?
    } else {
        commit_id.to_string()
    };

    vlog!("Restoring to full hash: {}", full_hash);
    self.oxen.checkout(&self.path, &full_hash)?;
    Ok(())
}

async fn find_commit_by_prefix(&self, prefix: &str) -> Result<String> {
    let commits = self.get_history(None).await?;

    let matches: Vec<_> = commits
        .iter()
        .filter(|c| c.id.starts_with(prefix))
        .collect();

    match matches.len() {
        0 => Err(anyhow!("No commit found matching: {}", prefix)),
        1 => Ok(matches[0].id.clone()),
        _ => Err(anyhow!("Ambiguous commit prefix '{}': {} matches", prefix, matches.len())),
    }
}
```

**Tests to Add**:

```rust
#[test]
fn test_restore_with_short_hash() {
    // Should expand short hash and restore successfully
}

#[test]
fn test_restore_with_invalid_hash() {
    // Should return error, not report false success
}

#[test]
fn test_restore_with_ambiguous_short_hash() {
    // Should error if prefix matches multiple commits
}
```

**Estimated Time**: 2-3 hours (implementation + tests)

---

## Testing Checklist

After implementing fix:

- [ ] Restore with full hash (32+ chars) - should work
- [ ] Restore with short hash (7-8 chars) - should expand and work
- [ ] Restore with invalid hash - should error clearly
- [ ] Restore with ambiguous prefix (e.g., "a") - should error with count
- [ ] Restore with non-existent short hash - should error
- [ ] Integration test with real Logic Pro project workflow

---

## Priority & Next Steps

**Priority**: 🔴 **P0 - Critical for MVP**

**Blocking**:
- MVP release
- User acceptance testing
- Production deployment

**Next Steps**:
1. Implement hybrid fix (2-3 hours)
2. Add comprehensive tests (1 hour)
3. Update user documentation (30 minutes)
4. Re-test end-to-end workflow (30 minutes)
5. Consider filing bug report with Oxen maintainers

**Owner**: TBD
**ETA**: Should be fixed before MVP ships

---

## Workaround for Users (Until Fixed)

1. Always use full commit hashes from `oxenvcs-cli log`
2. If restore appears to succeed but files haven't changed, use raw Oxen:
   ```bash
   oxen log  # Get full hash
   oxen checkout <FULL_HASH>
   ```

---

## Related Issues

- Oxen CLI returning exit code 0 on errors (upstream bug)
- Log command showing short hashes for UX but restore needs full hashes
- No validation of commit hash before calling Oxen

---

*Discovered during end-to-end workflow testing on 2025-11-14*
*Needs immediate attention before MVP release*
