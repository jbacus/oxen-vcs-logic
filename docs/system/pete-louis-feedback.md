# Pete & Louis Visual Feedback

## Overview

Pete (left) and Louis (right) provide visual feedback for test results and audio bounce comparisons throughout the Auxin system. This adds a fun, human touch to automated operations.

## Locations

### 1. Test Runner (`run_all_tests.sh`)

The test runner displays Pete & Louis at the end of the test suite:

- **All tests pass**: Pete & Louis give thumbs up 👍 with happy faces
- **Any tests fail**: Pete & Louis give thumbs down 👎 with concerned faces

Example output:
```
    ╔════════════════════════════════════════════════════════╗
    ║  Pete (L) & Louis (R) say: "Tests look good!" 🎉      ║
    ╚════════════════════════════════════════════════════════╝
         👍                              👍
        ___                             ___
       /   \                           /   \
      | 😊  |                         | 😄  |
      |_____|                         |_____|
        | |                             | |
```

### 2. Bounce Comparison (`src/bounce.rs`)

Audio bounce comparisons include Pete & Louis feedback based on the null test results:

- **≥80% cancellation**: Thumbs up 👍 - "Sounds good!"
  - Indicates identical or very similar audio
- **<80% cancellation**: Thumbs down 👎 - "Something changed..."
  - Indicates significant audio differences

This appears in the `BounceComparison::format_report()` output when null test results are available.

## Implementation Details

### Rust (Bounce Comparison)

The `BounceComparison` struct in `Auxin-CLI-Wrapper/src/bounce.rs` includes:

```rust
fn get_feedback_art(cancellation_percent: f64) -> &'static str
```

This function returns ASCII art based on the audio similarity percentage.

### Bash (Test Runner)

The test runner script at `/run_all_tests.sh` displays feedback at the end based on the `$FAILED_SUITES` count.

## Customization

To modify the feedback messages or thresholds:

1. **Bounce comparison threshold**: Edit `get_feedback_art()` in `src/bounce.rs` (currently 80%)
2. **Test feedback messages**: Edit the echo statements in `run_all_tests.sh`

## Who are Pete & Louis?

Pete and Louis are members of the Auxin development team, immortalized in code to provide encouragement and feedback to users.

---

*Last updated: 2025-11-22*
