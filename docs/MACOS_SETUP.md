# macOS Environment Setup Guide

## Prerequisites Checklist

Run these commands to verify your environment:

```bash
# Check macOS version (need 14.0+)
sw_vers

# Check Xcode (need 15+)
xcodebuild -version

# Check Swift (need 5.9+)
swift --version

# Check Rust
rustc --version
cargo --version
```

## Step-by-Step Setup

### 1. Install Oxen CLI (Required)

```bash
# Option A: Via pip (recommended)
pip3 install oxen-ai

# Option B: Via cargo (alternative)
cargo install oxen

# Verify installation
oxen --version

# Should output: oxen 0.x.x or similar
```

### 2. Clone and Build All Components

```bash
# Navigate to project
cd ~/oxen-vcs-logic  # Or wherever you cloned it

# Pull latest changes
git checkout claude/session-011CUa3sb9HKKJzkJ1nyr5ax
git pull

# Build Rust CLI
cd OxVCS-CLI-Wrapper
cargo build --release
cargo test  # Should see 127 tests pass

# Build Swift LaunchAgent
cd ../OxVCS-LaunchAgent
swift build -c release
swift test  # Should see LockManager tests pass

# Build Swift App
cd ../OxVCS-App
swift build -c release
swift test  # Should see MockXPCClient tests pass

cd ..
```

### 3. Create Test Logic Pro Project

If you don't have Logic Pro, create a minimal test structure:

```bash
# Create minimal .logicx structure for testing
mkdir -p ~/Desktop/TestProject.logicx/Alternatives/001
echo "<?xml version=\"1.0\" encoding=\"UTF-8\"?><test/>" > ~/Desktop/TestProject.logicx/Alternatives/001/ProjectData

# Verify detection works
./OxVCS-CLI-Wrapper/target/release/oxenvcs-cli --version
```

### 4. Initialize Test Repository

```bash
# Create a temp directory for testing
mkdir -p ~/oxen-test
cd ~/oxen-test

# Initialize an oxen repo to verify CLI works
oxen init
oxen status

# If that works, you're ready!
```

## Troubleshooting

### Issue: oxen command not found
```bash
# Find where pip installed it
pip3 show oxen-ai

# Add to PATH if needed
export PATH="$PATH:$HOME/.local/bin"
# Or add to ~/.zshrc or ~/.bash_profile
```

### Issue: Swift build fails
```bash
# Make sure Xcode command line tools are selected
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
sudo xcode-select --install
```

### Issue: Cargo build fails
```bash
# Update Rust
rustup update

# Clear cache and rebuild
cd OxVCS-CLI-Wrapper
cargo clean
cargo build
```

## Verification Checklist

Run these to verify everything is working:

```bash
# ✅ Oxen CLI works
oxen --version

# ✅ Rust CLI builds
cd OxVCS-CLI-Wrapper && cargo test

# ✅ Swift components build
cd ../OxVCS-LaunchAgent && swift test
cd ../OxVCS-App && swift test

# ✅ Can detect Logic Pro projects
./OxVCS-CLI-Wrapper/target/release/oxenvcs-cli --help
```

Once all checkmarks are complete, you're ready for Phase 2!
