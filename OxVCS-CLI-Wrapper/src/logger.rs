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
