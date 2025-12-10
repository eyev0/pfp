//! Filesystem utilities for path manipulation and directory scanning.
//!
//! This module provides functions for:
//! - Environment variable expansion in paths
//! - Path name manipulation for tmux windows/sessions
//! - Recursive directory scanning with configurable markers and ignore patterns
//!
//! # Key Functions
//!
//! - [`expand`] - Expand environment variables in paths
//! - [`scan_paths`] - Scan directories for project paths using resolved profile
//! - [`trim_window_name`] - Create short names for tmux windows
//! - [`trim_session_name`] - Sanitize session names for tmux

use crate::config::{Mode, ResolvedProfile};
use crate::Error;

use anyhow::anyhow;
use glob::Pattern;
use log::{error, trace};

use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, DirEntry, FileType};
use std::path::PathBuf;

/// Expands environment variables in a path string.
///
/// Supports two formats:
/// - Simple: `$VAR`
/// - Braced: `${VAR}`
///
/// # Arguments
///
/// * `path` - A string slice containing potential environment variable references
///
/// # Returns
///
/// * `Ok(String)` - The path with all environment variables expanded
/// * `Err(Error::EnvVar)` - If any referenced environment variable is not set
pub(crate) fn expand(path: &str) -> Result<String, Error> {
    let mut result = path.to_string();
    let mut i = 0;
    
    while let Some(dollar_pos) = result[i..].find('$') {
        let start = i + dollar_pos;
        let rest = &result[start + 1..];
        
        let (var_name, end_offset) = if rest.starts_with('{') {
            // ${VAR} format
            if let Some(close) = rest.find('}') {
                (&rest[1..close], close + 2)
            } else {
                i = start + 1;
                continue;
            }
        } else {
            // $VAR format - read until non-alphanumeric/underscore
            let end = rest
                .find(|c: char| !c.is_alphanumeric() && c != '_')
                .unwrap_or(rest.len());
            if end == 0 {
                i = start + 1;
                continue;
            }
            (&rest[..end], end + 1)
        };
        
        if var_name.is_empty() {
            i = start + 1;
            continue;
        }
        
        let value = env::var(OsStr::new(var_name))
            .map_err(|e| Error::EnvVar(e, var_name.to_string()))?;
        
        result = format!("{}{}{}", &result[..start], value, &result[start + end_offset..]);
        i = start + value.len();
    }
    
    Ok(result)
}

/// Creates a short window name from a full path.
///
/// Retains the last two path segments, with the parent directory truncated to 4 characters
/// if only one segment is present, the entire path is returned.
///
/// # Arguments
///
/// * `path` - The full path to the project
///
/// # Returns
///
/// * `Ok(String)` - The truncated window name
/// * `Err(anyhow::Error)` - If the path is not valid utf8
///
/// # Examples
///
/// ```
/// let path = "/home/user/projects/myapp";
/// let window_name = trim_window_name(path).unwrap();
/// assert_eq!(window_name, "proj/myapp");
/// ```
pub(crate) fn trim_window_name(path: &str) -> Result<String, anyhow::Error> {
    let parts: Vec<&str> = path.trim_end_matches('/').rsplit('/').take(2).collect();
    match parts.len() {
        2 => Ok(format!(
            "{}/{}",
            parts[1].chars().take(4).collect::<String>(),
            parts[0]
        )),
        1 => Ok(parts[0].to_string()),
        _ => Ok(path.to_string()),
    }
}

/// Removes dots from a session name for tmux compatibility.
pub(crate) fn trim_session_name(name: &String) -> String {
    let mut s = String::from(name);
    s.retain(|x| x != '.');
    s
}

/// Check if a name or path matches any of the patterns.
/// Patterns can be exact matches or glob patterns (containing * or ?).
/// 
/// If a pattern contains `/`, it's matched against the full relative path.
/// Otherwise, it's matched against just the name (last component).
pub(crate) fn matches_patterns(name: &str, patterns: &[String]) -> bool {
    matches_patterns_with_path(name, None, patterns)
}

/// Check if a name matches patterns, optionally using relative path for patterns with `/`.
/// 
/// - `name` - the directory/file name (last path component)
/// - `relative_path` - optional relative path from scan root (e.g., "mason/packages")
/// - `patterns` - list of patterns to match against
pub(crate) fn matches_patterns_with_path(name: &str, relative_path: Option<&str>, patterns: &[String]) -> bool {
    for pattern in patterns {
        if pattern == "*" {
            return true;
        }
        
        // If pattern contains '/', match against relative path
        let match_target = if pattern.contains('/') {
            relative_path.unwrap_or(name)
        } else {
            name
        };
        
        if pattern.contains('*') || pattern.contains('?') {
            // Glob pattern
            if let Ok(p) = Pattern::new(pattern) {
                if p.matches(match_target) {
                    return true;
                }
            }
        } else {
            // Exact match
            if match_target == pattern {
                return true;
            }
        }
    }
    false
}

/// Scan paths according to the resolved profile settings.
///
/// # Arguments
///
/// * `paths` - Root paths to start scanning from
/// * `profile` - Resolved profile with scan settings
/// * `output` - HashMap to populate with discovered paths
pub(crate) fn scan_paths(
    paths: &[&str],
    profile: &ResolvedProfile,
    output: &mut HashMap<String, ()>,
) -> Result<(), Error> {
    for path in paths {
        let expanded = expand(path)?;
        if profile.intermediate_paths {
            output.insert(expanded.clone(), ());
        }
        scan_directory(&expanded, &expanded, 0, profile, output)?;
    }
    Ok(())
}

/// Recursively scan a directory.
/// 
/// * `path` - Current directory path
/// * `root` - Root path (for computing relative paths for ignore patterns)
fn scan_directory(
    path: &str,
    root: &str,
    depth: u8,
    profile: &ResolvedProfile,
    output: &mut HashMap<String, ()>,
) -> Result<bool, Error> {
    let mut path_has_marker = false;

    // Read directory contents
    let read_dir = match fs::read_dir(path) {
        Ok(read) => read,
        Err(err) => {
            trace!("Error reading dir {}: {:#?}", path, err);
            return Ok(false);
        }
    };
    let dir_contents: Vec<DirEntry> = read_dir.flatten().collect();

    match profile.mode {
        Mode::Dir => {
            // Check for markers in current directory
            for entry in &dir_contents {
                let name = match entry.file_name().to_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                
                if matches_patterns(&name, &profile.markers) {
                    trace!("Marker found: {} in {}", name, path);
                    path_has_marker = true;
                    output.insert(path.to_string(), ());
                    
                    if profile.stop_on_marker {
                        return Ok(true);
                    }
                    break;
                }
            }

            // Check depth limit
            if depth >= profile.depth {
                return Ok(path_has_marker);
            }

            // Recurse into subdirectories
            for entry in &dir_contents {
                let name = match entry.file_name().to_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                
                // Skip hidden directories unless traverse_hidden_dirs is enabled
                if name.starts_with('.') && !profile.traverse_hidden_dirs {
                    continue;
                }

                let entry_path = match get_path_string(entry) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                // Compute relative path for ignore patterns with '/'
                let relative_path = entry_path.strip_prefix(root)
                    .map(|p| p.trim_start_matches('/'))
                    .unwrap_or(&name);
                
                // Skip ignored
                if matches_patterns_with_path(&name, Some(relative_path), &profile.ignore) {
                    continue;
                }

                let ft = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };

                if is_dir(&entry_path, &ft)? && scan_directory(&entry_path, root, depth + 1, profile, output)? {
                    path_has_marker = true;
                }
            }

            // Include intermediate paths if children matched
            if path_has_marker && profile.intermediate_paths {
                output.insert(path.to_string(), ());
            }
        }
        Mode::File => {
            // Collect files and recurse into directories
            for entry in &dir_contents {
                let name = match entry.file_name().to_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };

                let entry_path = match get_path_string(entry) {
                    Ok(p) => p,
                    Err(_) => continue,
                };

                // Compute relative path for ignore patterns with '/'
                let relative_path = entry_path.strip_prefix(root)
                    .map(|p| p.trim_start_matches('/'))
                    .unwrap_or(&name);
                
                // Skip ignored
                if matches_patterns_with_path(&name, Some(relative_path), &profile.ignore) {
                    continue;
                }

                let ft = match entry.file_type() {
                    Ok(ft) => ft,
                    Err(_) => continue,
                };

                if is_file(&entry_path, &ft)? && matches_patterns(&name, &profile.markers) {
                    // Skip hidden files unless show_hidden is enabled
                    if name.starts_with('.') && !profile.show_hidden {
                        continue;
                    }
                    path_has_marker = true;
                    output.insert(entry_path, ());
                } else if is_dir(&entry_path, &ft)? {
                    // Skip hidden directories unless traverse_hidden_dirs is enabled
                    if name.starts_with('.') && !profile.traverse_hidden_dirs {
                        continue;
                    }
                    if depth < profile.depth 
                        && scan_directory(&entry_path, root, depth + 1, profile, output)? 
                    {
                        path_has_marker = true;
                    }
                }
            }

            // Include intermediate paths if children matched
            if path_has_marker && profile.intermediate_paths {
                output.insert(path.to_string(), ());
            }
        }
    }

    Ok(path_has_marker)
}

fn get_path_string(entry: &DirEntry) -> Result<String, anyhow::Error> {
    Ok(String::from(entry.path().to_str().ok_or_else(|| {
        anyhow!("entry.path() is not valid utf8: {:#?}", entry.path())
    })?))
}

/// Checks if a path is a directory, following symlinks.
pub(crate) fn is_dir(path: &str, ft: &FileType) -> Result<bool, std::io::Error> {
    if ft.is_symlink() {
        Ok(read_link(path)
            .as_deref()
            .map(std::path::Path::is_dir)
            .unwrap_or(false))
    } else {
        Ok(ft.is_dir())
    }
}

/// Checks if a path is a regular file, following symlinks.
pub(crate) fn is_file(path: &str, ft: &FileType) -> Result<bool, std::io::Error> {
    if ft.is_symlink() {
        Ok(read_link(path)
            .as_deref()
            .map(std::path::Path::is_file)
            .unwrap_or(false))
    } else {
        Ok(ft.is_file())
    }
}

fn read_link(path: &str) -> Option<PathBuf> {
    match fs::read_link(PathBuf::from(path)) {
        Ok(rl) => Some(rl),
        Err(err) => {
            error!("error reading link: {:#?}", err);
            None
        }
    }
}

/// Convenience function to check if a path is a file using metadata.
pub(crate) fn path_is_file(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(meta) => meta.is_file(),
        Err(err) => {
            error!("error reading metadata of path {}: {}", path, err);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests for expand()
    
    #[test]
    fn test_expand_home_var() {
        // SAFETY: Tests run single-threaded by default
        unsafe { std::env::set_var("TEST_HOME", "/home/test") };
        let result = expand("$TEST_HOME/projects").unwrap();
        assert_eq!(result, "/home/test/projects");
    }

    #[test]
    fn test_expand_braced_var() {
        // SAFETY: Tests run single-threaded by default
        unsafe { std::env::set_var("TEST_VAR", "value") };
        let result = expand("${TEST_VAR}/path").unwrap();
        assert_eq!(result, "value/path");
    }

    #[test]
    fn test_expand_multiple_vars() {
        // SAFETY: Tests run single-threaded by default
        unsafe {
            std::env::set_var("TEST_A", "aaa");
            std::env::set_var("TEST_B", "bbb");
        }
        let result = expand("$TEST_A/$TEST_B/end").unwrap();
        assert_eq!(result, "aaa/bbb/end");
    }

    #[test]
    fn test_expand_no_vars() {
        let result = expand("/plain/path").unwrap();
        assert_eq!(result, "/plain/path");
    }

    #[test]
    fn test_expand_nonexistent_var() {
        let result = expand("$NONEXISTENT_VAR_12345/path");
        assert!(result.is_err());
    }

    // Tests for matches_patterns()

    #[test]
    fn test_matches_patterns_exact() {
        let patterns = vec![".git".to_string(), "Cargo.toml".to_string()];
        
        assert!(matches_patterns(".git", &patterns));
        assert!(matches_patterns("Cargo.toml", &patterns));
        assert!(!matches_patterns("README.md", &patterns));
    }

    #[test]
    fn test_matches_patterns_glob_star() {
        let patterns = vec!["*.rs".to_string()];
        
        assert!(matches_patterns("main.rs", &patterns));
        assert!(matches_patterns("lib.rs", &patterns));
        assert!(!matches_patterns("main.py", &patterns));
    }

    #[test]
    fn test_matches_patterns_glob_prefix() {
        let patterns = vec!["tree-sitter-*".to_string()];
        
        assert!(matches_patterns("tree-sitter-rust", &patterns));
        assert!(matches_patterns("tree-sitter-python", &patterns));
        assert!(!matches_patterns("tree-rust", &patterns));
    }

    #[test]
    fn test_matches_patterns_wildcard_all() {
        let patterns = vec!["*".to_string()];
        
        assert!(matches_patterns("anything", &patterns));
        assert!(matches_patterns(".hidden", &patterns));
        assert!(matches_patterns("file.txt", &patterns));
    }

    #[test]
    fn test_matches_patterns_question_mark() {
        let patterns = vec!["file?.txt".to_string()];
        
        assert!(matches_patterns("file1.txt", &patterns));
        assert!(matches_patterns("fileA.txt", &patterns));
        assert!(!matches_patterns("file12.txt", &patterns));
    }

    #[test]
    fn test_matches_patterns_empty() {
        let patterns: Vec<String> = vec![];
        
        assert!(!matches_patterns("anything", &patterns));
    }

    // Tests for matches_patterns_with_path (path-based ignore patterns)

    #[test]
    fn test_matches_patterns_with_path_simple() {
        // Simple pattern without / - matches against name
        let patterns = vec!["packages".to_string()];
        
        assert!(matches_patterns_with_path("packages", Some("mason/packages"), &patterns));
        assert!(!matches_patterns_with_path("mason", Some("mason"), &patterns));
    }

    #[test]
    fn test_matches_patterns_with_path_slash_pattern() {
        // Pattern with / - matches against relative path
        let patterns = vec!["mason/packages".to_string()];
        
        // Should match when relative path matches
        assert!(matches_patterns_with_path("packages", Some("mason/packages"), &patterns));
        
        // Should NOT match just the name
        assert!(!matches_patterns_with_path("packages", Some("other/packages"), &patterns));
        assert!(!matches_patterns_with_path("packages", None, &patterns));
    }

    #[test]
    fn test_matches_patterns_with_path_glob_slash() {
        // Glob pattern with /
        let patterns = vec!["mason/packages*".to_string()];
        
        assert!(matches_patterns_with_path("packages", Some("mason/packages"), &patterns));
        assert!(matches_patterns_with_path("packages-extra", Some("mason/packages-extra"), &patterns));
        assert!(!matches_patterns_with_path("packages", Some("other/packages"), &patterns));
    }

    #[test]
    fn test_matches_patterns_with_path_deep_glob() {
        // Deep path glob
        let patterns = vec!["nvim-data/mason/*".to_string()];
        
        assert!(matches_patterns_with_path("packages", Some("nvim-data/mason/packages"), &patterns));
        assert!(matches_patterns_with_path("bin", Some("nvim-data/mason/bin"), &patterns));
        assert!(!matches_patterns_with_path("packages", Some("other/mason/packages"), &patterns));
    }

    // Tests for trim_window_name()

    #[test]
    fn test_trim_window_name_normal() {
        let result = trim_window_name("/home/user/projects/myapp").unwrap();
        assert_eq!(result, "proj/myapp");
    }

    #[test]
    fn test_trim_window_name_short_parent() {
        let result = trim_window_name("/home/dev/app").unwrap();
        assert_eq!(result, "dev/app");
    }

    #[test]
    fn test_trim_window_name_trailing_slash() {
        let result = trim_window_name("/home/user/projects/myapp/").unwrap();
        assert_eq!(result, "proj/myapp");
    }

    #[test]
    fn test_trim_window_name_single_component() {
        let result = trim_window_name("/root").unwrap();
        assert_eq!(result, "/root");
    }

    // Tests for trim_session_name()

    #[test]
    fn test_trim_session_name_with_dots() {
        let result = trim_session_name(&"my.app.name".to_string());
        assert_eq!(result, "myappname");
    }

    #[test]
    fn test_trim_session_name_no_dots() {
        let result = trim_session_name(&"myapp".to_string());
        assert_eq!(result, "myapp");
    }

    #[test]
    fn test_trim_session_name_only_dots() {
        let result = trim_session_name(&"...".to_string());
        assert_eq!(result, "");
    }

    // Tests for ignore patterns in scan_paths

    use tempfile::TempDir;
    use crate::config::{Mode, ResolvedProfile};

    fn create_scan_test_structure(base: &std::path::Path) {
        use std::fs as stdfs;
        
        // Project with .git marker
        let project = base.join("myproject");
        stdfs::create_dir_all(project.join(".git")).unwrap();
        stdfs::write(project.join("README.md"), "# Project").unwrap();
        
        // node_modules should be ignored
        let nm = base.join("myproject/node_modules/lodash");
        stdfs::create_dir_all(&nm).unwrap();
        stdfs::write(nm.join("package.json"), "{}").unwrap();
        
        // target should be ignored  
        let target = base.join("myproject/target/debug");
        stdfs::create_dir_all(&target).unwrap();
        stdfs::write(target.join("binary"), "").unwrap();
        
        // custom_ignored should be ignored when specified
        let custom = base.join("myproject/custom_ignored/sub");
        stdfs::create_dir_all(&custom).unwrap();
        stdfs::write(custom.join("file.txt"), "").unwrap();
        
        // Another project inside node_modules (should be ignored)
        let nested_project = base.join("myproject/node_modules/some-pkg");
        stdfs::create_dir_all(nested_project.join(".git")).unwrap();
    }

    #[test]
    fn test_scan_ignores_default_patterns() {
        let temp = TempDir::new().unwrap();
        create_scan_test_structure(temp.path());
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec!["node_modules".to_string(), "target".to_string()],
            depth: 10,
            stop_on_marker: true,
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should find myproject
        assert!(paths.iter().any(|p| p.ends_with("myproject")), 
            "Should find myproject, got: {:?}", paths);
        
        // Should NOT find anything in node_modules
        assert!(!paths.iter().any(|p| p.contains("node_modules")),
            "Should ignore node_modules, got: {:?}", paths);
        
        // Should NOT find anything in target
        assert!(!paths.iter().any(|p| p.contains("target")),
            "Should ignore target, got: {:?}", paths);
    }

    #[test]
    fn test_scan_ignores_custom_pattern() {
        let temp = TempDir::new().unwrap();
        create_scan_test_structure(temp.path());
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec!["custom_ignored".to_string()],
            depth: 10,
            stop_on_marker: true,
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should NOT find anything in custom_ignored
        assert!(!paths.iter().any(|p| p.contains("custom_ignored")),
            "Should ignore custom_ignored, got: {:?}", paths);
    }

    #[test]
    fn test_scan_ignores_glob_pattern() {
        let temp = TempDir::new().unwrap();
        create_scan_test_structure(temp.path());
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec!["node_*".to_string(), "targ*".to_string()],
            depth: 10,
            stop_on_marker: true,
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should ignore node_modules (matches node_*)
        assert!(!paths.iter().any(|p| p.contains("node_modules")),
            "Should ignore node_modules via glob, got: {:?}", paths);
        
        // Should ignore target (matches targ*)
        assert!(!paths.iter().any(|p| p.contains("target")),
            "Should ignore target via glob, got: {:?}", paths);
    }

    #[test]
    fn test_scan_without_ignore_finds_all() {
        let temp = TempDir::new().unwrap();
        create_scan_test_structure(temp.path());
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec![], // No ignore patterns
            depth: 10,
            stop_on_marker: false, // Continue scanning into subdirs
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should find project in node_modules when ignore is empty
        assert!(paths.iter().any(|p| p.contains("node_modules")),
            "Without ignore, should find projects in node_modules, got: {:?}", paths);
    }

    // Tests for path-based ignore patterns (patterns with /)

    fn create_mason_test_structure(base: &std::path::Path) {
        use std::fs as stdfs;
        
        // nvim-data/mason/packages - should be ignored with "mason/packages*"
        let packages = base.join("nvim-data/mason/packages/lua-ls");
        stdfs::create_dir_all(&packages).unwrap();
        stdfs::write(packages.join("file.txt"), "").unwrap();
        
        // nvim-data/mason/bin - should NOT be ignored
        let bin = base.join("nvim-data/mason/bin");
        stdfs::create_dir_all(&bin).unwrap();
        stdfs::write(bin.join("script"), "").unwrap();
        
        // nvim-data/lazy - separate dir, might be ignored
        let lazy = base.join("nvim-data/lazy/plugin");
        stdfs::create_dir_all(lazy.join(".git")).unwrap();
        
        // Regular project - should be found
        let project = base.join("project");
        stdfs::create_dir_all(project.join(".git")).unwrap();
    }

    #[test]
    fn test_scan_ignores_path_based_pattern() {
        let temp = TempDir::new().unwrap();
        create_mason_test_structure(temp.path());
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec!["mason/packages*".to_string(), "lazy".to_string()],
            depth: 10,
            stop_on_marker: true,
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should find regular project
        assert!(paths.iter().any(|p| p.ends_with("project")), 
            "Should find project, got: {:?}", paths);
        
        // Should NOT find anything in mason/packages
        assert!(!paths.iter().any(|p| p.contains("mason/packages")),
            "Should ignore mason/packages, got: {:?}", paths);
        
        // Should NOT find anything in lazy (simple pattern)
        assert!(!paths.iter().any(|p| p.contains("lazy")),
            "Should ignore lazy, got: {:?}", paths);
    }

    #[test]
    fn test_scan_path_pattern_only_matches_specific_path() {
        let temp = TempDir::new().unwrap();
        use std::fs as stdfs;
        
        // Create two "packages" dirs - only one should be ignored
        // Pattern "mason/packages" should match relative path "mason/packages/..."
        let mason_packages = temp.path().join("mason/packages/tool");
        stdfs::create_dir_all(mason_packages.join(".git")).unwrap();
        
        let other_packages = temp.path().join("other/packages/tool");
        stdfs::create_dir_all(other_packages.join(".git")).unwrap();
        
        let profile = ResolvedProfile {
            mode: Mode::Dir,
            markers: vec![".git".to_string()],
            ignore: vec!["mason/packages".to_string()], // Should only ignore mason/packages, not other/packages
            depth: 10,
            stop_on_marker: true,
            intermediate_paths: false,
            show_hidden: false,
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should NOT find mason/packages
        assert!(!paths.iter().any(|p| p.contains("mason/packages")),
            "Should ignore mason/packages, got: {:?}", paths);
        
        // SHOULD find other/packages (different path)
        assert!(paths.iter().any(|p| p.contains("other/packages")),
            "Should find other/packages (different path), got: {:?}", paths);
    }

    // Tests for Mode::File with dotfiles

    #[test]
    fn test_file_mode_finds_dotfiles() {
        let temp = TempDir::new().unwrap();
        use std::fs as stdfs;
        
        // Create dotfiles (like $HOME)
        stdfs::write(temp.path().join(".zshrc"), "# zsh config").unwrap();
        stdfs::write(temp.path().join(".bashrc"), "# bash config").unwrap();
        stdfs::write(temp.path().join(".gitconfig"), "[user]").unwrap();
        stdfs::write(temp.path().join("visible.txt"), "visible").unwrap();
        
        let profile = ResolvedProfile {
            mode: Mode::File,
            markers: vec!["*".to_string()], // Match all files
            ignore: vec![],
            depth: 0, // Only current dir
            stop_on_marker: false,
            intermediate_paths: false,
            show_hidden: true,           // Enable to find dotfiles
            traverse_hidden_dirs: false,
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should find dotfiles
        assert!(paths.iter().any(|p| p.ends_with(".zshrc")),
            "Should find .zshrc, got: {:?}", paths);
        assert!(paths.iter().any(|p| p.ends_with(".bashrc")),
            "Should find .bashrc, got: {:?}", paths);
        assert!(paths.iter().any(|p| p.ends_with(".gitconfig")),
            "Should find .gitconfig, got: {:?}", paths);
        
        // Should also find regular files
        assert!(paths.iter().any(|p| p.ends_with("visible.txt")),
            "Should find visible.txt, got: {:?}", paths);
    }

    #[test]
    fn test_file_mode_skips_hidden_directories() {
        let temp = TempDir::new().unwrap();
        use std::fs as stdfs;
        
        // Create hidden directory with files
        let hidden_dir = temp.path().join(".hidden_dir");
        stdfs::create_dir_all(&hidden_dir).unwrap();
        stdfs::write(hidden_dir.join("secret.txt"), "secret").unwrap();
        
        // Create visible directory with files
        let visible_dir = temp.path().join("visible_dir");
        stdfs::create_dir_all(&visible_dir).unwrap();
        stdfs::write(visible_dir.join("file.txt"), "file").unwrap();
        
        let profile = ResolvedProfile {
            mode: Mode::File,
            markers: vec!["*.txt".to_string()],
            ignore: vec![],
            depth: 2,
            stop_on_marker: false,
            intermediate_paths: false,
            show_hidden: false,          // Don't show hidden files
            traverse_hidden_dirs: false, // Don't traverse hidden dirs
        };
        
        let mut output = std::collections::HashMap::new();
        let path_str = temp.path().to_str().unwrap();
        scan_paths(&[path_str], &profile, &mut output).unwrap();
        
        let paths: Vec<String> = output.into_keys().collect();
        
        // Should NOT recurse into hidden directories
        assert!(!paths.iter().any(|p| p.contains(".hidden_dir")),
            "Should skip hidden directories, got: {:?}", paths);
        
        // Should find files in visible directories
        assert!(paths.iter().any(|p| p.contains("visible_dir")),
            "Should find files in visible dirs, got: {:?}", paths);
    }
}
