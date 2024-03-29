use serde::Deserialize;

#[derive(thiserror::Error, Debug)]
pub(crate) enum ConfigError {
    #[error("Parse config: {0}")]
    Parse(#[from] serde_jsonc::Error),
    #[error("Read config: {0}")]
    Read(#[from] std::io::Error),
}

#[derive(Deserialize, Debug)]
pub(crate) struct Config<'a> {
    #[serde(default)]
    pub sessions: Vec<Session<'a>>,
    #[serde(default = "IncludeEntry::markers_default", borrow = "'a")]
    pub markers: Vec<&'a str>,
    #[serde(default = "IncludeEntry::ignore_default")]
    pub ignore: Vec<&'a str>,
    #[serde(default = "IncludeEntry::traverse_hidden_default")]
    pub traverse_hidden: bool,
    #[serde(default = "IncludeEntry::stop_on_marker_default")]
    pub stop_on_marker: bool,
    pub include: Vec<IncludeEntry<'a>>,
}

#[derive(Deserialize, Debug)]
pub(crate) struct Session<'a> {
    pub name: &'a str,
    pub windows: Vec<&'a str>,
}

impl<'a> ToString for Session<'a> {
    fn to_string(&self) -> String {
        format!(
            "{}:\n{}\n",
            self.name,
            self.windows
                .iter()
                .map(|p| crate::fs::expand_path(p).unwrap_or(p.to_string()))
                .collect::<Vec<_>>()
                .join("\n")
        )
    }
}

#[derive(Deserialize, Debug, Default)]
pub(crate) struct IncludeEntry<'a> {
    #[serde(borrow = "'a")]
    pub paths: Vec<&'a str>,
    #[serde(default)]
    pub markers: Vec<&'a str>,
    #[serde(default = "IncludeEntry::use_root_markers_default")]
    pub use_root_markers: bool,
    #[serde(default = "IncludeEntry::include_files_default")]
    pub include_files: bool,
    #[serde(default)]
    pub ignore: Vec<&'a str>,
    #[serde(default = "IncludeEntry::use_root_ignore_default")]
    pub use_root_ignore: bool,
    #[serde(default)]
    pub show_hidden: Option<bool>,
    #[serde(default)]
    pub stop_on_marker: Option<bool>,
    #[serde(default = "u8::max_value")]
    pub depth: u8,
}

const IGNORE_DEFAULT: [&str; 11] = [
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

const MARKERS_DEFAULT: [&str; 2] = [
    ".git",
    "Cargo.toml",
    // "package.json",
    // "pom.xml",
    // "build.gradle",
];

impl<'a> IncludeEntry<'a> {
    fn use_root_ignore_default() -> bool {
        true
    }
    fn use_root_markers_default() -> bool {
        true
    }
    fn traverse_hidden_default() -> bool {
        false
    }
    fn stop_on_marker_default() -> bool {
        true
    }
    fn markers_default() -> Vec<&'a str> {
        MARKERS_DEFAULT.to_vec()
    }
    fn include_files_default() -> bool {
        false
    }
    fn ignore_default() -> Vec<&'a str> {
        IGNORE_DEFAULT.to_vec()
    }
}

impl<'a> Default for Config<'a> {
    fn default() -> Self {
        Self {
            sessions: vec![],
            markers: IncludeEntry::markers_default(),
            ignore: IncludeEntry::ignore_default(),
            traverse_hidden: IncludeEntry::traverse_hidden_default(),
            stop_on_marker: IncludeEntry::stop_on_marker_default(),
            include: vec![IncludeEntry {
                paths: ["$HOME"].to_vec(),
                ..Default::default()
            }],
        }
    }
}

pub(crate) fn read_config(path: &str) -> Result<Config, ConfigError> {
    let contents = Box::leak(Box::new(std::fs::read_to_string(path)?));
    Ok(serde_jsonc::from_str(contents)?)
}
