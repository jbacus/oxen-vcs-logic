## Overview

Implement process efficiency improvements to reduce friction and improve developer productivity when working with the Oxen-VCS codebase.

**Full Workplan**: See [WORKPLAN_PROCESS_EFFICIENCY.md](https://github.com/jbacus/oxen-vcs-logic/blob/claude/workplan-process-efficiency-011CUdVbYXsN3Um2HtBueeKU/WORKPLAN_PROCESS_EFFICIENCY.md)

**Estimated Time**: 6-8 hours
**Branch**: `claude/workplan-process-efficiency-011CUdVbYXsN3Um2HtBueeKU`

---

## Improvements to Implement

### Phase 1: Foundation (HIGH Priority) - 2-3 hours

- [ ] **1. Create Unified Task Runner (Justfile)**
  - Single entry point for common tasks (`just test`, `just build`, `just lint`, etc.)
  - Self-documenting with `just --list`
  - Reduces cognitive load and standardizes workflows
  - **Files**: Create `justfile` at project root
  - **Update**: README.md with quick start commands

- [ ] **2. Standardize Rust Tooling Configuration**
  - Add `rustfmt.toml` for consistent formatting
  - Add `.clippy.toml` for linting configuration
  - Enforce project style locally (currently only in CI)
  - **Files**: `OxVCS-CLI-Wrapper/rustfmt.toml`, `OxVCS-CLI-Wrapper/.clippy.toml`
  - **Update**: CONTRIBUTING.md with formatting guidelines

### Phase 2: Developer Experience (MEDIUM Priority) - 2-3 hours

- [ ] **3. Create Pre-Commit Hook System**
  - Catch formatting/lint issues before commit, not in CI
  - Faster feedback loop, reduces CI failures
  - **Files**: Create `.pre-commit-config.yaml`
  - **Update**: CONTRIBUTING.md with installation instructions
  - **Hooks**: trailing-whitespace, cargo fmt, cargo clippy, check-yaml/json

- [ ] **4. Add Code Coverage Tracking**
  - Set up cargo-tarpaulin for Rust coverage reports
  - Add coverage badges to README
  - Track coverage trends over time
  - **Files**: Create `scripts/coverage.sh`
  - **CI**: Configure coverage upload (Codecov/Coveralls)
  - **Update**: README.md with badges, CONTRIBUTING.md with requirements

- [ ] **5. Create Quick Dev Environment Setup Script**
  - Zero to productive in <5 minutes
  - Checks and installs: Rust toolchain, Oxen CLI, pre-commit, just
  - Platform-aware (Linux vs macOS)
  - **Files**: Create `scripts/setup-dev-env.sh`
  - **Update**: INSTALL.md to recommend script first

### Phase 3: Quality of Life (LOW Priority) - 2 hours

- [ ] **6. Create Debugging Utilities**
  - `scripts/debug-daemon.sh` - Tail logs, show status, restart daemon
  - `scripts/debug-rust.sh` - Run CLI with verbose output and backtraces
  - `scripts/watch-tests.sh` - Continuous testing with cargo-watch
  - **Update**: TROUBLESHOOTING.md with script documentation

- [ ] **7. Improve Documentation Navigation**
  - Create `docs/INDEX.md` with categorized table of contents
  - "Start here" section for new developers
  - Links to all 16+ docs with descriptions
  - **Update**: README.md to link to documentation index

---

## Success Metrics

After completion:

- ✅ `just test` runs all tests with one command
- ✅ 100% of Rust code passes `cargo fmt --check` and `cargo clippy`
- ✅ New developer setup time: 30min → 5min
- ✅ Pre-commit hooks catch 90%+ of CI failures locally
- ✅ Coverage % tracked and displayed on README
- ✅ First contribution time: <1 hour from clone

---

## Implementation Notes

**Platform Awareness**: Current Linux environment can handle Rust improvements. Swift-specific tooling should be documented for macOS testing.

**Testing Strategy**: After each improvement, validate:
```bash
# Does it work?
just <new-command>
# Is it documented?
grep <new-command> README.md
# Does it fail gracefully?
# Test on wrong platform, with missing dependencies
```

**Incremental Commits**: Each improvement should be independently committable for easy review and rollback.

---

## Related Files

**Will Create**:
- `justfile`
- `OxVCS-CLI-Wrapper/rustfmt.toml`
- `OxVCS-CLI-Wrapper/.clippy.toml`
- `.pre-commit-config.yaml`
- `scripts/setup-dev-env.sh`
- `scripts/coverage.sh`
- `scripts/debug-daemon.sh`
- `scripts/debug-rust.sh`
- `scripts/watch-tests.sh`
- `docs/INDEX.md`

**Will Modify**:
- `README.md` (Quick Start, coverage badges, docs link)
- `CONTRIBUTING.md` (formatting guidelines, pre-commit)
- `TROUBLESHOOTING.md` (debug scripts)
- `CLAUDE.md` (new workflow commands)

---

## Suggested Labels

- `enhancement`
- `developer-experience`
- `documentation`
- `good first issue` (individual tasks can be split out)
