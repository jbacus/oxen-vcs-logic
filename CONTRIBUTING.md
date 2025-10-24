# Contributing to Oxen-VCS

## Development Status

This project is in early development (Phase 1). Contributions are welcome once the MVP is complete.

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

### Swift Tests
```bash
xcodebuild test -project OxVCS-App/OxVCS.xcodeproj -scheme OxVCS
```

### Rust Tests
```bash
cd OxVCS-CLI-Wrapper
cargo test
cargo bench
```

## Pull Request Process

1. Update documentation for any API changes
2. Add tests covering new functionality
3. Ensure all tests pass
4. Update CHANGELOG.md (once established)
5. Request review from maintainers

## Architecture Decisions

Major architectural changes should be discussed in an issue before implementation. Reference the [Architecture Blueprint](docs/ARCHITECTURE.md) for design rationale.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
