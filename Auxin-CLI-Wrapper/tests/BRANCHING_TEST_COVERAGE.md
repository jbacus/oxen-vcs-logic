# Branching Test Coverage: Auxin vs Git

**Date**: 2025-11-23
**Analysis**: Comparison of Auxin's branching test coverage to Git's branching model

---

## Executive Summary

**Key Finding**: Auxin uses a **fundamentally different collaboration model** than Git.

- **Git**: Optimistic locking with branching + merging for parallel development
- **Auxin**: Pessimistic locking on a single main branch to prevent merge conflicts with binary files

**Test Coverage**: ✅ **Good** for Auxin's intended workflow, ❌ **Incomplete** compared to Git's branching model

---

## Git's Branching Model (For Reference)

### Core Branching Operations in Git

| Operation | Purpose | Git Usage | Oxen Support | Auxin Tests |
|-----------|---------|-----------|--------------|-------------|
| **Branch Creation** | Create new branch from HEAD | `git branch feature` | ✅ Yes | ✅ Tested (9 tests) |
| **Branch Switching** | Switch to existing branch | `git checkout feature` | ✅ Yes | ✅ Tested (9 tests) |
| **Branch Listing** | List all branches | `git branch -a` | ✅ Yes | ✅ Tested (2 tests) |
| **Branch Deletion** | Delete merged/unneeded branch | `git branch -d feature` | ✅ Yes | ❌ **Not tested** |
| **Merging** | Integrate changes from branch | `git merge feature` | ❌ **No** | ❌ N/A |
| **Rebasing** | Replay commits on new base | `git rebase main` | ❌ **No** | ❌ N/A |
| **Fast-forward** | Linear history merge | `git merge --ff` | ❌ **No** | ❌ N/A |
| **3-way merge** | Merge with common ancestor | Auto in `git merge` | ❌ **No** | ❌ N/A |
| **Conflict resolution** | Resolve merge conflicts | Manual editing | ❌ **No** | ❌ N/A |
| **Branch tracking** | Track upstream branches | `git branch -u origin/main` | Partial | ❌ Not tested |

### Git's Typical Branching Workflows

#### 1. **Feature Branch Workflow**
```bash
# Developer creates feature branch
git checkout -b feature/new-drum-pattern

# Makes commits on feature branch
git commit -m "Add drum pattern"
git commit -m "Tweak timing"

# Merges back to main when done
git checkout main
git merge feature/new-drum-pattern
```

**Auxin Equivalent**: NOT APPLICABLE
- Auxin uses pessimistic locks on main branch
- No parallel feature branches needed
- Lock prevents conflicts before they occur

#### 2. **Parallel Development (Multiple Developers)**
```bash
# Developer A: works on vocals
git checkout -b feature/vocals
git commit -m "Record vocals"

# Developer B: works on bass (parallel!)
git checkout -b feature/bass
git commit -m "Add bass line"

# Both merge to main independently
```

**Auxin Equivalent**: Sequential handoff with locks (tested in `multi_user_workflow_test.rs`)

#### 3. **Long-running Branches**
```bash
# Development branch for integration
git checkout -b develop

# Multiple feature branches merge here first
git merge feature/vocals
git merge feature/drums

# Eventually merges to main for release
```

**Auxin Equivalent**: NOT APPLICABLE
- Binary audio files don't merge well
- Pessimistic locking prevents this need

---

## Auxin's Branching Model

### Supported Branch Operations

| Operation | Implementation | Purpose in Auxin |
|-----------|----------------|------------------|
| `create_branch` | `oxen checkout -b <name>` | Create draft branch |
| `checkout` | `oxen checkout <branch>` | Switch between branches |
| `list_branches` | `oxen branch` | View available branches |
| `delete_branch` | `oxen branch -d <name>` | Clean up branches |
| `current_branch` | Parse `oxen branch` output | Identify active branch |

### **NOT** Supported

- ❌ `merge` - No merge command
- ❌ `rebase` - No rebase command
- ❌ Conflict resolution - Binary files can't be merged
- ❌ 3-way merge - Not applicable to binary formats
- ❌ Cherry-pick - Not implemented

---

## Current Test Coverage

### ✅ **Well-Tested: Draft Branch Workflow**

**File**: `draft_manager_integration_test.rs` (9 tests)

| Test | What It Covers | Line |
|------|----------------|------|
| `test_draft_manager_initialization` | Creates draft branch, verifies it exists | 50 |
| `test_draft_manager_auto_commit` | Commits to draft branch | 81 |
| `test_draft_manager_switch_branches` | Switch between `main` and `__drafts__` | 116 |
| `test_draft_manager_multiple_commits` | Multiple sequential commits on draft | 150 |
| `test_draft_manager_reset_to_main` | Reset draft branch to match main | 184 |
| `test_draft_manager_stats` | Branch statistics and status | 223 |
| `test_draft_manager_custom_config` | Custom branch names (not just `__drafts__`) | 255 |
| `test_draft_manager_auto_switch_to_draft` | Auto-switch to draft on commit | 283 |
| `test_draft_manager_prune_if_needed` | Prune old draft commits | 311 |

**Coverage**: ✅ **Excellent** for draft workflow

---

### ✅ **Tested: Basic Branch Operations**

**File**: `oxen_subprocess_integration_test.rs` (2 tests)

| Test | What It Covers | Line |
|------|----------------|------|
| `test_list_branches` | List all branches, verify main exists | 483 |
| `test_current_branch_is_marked` | Current branch marked correctly | 513 |

**Coverage**: ✅ **Adequate** for listing

---

### ❌ **NOT Tested: Branch Deletion**

**Missing Tests**:
- Deleting a branch
- Preventing deletion of current branch
- Preventing deletion of main branch
- Clean up after branch deletion (refs, working tree)

**Risk**: Low (rarely delete branches in Auxin workflow)

---

### ❌ **NOT Tested: Feature Branches (By Design)**

**Why Git Uses Feature Branches**:
```bash
# Scenario: 2 developers work in parallel
Dev A: git checkout -b feature/vocals
Dev B: git checkout -b feature/drums

# Both merge independently
git checkout main
git merge feature/vocals  # ✅ Works
git merge feature/drums   # ✅ Works (Git resolves)
```

**Why Auxin Doesn't**:
- Audio files (`.wav`, `.aiff`) are **binary** and cannot be auto-merged
- Merge conflicts in binary files require manual "winner" selection (lose data)
- Pessimistic locking ensures only ONE person modifies at a time
- Sequential workflow is intentional: Producer → Mixer → Mastering

**Test Coverage**:
- ✅ Sequential handoff tested in `multi_user_workflow_test.rs`
- ✅ Lock coordination tested in `multi_user_workflow_test.rs`
- ❌ Parallel feature branches NOT tested (not needed)

---

### ❌ **NOT Tested: Merging (Not Supported by Design)**

**Why Git Merge is Powerful**:
```bash
# Merge strategies
git merge --ff              # Fast-forward (linear history)
git merge --no-ff          # Force merge commit
git merge --squash         # Squash commits

# Auto-merge logic
- Text files: 3-way merge with common ancestor
- Conflicts: Mark conflicts, user resolves
```

**Why Oxen Doesn't Support**:
- Binary files can't be auto-merged (no line-by-line diff)
- Audio waveforms can't be "merged" meaningfully
- `.logicx` bundles contain binary plists and samples
- Merging would require custom audio mixing (out of scope)

**Test Coverage**:
- ❌ Not tested (feature doesn't exist)
- ✅ Lock-based coordination tested instead

---

### ❌ **NOT Tested: Branch Tracking (Partially Supported)**

**What Git Tracks**:
```bash
# Upstream branch relationships
git branch --set-upstream-to=origin/main main
git status
# Output: Your branch is behind 'origin/main' by 3 commits

# Ahead/behind tracking
git branch -vv
# * main 1a2b3c4 [origin/main: ahead 2, behind 1] Latest commit
```

**What Oxen Supports**:
- Oxen has remote branches via `oxen push origin main`
- Oxen can fetch remote branches
- Unclear if Oxen tracks ahead/behind counts

**Test Coverage**:
- ❌ Not tested
- ❌ No tests for `oxen fetch` with branches
- ❌ No tests for ahead/behind tracking

**Recommendation**: Add tests if needed for remote collaboration

---

## Test Gaps Summary

### Critical Gaps (Should Fix)
None. Auxin's branching tests cover the intended workflow.

### Nice-to-Have Gaps (Low Priority)

1. **Branch Deletion** (1-2 tests)
   - Test `delete_branch()` on non-current branch
   - Test error handling (delete current branch, delete main)

2. **Remote Branch Tracking** (3-4 tests)
   - Test `oxen fetch` updates remote branch refs
   - Test ahead/behind tracking (if supported)
   - Test push creates remote branch

3. **Multiple Feature Branches** (Educational Only)
   - Create 2 feature branches in parallel
   - Document why merging them is NOT supported
   - Show lock coordination as the alternative

---

## Comparison Table: Git vs Auxin Branching Tests

| Workflow | Git Tests (Typical) | Auxin Tests (Current) | Gap? |
|----------|---------------------|----------------------|------|
| **Create branch** | ✅ Tested | ✅ Tested (9 tests) | ✅ No gap |
| **Switch branch** | ✅ Tested | ✅ Tested (9 tests) | ✅ No gap |
| **Delete branch** | ✅ Tested | ❌ Not tested | ⚠️ Minor gap |
| **List branches** | ✅ Tested | ✅ Tested (2 tests) | ✅ No gap |
| **Merge branches** | ✅ Tested (critical!) | ❌ N/A (not supported) | ✅ By design |
| **Rebase** | ✅ Tested | ❌ N/A (not supported) | ✅ By design |
| **Conflict resolution** | ✅ Tested | ❌ N/A (locks prevent) | ✅ By design |
| **Feature branches** | ✅ Tested (10+ tests) | ❌ Not applicable | ✅ By design |
| **Remote branches** | ✅ Tested | ⚠️ Partial (via push/pull) | ⚠️ Minor gap |
| **Ahead/behind** | ✅ Tested | ❌ Not tested | ⚠️ Unknown if needed |
| **Long-running branches** | ✅ Tested | ❌ N/A (draft only) | ✅ By design |

**Overall**: ✅ **Auxin's branching tests are appropriate for its collaboration model**

---

## Recommendations

### 1. ✅ **No Action Needed for Core Workflow**

Auxin's branching tests adequately cover the **draft branch workflow**, which is the primary use case:
- ✅ Create `__drafts__` branch
- ✅ Auto-commit drafts to branch
- ✅ Switch between `main` and `__drafts__`
- ✅ Reset drafts to main

This matches the **intended design**: auto-save drafts, then explicit commits to main.

### 2. ⚠️ **Consider Adding: Branch Deletion Tests** (Low Priority)

**Why**: Cleanup is part of good branch hygiene.

**Tests to Add** (2 tests):
```rust
#[tokio::test]
async fn test_delete_non_current_branch() {
    // Create feature branch, switch to main, delete feature
    draft_manager.create_branch("feature/test").await.unwrap();
    draft_manager.switch_to_main().await.unwrap();

    let result = draft_manager.delete_branch("feature/test").await;
    assert!(result.is_ok(), "Should delete non-current branch");
}

#[tokio::test]
async fn test_cannot_delete_current_branch() {
    draft_manager.switch_to_draft().await.unwrap();

    let result = draft_manager.delete_branch("__drafts__").await;
    assert!(result.is_err(), "Cannot delete current branch");
}
```

### 3. ⚠️ **Consider Adding: Remote Branch Tests** (Medium Priority)

**Why**: Users collaborate via remotes, and branch synchronization is important.

**Tests to Add** (3-4 tests):
```rust
#[tokio::test]
async fn test_push_creates_remote_branch() {
    // Create local branch, push to remote, verify remote has it
}

#[tokio::test]
async fn test_fetch_updates_remote_branches() {
    // Push from user1, fetch from user2, verify branch visible
}

#[tokio::test]
async fn test_checkout_remote_branch() {
    // Checkout remote branch as local tracking branch
}
```

### 4. 📚 **Document Why No Merge Tests** (Documentation)

**Action**: Add section to `COLLABORATION_TEST_COVERAGE.md` explaining:
- Why Git uses branching/merging (parallel development of text files)
- Why Auxin uses pessimistic locking (binary files can't merge)
- How Auxin's lock coordination replaces Git's merge workflow
- Trade-offs: Auxin prevents conflicts upfront vs Git resolves conflicts after

---

## Conclusion

### Key Insights

1. **Different Models, Different Tests**
   - Git: Optimistic locking → needs merge/conflict tests
   - Auxin: Pessimistic locking → needs lock coordination tests

2. **Auxin's Branching is Intentionally Limited**
   - Branches exist (for drafts)
   - Merging does NOT exist (by design)
   - This is correct for binary file workflows

3. **Test Coverage is Appropriate**
   - ✅ Draft workflow: 9 comprehensive tests
   - ✅ Lock coordination: 7 multi-user tests
   - ✅ Push/pull: 13 synchronization tests
   - ⚠️ Branch deletion: 0 tests (minor gap)
   - ⚠️ Remote branches: Partial coverage

### Final Verdict

**Question**: "Have we covered branching?"

**Answer**:
- ✅ **Yes** for Auxin's **draft branch workflow** (excellent coverage)
- ✅ **Yes** for Auxin's **intended collaboration model** (pessimistic locking)
- ⚠️ **Partially** for **remote branch operations** (minor gap)
- ❌ **No** for **Git-style feature branches and merging** (by design, not needed)

**Overall**: ✅ **Branching tests are sufficient for Auxin's use case**

---

## Appendix: Test Files Reference

### Existing Test Files

1. **`draft_manager_integration_test.rs`** (9 tests)
   - Draft branch creation, switching, auto-commit
   - Reset, pruning, custom branch names

2. **`oxen_subprocess_integration_test.rs`** (2 branch tests)
   - List branches
   - Current branch detection

3. **`multi_user_workflow_test.rs`** (7 tests)
   - Lock-based collaboration (replaces Git merging)
   - Sequential handoff workflow

4. **`push_pull_integration_test.rs`** (13 tests)
   - Push/pull with local remotes
   - Metadata synchronization

### Suggested New Tests

**File**: `branch_operations_test.rs` (NEW - optional)

```rust
// Test branch deletion
test_delete_branch()
test_cannot_delete_current_branch()
test_cannot_delete_main_branch()

// Test remote branches (if needed)
test_push_creates_remote_branch()
test_fetch_updates_remote_branches()
test_checkout_remote_tracking_branch()
test_delete_remote_branch()
```

---

**Generated**: 2025-11-23
**Tool**: Claude Code (Sonnet 4.5)
