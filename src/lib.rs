//! PFP Library - Re-exports for tests and external use.

use std::env::VarError;
use std::string::FromUtf8Error;

pub mod cli;
pub mod config;
pub mod context;
pub mod fs;
pub mod fzf;
pub mod selectors;
pub mod tmux;

pub use config::ConfigError;

/// Application-wide error type encompassing all possible failure modes.
///
/// This enum provides structured error handling for the entire application,
/// with automatic conversions from underlying error types.
#[derive(thiserror::Error, Debug)]
pub enum Error {
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
    #[error("Glob pattern error: {0}")]
    GlobPattern(#[from] glob::PatternError),
    #[error("Env var error: {0}: {1}")]
    EnvVar(VarError, String),
    #[error("Parse utf8 error: {0}")]
    ParseUTF8(#[from] FromUtf8Error),
    #[error("Empty pick!")]
    EmptyPick(),
}
