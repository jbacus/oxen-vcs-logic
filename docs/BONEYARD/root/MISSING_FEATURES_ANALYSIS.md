# Missing Features & Workflow Gaps Analysis

**Generated from**: Test compilation failures
**Date**: 2025-11-20

---

## Executive Summary

Testing against the documented API reveals **significant gaps** between what's documented/tested and what's actually implemented. The tests serve as specifications showing what SHOULD exist but doesn't.

**Critical Finding**: Many Phase 6 features are documented as "complete" but the public API is incomplete.

---

## 1. Network Resilience Module (Phase 6)

### Missing Public Exports

The following types exist internally but are NOT exported in `lib.rs`:

| Type | Status | Impact |
|------|--------|--------|
| `NetworkHealthMonitor` | Not exported | Cannot monitor network quality programmatically |
| `OfflineQueue` | Not exported | Cannot access offline queue from external code |
| `ChunkedUploadManager` | Not exported | Cannot manage chunked uploads externally |
| `RetryableError` | Not implemented | No structured error classification |
| `ErrorKind` enum | Not implemented | Cannot distinguish error types |

### Missing RetryPolicy Methods

`RetryPolicy` struct exists but is missing critical methods:

```rust
// MISSING - Need these for proper retry logic:
fn delay_for_attempt(&self, attempt: usize) -> Duration
fn should_retry(&self, attempt: usize) -> bool
fn is_retryable(&self, error: &str) -> bool

// MISSING - Need these fields exposed:
pub max_attempts: u32
pub base_delay_ms: u64
pub max_delay_ms: u64
```

**Impact**: Cannot implement proper exponential backoff from CLI commands.

### Workflow Gap: Retry Logic

**Documented**: Smart retry with exponential backoff (2s → 4s → 8s → 16s)
**Actual**: RetryPolicy exists but doesn't expose methods to calculate delays

**Missing workflow**:
1. Classify error as retryable vs fatal
2. Calculate appropriate delay for attempt N
3. Check if max attempts exceeded

---

## 2. Console TUI Module

### Missing Public Types

The console module has many private types that need to be public for testing:

| Type | Current | Should Be |
|------|---------|-----------|
| `ConsoleMode` | private enum | public enum |
| `KeyEvent` | not defined | public enum |
| `ConsoleState` | not defined | public struct |
| `StatusView` | internal struct | public struct |
| `HistoryView` | internal struct | public struct |
| `CommitDialog` | `CommitDialogState` (private) | public struct |
| `SearchView` | `SearchState` (private) | public struct |
| `HooksView` | `HooksState` (private) | public struct |
| `DiffView` | `CompareState` (private) | public struct |
| `HelpView` | not defined | public struct |

### Missing Console Methods

```rust
// MISSING on Console struct:
fn current_mode(&self) -> ConsoleMode
fn get_state(&self) -> ConsoleState
fn switch_mode(&mut self, mode: ConsoleMode)
fn handle_key(&mut self, key: KeyEvent) -> KeyResult
fn render(&self) -> String
fn handle_resize(&mut self, width: u16, height: u16)
fn set_colored(&mut self, enabled: bool)
fn save_state(&self) -> Result<()>
```

**Impact**: Console is essentially a black box - cannot test individual modes or key handling.

### Workflow Gap: Mode Switching

**Documented**: 7 modes (Status, History, Commit, Search, Diff, Hooks, Help)
**Actual**: Modes exist internally but no public API to switch or query current mode

---

## 3. Hooks Module

### API Mismatch

**Test expects**:
```rust
manager.list_hooks(HookType::PreCommit) -> Vec<String>
```

**Actual API**:
```rust
manager.list_hooks() -> Vec<(HookType, String)>
```

### Missing Filter Capability

Cannot filter hooks by type without manually iterating.

**Workflow Gap**: Users expect to list only pre-commit or only post-commit hooks.

### Missing remove_hook Implementation

Tests reveal `remove_hook` may not be fully implemented:
- Test expects: `manager.remove_hook(name, hook_type)`
- Need to verify this method exists and works

---

## 4. Bounce Module

### Missing BounceFilter Fields

```rust
// BounceFilter is missing:
pub pattern: Option<String>  // For filename pattern matching
```

The `search_bounces` function expects pattern-based filtering but the struct doesn't have the field.

### Workflow Gap: Search Functionality

**Documented**: Search bounces by pattern, format, size, date
**Actual**: `BounceFilter` struct exists but is incomplete

---

## 5. Circuit Breaker

### Missing Methods

```rust
// CircuitBreaker is missing:
fn is_closed(&self) -> bool
fn is_open(&self) -> bool
fn record_failure(&mut self)
fn record_success(&mut self)
fn allow_request(&self) -> bool
```

**Actual**: CircuitBreaker exists with internal state but methods may differ.

---

## 6. Offline Queue

### Not Exported

`OfflineQueue` struct exists in `lib.rs` but the tests can't import it from `network_resilience` module.

**Issue**: Module organization - types defined in one place, not re-exported where expected.

---

## Recommended Fixes

### Priority 1: Export Public API (1 day)

Update `src/lib.rs` to export:
```rust
pub use network_resilience::{
    NetworkHealthMonitor,
    ChunkedUploadManager,
    RetryableError,
    ErrorKind,
};
```

### Priority 2: Complete RetryPolicy API (0.5 day)

Add methods to `RetryPolicy`:
- `delay_for_attempt()`
- `should_retry()`
- `is_retryable()`
- Make fields public or add getters

### Priority 3: Make Console Types Public (1 day)

- Change `enum ConsoleMode` to `pub enum`
- Define and export `KeyEvent`, `ConsoleState`
- Add view structs as public types
- Add missing methods to `Console`

### Priority 4: Fix Hooks API (0.5 day)

Either:
- Add `list_hooks(hook_type: HookType)` overload, OR
- Document that filtering must be done client-side

### Priority 5: Complete BounceFilter (0.25 day)

Add `pattern` field to `BounceFilter` struct.

---

## Workflow Continuity Gaps

### 1. Network Retry Flow
```
DOCUMENTED:
  Error occurs → Classify error → Calculate delay → Retry → Track attempts

ACTUAL:
  Error occurs → ??? (no public API for classification or delay calculation)
```

### 2. Console Interaction Flow
```
DOCUMENTED:
  Launch → Show status → Handle keypress → Switch mode → Render

ACTUAL:
  Launch → ??? (no public API for mode/key handling)
```

### 3. Offline Operation Flow
```
DOCUMENTED:
  Go offline → Queue operations → Come online → Sync automatically

ACTUAL:
  OfflineQueue exists but isn't accessible from network_resilience module
```

### 4. Hook Execution Flow
```
DOCUMENTED:
  Commit → Run pre-commit hooks → If fail, abort → If pass, commit → Run post-commit

ACTUAL:
  Hooks exist but remove/execution testing blocked by API issues
```

---

## Impact on Production Readiness

These gaps mean:

1. **Phase 6 "Complete" is overstated** - Network resilience internals exist but aren't usable programmatically
2. **TUI can't be tested** - Console is a monolithic black box
3. **Error handling is ad-hoc** - No structured error classification
4. **CI can't verify features** - Tests can't compile due to missing exports

---

## Metrics

| Category | Missing Items | Effort to Fix |
|----------|---------------|---------------|
| Public exports | 8 types | 1 day |
| Missing methods | 15+ methods | 2 days |
| Missing structs | 5 structs | 1 day |
| API mismatches | 3 functions | 0.5 day |

**Total effort**: ~4-5 days to make tests compile

---

## Conclusion

The tests successfully identified that **the codebase has the features internally but doesn't expose them properly**. This is a classic case of "works in manual testing but can't be automated".

**Next steps**:
1. Export all public types
2. Complete the public API surface
3. Make tests compile
4. Run tests to find actual bugs

The 88% code coverage number is misleading - it covers internal logic but doesn't verify the public API is complete or usable.

---

*Generated by analyzing test compilation failures against actual implementation*
