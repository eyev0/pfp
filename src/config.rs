//! Configuration parsing and data structures.
//!
//! This module provides types for parsing and representing the application
//! configuration with profile-based scanning settings.
//!
//! # Configuration File Location
//!
//! By default, the configuration is read from `${XDG_CONFIG_HOME}/pfp/config.json`.
//!
//! # Default Profiles
//!
//! Built-in profiles are loaded from `defaults.json` (embedded at compile time).
//! See the file for reference values you can copy and customize.
//!
//! # Example Configuration
//!
//! ```json
//! {
//!   "profiles": {
//!     "projects": {
//!       "markers": [".git", "Cargo.toml", "pom.xml"]
//!     }
//!   },
//!   "include": [
//!     "$HOME/dev",
//!     { "paths": ["$HOME/Downloads"], "profile": "browse" }
//!   ]
//! }
//! ```

use serde::Deserialize;
use std::collections::HashMap;
use std::fmt;

/// Embedded default profiles from defaults.json
const DEFAULTS_JSON: &str = include_str!("../defaults.json");

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

/// Scanning mode for profiles.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub(crate) enum Mode {
    /// Scan for directories containing project markers (default)
    #[default]
    Dir,
    /// Scan for individual files
    File,
}

/// A profile defining scan behavior.
///
/// All fields are optional to support partial overrides.
/// When merging, specified fields override the base profile.
#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct Profile {
    /// Scanning mode: `dir` for directories, `file` for files
    pub mode: Option<Mode>,
    /// Markers that identify targets (glob patterns supported)
    pub markers: Option<Vec<String>>,
    /// Patterns to ignore during scanning (glob patterns supported)
    pub ignore: Option<Vec<String>>,
    /// Maximum directory depth to descend
    pub depth: Option<u8>,
    /// Stop descending when a marker is found
    pub stop_on_marker: Option<bool>,
    /// Include parent directories of matches in results
    pub intermediate_paths: Option<bool>,
    /// Show hidden files in results (dotfiles like .zshrc)
    pub show_hidden: Option<bool>,
    /// Traverse into hidden directories (starting with .)
    pub traverse_hidden_dirs: Option<bool>,
}

impl Profile {
    /// Merge another profile on top of this one.
    /// Fields from `other` override fields in `self` if they are Some.
    pub fn merge(&self, other: &Profile) -> Profile {
        Profile {
            mode: other.mode.or(self.mode),
            markers: other.markers.clone().or_else(|| self.markers.clone()),
            ignore: other.ignore.clone().or_else(|| self.ignore.clone()),
            depth: other.depth.or(self.depth),
            stop_on_marker: other.stop_on_marker.or(self.stop_on_marker),
            intermediate_paths: other.intermediate_paths.or(self.intermediate_paths),
            show_hidden: other.show_hidden.or(self.show_hidden),
            traverse_hidden_dirs: other.traverse_hidden_dirs.or(self.traverse_hidden_dirs),
        }
    }

    /// Convert to a resolved profile with all defaults filled in.
    pub fn resolve(&self) -> ResolvedProfile {
        ResolvedProfile {
            mode: self.mode.unwrap_or_default(),
            markers: self.markers.clone().unwrap_or_default(),
            ignore: self.ignore.clone().unwrap_or_default(),
            depth: self.depth.unwrap_or(255),
            stop_on_marker: self.stop_on_marker.unwrap_or(true),
            intermediate_paths: self.intermediate_paths.unwrap_or(true),
            show_hidden: self.show_hidden.unwrap_or(false),
            traverse_hidden_dirs: self.traverse_hidden_dirs.unwrap_or(false),
        }
    }
}

/// A fully resolved profile with no optional fields.
#[derive(Debug, Clone)]
pub(crate) struct ResolvedProfile {
    pub mode: Mode,
    pub markers: Vec<String>,
    pub ignore: Vec<String>,
    pub depth: u8,
    pub stop_on_marker: bool,
    pub intermediate_paths: bool,
    /// Show hidden files in results (dotfiles like .zshrc)
    pub show_hidden: bool,
    /// Whether to traverse into hidden directories (starting with .)
    pub traverse_hidden_dirs: bool,
}

/// An entry in the include array - either a simple path string or a detailed object.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub(crate) enum IncludeEntry {
    /// Simple path string - uses "projects" profile by default
    Simple(String),
    /// Detailed entry with paths, profile, and optional overrides
    Detailed(IncludeEntryDetailed),
}

impl IncludeEntry {
    /// Get the paths for this entry.
    pub fn paths(&self) -> Vec<&str> {
        match self {
            IncludeEntry::Simple(path) => vec![path.as_str()],
            IncludeEntry::Detailed(d) => d.paths.iter().map(|s| s.as_str()).collect(),
        }
    }

    /// Get the profile name for this entry.
    pub fn profile_name(&self) -> &str {
        match self {
            IncludeEntry::Simple(_) => "projects",
            IncludeEntry::Detailed(d) => d.profile.as_deref().unwrap_or("projects"),
        }
    }

    /// Get inline overrides as a Profile (for merging).
    pub fn overrides(&self) -> Profile {
        match self {
            IncludeEntry::Simple(_) => Profile::default(),
            IncludeEntry::Detailed(d) => Profile {
                mode: d.mode,
                markers: d.markers.clone(),
                ignore: d.ignore.clone(),
                depth: d.depth,
                stop_on_marker: d.stop_on_marker,
                intermediate_paths: d.intermediate_paths,
                show_hidden: d.show_hidden,
                traverse_hidden_dirs: d.traverse_hidden_dirs,
            },
        }
    }
}

/// Detailed include entry with all options.
#[derive(Deserialize, Debug)]
pub(crate) struct IncludeEntryDetailed {
    /// Paths to scan (always an array)
    pub paths: Vec<String>,
    /// Profile name to use as base
    pub profile: Option<String>,
    // Inline overrides (all optional)
    /// Override mode
    pub mode: Option<Mode>,
    /// Override markers
    pub markers: Option<Vec<String>>,
    /// Override ignore patterns
    pub ignore: Option<Vec<String>>,
    /// Override depth
    pub depth: Option<u8>,
    /// Override stop_on_marker
    pub stop_on_marker: Option<bool>,
    /// Override intermediate_paths
    pub intermediate_paths: Option<bool>,
    /// Override show_hidden
    pub show_hidden: Option<bool>,
    /// Override traverse_hidden_dirs
    pub traverse_hidden_dirs: Option<bool>,
}

/// A predefined tmux session configuration.
#[derive(Deserialize, Debug)]
pub(crate) struct Session {
    /// The session name (displayed in tmux status bar)
    pub name: String,
    /// List of paths for windows in this session (supports env vars)
    pub windows: Vec<String>,
}

impl fmt::Display for Session {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:\n{}\n",
            self.name,
            self.windows
                .iter()
                .map(|p| crate::fs::expand(p).unwrap_or_else(|_| p.to_string()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

/// Raw configuration as parsed from JSON.
#[derive(Deserialize, Debug, Default)]
struct RawConfig {
    #[serde(default)]
    profiles: HashMap<String, Profile>,
    #[serde(default)]
    include: Vec<IncludeEntry>,
    #[serde(default)]
    sessions: Vec<Session>,
}

/// Main configuration structure with resolved profiles.
#[derive(Debug)]
pub(crate) struct Config {
    /// All profiles (defaults merged with user overrides)
    pub profiles: HashMap<String, Profile>,
    /// List of paths to scan with their settings
    pub include: Vec<IncludeEntry>,
    /// Predefined tmux sessions
    pub sessions: Vec<Session>,
}

impl Default for Config {
    fn default() -> Self {
        let defaults: RawConfig =
            serde_jsonc::from_str(DEFAULTS_JSON).expect("Invalid defaults.json");
        Config {
            profiles: defaults.profiles,
            include: vec![IncludeEntry::Simple("$HOME".to_string())],
            sessions: vec![],
        }
    }
}

impl Config {
    /// Resolve a profile by name, applying the merge chain:
    /// defaults[profile] <- user_profiles[profile] <- inline_overrides
    pub fn resolve_profile(&self, entry: &IncludeEntry) -> ResolvedProfile {
        let profile_name = entry.profile_name();
        let base_profile = self.profiles.get(profile_name).cloned().unwrap_or_default();
        let overrides = entry.overrides();
        base_profile.merge(&overrides).resolve()
    }
}

/// Load default profiles from embedded defaults.json.
fn load_defaults() -> HashMap<String, Profile> {
    let defaults: RawConfig = serde_jsonc::from_str(DEFAULTS_JSON).expect("Invalid defaults.json");
    defaults.profiles
}

/// Reads and parses a configuration file.
///
/// Supports JSON with comments (JSONC format). Default profiles are loaded
/// from the embedded defaults.json and merged with user configuration.
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
pub(crate) fn read_config(path: &str) -> Result<Config, ConfigError> {
    let contents = std::fs::read_to_string(path)?;
    let raw: RawConfig = serde_jsonc::from_str(&contents)?;

    // Load defaults and merge user profiles on top
    let mut profiles = load_defaults();
    for (name, user_profile) in raw.profiles {
        let merged = profiles
            .get(&name)
            .map(|base| base.merge(&user_profile))
            .unwrap_or(user_profile);
        profiles.insert(name, merged);
    }

    Ok(Config {
        profiles,
        include: if raw.include.is_empty() {
            vec![IncludeEntry::Simple("$HOME".to_string())]
        } else {
            raw.include
        },
        sessions: raw.sessions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_defaults_json_parses() {
        let defaults: RawConfig = serde_jsonc::from_str(DEFAULTS_JSON).unwrap();
        assert!(defaults.profiles.contains_key("projects"));
        assert!(defaults.profiles.contains_key("browse"));
        assert!(defaults.profiles.contains_key("files"));
    }

    #[test]
    fn test_profile_merge_overrides_some_fields() {
        let base = Profile {
            mode: Some(Mode::Dir),
            markers: Some(vec![".git".to_string()]),
            ignore: Some(vec!["node_modules".to_string()]),
            depth: Some(255),
            stop_on_marker: Some(true),
            intermediate_paths: Some(true),
            show_hidden: Some(false),
            traverse_hidden_dirs: Some(false),
        };
        
        let override_profile = Profile {
            mode: None,
            markers: Some(vec![".git".to_string(), "pom.xml".to_string()]),
            ignore: None,
            depth: Some(5),
            stop_on_marker: None,
            intermediate_paths: None,
            show_hidden: None,
            traverse_hidden_dirs: None,
        };
        
        let merged = base.merge(&override_profile);
        
        // Overridden fields
        assert_eq!(merged.markers, Some(vec![".git".to_string(), "pom.xml".to_string()]));
        assert_eq!(merged.depth, Some(5));
        
        // Inherited fields
        assert_eq!(merged.mode, Some(Mode::Dir));
        assert_eq!(merged.ignore, Some(vec!["node_modules".to_string()]));
        assert_eq!(merged.stop_on_marker, Some(true));
        assert_eq!(merged.intermediate_paths, Some(true));
    }

    #[test]
    fn test_profile_resolve_with_defaults() {
        let profile = Profile {
            mode: Some(Mode::File),
            markers: Some(vec!["*.rs".to_string()]),
            ignore: None,
            depth: None,
            stop_on_marker: None,
            intermediate_paths: None,
            show_hidden: None,
            traverse_hidden_dirs: None,
        };
        
        let resolved = profile.resolve();
        
        assert_eq!(resolved.mode, Mode::File);
        assert_eq!(resolved.markers, vec!["*.rs".to_string()]);
        assert!(resolved.ignore.is_empty()); // Default empty
        assert_eq!(resolved.depth, 255); // Default
        assert!(resolved.stop_on_marker); // Default true
        assert!(resolved.intermediate_paths); // Default true
        assert!(!resolved.show_hidden); // Default false
        assert!(!resolved.traverse_hidden_dirs); // Default false
    }

    #[test]
    fn test_include_entry_simple_string() {
        let json = r#""$HOME/dev""#;
        let entry: IncludeEntry = serde_jsonc::from_str(json).unwrap();
        
        assert_eq!(entry.paths(), vec!["$HOME/dev"]);
        assert_eq!(entry.profile_name(), "projects");
        
        let overrides = entry.overrides();
        assert!(overrides.mode.is_none());
        assert!(overrides.markers.is_none());
    }

    #[test]
    fn test_include_entry_detailed_object() {
        let json = r#"{
            "paths": ["/tmp", "/var"],
            "profile": "browse",
            "depth": 3
        }"#;
        let entry: IncludeEntry = serde_jsonc::from_str(json).unwrap();
        
        assert_eq!(entry.paths(), vec!["/tmp", "/var"]);
        assert_eq!(entry.profile_name(), "browse");
        
        let overrides = entry.overrides();
        assert_eq!(overrides.depth, Some(3));
        assert!(overrides.mode.is_none());
    }

    #[test]
    fn test_include_entry_detailed_with_inline_overrides() {
        let json = r#"{
            "paths": ["$HOME/scripts"],
            "profile": "files",
            "mode": "file",
            "markers": ["*.sh"],
            "depth": 2
        }"#;
        let entry: IncludeEntry = serde_jsonc::from_str(json).unwrap();
        
        let overrides = entry.overrides();
        assert_eq!(overrides.mode, Some(Mode::File));
        assert_eq!(overrides.markers, Some(vec!["*.sh".to_string()]));
        assert_eq!(overrides.depth, Some(2));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        
        assert!(config.profiles.contains_key("projects"));
        assert!(config.profiles.contains_key("browse"));
        assert!(config.profiles.contains_key("files"));
        assert_eq!(config.include.len(), 1);
    }

    #[test]
    fn test_config_resolve_profile() {
        let config = Config::default();
        let entry = IncludeEntry::Detailed(IncludeEntryDetailed {
            paths: vec!["/tmp".to_string()],
            profile: Some("projects".to_string()),
            mode: None,
            markers: None,
            ignore: None,
            depth: Some(3),
            stop_on_marker: None,
            intermediate_paths: None,
            show_hidden: None,
            traverse_hidden_dirs: None,
        });
        
        let resolved = config.resolve_profile(&entry);
        
        // From profile
        assert_eq!(resolved.mode, Mode::Dir);
        assert!(resolved.markers.contains(&".git".to_string()));
        
        // From inline override
        assert_eq!(resolved.depth, 3);
    }

    #[test]
    fn test_parse_full_config() {
        let json = r#"{
            "profiles": {
                "projects": {
                    "markers": [".git", "pom.xml"]
                }
            },
            "include": [
                "$HOME/dev",
                { "paths": ["/tmp"], "profile": "browse" }
            ],
            "sessions": [
                { "name": "work", "windows": ["/a", "/b"] }
            ]
        }"#;
        
        let raw: RawConfig = serde_jsonc::from_str(json).unwrap();
        
        assert_eq!(raw.include.len(), 2);
        assert_eq!(raw.sessions.len(), 1);
        assert_eq!(raw.sessions[0].name, "work");
        
        // Check profile override
        let projects = raw.profiles.get("projects").unwrap();
        assert_eq!(projects.markers, Some(vec![".git".to_string(), "pom.xml".to_string()]));
    }

    #[test]
    fn test_profile_merge_ignore_override() {
        let base = Profile {
            mode: Some(Mode::Dir),
            markers: Some(vec![".git".to_string()]),
            ignore: Some(vec!["node_modules".to_string(), "target".to_string()]),
            depth: Some(10),
            stop_on_marker: Some(true),
            intermediate_paths: Some(true),
            show_hidden: Some(false),
            traverse_hidden_dirs: Some(false),
        };
        
        // User wants to add custom ignore
        let override_profile = Profile {
            ignore: Some(vec!["my_ignored_dir".to_string()]),
            ..Default::default()
        };
        
        let merged = base.merge(&override_profile);
        
        // Override completely replaces ignore, NOT appends
        assert_eq!(merged.ignore, Some(vec!["my_ignored_dir".to_string()]));
        
        // Other fields should remain from base
        assert_eq!(merged.markers, Some(vec![".git".to_string()]));
    }

    #[test]
    fn test_resolve_profile_with_user_ignore() {
        // Simulate: defaults has ignore, user profile overrides ignore
        let defaults_profile = Profile {
            mode: Some(Mode::Dir),
            markers: Some(vec![".git".to_string()]),
            ignore: Some(vec!["node_modules".to_string()]),
            depth: Some(255),
            stop_on_marker: Some(true),
            intermediate_paths: Some(true),
            show_hidden: Some(false),
            traverse_hidden_dirs: Some(false),
        };
        
        let user_profile = Profile {
            ignore: Some(vec!["my_custom_dir".to_string(), "build".to_string()]),
            ..Default::default()
        };
        
        let merged = defaults_profile.merge(&user_profile);
        let resolved = merged.resolve();
        
        // User's ignore should completely replace default ignore
        assert_eq!(resolved.ignore, vec!["my_custom_dir".to_string(), "build".to_string()]);
        assert!(!resolved.ignore.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_inline_ignore_override() {
        let config = Config::default();
        
        // Inline override with custom ignore
        let entry = IncludeEntry::Detailed(IncludeEntryDetailed {
            paths: vec!["/tmp".to_string()],
            profile: Some("projects".to_string()),
            mode: None,
            markers: None,
            ignore: Some(vec!["custom_ignored".to_string()]),
            depth: None,
            stop_on_marker: None,
            intermediate_paths: None,
            show_hidden: None,
            traverse_hidden_dirs: None,
        });
        
        let resolved = config.resolve_profile(&entry);
        
        // Inline ignore should override profile's ignore
        assert_eq!(resolved.ignore, vec!["custom_ignored".to_string()]);
        assert!(!resolved.ignore.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_default_ignore_preserved_when_not_overridden() {
        let config = Config::default();
        
        // Entry without ignore override
        let entry = IncludeEntry::Detailed(IncludeEntryDetailed {
            paths: vec!["/tmp".to_string()],
            profile: Some("projects".to_string()),
            mode: None,
            markers: None,
            ignore: None, // No override
            depth: None,
            stop_on_marker: None,
            intermediate_paths: None,
            show_hidden: None,
            traverse_hidden_dirs: None,
        });
        
        let resolved = config.resolve_profile(&entry);
        
        // Default ignore should be preserved
        assert!(resolved.ignore.contains(&"node_modules".to_string()));
        assert!(resolved.ignore.contains(&"target".to_string()));
        assert!(resolved.ignore.contains(&"venv".to_string()));
    }
}
