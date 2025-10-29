# Oxen Subprocess Integration Guide

## Goal
Replace the stub implementation with real oxen CLI subprocess calls.

## Current State
- `oxen_subprocess.rs` is complete and tested (parsing logic)
- `oxen_ops.rs` uses the stub
- Main CLI calls `oxen_ops.rs`

## Integration Steps

### Step 1: Add Feature Flag (Optional but Recommended)

Edit `OxVCS-CLI-Wrapper/Cargo.toml`:

```toml
[features]
default = ["subprocess"]
subprocess = []
stub = []
```

This allows switching between stub and subprocess during development.

### Step 2: Update oxen_ops.rs

Create a new implementation that wraps `OxenSubprocess`:

```rust
// OxVCS-CLI-Wrapper/src/oxen_ops.rs

use crate::oxen_subprocess::OxenSubprocess;
use anyhow::Result;
use std::path::Path;

pub struct OxenRepository {
    subprocess: OxenSubprocess,
    path: std::path::PathBuf,
}

impl OxenRepository {
    pub fn init(path: &Path) -> Result<Self> {
        let subprocess = OxenSubprocess::new();
        subprocess.init(path)?;

        Ok(Self {
            subprocess,
            path: path.to_path_buf(),
        })
    }

    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            subprocess: OxenSubprocess::new(),
            path: path.to_path_buf(),
        })
    }

    pub fn add(&self, files: &[&Path]) -> Result<()> {
        self.subprocess.add(&self.path, files)
    }

    pub fn add_all(&self) -> Result<()> {
        self.subprocess.add_all(&self.path)
    }

    pub fn commit(&self, message: &str) -> Result<String> {
        let commit = self.subprocess.commit(&self.path, message)?;
        Ok(commit.id)
    }

    pub fn log(&self, limit: Option<usize>) -> Result<Vec<crate::oxen_subprocess::CommitInfo>> {
        self.subprocess.log(&self.path, limit)
    }

    pub fn status(&self) -> Result<crate::oxen_subprocess::StatusInfo> {
        self.subprocess.status(&self.path)
    }

    pub fn checkout(&self, target: &str) -> Result<()> {
        self.subprocess.checkout(&self.path, target)
    }

    pub fn current_branch(&self) -> Result<String> {
        self.subprocess.current_branch(&self.path)
    }
}
```

### Step 3: Test the Integration

Create `OxVCS-CLI-Wrapper/tests/integration_basic.rs`:

```rust
use oxenvcs_cli::*;
use std::path::PathBuf;
use tempfile::TempDir;

#[test]
fn test_init_and_commit_workflow() {
    // Only run if oxen CLI is available
    if !OxenSubprocess::new().is_available() {
        eprintln!("Skipping test: oxen CLI not installed");
        return;
    }

    let temp = TempDir::new().unwrap();
    let project_path = temp.path().join("test.logicx");
    std::fs::create_dir_all(&project_path.join("Alternatives/001")).unwrap();
    std::fs::write(
        project_path.join("Alternatives/001/ProjectData"),
        b"test data"
    ).unwrap();

    // Test initialization
    let repo = OxenRepository::init(&project_path).unwrap();

    // Add files
    repo.add_all().unwrap();

    // Create commit
    let commit_id = repo.commit("Initial commit").unwrap();
    assert!(!commit_id.is_empty());

    // Verify commit in log
    let log = repo.log(Some(10)).unwrap();
    assert_eq!(log.len(), 1);
    assert!(log[0].message.contains("Initial commit"));

    println!("✓ Integration test passed!");
}

#[test]
fn test_status_shows_changes() {
    if !OxenSubprocess::new().is_available() {
        return;
    }

    let temp = TempDir::new().unwrap();
    let repo_path = temp.path();

    // Init repo
    let repo = OxenRepository::init(repo_path).unwrap();

    // Create a file
    std::fs::write(repo_path.join("test.txt"), b"hello").unwrap();

    // Check status
    let status = repo.status().unwrap();
    assert!(!status.untracked.is_empty());

    println!("✓ Status test passed!");
}
```

Add to `Cargo.toml`:

```toml
[dev-dependencies]
tempfile = "3.8"
```

Run tests:

```bash
cd OxVCS-CLI-Wrapper
cargo test --test integration_basic
```

### Step 4: Update Main CLI

The main CLI in `src/main.rs` should already use `OxenRepository`, so it should just work! Test it:

```bash
# Build
cargo build --release

# Test with a temp directory
mkdir -p /tmp/test-project.logicx/Alternatives/001
echo "test" > /tmp/test-project.logicx/Alternatives/001/ProjectData

# Initialize
./target/release/oxenvcs-cli init --logic /tmp/test-project.logicx

# Check status
./target/release/oxenvcs-cli status /tmp/test-project.logicx

# Add files
./target/release/oxenvcs-cli add --all /tmp/test-project.logicx

# Commit
./target/release/oxenvcs-cli commit -m "Test commit" /tmp/test-project.logicx

# View log
./target/release/oxenvcs-cli log /tmp/test-project.logicx
```

### Step 5: Test with Real Logic Pro Project (if available)

```bash
# Navigate to an actual Logic Pro project
cd ~/Music/YourActualProject.logicx

# Initialize
/path/to/oxenvcs-cli init --logic .

# Make a change in Logic Pro, save, then:
/path/to/oxenvcs-cli status .
/path/to/oxenvcs-cli add --all .
/path/to/oxenvcs-cli commit -m "Saved changes from Logic Pro" --bpm 120 .

# View history
/path/to/oxenvcs-cli log .
```

## Expected Results

✅ Repository initializes without errors
✅ Files are added to staging
✅ Commits are created successfully
✅ Log shows commit history
✅ Status shows modified/untracked files
✅ Checkout switches between commits

## Common Issues

### Issue: "oxen command not found"
```bash
# Subprocess can't find oxen
# Solution: Add oxen to PATH or use full path

export PATH="$PATH:$(pip3 show -f oxen-ai | grep Location | cut -d' ' -f2)/bin"
```

### Issue: Permission denied
```bash
# Make sure oxen CLI is executable
which oxen
ls -la $(which oxen)
chmod +x $(which oxen)
```

### Issue: Tests fail with "Repository not found"
This is expected if using the stub. Make sure:
1. Oxen CLI is installed
2. Tests are using real `OxenSubprocess`
3. Repository was actually initialized

## Verification Checklist

Run through this checklist to verify integration:

- [ ] `cargo test` passes (all 127+ tests)
- [ ] `cargo test --test integration_basic` passes
- [ ] CLI can initialize a repo
- [ ] CLI can add files
- [ ] CLI can commit changes
- [ ] CLI can show log
- [ ] CLI can show status
- [ ] CLI works with Logic Pro project structure

## Next Steps

Once integration is working:
1. Write more integration tests (see TESTING_ROADMAP.md Phase 1)
2. Test with large files (audio)
3. Test with real Logic Pro project
4. Begin Swift component testing
