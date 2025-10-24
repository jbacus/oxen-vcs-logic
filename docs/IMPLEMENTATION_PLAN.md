# Oxen-VCS Implementation Plan

## Prerequisites

**Environment Setup**
- macOS development machine (14.0+)
- Xcode 15+ with Swift 5.9+
- Oxen.ai CLI installed and tested
- Rust toolchain (for FFI wrapper optimization)
- Logic Pro 11.x for testing

**Technical Dependencies**
- Oxen Python client or liboxen Rust crate
- FSEvents framework access
- SMAppService framework (macOS 13+)
- IPC mechanism (XPC or Darwin notifications)

---

## Phase 1: Core Data Management (MVP)

**Objective:** Prove the versioning model works with Logic Pro's folder structure.

### 1.1 Repository Structure Layer
- [ ] Implement folder-based project detection/enforcement
- [ ] Generate .oxenignore template with asset classification rules
- [ ] Build Oxen initialization wrapper (`oxen.init()` + populate ignore file)

### 1.2 Basic Oxen Integration
- [ ] Implement core operations module (init, add, commit, log, restore)
- [ ] Add structured commit message format (BPM, sample rate, key signature)
- [ ] Test with sample Logic Pro folder projects

### 1.3 Minimal FSEvents Monitor
- [ ] Create standalone FSEvents listener in Swift
- [ ] Implement basic debounce logic (30-60s inactivity threshold)
- [ ] Test detection of projectData file changes

**Deliverable:** Command-line tool that can initialize, stage, and commit Logic Pro folder projects using Oxen.

---

## Phase 2: Service Architecture & Resilience

**Objective:** Build the production-grade macOS service layer.

### 2.1 LaunchAgent Implementation
- [ ] Create LaunchAgent plist configuration
- [ ] Implement SMAppService registration
- [ ] Build daemon with FSEvents monitoring and IPC listener

### 2.2 Power Management Integration
- [ ] Register for system notifications (sleep/shutdown)
- [ ] Implement emergency commit logic
- [ ] Test with forced sleep/shutdown scenarios

### 2.3 Oxen CLI Wrapper Optimization
- [ ] Build Rust FFI wrapper around liboxen
- [ ] Package as embedded helper tool
- [ ] Implement secure IPC (XPC)
- [ ] Benchmark performance

### 2.4 Draft Tracking System
- [ ] Create local "draft" branch on init
- [ ] Implement auto-commit workflow
- [ ] Add draft pruning logic
- [ ] Test continuous editing sessions

**Deliverable:** Background daemon with automatic tracking and power-safe commits.

---

## Phase 3: UI Application & Collaboration

**Objective:** Complete user-facing application and team workflow features.

### 3.1 Main UI Application
- [ ] Repository browser view
- [ ] Project initialization wizard
- [ ] Milestone commit interface
- [ ] Rollback/restore interface
- [ ] Settings panel

### 3.2 Exclusive File Locking System
- [ ] Design lock manifest schema
- [ ] Implement lock acquisition/release
- [ ] Enforce lock in LaunchAgent
- [ ] Build admin force-break mechanism

### 3.3 Manual Merge Protocol
- [ ] Document FCP XML reconciliation workflow
- [ ] Add UI helpers for export/import
- [ ] Test with divergent branches

### 3.4 Milestone Commit Pre-Flight
- [ ] Implement cleanup automation
- [ ] Add confirmation dialog
- [ ] Execute staging → commit → push sequence

**Deliverable:** Complete macOS application with collaboration features.

---

## Testing Strategy

- Unit tests for all core functions
- Integration tests for FSEvents → commit pipeline
- System tests with 8+ hour sessions, multiple users
- Performance benchmarks for large projects (50+ GB)

---

## Critical Path Items

1. Prove Oxen performance with real Logic Pro project
2. Validate FSEvents debounce accuracy
3. Test power event handling thoroughly
4. Implement locking before multi-user testing

**Estimated Timeline:** 12-16 weeks with one developer.
