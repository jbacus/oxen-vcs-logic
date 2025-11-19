# Contributing Guide

**See the main contributing guide**: [CONTRIBUTING.md](../../CONTRIBUTING.md)

The root CONTRIBUTING.md contains:
- Getting started instructions
- Code style guidelines (Swift, Rust)
- Commit message format
- Testing requirements
- Pull request process

---

## Quick Reference

### Code Style

**Swift**: Follow Swift API Design Guidelines, max 120 char lines

**Rust**: Run `cargo fmt` and `cargo clippy` before committing

### Commit Messages

```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

### Testing

```bash
# All tests
./run_all_tests.sh

# Rust
cd Auxin-CLI-Wrapper && cargo test

# Swift
cd Auxin-LaunchAgent && swift test
cd Auxin-App && swift test
```

### Pull Request Checklist

- [ ] Tests pass
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Documentation updated
- [ ] Changelog updated (if applicable)

---

For full details, see [CONTRIBUTING.md](../../CONTRIBUTING.md).
