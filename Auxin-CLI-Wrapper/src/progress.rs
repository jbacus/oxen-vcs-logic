//! Progress indicators and visual feedback utilities
//!
//! This module provides consistent progress bars, spinners, and status messages
//! across all CLI commands for better user experience.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Creates a spinner for indeterminate operations
pub fn spinner(message: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .expect("Failed to set template"),
    );
    pb.set_message(message.to_string());
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Creates a progress bar for operations with known size
pub fn progress_bar(total: u64, message: &str) -> ProgressBar {
    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})")
            .expect("Failed to set template")
            .progress_chars("#>-"),
    );
    pb.set_message(message.to_string());
    pb
}

/// Finish spinner with success message
pub fn finish_success(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("✓ {}", message));
}

/// Finish spinner with error message
pub fn finish_error(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("✗ {}", message));
}

/// Finish spinner with info message
pub fn finish_info(pb: &ProgressBar, message: &str) {
    pb.finish_with_message(format!("ℹ {}", message));
}

/// Print a success message (without spinner)
pub fn success(message: &str) {
    println!("✓ {}", message);
}

/// Print an error message (without spinner)
pub fn error(message: &str) {
    eprintln!("✗ {}", message);
}

/// Print an info message (without spinner)
pub fn info(message: &str) {
    println!("ℹ {}", message);
}

/// Print a warning message
pub fn warning(message: &str) {
    println!("⚠ {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_creation() {
        let pb = spinner("Testing...");
        assert!(pb.is_finished() == false);
        finish_success(&pb, "Test complete");
        assert!(pb.is_finished());
    }

    #[test]
    fn test_progress_bar_creation() {
        let pb = progress_bar(100, "Processing");
        assert_eq!(pb.length().unwrap(), 100);
        assert_eq!(pb.position(), 0);
    }
}
