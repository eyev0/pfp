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
//!
//! # Mocking Support
//!
//! The `TmuxExecutor` trait allows mocking tmux commands in tests.

use std::io;
use std::process::{self, Output};

use crate::fs::{expand, path_is_file};

/// Trait for executing tmux commands.
/// 
/// This trait allows for mocking tmux in tests.
pub trait TmuxExecutor: Send + Sync {
    /// Execute a tmux command with the given arguments.
    fn execute(&self, args: &[&str]) -> io::Result<Output>;
    
    /// Execute a tmux command with custom stdin handling.
    fn execute_with_stdin(&self, args: &[&str], stdin: process::Stdio) -> io::Result<Output>;
}

/// Real tmux executor that runs actual tmux commands.
pub struct RealTmux;

impl TmuxExecutor for RealTmux {
    fn execute(&self, args: &[&str]) -> io::Result<Output> {
        process::Command::new("tmux")
            .stdin(process::Stdio::piped())
            .args(args)
            .output()
    }

    fn execute_with_stdin(&self, args: &[&str], stdin: process::Stdio) -> io::Result<Output> {
        process::Command::new("tmux")
            .stdin(stdin)
            .args(args)
            .output()
    }
}

/// Default global tmux executor.
#[allow(dead_code)]
static TMUX: RealTmux = RealTmux;

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
#[allow(dead_code)]
pub(crate) fn execute_tmux_command_with_stdin(
    cmd: &str,
    stdin: process::Stdio,
) -> io::Result<Output> {
    let args: Vec<&str> = cmd.split(' ').skip(1).collect();
    TMUX.execute_with_stdin(&args, stdin)
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
#[allow(dead_code)]
pub(crate) fn execute_tmux_command(cmd: &str) -> io::Result<Output> {
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
#[allow(dead_code)]
pub(crate) fn execute_tmux_window_command(cmd: &str, target: &str) -> Result<Output, anyhow::Error> {
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

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock tmux executor for testing.
    pub struct MockTmux {
        pub calls: Mutex<Vec<Vec<String>>>,
        pub return_output: Output,
    }

    impl MockTmux {
        pub fn new() -> Self {
            MockTmux {
                calls: Mutex::new(Vec::new()),
                return_output: Output {
                    status: std::process::ExitStatus::default(),
                    stdout: Vec::new(),
                    stderr: Vec::new(),
                },
            }
        }

        pub fn with_output(stdout: &str) -> Self {
            MockTmux {
                calls: Mutex::new(Vec::new()),
                return_output: Output {
                    status: std::process::ExitStatus::default(),
                    stdout: stdout.as_bytes().to_vec(),
                    stderr: Vec::new(),
                },
            }
        }

        pub fn get_calls(&self) -> Vec<Vec<String>> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl TmuxExecutor for MockTmux {
        fn execute(&self, args: &[&str]) -> io::Result<Output> {
            self.calls.lock().unwrap().push(args.iter().map(|s| s.to_string()).collect());
            Ok(self.return_output.clone())
        }

        fn execute_with_stdin(&self, args: &[&str], _stdin: process::Stdio) -> io::Result<Output> {
            self.calls.lock().unwrap().push(args.iter().map(|s| s.to_string()).collect());
            Ok(self.return_output.clone())
        }
    }

    #[test]
    fn test_mock_tmux_records_calls() {
        let mock = MockTmux::new();
        
        mock.execute(&["list-sessions", "-F", "#S"]).unwrap();
        mock.execute(&["new-window", "-n", "test"]).unwrap();
        
        let calls = mock.get_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], vec!["list-sessions", "-F", "#S"]);
        assert_eq!(calls[1], vec!["new-window", "-n", "test"]);
    }

    #[test]
    fn test_mock_tmux_returns_output() {
        let mock = MockTmux::with_output("session1\nsession2\n");
        
        let output = mock.execute(&["list-sessions"]).unwrap();
        
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "session1\nsession2\n");
    }
}
