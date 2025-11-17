# Homebrew Formula for OxVCS CLI

This directory contains the Homebrew formula for installing `oxenvcs-cli` on macOS.

## Quick Installation (macOS)

### Option 1: Via Homebrew Tap (Recommended)

Once the formula is published to a tap:

```bash
brew tap jbacus/oxenvcs
brew install oxenvcs-cli
```

### Option 2: Direct Installation from Formula

Install directly from this repository:

```bash
brew install --build-from-source /path/to/oxen-vcs-logic/OxVCS-CLI-Wrapper/formula/oxenvcs-cli.rb
```

### Option 3: Manual Installation Script

Use the installation script (works on both macOS and Linux):

```bash
cd OxVCS-CLI-Wrapper
./install.sh
```

---

## Publishing to Homebrew

### Creating a Homebrew Tap

1. **Create a tap repository**:
   ```bash
   # Create a new GitHub repository named: homebrew-oxenvcs
   # Repository URL will be: https://github.com/jbacus/homebrew-oxenvcs
   ```

2. **Add the formula**:
   ```bash
   git clone https://github.com/jbacus/homebrew-oxenvcs.git
   cd homebrew-oxenvcs
   mkdir Formula
   cp /path/to/oxen-vcs-logic/OxVCS-CLI-Wrapper/formula/oxenvcs-cli.rb Formula/
   git add Formula/oxenvcs-cli.rb
   git commit -m "Add oxenvcs-cli formula"
   git push
   ```

3. **Update SHA256 hash**:
   ```bash
   # After creating a GitHub release with a tarball
   shasum -a 256 /path/to/v0.1.0.tar.gz

   # Update the sha256 in oxenvcs-cli.rb
   ```

4. **Test the formula**:
   ```bash
   brew install --build-from-source Formula/oxenvcs-cli.rb
   brew test oxenvcs-cli
   brew audit --strict oxenvcs-cli
   ```

### Creating a GitHub Release

Before publishing, create a tagged release:

```bash
cd oxen-vcs-logic
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

Then create a release on GitHub with the tarball, and update the formula's `url` and `sha256`.

---

## Formula Development

### Testing the Formula Locally

```bash
# Install from local formula
brew install --build-from-source formula/oxenvcs-cli.rb

# Run tests
brew test oxenvcs-cli

# Audit formula
brew audit --strict oxenvcs-cli

# Uninstall
brew uninstall oxenvcs-cli
```

### Updating the Formula

When releasing a new version:

1. Update version in `url` field
2. Download the new tarball and compute SHA256:
   ```bash
   curl -L https://github.com/jbacus/oxen-vcs-logic/archive/refs/tags/vX.Y.Z.tar.gz -o archive.tar.gz
   shasum -a 256 archive.tar.gz
   ```
3. Update `sha256` field in formula
4. Test installation: `brew reinstall --build-from-source formula/oxenvcs-cli.rb`
5. Commit and push to tap repository

---

## What the Formula Does

The Homebrew formula automates:

1. **Building from source** using Cargo
2. **Installing the binary** to `/usr/local/bin` (or `/opt/homebrew/bin` on Apple Silicon)
3. **Shell completions** for bash, zsh, and fish
4. **Config template** at `$(brew --prefix)/share/oxenvcs-cli/config.toml.example`
5. **Documentation** in `$(brew --prefix)/share/doc/oxenvcs-cli`

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
   mkdir -p ~/.oxenvcs
   cp $(brew --prefix)/share/oxenvcs-cli/config.toml.example ~/.oxenvcs/config.toml
   ```

3. **Initialize a project**:
   ```bash
   cd /path/to/your-project.logicx
   oxenvcs-cli init
   ```

---

## Troubleshooting

### Formula not found
```bash
brew tap jbacus/oxenvcs
brew update
```

### Build failures
```bash
# Check Rust installation
rustc --version

# View build logs
brew install --build-from-source --verbose formula/oxenvcs-cli.rb
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
class OxenvcsLi < Formula
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
