//! Application context for dependency injection.
//!
//! This module provides `AppContext` which holds all external dependencies
//! (fzf, tmux) allowing them to be mocked in tests.

use std::process::{Output, Stdio};
use std::io;

use crate::fzf::{FzfExecutor, RealFzf};
use crate::tmux::{TmuxExecutor, RealTmux};
use crate::Error;

/// Application context holding all external dependencies.
///
/// Use `AppContext::default()` for production code with real implementations.
/// Create custom contexts with mock implementations for testing.
pub struct AppContext {
    fzf: Box<dyn FzfExecutor>,
    tmux: Box<dyn TmuxExecutor>,
}

impl Default for AppContext {
    fn default() -> Self {
        Self {
            fzf: Box::new(RealFzf),
            tmux: Box::new(RealTmux),
        }
    }
}

impl AppContext {
    /// Create a new context with custom implementations.
    #[allow(dead_code)]
    pub fn new(fzf: Box<dyn FzfExecutor>, tmux: Box<dyn TmuxExecutor>) -> Self {
        Self { fzf, tmux }
    }

    /// Execute fzf with given arguments and input.
    pub fn fzf_execute(&self, args: &[&str], input: &str) -> Result<String, Error> {
        self.fzf.execute(args, input)
    }

    /// Execute a tmux command (from full command string like "tmux list-sessions").
    pub fn tmux_execute(&self, cmd: &str) -> io::Result<Output> {
        let args: Vec<&str> = cmd.split(' ').skip(1).collect();
        self.tmux.execute(&args)
    }

    /// Execute a tmux command with custom stdin handling.
    pub fn tmux_execute_with_stdin(&self, cmd: &str, stdin: Stdio) -> io::Result<Output> {
        let args: Vec<&str> = cmd.split(' ').skip(1).collect();
        self.tmux.execute_with_stdin(&args, stdin)
    }

    /// Execute tmux new-window/new-session with smart file handling.
    pub fn tmux_window_command(&self, cmd: &str, target: &str) -> Result<Output, anyhow::Error> {
        use crate::fs::{expand, path_is_file};
        
        if path_is_file(target) {
            let split = cmd.split('/');
            let modified_cmd = format!(
                "{} {} {}",
                split
                    .clone()
                    .take(split.count() - 1)
                    .collect::<Vec<&str>>()
                    .join("/"),
                expand("$EDITOR")?,
                target
            );
            Ok(self.tmux_execute_with_stdin(&modified_cmd, Stdio::piped())?)
        } else {
            Ok(self.tmux_execute_with_stdin(cmd, Stdio::piped())?)
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::fzf::tests::MockFzf;
    use crate::tmux::tests::MockTmux;

    /// Create a test context with mock fzf and tmux.
    pub fn test_context(fzf_return: &str, tmux_stdout: &str) -> AppContext {
        AppContext::new(
            Box::new(MockFzf::new(fzf_return)),
            Box::new(MockTmux::with_output(tmux_stdout)),
        )
    }

    #[test]
    fn test_context_fzf() {
        let ctx = test_context("/tmp/project\n", "");
        let result = ctx.fzf_execute(&["--layout", "reverse"], "a\nb\nc").unwrap();
        assert_eq!(result, "/tmp/project\n");
    }

    #[test]
    fn test_context_tmux() {
        let ctx = test_context("", "session1\nsession2\n");
        let output = ctx.tmux_execute("tmux list-sessions").unwrap();
        assert_eq!(String::from_utf8(output.stdout).unwrap(), "session1\nsession2\n");
    }
}

