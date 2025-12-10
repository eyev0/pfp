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
    context::AppContext,
    fs::scan_paths,
    Error,
};

/// Presents a list of items to the user for fuzzy selection.
///
/// This is a thin wrapper around fzf that adds a header and handles empty selections.
///
/// # Arguments
///
/// * `ctx` - Application context with fzf executor
/// * `list` - Newline-separated items to present
/// * `header` - Header text displayed at the top of fzf
/// * `args` - Additional fzf arguments (layout, preview, etc.)
///
/// # Returns
///
/// * `Ok(String)` - The user's selection
/// * `Err(Error::EmptyPick)` - If the user cancelled without selecting
/// * `Err(Error)` - On other errors
pub(crate) fn select_from_list(
    ctx: &AppContext,
    list: &str,
    header: &'static str,
    args: &[&str],
) -> Result<String, Error> {
    let mut full_args: Vec<&str> = args.to_vec();
    full_args.extend(&["--header", header]);
    
    let result = ctx.fzf_execute(&full_args, list)?;
    if result.is_empty() {
        trace!("Empty pick");
        Err(Error::EmptyPick())
    } else {
        trace!("Pick: {}", result);
        Ok(result)
    }
}

/// Scans for project directories and presents them for user selection.
///
/// This is the main project picker function that:
/// 1. Scans all configured include paths using their resolved profiles
/// 2. Presents the found projects in fzf with tree preview
/// 3. Returns the user's selection
///
/// # Arguments
///
/// * `ctx` - Application context with fzf executor
/// * `config` - Application configuration containing scan settings
/// * `header` - Header text for the fzf interface
///
/// # Returns
///
/// * `Ok(String)` - The absolute path to the selected project
/// * `Err(Error)` - On scan errors, cancelled selection, etc.
pub(crate) fn pick_project(
    ctx: &AppContext,
    config: &Config,
    header: &'static str,
) -> Result<String, Error> {
    let mut paths_set: HashMap<String, ()> = HashMap::new();

    // Scan all include entries
    for entry in &config.include {
        let profile = config.resolve_profile(entry);
        let paths: Vec<&str> = entry.paths();
        scan_paths(&paths, &profile, &mut paths_set)?;
    }

    let dirs = paths_set.into_keys().collect::<Vec<String>>().join("\n");

    // Pick one from list with fzf
    let pick = select_from_list(
        ctx,
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
