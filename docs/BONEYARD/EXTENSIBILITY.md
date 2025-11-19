# Oxen-VCS Extensibility Guide

**Last Updated**: 2025-11-17
**Purpose**: Track shared vs. application-specific code for future multi-application support

---

## Overview

Oxen-VCS is currently implemented for **Logic Pro**, but the architecture is designed to support any application with similar version control challenges:
- Binary, non-mergeable project files
- Large embedded assets (audio, textures, models)
- Collaboration workflows requiring pessimistic locking
- Generated/volatile files that should be excluded

**Potential applications**: SketchUp, Ableton Live, Pro Tools, Cubase, Blender, Unreal Engine, Unity

This document categorizes the codebase into:
- ✅ **Shared/Generic** - Works for any application (85% of codebase)
- 🎵 **Logic Pro-specific** - Needs custom implementation per application (15%)
- 🔄 **Refactoring opportunities** - How to improve extensibility

---

## Codebase Categorization

### Rust CLI Wrapper (`OxVCS-CLI-Wrapper/src/`)

#### ✅ Shared/Generic Components (Reusable for Any Application)

| File | Purpose | Shared % | Notes |
|------|---------|----------|-------|
| `oxen_subprocess.rs` | Oxen CLI subprocess wrapper | **100%** | Application-agnostic Oxen integration |
| `oxen_ops.rs` | High-level Oxen operations | **100%** | Generic init/commit/push/pull wrappers |
| `logger.rs` | Logging infrastructure | **100%** | No application dependencies |
| `progress.rs` | Progress bars for long operations | **100%** | Generic UI feedback |
| `backup_recovery.rs` | Emergency recovery system | **100%** | Works with any file structure |
| `network_resilience.rs` | Retry logic for network ops | **100%** | Generic network handling |
| `remote_lock.rs` | Remote lock management | **100%** | Generic locking protocol |
| `lock_integration.rs` | Lock enforcement logic | **100%** | Application-agnostic |
| `auth.rs` | Authentication handling | **100%** | Generic credential management |
| `daemon_client.rs` | IPC client for daemon | **100%** | Generic XPC communication |
| `operation_history.rs` | Operation history tracking | **100%** | Generic audit log |
| `hooks.rs` | Git-style hooks system | **100%** | Generic lifecycle hooks |
| `search.rs` | Commit history search | **100%** | Generic search across metadata |
| `collaboration.rs` | Collaboration workflows | **100%** | Generic multi-user patterns |
| `conflict_detection.rs` | Conflict detection logic | **95%** | Mostly generic, uses metadata trait |
| `workflow_automation.rs` | Automated workflows | **90%** | Generic framework, custom triggers |
| `draft_manager.rs` | Draft branch management | **100%** | Generic branching strategy |
| `console/mod.rs` | Console UI utilities | **100%** | Generic colored output |
| `liboxen_stub/` | Oxen stub implementation | **100%** | Fallback, no dependencies |

**Total Shared**: ~19 files (**~2,200 lines**, 85% of CLI codebase)

---

#### 🎵 Logic Pro-Specific Components (Need Custom Implementations)

| File | Purpose | Custom % | SketchUp Equivalent | Other Apps |
|------|---------|----------|---------------------|------------|
| `logic_project.rs` | Project detection & validation | **100%** | `sketchup_project.rs` | `ableton_project.rs`, `protool_project.rs` |
| `commit_metadata.rs` | Structured commit metadata | **80%** | `sketchup_metadata.rs` (different fields) | `daw_metadata.rs` (shared struct) |
| `ignore_template.rs` | .oxenignore generation | **100%** | `sketchup_ignore_template.rs` | Per-application templates |
| `logic_parser/` | Binary .logicx parser | **100%** | `sketchup_parser/` (optional) | Optional per-app parsers |
| `metadata_diff/` | Metadata diffing engine | **60%** | Reuse framework, custom fields | Generic trait + implementations |

**Total Custom**: ~5 files/modules (**~800 lines**, 15% of CLI codebase)

---

#### 🔄 Refactoring Opportunities for Better Extensibility

**1. Create Abstract Project Trait**

**Current**: `logic_project.rs` hardcodes Logic Pro logic
```rust
pub struct LogicProject {
    pub path: PathBuf,
    pub project_data_path: PathBuf,
}

impl LogicProject {
    pub fn detect(path: impl AsRef<Path>) -> Result<Self> { ... }
    pub fn tracked_paths(&self) -> Vec<PathBuf> { ... }
    pub fn ignored_patterns() -> Vec<&'static str> { ... }
}
```

**Refactored**: Generic trait with per-app implementations
```rust
// src/project.rs (new shared module)
pub trait Project {
    fn detect(path: impl AsRef<Path>) -> Result<Self> where Self: Sized;
    fn name(&self) -> String;
    fn tracked_paths(&self) -> Vec<PathBuf>;
    fn ignored_patterns() -> Vec<&'static str>;
    fn root_path(&self) -> &Path;
}

// src/logic/project.rs
pub struct LogicProject { ... }
impl Project for LogicProject { ... }

// src/sketchup/project.rs (future)
pub struct SketchUpProject { ... }
impl Project for SketchUpProject { ... }
```

**Benefits**:
- Single interface for all project types
- Easy to add new applications
- CLI commands work with `dyn Project` trait objects

---

**2. Abstract Metadata System**

**Current**: `commit_metadata.rs` hardcodes BPM/sample rate/key
```rust
pub struct CommitMetadata {
    pub message: String,
    pub bpm: Option<f32>,
    pub sample_rate: Option<u32>,
    pub key_signature: Option<String>,
    pub tags: Vec<String>,
}
```

**Refactored**: Generic metadata with app-specific extensions
```rust
// src/metadata.rs (new shared module)
pub trait ProjectMetadata: Serialize + Deserialize {
    fn format_commit_message(&self) -> String;
    fn parse_commit_message(msg: &str) -> Self;
    fn compare_with(&self, other: &Self) -> String;
}

// Base struct shared by all apps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseMetadata {
    pub message: String,
    pub tags: Vec<String>,
    pub timestamp: Option<i64>,
}

// src/logic/metadata.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicMetadata {
    #[serde(flatten)]
    pub base: BaseMetadata,
    pub bpm: Option<f32>,
    pub sample_rate: Option<u32>,
    pub key_signature: Option<String>,
}
impl ProjectMetadata for LogicMetadata { ... }

// src/sketchup/metadata.rs (future)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SketchUpMetadata {
    #[serde(flatten)]
    pub base: BaseMetadata,
    pub face_count: Option<u64>,
    pub component_count: Option<u32>,
    pub file_size: Option<u64>,
}
impl ProjectMetadata for SketchUpMetadata { ... }
```

**Benefits**:
- Shared commit message parsing logic
- Type-safe metadata per application
- Easy to add custom fields

---

**3. Plugin-Style Architecture**

**Current**: All application logic compiled into single binary

**Refactored**: Dynamic application plugins
```rust
// src/app_registry.rs (new)
pub struct AppPlugin {
    pub name: &'static str,
    pub detect_fn: fn(&Path) -> Result<Box<dyn Project>>,
    pub metadata_parser: fn(&str) -> Box<dyn ProjectMetadata>,
    pub ignore_template_fn: fn() -> String,
}

pub static SUPPORTED_APPS: &[AppPlugin] = &[
    AppPlugin {
        name: "logic-pro",
        detect_fn: |p| Ok(Box::new(LogicProject::detect(p)?)),
        metadata_parser: |s| Box::new(LogicMetadata::parse_commit_message(s)),
        ignore_template_fn: || generate_logic_oxenignore(),
    },
    AppPlugin {
        name: "sketchup",
        detect_fn: |p| Ok(Box::new(SketchUpProject::detect(p)?)),
        metadata_parser: |s| Box::new(SketchUpMetadata::parse_commit_message(s)),
        ignore_template_fn: || generate_sketchup_oxenignore(),
    },
];

// Auto-detect application
pub fn detect_project(path: &Path) -> Result<(Box<dyn Project>, &'static AppPlugin)> {
    for plugin in SUPPORTED_APPS {
        if let Ok(project) = (plugin.detect_fn)(path) {
            return Ok((project, plugin));
        }
    }
    Err(anyhow!("No supported project found at {}", path.display()))
}
```

**CLI usage**:
```rust
// main.rs
let (project, app_plugin) = detect_project(&args.path)?;
println!("Detected {} project: {}", app_plugin.name, project.name());
```

**Benefits**:
- Auto-detect any supported application
- Single CLI binary supports all apps
- Easy to add new applications without refactoring

---

### Swift LaunchAgent (`OxVCS-LaunchAgent/Sources/`)

#### ✅ Shared/Generic Components

| File | Purpose | Shared % | Notes |
|------|---------|----------|-------|
| `Daemon.swift` | Main daemon orchestration | **95%** | Minimal app-specific logic |
| `PowerManagement.swift` | Sleep/shutdown handlers | **100%** | Generic power events |
| `XPCService.swift` | XPC IPC service | **100%** | Generic message passing |
| `LockManager.swift` | File lock enforcement | **100%** | Generic locking logic |
| `ServiceManager.swift` | LaunchAgent registration | **100%** | Generic macOS service mgmt |
| `CommitOrchestrator.swift` | Auto-commit workflow | **90%** | Generic commit sequence |

**Total Shared**: 6 files (**~800 lines**)

---

#### 🎵 Application-Specific Components

| File | Purpose | Custom % | What Needs Customization |
|------|---------|----------|--------------------------|
| `FSEventsMonitor.swift` | File system monitoring | **20%** | Watch paths, file extensions |
| `OxVCSDaemon.swift` | Daemon entry point | **10%** | Plist path, bundle ID |

**Customization needed**:
```swift
// FSEventsMonitor.swift
// Logic Pro version:
let watchPaths = ["\(projectPath)/Alternatives"]
let fileExtension = "ProjectData"

// SketchUp version:
let watchPaths = [projectPath] // Watch parent directory
let fileExtension = "skp"
```

---

#### 🔄 Refactoring for Extensibility

**Configuration-Driven Monitoring**:
```swift
// AppConfig.swift (new)
protocol AppConfiguration {
    var watchSubpaths: [String] { get }
    var projectFileExtension: String { get }
    var debounceInterval: TimeInterval { get }
}

struct LogicProConfig: AppConfiguration {
    let watchSubpaths = ["Alternatives"]
    let projectFileExtension = "ProjectData"
    let debounceInterval: TimeInterval = 30.0
}

struct SketchUpConfig: AppConfiguration {
    let watchSubpaths = [] // Watch root
    let projectFileExtension = "skp"
    let debounceInterval: TimeInterval = 60.0
}

// FSEventsMonitor becomes generic
class FSEventsMonitor {
    let config: AppConfiguration

    init(config: AppConfiguration) {
        self.config = config
    }

    func startMonitoring(path: String) {
        let watchPaths = config.watchSubpaths.map { "\(path)/\($0)" }
        // Use config.projectFileExtension for filtering
    }
}
```

---

### Swift App (`OxVCS-App/Sources/`)

#### ✅ Shared/Generic Components

| Component | Purpose | Shared % | Notes |
|-----------|---------|----------|-------|
| `OxVCSApp.swift` | SwiftUI app entry point | **100%** | Generic app lifecycle |
| `AppDelegate.swift` | Menu bar, system integration | **90%** | Minor app name changes |
| `Services/OxenDaemonXPCClient.swift` | XPC client | **100%** | Generic daemon communication |
| `ViewModels/ProjectListViewModel.swift` | Project list logic | **100%** | Generic MVVM pattern |
| `ViewModels/ProjectDetailViewModel.swift` | Project detail logic | **95%** | Metadata parsing is custom |
| `Views/SwiftUI/ContentView.swift` | Main navigation | **100%** | Generic NavigationSplitView |
| `Views/SwiftUI/ProjectListContentView.swift` | Sidebar | **100%** | Generic list view |
| `Views/SwiftUI/SwiftUIStatusBar.swift` | Status indicator | **100%** | Generic status display |
| `Views/LockManagementView.swift` | Lock UI | **100%** | Generic lock display |
| `Views/RollbackWindow.swift` | Rollback interface | **100%** | Generic commit selection |
| `Views/SettingsWindow.swift` | Settings UI | **100%** | Generic preferences |
| `Models/Project.swift` | Project data model | **100%** | Generic project struct |

**Total Shared**: ~12 files (**~1,200 lines**)

---

#### 🎵 Application-Specific Components

| Component | Purpose | Custom % | What Changes |
|-----------|---------|----------|--------------|
| `Views/SwiftUI/ProjectDetailContentView.swift` | Commit history with metadata | **40%** | Metadata display fields |
| `Views/SwiftUI/MilestoneCommitView.swift` | Milestone commit form | **60%** | Metadata input fields |
| `Views/ProjectWizardWindow.swift` | New project wizard | **30%** | Project type detection |
| `Views/MergeHelperWindow.swift` | Manual merge UI | **80%** | Export format (FCP XML vs IFC) |

**Example customization**:
```swift
// ProjectDetailContentView.swift

// Logic Pro version:
struct MetadataView: View {
    var body: some View {
        VStack(alignment: .leading) {
            if let bpm = metadata.bpm {
                Label("\(bpm, specifier: "%.1f") BPM", systemImage: "metronome")
            }
            if let sr = metadata.sampleRate {
                Label("\(sr / 1000) kHz", systemImage: "waveform")
            }
        }
    }
}

// SketchUp version:
struct MetadataView: View {
    var body: some View {
        VStack(alignment: .leading) {
            if let faces = metadata.faceCount {
                Label("\(faces) faces", systemImage: "cube")
            }
            if let components = metadata.componentCount {
                Label("\(components) components", systemImage: "square.stack.3d.up")
            }
        }
    }
}
```

---

#### 🔄 Refactoring for Extensibility

**Generic Metadata Display**:
```swift
// MetadataDisplayable protocol
protocol MetadataDisplayable {
    func displayRows() -> [MetadataRow]
}

struct MetadataRow {
    let label: String
    let value: String
    let icon: String
}

// Logic implementation
extension LogicMetadata: MetadataDisplayable {
    func displayRows() -> [MetadataRow] {
        var rows = [MetadataRow]()
        if let bpm = self.bpm {
            rows.append(MetadataRow(label: "BPM", value: "\(bpm)", icon: "metronome"))
        }
        if let sr = self.sampleRate {
            rows.append(MetadataRow(label: "Sample Rate", value: "\(sr/1000) kHz", icon: "waveform"))
        }
        return rows
    }
}

// Generic view
struct MetadataView: View {
    let metadata: MetadataDisplayable

    var body: some View {
        VStack(alignment: .leading) {
            ForEach(metadata.displayRows(), id: \.label) { row in
                Label(row.value, systemImage: row.icon)
            }
        }
    }
}
```

---

## Application Comparison Matrix

### Logic Pro vs SketchUp Implementation Differences

| Aspect | Logic Pro | SketchUp | Shared Code |
|--------|-----------|----------|-------------|
| **Project structure** | Folder (`.logicx/`) | Single file (`.skp`) | ❌ Custom detection |
| **Binary file** | `ProjectData` | `.skp` file | ❌ Different paths |
| **Asset location** | `Resources/` subfolder | Embedded in `.skp` | ❌ Different tracking |
| **Metadata fields** | BPM, sample rate, key | Faces, components, size | ❌ Custom fields |
| **Ignored files** | Bounces/, Freeze Files/ | .skb, _AutoSave_* | ❌ Different patterns |
| **FSEvents watch** | `Alternatives/` subfolder | Parent directory | 🟡 Configurable |
| **Debounce time** | 30-60 seconds | 60-120 seconds | 🟡 Configurable |
| **Lock enforcement** | On folder | On file | ✅ 100% shared |
| **Oxen operations** | init/commit/push/pull | (same) | ✅ 100% shared |
| **Power management** | Sleep/shutdown hooks | (same) | ✅ 100% shared |
| **XPC communication** | Daemon IPC | (same) | ✅ 100% shared |
| **UI framework** | SwiftUI | (same) | ✅ 95% shared |

**Overall code reuse**: **85% shared**, **15% custom per application**

---

## Recommended Directory Structure for Multi-App Support

```
OxVCS-CLI-Wrapper/src/
├── main.rs                          # ✅ Generic CLI entry point
├── lib.rs                           # ✅ Generic library exports
├── project.rs                       # 🆕 Generic Project trait
├── metadata.rs                      # 🆕 Generic Metadata trait
├── app_registry.rs                  # 🆕 Application plugin registry
│
├── core/                            # ✅ All shared/generic modules
│   ├── oxen_subprocess.rs
│   ├── oxen_ops.rs
│   ├── logger.rs
│   ├── progress.rs
│   ├── backup_recovery.rs
│   ├── network_resilience.rs
│   ├── remote_lock.rs
│   ├── lock_integration.rs
│   ├── auth.rs
│   ├── daemon_client.rs
│   ├── operation_history.rs
│   ├── hooks.rs
│   ├── search.rs
│   ├── collaboration.rs
│   ├── conflict_detection.rs
│   ├── workflow_automation.rs
│   ├── draft_manager.rs
│   └── console/
│
├── apps/                            # 🎵 Application-specific modules
│   ├── logic_pro/
│   │   ├── mod.rs
│   │   ├── project.rs               # LogicProject impl
│   │   ├── metadata.rs              # LogicMetadata impl
│   │   ├── ignore_template.rs
│   │   └── parser/                  # Optional binary parser
│   │
│   ├── sketchup/                    # 🔮 Future
│   │   ├── mod.rs
│   │   ├── project.rs               # SketchUpProject impl
│   │   ├── metadata.rs              # SketchUpMetadata impl
│   │   ├── ignore_template.rs
│   │   └── parser/                  # Optional .skp header parser
│   │
│   └── ableton/                     # 🔮 Future
│       └── ...
│
└── liboxen_stub/                    # ✅ Generic fallback
```

**Benefits**:
- Clear separation of concerns
- Easy to add new apps in `apps/` directory
- Single CLI binary supports all apps via auto-detection
- Shared code in `core/` never changes when adding apps

---

## Adding a New Application: Step-by-Step

### Example: Adding Ableton Live Support

**1. Create application module**
```bash
mkdir -p OxVCS-CLI-Wrapper/src/apps/ableton
```

**2. Implement Project trait**
```rust
// src/apps/ableton/project.rs
pub struct AbletonProject {
    pub path: PathBuf, // Path to .als file
}

impl Project for AbletonProject {
    fn detect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        if path.extension() != Some(OsStr::new("als")) {
            return Err(anyhow!("Not an Ableton project"));
        }
        Ok(Self { path: path.to_path_buf() })
    }

    fn tracked_paths(&self) -> Vec<PathBuf> {
        vec![self.path.clone()]
    }

    fn ignored_patterns() -> Vec<&'static str> {
        vec!["Backup/", "*.asd", "Ableton Project Info/"]
    }
}
```

**3. Implement Metadata**
```rust
// src/apps/ableton/metadata.rs
#[derive(Serialize, Deserialize)]
pub struct AbletonMetadata {
    #[serde(flatten)]
    pub base: BaseMetadata,
    pub bpm: Option<f32>,
    pub time_signature: Option<String>,
    pub track_count: Option<u32>,
}

impl ProjectMetadata for AbletonMetadata {
    fn format_commit_message(&self) -> String { ... }
    fn parse_commit_message(msg: &str) -> Self { ... }
}
```

**4. Register in app_registry.rs**
```rust
pub static SUPPORTED_APPS: &[AppPlugin] = &[
    // ... existing apps ...
    AppPlugin {
        name: "ableton-live",
        detect_fn: |p| Ok(Box::new(AbletonProject::detect(p)?)),
        metadata_parser: |s| Box::new(AbletonMetadata::parse_commit_message(s)),
        ignore_template_fn: || generate_ableton_oxenignore(),
    },
];
```

**5. No changes needed in**:
- `oxen_subprocess.rs` ✅
- `oxen_ops.rs` ✅
- All of `core/` ✅
- Swift LaunchAgent (just update config)
- Swift App (metadata view updates only)

**Effort**: ~2-3 days for new application support

---

## Testing Strategy for Multi-App Support

### Unit Tests (App-Specific)
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sketchup_detection() {
        let project = SketchUpProject::detect("test.skp").unwrap();
        assert_eq!(project.file_extension(), "skp");
    }

    #[test]
    fn test_logic_detection() {
        let project = LogicProject::detect("test.logicx").unwrap();
        assert!(project.project_data_path.exists());
    }
}
```

### Integration Tests (Generic)
```rust
#[test]
fn test_auto_detection() {
    // Should detect any supported project type
    let (project, plugin) = detect_project("test.logicx").unwrap();
    assert_eq!(plugin.name, "logic-pro");

    let (project, plugin) = detect_project("test.skp").unwrap();
    assert_eq!(plugin.name, "sketchup");
}
```

---

## Migration Path from Current Codebase

### Phase 1: Extract Generic Traits (1 week)
- [ ] Create `project.rs` trait
- [ ] Create `metadata.rs` trait
- [ ] Create `app_registry.rs`
- [ ] All tests still pass

### Phase 2: Refactor Logic Pro to Plugin (2 days)
- [ ] Move `logic_project.rs` → `apps/logic_pro/project.rs`
- [ ] Move `commit_metadata.rs` → `apps/logic_pro/metadata.rs`
- [ ] Move `ignore_template.rs` → `apps/logic_pro/ignore_template.rs`
- [ ] Update imports
- [ ] All tests still pass

### Phase 3: Add Second Application (1 week)
- [ ] Implement SketchUp plugin
- [ ] Test with real .skp files
- [ ] Validate auto-detection works

### Phase 4: Update Swift Components (3 days)
- [ ] Make LaunchAgent config-driven
- [ ] Update App UI for generic metadata display
- [ ] Test both Logic Pro and SketchUp workflows

**Total migration effort**: ~3 weeks (preserves all existing functionality)

---

## Current Status vs. Extensible Architecture

### Current (Logic Pro Only)
```
✅ Fully functional for Logic Pro
❌ Hardcoded Logic-specific logic in 5 key files
❌ Adding SketchUp requires forking or heavy refactoring
❌ No abstraction for project types
```

### After Refactoring
```
✅ Fully functional for Logic Pro (same features)
✅ SketchUp support in ~3 days
✅ Future apps (Ableton, Pro Tools) in ~2-3 days each
✅ Clean separation: 85% shared core, 15% custom per app
✅ Single CLI binary auto-detects project type
```

---

## Maintenance Considerations

### Shared Code Updates (Benefits All Apps)
- Oxen subprocess improvements → All apps benefit
- Network resilience enhancements → All apps benefit
- Lock management fixes → All apps benefit
- UI framework updates → All apps benefit

### Per-App Updates (Isolated)
- Logic Pro parser changes → No impact on SketchUp
- SketchUp metadata fields → No impact on Logic Pro
- Each app has independent test suite

**Risk**: Shared code changes require testing across all apps

**Mitigation**: Comprehensive integration test suite that runs on all registered apps

---

## Future Extensibility Wishlist

### 1. Web-Based Configuration
```rust
// Allow users to define custom applications via config file
// ~/.oxenvcs/apps/my-custom-app.toml
[app]
name = "my-custom-app"
project_extension = "myproj"
watch_paths = ["Data/"]
ignored_patterns = ["Cache/", "*.tmp"]

[metadata]
fields = [
    { name = "version", type = "string" },
    { name = "complexity", type = "int" },
]
```

### 2. Plugin System
```rust
// Allow third-party plugins as dynamic libraries
// ~/.oxenvcs/plugins/my-app-plugin.dylib
```

### 3. Universal Binary Parser
```rust
// Generic binary parsing framework
trait BinaryProjectParser {
    fn extract_metadata(&self, file: &Path) -> Result<Box<dyn ProjectMetadata>>;
}
```

---

## Summary

### Current Codebase Breakdown

| Component | Shared Code | App-Specific | Total |
|-----------|-------------|--------------|-------|
| **Rust CLI** | ~2,200 lines (85%) | ~800 lines (15%) | ~3,000 lines |
| **Swift LaunchAgent** | ~800 lines (95%) | ~100 lines (5%) | ~900 lines |
| **Swift App** | ~1,200 lines (90%) | ~200 lines (10%) | ~1,400 lines |
| **Total** | **~4,200 lines** | **~1,100 lines** | **~5,300 lines** |

### Code Reuse for New Applications

- **SketchUp**: ~85% code reuse (2-3 weeks implementation)
- **Ableton Live**: ~85% code reuse (2-3 weeks implementation)
- **Blender/Unity**: ~80% code reuse (file-based projects)

### Key Takeaway

**Oxen-VCS has excellent bones for multi-application support.** With minimal refactoring (~3 weeks), the codebase can cleanly support any application with similar version control challenges. The pessimistic locking model, Oxen integration, and macOS daemon architecture are all application-agnostic.

---

**Document Maintenance**: Update this file when:
- Adding/modifying shared components
- Implementing application-specific features
- Refactoring for better extensibility
- Adding new applications

**Next Review**: When SketchUp prototype is implemented
