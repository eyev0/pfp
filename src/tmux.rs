//! Tmux command execution utilities.
//!
//! This module provides functions for executing tmux commands,
//! with special handling for window creation and file targets.
//!
//! # Functions
//!
//! - [`execute_tmux_command`] - Execute a tmux command with piped stdin
//! - [`execute_tmux_command_with_stdin`] - Execute with custom stdin handling
//! - [`execute_tmux_window_command`] - Smart window creation with file support

use std::process;

use crate::fs::{expand, path_is_file};

/// Executes a tmux command with custom stdin handling.
///
/// This is the low-level function for running tmux commands.
/// The command string should start with "tmux" - the first word is stripped
/// and the rest are passed as arguments to the tmux binary.
///
/// # Arguments
///
/// * `cmd` - The full tmux command (e.g., "tmux list-sessions")
/// * `stdin` - How to handle stdin (piped or inherited)
///
/// # Returns
///
/// The command output including stdout, stderr, and exit status.
///
/// # Example
///
/// ```ignore
/// // Piped stdin (non-interactive)
/// let output = execute_tmux_command_with_stdin(
///     "tmux list-sessions -F '#S'",
///     process::Stdio::piped()
/// )?;
///
/// // Inherited stdin (interactive)
/// let output = execute_tmux_command_with_stdin(
///     "tmux attach",
///     process::Stdio::inherit()
/// )?;
/// ```
pub(crate) fn execute_tmux_command_with_stdin(
    cmd: &str,
    stdin: process::Stdio,
) -> std::io::Result<process::Output> {
    let args = cmd.split(' ').skip(1);
    process::Command::new("tmux").stdin(stdin).args(args).output()
}

/// Executes a tmux command with piped stdin.
///
/// Convenience wrapper around [`execute_tmux_command_with_stdin`] that uses
/// piped stdin (non-interactive mode).
///
/// # Arguments
///
/// * `cmd` - The full tmux command string
///
/// # Returns
///
/// The command output.
///
/// # Example
///
/// ```ignore
/// let output = execute_tmux_command("tmux display-message -p '#S'")?;
/// let session_name = String::from_utf8(output.stdout)?;
/// ```
pub(crate) fn execute_tmux_command(cmd: &str) -> std::io::Result<process::Output> {
    execute_tmux_command_with_stdin(cmd, process::Stdio::piped())
}

/// Executes tmux new-window/new-session with smart file handling.
///
/// If the target path is a file, this function modifies the command to:
/// 1. Use the file's parent directory as the working directory
/// 2. Open the file in `$EDITOR`
///
/// If the target is a directory, the command is executed as-is.
///
/// # Arguments
///
/// * `cmd` - The tmux command. **IMPORTANT**: The `-c` flag (working directory)
///   must be at the end of the command for file path trimming to work correctly.
/// * `target` - The target path (file or directory)
///
/// # Returns
///
/// The command output.
///
/// # Example
///
/// ```ignore
/// // For a directory target:
/// execute_tmux_window_command(
///     "tmux new-window -n myproject -c /home/user/projects/myproject",
///     "/home/user/projects/myproject"
/// )?;
///
/// // For a file target, the command is modified to open in $EDITOR:
/// execute_tmux_window_command(
///     "tmux new-window -n script -c /home/user/scripts/deploy.sh",
///     "/home/user/scripts/deploy.sh"
/// )?;
/// // Actually runs: tmux new-window -n script -c /home/user/scripts $EDITOR /home/user/scripts/deploy.sh
/// ```
pub(crate) fn execute_tmux_window_command(cmd: &str, target: &str) -> Result<process::Output, anyhow::Error> {
    if path_is_file(target) {
        let split = cmd.split('/');
        Ok(execute_tmux_command_with_stdin(
            &format!(
                "{} {} {}",
                split
                    .clone()
                    .take(split.count() - 1)
                    .collect::<Vec<&str>>()
                    .join("/"),
                expand("$EDITOR")?,
                target
            ),
            process::Stdio::piped(),
        )?)
    } else {
        Ok(execute_tmux_command_with_stdin(cmd, process::Stdio::piped())?)
    }
}
