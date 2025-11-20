# Contributing to Auxin

## Development Status

All three development phases are **COMPLETE** and the project is **production-ready**:
- ✅ Phase 1: Core Data Management (MVP)
- ✅ Phase 2: Service Architecture & Resilience
- ✅ Phase 3: UI Application & Collaboration

Contributions are welcome! We're actively accepting pull requests for bug fixes, new features, performance improvements, and documentation updates.

## Getting Started

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/your-feature`
3. Make your changes
4. Write or update tests
5. Submit a pull request

## Code Style

### Swift
- Follow Swift API Design Guidelines
- Use SwiftLint configuration (TBD)
- Maximum line length: 120 characters
- Use meaningful variable names

### Rust
- Use `cargo fmt` before committing
- Run `cargo clippy` and address warnings
- Write documentation comments for public APIs
- Add tests for new functionality

## Commit Messages

Use conventional commit format:

```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

Examples:
- `feat(daemon): add FSEvents debounce logic`
- `fix(ui): correct milestone commit cleanup sequence`
- `docs(readme): update installation instructions`

## Testing

We have a comprehensive testing strategy to ensure quality and reliability. Please refer to:
- **[Testing Strategy](docs/developer/testing.md)** - Overall testing philosophy and approach
- **[Architecture Guide](docs/developer/architecture.md)** - Technical architecture and development roadmap

### Running Tests

#### All Tests (CI Pipeline)
```bash
# Runs all test suites via GitHub Actions locally
act -j rust-tests
act -j swift-daemon-tests
act -j swift-app-tests
```

#### Rust Tests (CLI Wrapper)
```bash
cd Auxin-CLI-Wrapper

# Run all tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

# Run benchmarks
cargo bench

# Run with verbose output
cargo test -- --nocapture
```

#### Swift Tests (LaunchAgent Daemon)
```bash
cd Auxin-LaunchAgent

# Run all tests
swift test

# Run with coverage
swift test --enable-code-coverage

# Run specific test
swift test --filter LockManagerTests
```

#### Swift Tests (App UI)
```bash
cd Auxin-App

# Run all tests
swift test

# Run with coverage
swift test --enable-code-coverage
```

### Writing Tests

#### Test Requirements
- **All new features** must include tests
- **Bug fixes** should include regression tests
- Aim for **70-80% code coverage** overall
- Critical paths require **90%+ coverage**

#### Test Utilities
We provide test utilities to make testing easier:

**Rust**:
- `TestFixture` - Creates temporary Logic Pro projects
- See `Auxin-CLI-Wrapper/tests/common/mod.rs`
- Example: `Auxin-CLI-Wrapper/tests/example_test.rs`

**Swift (LaunchAgent)**:
- `TestFixtures` - Creates test projects and environments
- See `Auxin-LaunchAgent/Tests/TestUtils/TestFixtures.swift`

**Swift (App)**:
- `MockOxenDaemonXPCClient` - Mock XPC client for UI testing
- See `Auxin-App/Tests/TestUtils/MockXPCClient.swift`

#### Test Naming Conventions

**Swift**:
```swift
// Pattern: test_{function}_{scenario}_{expectedResult}
func testLockAcquisition_WhenAvailable_Succeeds()
func testLockAcquisition_WhenAlreadyLocked_Fails()
```

**Rust**:
```rust
// Pattern: test_{function}_{scenario}
#[test]
fn test_is_logic_project_valid_structure()
#[test]
fn test_is_logic_project_missing_alternatives()
```

### Code Coverage

We use coverage tracking to ensure test quality:

**View Coverage Reports**:
```bash
# Rust
cd Auxin-CLI-Wrapper
cargo tarpaulin --out Html
open tarpaulin-report.html

# Swift
cd Auxin-LaunchAgent
swift test --enable-code-coverage
xcrun llvm-cov show .build/debug/...xctest -instr-profile=... -format=html -output-dir=coverage
open coverage/index.html
```

**Coverage Requirements**:
- Overall: ≥70%
- Critical paths (locks, commits, power management): ≥90%
- New code should not decrease coverage (ratcheting)

### Continuous Integration

All tests run automatically on:
- Every push to `main`, `develop`, or `claude/*` branches
- Every pull request
- See `.github/workflows/test.yml` for CI configuration

**Quality Gates** (must pass for PR merge):
- All tests pass
- No linting errors (`cargo clippy`, `cargo fmt`)
- Code coverage ≥ baseline (no decrease)

### Performance Testing

We use benchmarking to track performance:

```bash
cd Auxin-CLI-Wrapper
cargo bench

# Compare with baseline
cargo bench -- --baseline main
```

**Performance Requirements**:
- Commit 1GB project: <10 seconds
- Lock acquisition: <100ms
- Daemon CPU usage: <5% average

## Pull Request Process

1. Update documentation for any API changes
2. Add tests covering new functionality
3. Ensure all tests pass
4. Update CHANGELOG.md (once established)
5. Request review from maintainers

## Architecture Decisions

Major architectural changes should be discussed in an issue before implementation. Reference the [Architecture Guide](docs/developer/architecture.md) for system architecture and design rationale.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
