# Homebrew Formula for Auxin CLI

This directory contains the Homebrew formula for installing `auxin` on macOS.

## Quick Installation (macOS)

### Option 1: Via Homebrew Tap (Recommended)

Once the formula is published to a tap:

```bash
brew tap jbacus/auxin
brew install auxin
```

### Option 2: Direct Installation from Formula

Install directly from this repository:

```bash
brew install --build-from-source /path/to/auxin/Auxin-CLI-Wrapper/formula/auxin.rb
```

### Option 3: Manual Installation Script

Use the installation script (works on both macOS and Linux):

```bash
cd Auxin-CLI-Wrapper
./install.sh
```

---

## Publishing to Homebrew

### Creating a Homebrew Tap

1. **Create a tap repository**:
   ```bash
   # Create a new GitHub repository named: homebrew-auxin
   # Repository URL will be: https://github.com/jbacus/homebrew-auxin
   ```

2. **Add the formula**:
   ```bash
   git clone https://github.com/jbacus/homebrew-auxin.git
   cd homebrew-auxin
   mkdir Formula
   cp /path/to/auxin/Auxin-CLI-Wrapper/formula/auxin.rb Formula/
   git add Formula/auxin.rb
   git commit -m "Add auxin formula"
   git push
   ```

3. **Update SHA256 hash**:
   ```bash
   # After creating a GitHub release with a tarball
   shasum -a 256 /path/to/v0.1.0.tar.gz

   # Update the sha256 in auxin.rb
   ```

4. **Test the formula**:
   ```bash
   brew install --build-from-source Formula/auxin.rb
   brew test auxin
   brew audit --strict auxin
   ```

### Creating a GitHub Release

Before publishing, create a tagged release:

```bash
cd auxin
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

Then create a release on GitHub with the tarball, and update the formula's `url` and `sha256`.

---

## Formula Development

### Testing the Formula Locally

```bash
# Install from local formula
brew install --build-from-source formula/auxin.rb

# Run tests
brew test auxin

# Audit formula
brew audit --strict auxin

# Uninstall
brew uninstall auxin
```

### Updating the Formula

When releasing a new version:

1. Update version in `url` field
2. Download the new tarball and compute SHA256:
   ```bash
   curl -L https://github.com/jbacus/auxin/archive/refs/tags/vX.Y.Z.tar.gz -o archive.tar.gz
   shasum -a 256 archive.tar.gz
   ```
3. Update `sha256` field in formula
4. Test installation: `brew reinstall --build-from-source formula/auxin.rb`
5. Commit and push to tap repository

---

## What the Formula Does

The Homebrew formula automates:

1. **Building from source** using Cargo
2. **Installing the binary** to `/usr/local/bin` (or `/opt/homebrew/bin` on Apple Silicon)
3. **Shell completions** for bash, zsh, and fish
4. **Config template** at `$(brew --prefix)/share/auxin/config.toml.example`
5. **Documentation** in `$(brew --prefix)/share/doc/auxin`

---

## Post-Installation

After installation via Homebrew, users should:

1. **Install Oxen CLI** (required dependency):
   ```bash
   pip3 install oxen-ai
   # OR
   cargo install oxen
   ```

2. **Create config file** (optional):
   ```bash
   mkdir -p ~/.auxin
   cp $(brew --prefix)/share/auxin/config.toml.example ~/.auxin/config.toml
   ```

3. **Initialize a project**:
   ```bash
   cd /path/to/your-project.logicx
   auxin init
   ```

---

## Troubleshooting

### Formula not found
```bash
brew tap jbacus/auxin
brew update
```

### Build failures
```bash
# Check Rust installation
rustc --version

# View build logs
brew install --build-from-source --verbose formula/auxin.rb
```

### Completions not working
```bash
# Zsh users may need to refresh completions
rm -f ~/.zcompdump*
compinit
```

---

## Formula Structure

```ruby
class Auxin < Formula
  desc "..."              # Short description
  homepage "..."          # Project homepage
  url "..."               # Source tarball URL
  sha256 "..."            # SHA256 of tarball
  license "MIT"           # License

  depends_on "rust" => :build    # Build dependencies

  def install
    # Build and install binary
    # Install completions
    # Install config template
    # Install documentation
  end

  def caveats
    # Post-installation instructions
  end

  test do
    # Installation verification tests
  end
end
```

---

## References

- [Homebrew Formula Cookbook](https://docs.brew.sh/Formula-Cookbook)
- [Homebrew Acceptable Formulae](https://docs.brew.sh/Acceptable-Formulae)
- [Creating Homebrew Taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap)

---

*Last updated: 2025-11-17*
