# Code Formatting Guide

This document describes the code formatting standards and tools used in the Auxin project.

## Overview

The Auxin project enforces consistent code formatting across all codebases:

- **Rust**: Formatted using `rustfmt` with configuration in `rustfmt.toml`
- **Swift**: Formatted using `swiftformat` with configuration in `.swiftformat`

All formatting checks are automatically enforced in CI/CD pipelines.

## Rust Formatting

### Configuration

Rust code formatting is configured in [`rustfmt.toml`](rustfmt.toml) at the repository root.

### Installing rustfmt

rustfmt is included with the standard Rust toolchain:

```bash
rustup component add rustfmt
```

### Running rustfmt Locally

**Check formatting (without making changes):**

```bash
# For CLI wrapper
cd Auxin-CLI-Wrapper
cargo fmt -- --check

# For server
cd auxin-server
cargo fmt -- --check

# For all Rust packages
cd auxin-oxen
cargo fmt -- --check
```

**Auto-format your code:**

```bash
# For CLI wrapper
cd Auxin-CLI-Wrapper
cargo fmt

# For server
cd auxin-server
cargo fmt

# For all Rust packages
cd auxin-oxen
cargo fmt
```

### Editor Integration

Most Rust-aware editors support rustfmt integration:

- **VS Code**: Install the "rust-analyzer" extension, enable "Format on Save"
- **IntelliJ/CLion**: Built-in support, enable in Preferences → Rust → Rustfmt
- **Vim/Neovim**: Use `ale` or `coc-rust-analyzer`

## Swift Formatting

### Configuration

Swift code formatting is configured in [`.swiftformat`](.swiftformat) at the repository root.

### Installing SwiftFormat

**macOS (Homebrew):**

```bash
brew install swiftformat
```

**Building from source:**

```bash
git clone https://github.com/nicklockwood/SwiftFormat
cd SwiftFormat
swift build -c release
```

### Running SwiftFormat Locally

**Check formatting (without making changes):**

```bash
# For LaunchAgent
cd Auxin-LaunchAgent
swiftformat --lint --config ../.swiftformat .

# For App
cd Auxin-App
swiftformat --lint --config ../.swiftformat .
```

**Auto-format your code:**

```bash
# For LaunchAgent
cd Auxin-LaunchAgent
swiftformat --config ../.swiftformat .

# For App
cd Auxin-App
swiftformat --config ../.swiftformat .
```

### Editor Integration

- **Xcode**: Install SwiftFormat for Xcode from [releases](https://github.com/nicklockwood/SwiftFormat/releases)
- **VS Code**: Install "SwiftFormat" extension
- **Vim/Neovim**: Use `ale` with `swiftformat`

## CI/CD Enforcement

All formatting checks are automatically run in the CI/CD pipeline:

### Rust Projects

The following jobs check Rust formatting:
- `rust-unit-tests` - Checks CLI wrapper formatting
- `rust-server-tests` - Checks server formatting

Both jobs run `cargo fmt -- --check` which fails if code is not properly formatted.

### Swift Projects

The following jobs check Swift formatting:
- `swift-launchagent` - Checks LaunchAgent formatting
- `swift-app` - Checks App formatting

Both jobs run `swiftformat --lint` which fails if code is not properly formatted.

## Pre-commit Hooks (Optional)

You can set up pre-commit hooks to automatically format code before committing:

### Git Hook for Rust

Create `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Format Rust code before commit

cd Auxin-CLI-Wrapper && cargo fmt
cd ../auxin-server && cargo fmt
cd ../auxin-oxen && cargo fmt

git add -u
```

### Git Hook for Swift

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Format Swift code before commit

cd Auxin-LaunchAgent && swiftformat --config ../.swiftformat .
cd ../Auxin-App && swiftformat --config ../.swiftformat .

git add -u
```

Make the hook executable:

```bash
chmod +x .git/hooks/pre-commit
```

## Troubleshooting

### "rustfmt not found"

Ensure rustfmt is installed:
```bash
rustup component add rustfmt
```

### "swiftformat not found"

Install SwiftFormat via Homebrew:
```bash
brew install swiftformat
```

### Formatting conflicts

If you encounter merge conflicts in formatted code:

1. Resolve the conflicts manually
2. Run the formatter on the resolved file
3. Commit the formatted result

### CI formatting check fails

If CI fails due to formatting:

1. Pull the latest changes
2. Run the formatter locally (see commands above)
3. Commit the formatting changes
4. Push to your branch

## Configuration Updates

### Modifying rustfmt.toml

When updating `rustfmt.toml`:

1. Test the changes locally on representative code
2. Document significant changes in commit message
3. Run CI to ensure no unexpected formatting changes
4. Consider reformatting the entire codebase if needed

### Modifying .swiftformat

When updating `.swiftformat`:

1. Test locally on both Swift projects
2. Review formatting changes carefully
3. Document the reason for configuration changes
4. Run CI to validate

## Best Practices

1. **Format before committing**: Always run formatters before creating commits
2. **Use editor integration**: Enable format-on-save in your editor
3. **Don't mix formatting with logic changes**: Keep formatting changes in separate commits when reformatting existing code
4. **Review formatting changes**: Sometimes auto-formatters make code less readable - use your judgment
5. **Keep configurations in sync**: Both projects should use consistent formatting rules where applicable

## Additional Resources

- [rustfmt documentation](https://rust-lang.github.io/rustfmt/)
- [SwiftFormat documentation](https://github.com/nicklockwood/SwiftFormat)
- [Rust style guide](https://doc.rust-lang.org/1.0.0/style/)
- [Swift API Design Guidelines](https://swift.org/documentation/api-design-guidelines/)

---

*Last updated: 2025-11-23*
