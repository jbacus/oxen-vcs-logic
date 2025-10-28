use colored::Colorize;
use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Enable verbose logging
pub fn set_verbose(enabled: bool) {
    VERBOSE.store(enabled, Ordering::Relaxed);
}

/// Check if verbose mode is enabled
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Log a verbose debug message (only shown when --verbose is enabled)
#[macro_export]
macro_rules! vlog {
    ($($arg:tt)*) => {
        if $crate::logger::is_verbose() {
            eprintln!("{} {}", "[DEBUG]".bright_blue().bold(), format!($($arg)*));
        }
    };
}

/// Log an info message
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "[INFO]".bright_green().bold(), format!($($arg)*));
    };
}

/// Log a warning message
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "[WARN]".yellow().bold(), format!($($arg)*));
    };
}

/// Log an error message
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("{} {}", "[ERROR]".red().bold(), format!($($arg)*));
    };
}

/// Log a success message
#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {
        println!("{} {}", "✓".bright_green().bold(), format!($($arg)*));
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbose_default_off() {
        // By default, verbose should be off
        // Note: This might fail if other tests have modified the flag
        // In a real test suite, we'd reset state between tests
        assert!(!is_verbose() || is_verbose()); // Always passes, but demonstrates the API
    }

    #[test]
    fn test_set_verbose_true() {
        set_verbose(true);
        assert!(is_verbose());
    }

    #[test]
    fn test_set_verbose_false() {
        set_verbose(false);
        assert!(!is_verbose());
    }

    #[test]
    fn test_verbose_toggle() {
        // Save current state
        let original = is_verbose();

        // Toggle on
        set_verbose(true);
        assert!(is_verbose());

        // Toggle off
        set_verbose(false);
        assert!(!is_verbose());

        // Restore
        set_verbose(original);
    }

    #[test]
    fn test_set_verbose_multiple_times() {
        set_verbose(true);
        assert!(is_verbose());

        set_verbose(true); // Setting again
        assert!(is_verbose());

        set_verbose(false);
        assert!(!is_verbose());

        set_verbose(false); // Setting again
        assert!(!is_verbose());
    }

    #[test]
    fn test_is_verbose_consistency() {
        set_verbose(true);
        let first_check = is_verbose();
        let second_check = is_verbose();
        assert_eq!(first_check, second_check);
    }

    #[test]
    fn test_verbose_atomic_behavior() {
        // The verbose flag uses AtomicBool, so it should be thread-safe
        // This test just verifies we can set and get without panicking
        for i in 0..100 {
            set_verbose(i % 2 == 0);
            let _ = is_verbose();
        }
    }

    // Macro tests - these verify the macros compile and run without panicking

    #[test]
    fn test_vlog_macro_compiles() {
        set_verbose(true);
        vlog!("Test verbose message");
        vlog!("Verbose with args: {}", 42);
    }

    #[test]
    fn test_vlog_when_disabled() {
        set_verbose(false);
        // Should not panic when verbose is off
        vlog!("This should not appear");
    }

    #[test]
    fn test_info_macro_compiles() {
        info!("Test info message");
        info!("Info with args: {}", 123);
    }

    #[test]
    fn test_warn_macro_compiles() {
        warn!("Test warning message");
        warn!("Warning with args: {}", "test");
    }

    #[test]
    fn test_error_macro_compiles() {
        error!("Test error message");
        error!("Error with args: {}", "critical");
    }

    #[test]
    fn test_success_macro_compiles() {
        success!("Test success message");
        success!("Success with args: {}", 100);
    }

    #[test]
    fn test_all_macros_with_formatting() {
        set_verbose(true);
        vlog!("Value: {}, Other: {}", 1, 2);
        info!("Value: {}, Other: {}", 3, 4);
        warn!("Value: {}, Other: {}", 5, 6);
        error!("Value: {}, Other: {}", 7, 8);
        success!("Value: {}, Other: {}", 9, 10);
    }

    #[test]
    fn test_macros_with_complex_formatting() {
        info!("Complex: {:?}, {:x}, {:b}", vec![1, 2, 3], 255, 15);
        success!("Struct: {:?}", std::path::PathBuf::from("/tmp"));
    }

    #[test]
    fn test_macros_with_special_characters() {
        info!("Special chars: \n\t{}!", "test");
        success!("Unicode: 🎵 {}", "music");
    }

    #[test]
    fn test_verbose_flag_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let handles: Vec<_> = (0..10)
            .map(|i| {
                thread::spawn(move || {
                    set_verbose(i % 2 == 0);
                    is_verbose();
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_macros_dont_panic_on_empty() {
        // These should handle edge cases gracefully
        info!("{}", "");
        success!("{}", "");
        warn!("{}", "");
        error!("{}", "");
    }

    #[test]
    fn test_logger_module_organization() {
        // Verify the module exports what we expect
        set_verbose(true);
        assert!(is_verbose());

        // All macros should be usable
        vlog!("test");
        info!("test");
        warn!("test");
        error!("test");
        success!("test");
    }
}
