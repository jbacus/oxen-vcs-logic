# OxVCS CLI Wrapper

Optimized Rust wrapper around liboxen for high-performance VCS operations.

## Responsibilities

- FFI interface to liboxen (Oxen's Rust core)
- Minimal-latency execution of Oxen commands
- Low memory footprint for daemon usage
- IPC server for LaunchAgent communication

## Structure

```
OxVCS-CLI-Wrapper/
├── Cargo.toml
├── src/
│   ├── main.rs          # CLI entry point
│   ├── oxen_ops.rs      # Oxen operation wrappers
│   ├── ipc.rs           # IPC server (XPC or Darwin)
│   └── lib.rs           # FFI exports
└── tests/
```

## Dependencies

```toml
[dependencies]
liboxen = "0.x.x"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
```

## Build

```bash
cargo build --release
# Output: target/release/oxenvcs-cli
```

## Benchmarks

```bash
cargo bench
```

Expected performance targets:
- `oxen.add()` single file: <10ms
- `oxen.commit()`: <100ms
- Memory footprint: <50MB resident

## Embedding

Built binary is embedded in main app bundle:
```
OxVCS.app/Contents/Helpers/oxenvcs-cli
```

## Implementation Status

See [IMPLEMENTATION_PLAN.md](../docs/IMPLEMENTATION_PLAN.md) Phase 2.3
