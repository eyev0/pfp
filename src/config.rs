//! Configuration parsing and data structures.
//!
//! This module provides types for parsing and representing the application
//! configuration, which is stored as JSON (with optional comments via JSONC).
//!
//! # Configuration File Location
//!
//! By default, the configuration is read from `${XDG_CONFIG_HOME}/pfp/config.json`.
//!
//! # Example Configuration
//!
//! ```json
//! {
//!   "include": [
//!     {
//!       "paths": ["$HOME/projects"],
//!       "depth": 3
//!     }
//!   ],
//!   "markers": {
//!     "exact": [".git", "Cargo.toml"]
//!   }
//! }
//! ```

use serde::Deserialize;

/// Errors that can occur during configuration parsing.
#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
    /// JSON parsing error (syntax error, type mismatch, etc.)
    #[error("Parse config: {0}")]
    Parse(#[from] serde_jsonc::Error),
    /// File I/O error (file not found, permission denied, etc.)
    #[error("Read config: {0}")]
    Read(#[from] std::io::Error),
}

/// Main configuration structure.
///
/// Contains all application settings including:
/// - Predefined tmux sessions
/// - Global project markers
/// - Global ignore patterns
/// - Directory scanning entries
#[derive(Deserialize, Debug)]
pub(crate) struct Config<'a> {
    /// Predefined tmux sessions that can be started with `pfp start`
    #[serde(default)]
    pub sessions: Vec<Session<'a>>,
    /// Global markers that identify project root directories
    #[serde(default, borrow = "'a")]
    pub markers: Markers<'a>,
    /// Global patterns for directories/files to ignore during scanning
    #[serde(default)]
    pub ignore: Ignore<'a>,
    /// List of directory entries to scan for projects
    pub include: Vec<IncludeEntry<'a>>,
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            sessions: vec![],
            markers: Markers::default(),
            ignore: Ignore::default(),
            include: vec![IncludeEntry {
                paths: ["$HOME"].to_vec(),
                ..Default::default()
            }],
        }
    }
}

/// A predefined tmux session configuration.
///
/// Sessions can be started using the `pfp start` command, which allows
/// selecting from configured sessions and creating them with their
/// predefined windows.
///
/// # Example
///
/// ```json
/// {
///   "name": "work",
///   "windows": [
///     "$HOME/projects/api",
///     "$HOME/projects/frontend"
///   ]
/// }
/// ```
#[derive(Deserialize, Debug)]
pub(crate) struct Session<'a> {
    /// The session name (displayed in tmux status bar)
    pub name: &'a str,
    /// List of paths for windows in this session (supports env vars)
    pub windows: Vec<&'a str>,
}

impl<'a> ToString for Session<'a> {
    fn to_string(&self) -> String {
        format!(
            "{}:\n{}\n",
            self.name,
            self.windows
                .iter()
                .map(|p| crate::fs::expand(p).unwrap_or(p.to_string()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

fn default_yield_on_marker() -> bool {
    true
}

fn default_include_intermediate_paths() -> bool {
    true
}

/// Configuration for a set of directories to scan for projects.
///
/// Each include entry defines root paths to scan along with specific
/// settings for that scan operation.
///
/// # Fields
///
/// - `paths`: Root directories to start scanning from
/// - `mode`: Whether to look for directories or files
/// - `depth`: Maximum recursion depth (default: 255)
/// - `markers`: Entry-specific project markers
/// - `ignore`: Entry-specific ignore patterns
/// - `include_intermediate_paths`: Include parent directories in results
/// - `yield_on_marker`: Stop descending when marker is found
#[derive(Deserialize, Debug)]
pub(crate) struct IncludeEntry<'a> {
    /// Root paths to scan (supports environment variable expansion)
    #[serde(borrow = "'a")]
    pub paths: Vec<&'a str>,
    /// Scanning mode: `dir` for project directories, `file` for individual files
    #[serde(default)]
    pub mode: Mode,
    /// Entry-specific markers (merged with global if `chain_root_markers` is true)
    #[serde(default)]
    pub markers: Markers<'a>,
    /// Entry-specific ignore patterns (merged with global if `chain_root_ignore` is true)
    #[serde(default)]
    pub ignore: Ignore<'a>,
    /// Include all directories in the path to a match, not just the match itself
    #[serde(default = "default_include_intermediate_paths")]
    pub include_intermediate_paths: bool,
    /// Stop descending into subdirectories when a marker is found
    #[serde(default = "default_yield_on_marker")]
    pub yield_on_marker: bool,
    /// Maximum directory depth to descend (0 = root only)
    #[serde(default = "u8::max_value")]
    pub depth: u8,
}

impl<'a> Default for IncludeEntry<'a> {
    fn default() -> Self {
        Self {
            paths: vec![],
            mode: Mode::Dir,
            markers: Markers::default(),
            ignore: Ignore::default(),
            include_intermediate_paths: default_include_intermediate_paths(),
            yield_on_marker: default_yield_on_marker(),
            depth: u8::max_value(),
        }
    }
}

/// Scanning mode for include entries.
///
/// Determines whether the scanner looks for project directories
/// or individual files.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mode {
    /// Scan for directories containing project markers (default)
    #[default]
    Dir,
    /// Scan for individual files (useful for scripts, configs, etc.)
    File,
}

const MARKERS_EXACT_DEFAULT: [&str; 3] = [
    ".git",
    "Cargo.toml",
    "go.mod",
    // "package.json",
    // "pom.xml",
    // "build.gradle",
];

const MARKERS_PATTERN_DEFAULT: [&str; 0] = [];

fn default_traverse_hidden() -> bool {
    true
}

fn default_chain_root_markers() -> bool {
    true
}

/// Configuration for project markers.
///
/// Markers are files or directories whose presence indicates a project root.
/// When scanning, directories containing these markers are considered projects.
///
/// # Defaults
///
/// - `exact`: `[".git", "Cargo.toml", "go.mod"]`
/// - `pattern`: `[]`
/// - `traverse_hidden`: `true`
/// - `chain_root_markers`: `true`
#[derive(Deserialize, Debug)]
pub(crate) struct Markers<'a> {
    /// Exact file/directory names that indicate a project root
    #[serde(default, borrow = "'a")]
    pub exact: Vec<&'a str>,
    /// Regex patterns for matching marker names
    #[serde(default)]
    pub pattern: Vec<&'a str>,
    /// Whether to descend into hidden directories (starting with `.`)
    #[serde(default = "default_traverse_hidden")]
    pub traverse_hidden: bool,
    /// Whether to merge with global markers configuration
    #[serde(default = "default_chain_root_markers")]
    pub chain_root_markers: bool,
}

impl<'a> Default for Markers<'a> {
    fn default() -> Self {
        Markers {
            exact: Vec::from(MARKERS_EXACT_DEFAULT),
            pattern: Vec::from(MARKERS_PATTERN_DEFAULT),
            chain_root_markers: default_chain_root_markers(),
            traverse_hidden: default_traverse_hidden(),
        }
    }
}

const IGNORE_EXACT_DEFAULT: [&str; 12] = [
    ".DS_Store",
    "node_modules",
    "venv",
    "bin",
    "target",
    "debug",
    "src",
    "test",
    "tests",
    "lib",
    "docs",
    "pkg",
];
const IGNORE_PATTERN_DEFAULT: [&str; 0] = [];

fn default_chain_root_ignore() -> bool {
    true
}

/// Configuration for ignore patterns.
///
/// Directories and files matching these patterns are skipped during scanning.
///
/// # Defaults
///
/// - `exact`: Common build/dependency directories like `node_modules`, `target`, `venv`, etc.
/// - `pattern`: `[]`
/// - `chain_root_ignore`: `true`
#[derive(Deserialize, Debug)]
pub(crate) struct Ignore<'a> {
    /// Exact names to ignore (e.g., `"node_modules"`, `"target"`)
    #[serde(default, borrow = "'a")]
    pub exact: Vec<&'a str>,
    /// Regex patterns for matching names to ignore
    #[serde(default)]
    pub pattern: Vec<&'a str>,
    /// Whether to merge with global ignore configuration
    #[serde(default = "default_chain_root_ignore")]
    pub chain_root_ignore: bool,
}

impl<'a> Default for Ignore<'a> {
    fn default() -> Self {
        Ignore {
            exact: Vec::from(IGNORE_EXACT_DEFAULT),
            pattern: Vec::from(IGNORE_PATTERN_DEFAULT),
            chain_root_ignore: default_chain_root_ignore(),
        }
    }
}

/// Reads and parses a configuration file.
///
/// Supports JSON with comments (JSONC format). The file contents are leaked
/// to enable zero-copy deserialization with borrowed string references.
///
/// # Arguments
///
/// * `path` - Path to the configuration file
///
/// # Returns
///
/// * `Ok(Config)` - Successfully parsed configuration
/// * `Err(ConfigError::Read)` - File could not be read
/// * `Err(ConfigError::Parse)` - File contents are not valid JSON
///
/// # Example
///
/// ```ignore
/// let config = read_config("~/.config/pfp/config.json")?;
/// println!("Found {} include entries", config.include.len());
/// ```
pub(crate) fn read_config(path: &str) -> Result<Config<'_>, ConfigError> {
    let contents = Box::leak(Box::new(std::fs::read_to_string(path)?));
    Ok(serde_jsonc::from_str(contents)?)
}
