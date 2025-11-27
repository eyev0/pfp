//! High-level selection functions for user interaction.
//!
//! This module provides higher-level abstractions over the fzf integration,
//! combining directory scanning with fuzzy selection for common use cases.
//!
//! # Functions
//!
//! - [`select_from_list`] - Present a list for user selection
//! - [`pick_project`] - Scan for projects and let user pick one

use std::collections::HashMap;

use log::trace;

use crate::{
    config::Config,
    fs::{expand, get_included_paths_list},
    fzf::execute_fzf_command,
    Error,
};

/// Presents a list of items to the user for fuzzy selection.
///
/// This is a thin wrapper around [`execute_fzf_command`]
/// that adds a header and handles empty selections.
///
/// # Arguments
///
/// * `list` - Newline-separated items to present
/// * `header` - Header text displayed at the top of fzf
/// * `args` - Additional fzf arguments (layout, preview, etc.)
///
/// # Returns
///
/// * `Ok(String)` - The user's selection
/// * `Err(Error::EmptyPick)` - If the user cancelled without selecting
/// * `Err(Error)` - On other errors
///
/// # Example
///
/// ```ignore
/// let result = select_from_list(
///     "option1\noption2\noption3",
///     "Choose an option:",
///     &["--layout", "reverse"]
/// )?;
/// ```
pub(crate) fn select_from_list(
    list: &str,
    header: &'static str,
    args: &[&str],
) -> Result<String, crate::Error> {
    let result = execute_fzf_command(args.iter().chain(&["--header", header]).cloned(), list)?;
    if result.is_empty() {
        trace!("Empty pick");
        Err(crate::Error::EmptyPick())
    } else {
        trace!("Pick: {}", result);
        Ok(result)
    }
}

/// Scans for project directories and presents them for user selection.
///
/// This is the main project picker function that:
/// 1. Scans all configured include paths for project directories
/// 2. Presents the found projects in fzf with tree preview
/// 3. Returns the user's selection
///
/// # Arguments
///
/// * `config` - Application configuration containing scan settings
/// * `header` - Header text for the fzf interface
///
/// # Returns
///
/// * `Ok(String)` - The absolute path to the selected project
/// * `Err(Error)` - On scan errors, cancelled selection, etc.
///
/// # Features
///
/// - Expands environment variables in configured paths
/// - Shows `tree -C` preview of directories
/// - Respects all marker and ignore settings from config
///
/// # Example
///
/// ```ignore
/// let project_path = pick_project(&config, "Select a project:")?;
/// println!("Opening: {}", project_path);
/// ```
pub(crate) fn pick_project(config: &Config, header: &'static str) -> Result<String, Error> {
    // get dirs' paths
    let dirs = {
        let mut paths_set = HashMap::new();
        for include_entry in config.include.iter() {
            for path in &include_entry.paths {
                let expanded_path = expand(path)?;
                if include_entry.include_intermediate_paths {
                    paths_set.insert(expanded_path.clone(), ());
                }
                get_included_paths_list(&expanded_path, 0, &mut paths_set, include_entry, config)?;
            }
        }
        paths_set.into_keys().collect::<Vec<String>>().join("\n")
    };

    // pick one from list with fzf
    let pick = select_from_list(
        &dirs,
        header,
        &[
            "--layout",
            "reverse",
            "--preview",
            "tree -C '{}'",
            "--preview-window",
            "right:nohidden",
        ],
    )?
    .trim_end()
    .to_owned();
    Ok(pick)
}
