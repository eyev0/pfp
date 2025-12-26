//! # PFP - Project Fuzzy Picker
//!
//! A command-line tool for managing tmux sessions and windows with fuzzy project selection.
//!
//! PFP scans your filesystem for project directories (identified by markers like `.git`,
//! `Cargo.toml`, etc.) and provides an interactive fuzzy finder interface for quick navigation.
//!
//! ## Features
//!
//! - Smart project detection using configurable markers
//! - Fast filesystem traversal with depth limits and ignore patterns
//! - Interactive fuzzy selection powered by fzf
//! - Tmux session and window management
//! - Predefined session configurations
//!
//! ## Usage
//!
//! ```bash
//! # Create a new tmux session from a selected project
//! pfp new-session
//!
//! # Create a new window in the current session
//! pfp new-window
//!
//! # Switch between active sessions
//! pfp sessions
//! ```

use log::info;
use pfp::cli;
use std::time::Instant;

fn main() {
    match cli::cli() {
        Ok(_) => std::process::exit(exitcode::OK),
        Err(error) => {
            eprintln!("{}", error);
            std::process::exit(exitcode::DATAERR);
        }
    }
}

/// Measures and logs the execution time of a closure.
///
/// This utility function wraps any closure and logs its execution time
/// at the `info` level using the `log` crate.
///
/// # Arguments
///
/// * `name` - A descriptive name for the operation being measured
/// * `f` - The closure to execute and measure
///
/// # Examples
///
/// ```ignore
/// measure("directory_scan", || {
///     scan_directories(&config);
/// });
/// // Logs: "Time elapsed for directory_scan is: 150ms"
/// ```
#[allow(dead_code)]
pub fn measure<F>(name: &str, mut f: F)
where
    F: FnMut(),
{
    let start = Instant::now();
    f();
    info!("Time elapsed for {} is: {:?}", name, start.elapsed());
}
