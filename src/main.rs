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

mod cli;
mod config;
mod fs;
mod fzf;
mod selectors;
mod tmux;

use crate::config::ConfigError;
use log::info;

use std::env::VarError;
use std::string::FromUtf8Error;
use std::time::Instant;

/// Application-wide error type encompassing all possible failure modes.
///
/// This enum provides structured error handling for the entire application,
/// with automatic conversions from underlying error types.
#[derive(thiserror::Error, Debug)]
enum Error {
    #[error("Config error: {0}")]
    Config(#[from] ConfigError),
    #[error("Cmd arguments error: {0}")]
    CmdArg(String),
    #[error("Descend error: {0}")]
    Descend(#[from] anyhow::Error),
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Unwrap IO stream error: {0}")]
    UnwrapIOStream(&'static str),
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    #[error("Env var error: {0}: {1}")]
    EnvVar(VarError, String),
    #[error("Parse utf8 error: {0}")]
    ParseUTF8(#[from] FromUtf8Error),
    #[error("Empty pick!")]
    EmptyPick(),
}

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
