# Process Efficiency Workplan - 2025-10-30

## Executive Summary

This workplan focuses on building process efficiency in how Claude (and other developers) work with the Oxen-VCS codebase. The goal is to reduce friction, improve consistency, and automate common workflows.

**Target Completion**: 1 day (6-8 hours)
**Environment**: Linux (Rust-focused improvements; Swift components require macOS)
**Priority**: High - These improvements will compound over time

---

## Current State Assessment

### Strengths ✅
- Comprehensive CI/CD with GitHub Actions (test.yml, 208 lines, well-structured)
- Good test coverage in Rust (85%, 121 tests)
- Consolidated documentation (CLAUDE.md is excellent, 16 docs in docs/)
- Installation automation (install.sh, run_all_tests.sh)
- Component-specific helper scripts exist

### Identified Gaps ❌

| Category | Issue | Impact | Priority |
|----------|-------|--------|----------|
| **Build/Dev** | No Makefile/Justfile for common tasks | High friction for repetitive commands | **HIGH** |
| **Code Quality** | No rustfmt.toml/clippy.toml configuration | Inconsistent formatting, manual style enforcement | **HIGH** |
| **Git Workflow** | No pre-commit hooks | Formatting issues caught in CI, not locally | **MEDIUM** |
| **Testing** | Code coverage not tracked/reported | Unknown test gaps, no coverage trends | **MEDIUM** |
| **Onboarding** | No quick dev environment setup | Slow onboarding for new developers | **MEDIUM** |
| **Debugging** | Limited debugging utilities/scripts | Manual log searching, no helper tools | **LOW** |
| **Documentation** | 16+ doc files, no clear navigation | Documentation discovery friction | **LOW** |
| **Dependencies** | Network issues with crates.io (403 error) | Blocks Rust builds in current environment | **HIGH** |

---

## Workplan: 8 Process Efficiency Improvements

### Phase 1: Foundation (High Priority) - 2-3 hours

#### 1. Create Unified Task Runner (Makefile/Justfile)
**Goal**: Single entry point for all common development tasks

**Rationale**:
- Reduces cognitive load (memorize `make test` vs. `cd OxVCS-CLI-Wrapper && cargo test --lib`)
- Standardizes workflows across team
- Self-documenting via `make help`

**Tasks**:
```bash
# Create Justfile (more modern than Make, better syntax)
# Commands to include:
- just test              # Run all tests
- just test-rust         # Rust tests only
- just test-swift        # Swift tests (macOS only)
- just build             # Build all components
- just build-release     # Release builds
- just fmt               # Format all code
- just lint              # Run all linters
- just check             # Pre-commit checks (fmt + lint + test)
- just clean             # Clean build artifacts
- just install           # Run install.sh
- just watch-rust        # Watch mode for Rust tests
- just coverage          # Generate coverage reports
- just docs              # Generate and serve docs
```

**Acceptance Criteria**:
- ✓ Justfile created with 12+ common commands
- ✓ `just --list` shows all available commands with descriptions
- ✓ Works on both Linux and macOS (with appropriate platform checks)
- ✓ README.md updated with "Quick Start" using just commands

---

#### 2. Standardize Rust Tooling Configuration
**Goal**: Enforce consistent Rust code style and catch common issues

**Rationale**:
- CI enforces `cargo fmt --check` and `cargo clippy` but no local config
- Prevents "works on my machine" formatting issues
- Catches potential bugs earlier (clippy lints)

**Tasks**:
```toml
# Create OxVCS-CLI-Wrapper/rustfmt.toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true

# Create OxVCS-CLI-Wrapper/clippy.toml
# Enforce stricter lints
cognitive-complexity-threshold = 30
```

**Acceptance Criteria**:
- ✓ rustfmt.toml and .clippy.toml created
- ✓ Running `cargo fmt` and `cargo clippy` uses project config
- ✓ CONTRIBUTING.md updated with formatting guidelines
- ✓ No clippy warnings in current codebase

---

#### 3. Fix Crates.io Network Issue
**Goal**: Resolve 403 error preventing Rust builds

**Investigation**:
```bash
# Check current .cargo/config.toml
# May need to:
# - Remove protocol = "sparse" if causing issues
# - Configure alternative registry mirror
# - Check network/firewall settings
```

**Tasks**:
- Diagnose root cause of 403 error
- Test alternative registry configurations
- Document solution in TROUBLESHOOTING.md
- Update CI if registry config changes

**Acceptance Criteria**:
- ✓ `cargo build` succeeds without network errors
- ✓ `cargo test` can download dependencies
- ✓ Solution documented for future reference

---

### Phase 2: Developer Experience (Medium Priority) - 2-3 hours

#### 4. Create Pre-Commit Hook System
**Goal**: Catch formatting/lint issues before commit, not in CI

**Rationale**:
- Faster feedback loop (local vs. CI)
- Prevents "fix formatting" commits
- Reduces CI failures and wasted cycles

**Tasks**:
```bash
# Install pre-commit framework: pip install pre-commit
# Create .pre-commit-config.yaml
# Hooks to include:
- trailing-whitespace removal
- end-of-file-fixer
- check-yaml
- check-json
- cargo fmt
- cargo clippy
- swift format (if on macOS)
```

**Acceptance Criteria**:
- ✓ .pre-commit-config.yaml created
- ✓ Installation documented in CONTRIBUTING.md
- ✓ Hooks run automatically on `git commit`
- ✓ Can bypass with `--no-verify` if needed

---

#### 5. Add Code Coverage Tracking
**Goal**: Visibility into test coverage trends and gaps

**Rationale**:
- Rust has 85% coverage but no tracking mechanism
- Swift components have <30% coverage - need to track improvements
- Prevents coverage regressions

**Tasks**:
```bash
# Set up cargo-tarpaulin for Rust
# Create scripts/coverage.sh
# Add coverage badges to README.md
# Configure CI to upload coverage reports (Codecov/Coveralls)
```

**Acceptance Criteria**:
- ✓ `just coverage` generates HTML report
- ✓ Coverage data uploaded to coverage service in CI
- ✓ README.md shows coverage badges
- ✓ CONTRIBUTING.md documents coverage requirements (e.g., "new code must have >80% coverage")

---

#### 6. Create Quick Dev Environment Setup Script
**Goal**: Zero to productive in <5 minutes

**Rationale**:
- Onboarding friction = lost productivity
- Different setups between Linux (Rust only) and macOS (full stack)
- Easy to forget a dependency

**Tasks**:
```bash
# Create scripts/setup-dev-env.sh
# Checks and installs:
- Rust toolchain (rustup, cargo, rustfmt, clippy)
- Oxen CLI (pip install oxen-ai or cargo install oxen)
- pre-commit hooks
- just command runner
- Platform-specific tools (macOS: Xcode, Swift)
# Validates installation
# Sets up git hooks
```

**Acceptance Criteria**:
- ✓ Script runs on fresh Linux and macOS environments
- ✓ Idempotent (safe to run multiple times)
- ✓ Clear success/failure messages
- ✓ INSTALL.md updated to recommend this script first

---

### Phase 3: Quality of Life (Low Priority) - 2 hours

#### 7. Create Debugging Utilities
**Goal**: Common debugging tasks are scriptable

**Tasks**:
```bash
# Create scripts/debug-daemon.sh
- Tails daemon logs with filtering
- Shows daemon status
- Restarts daemon if needed

# Create scripts/debug-rust.sh
- Sets RUST_LOG=debug
- Runs CLI with verbose output
- Captures backtrace on panic

# Create scripts/watch-tests.sh
- Runs cargo-watch for continuous testing
- Shows only failures and summaries
```

**Acceptance Criteria**:
- ✓ 3 debugging scripts created in scripts/
- ✓ Scripts documented in TROUBLESHOOTING.md
- ✓ Added to `just debug-*` commands

---

#### 8. Improve Documentation Navigation
**Goal**: Make 16+ doc files easily discoverable

**Tasks**:
```bash
# Create docs/INDEX.md
- Categorized table of contents
- "Start here" section for new developers
- Links to all docs with one-sentence descriptions

# Update README.md
- Add "Documentation" section
- Link to docs/INDEX.md as single entry point
```

**Acceptance Criteria**:
- ✓ docs/INDEX.md created with categorized TOC
- ✓ README.md links to documentation index
- ✓ CLAUDE.md references doc index for deep dives

---

## Success Metrics

After completion, the following should be true:

1. **Build Speed**: `just test` runs all available tests with one command ✅
2. **Code Quality**: 100% of Rust code passes `cargo fmt --check` and `cargo clippy` ✅
3. **Developer Time**: New developer setup time reduced from ~30min to ~5min ✅
4. **CI Efficiency**: Pre-commit hooks catch 90%+ of CI failures locally ✅
5. **Coverage Visibility**: Coverage % tracked and displayed on README ✅
6. **Onboarding**: A developer can go from git clone to first contribution in <1 hour ✅

---

## Implementation Order (Prioritized)

**Today (6-8 hours):**
1. ✅ Fix crates.io network issue (BLOCKING) - 30 min
2. ✅ Create Justfile task runner - 1 hour
3. ✅ Add rustfmt.toml and .clippy.toml - 30 min
4. ✅ Set up pre-commit hooks - 45 min
5. ✅ Add code coverage tracking - 1 hour
6. ✅ Create dev environment setup script - 1 hour
7. ⏭️ Create debugging utilities - 1 hour (if time permits)
8. ⏭️ Create docs/INDEX.md - 30 min (if time permits)

**Deferred (requires macOS or lower priority):**
- Swift-specific tooling improvements
- Advanced debugging integrations
- Performance profiling setup

---

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Crates.io issue is environmental, not fixable | Medium | High | Document workaround, test on different network |
| Pre-commit hooks too slow | Low | Medium | Make hooks optional, optimize hook performance |
| Justfile unfamiliar to team | Low | Low | Provide `make` alternative, document extensively |
| Coverage tools don't work in Linux env | Medium | Low | Focus on Rust coverage only, defer Swift |

---

## Notes for Claude Code

**Context Switching**: Each improvement should be independently committable. If interrupted, prior work remains valuable.

**Testing**: After each improvement, validate:
```bash
# Does it work?
just <new-command>
# Is it documented?
grep <new-command> README.md || grep <new-command> CONTRIBUTING.md
# Does it fail gracefully?
# Try on wrong platform, try with missing dependencies
```

**Platform Awareness**: Current environment is Linux - Swift tooling will be mocked/documented for macOS testing later.

---

## Appendix: Related Files

**Will Create:**
- `justfile` (root)
- `OxVCS-CLI-Wrapper/rustfmt.toml`
- `OxVCS-CLI-Wrapper/.clippy.toml`
- `.pre-commit-config.yaml` (root)
- `scripts/setup-dev-env.sh`
- `scripts/coverage.sh`
- `scripts/debug-daemon.sh`
- `scripts/debug-rust.sh`
- `scripts/watch-tests.sh`
- `docs/INDEX.md`

**Will Modify:**
- `README.md` (add Quick Start, coverage badges, link to docs)
- `CONTRIBUTING.md` (add formatting guidelines, pre-commit instructions)
- `TROUBLESHOOTING.md` (add crates.io fix, link to debug scripts)
- `OxVCS-CLI-Wrapper/.cargo/config.toml` (potentially, to fix network issue)
- `CLAUDE.md` (update with new workflow commands)

---

**Last Updated**: 2025-10-30
**Status**: Ready for implementation
**Estimated Time**: 6-8 hours (full implementation)
