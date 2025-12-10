//! FZF integration for fuzzy selection.
//!
//! This module provides the interface to the [fzf](https://github.com/junegunn/fzf)
//! fuzzy finder, enabling interactive selection from lists of items.
//!
//! # Mocking Support
//!
//! The `FzfExecutor` trait allows mocking fzf commands in tests.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

use crate::Error;

/// Trait for executing fzf commands.
///
/// This trait allows for mocking fzf in tests.
pub trait FzfExecutor: Send + Sync {
    /// Execute fzf with given arguments and input, return selected item(s).
    fn execute(&self, args: &[&str], input: &str) -> Result<String, Error>;
}

/// Real fzf executor that runs actual fzf commands.
#[derive(Default)]
pub struct RealFzf;

impl FzfExecutor for RealFzf {
    fn execute(&self, args: &[&str], input: &str) -> Result<String, Error> {
        let mut child = Command::new("fzf")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .args(args)
            .spawn()?;

        let mut result = String::new();
        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| Error::UnwrapIOStream("Could not get cmd.stdin"))?;
            stdin.write_all(input.as_bytes())?;
            stdin.flush()?;
            child.wait()?;
        }
        {
            let stdout = child
                .stdout
                .as_mut()
                .ok_or_else(|| Error::UnwrapIOStream("Could not get cmd.stdout"))?;
            stdout.read_to_string(&mut result)?;
        }
        Ok(result)
    }
}

/// Default global fzf executor.
#[allow(dead_code)]
static FZF: RealFzf = RealFzf;

/// Executes fzf with the given arguments and input.
///
/// Spawns an fzf process, writes the input items to its stdin,
/// and returns the user's selection from stdout.
///
/// # Arguments
///
/// * `args` - Iterator of command-line arguments for fzf
/// * `input` - Newline-separated list of items to select from
///
/// # Returns
///
/// * `Ok(String)` - The selected item(s), possibly empty if user cancelled
/// * `Err(Error)` - On I/O errors or if fzf is not installed
///
/// # Note
///
/// This function blocks until the user makes a selection or cancels.
/// The fzf process inherits the current terminal for display.
#[allow(dead_code)]
pub(crate) fn execute_fzf_command<'a>(
    args: impl Iterator<Item = &'a str>,
    input: &str,
) -> Result<String, Error> {
    let args_vec: Vec<&str> = args.collect();
    FZF.execute(&args_vec, input)
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Mock fzf executor for testing.
    pub struct MockFzf {
        /// Value to return when execute is called
        pub return_value: Mutex<String>,
        /// Record of all calls made
        pub calls: Mutex<Vec<(Vec<String>, String)>>,
    }

    impl MockFzf {
        pub fn new(return_value: &str) -> Self {
            MockFzf {
                return_value: Mutex::new(return_value.to_string()),
                calls: Mutex::new(Vec::new()),
            }
        }

        /// Set the next return value
        #[allow(dead_code)]
        pub fn set_return(&self, value: &str) {
            *self.return_value.lock().unwrap() = value.to_string();
        }

        /// Get recorded calls
        pub fn get_calls(&self) -> Vec<(Vec<String>, String)> {
            self.calls.lock().unwrap().clone()
        }
    }

    impl FzfExecutor for MockFzf {
        fn execute(&self, args: &[&str], input: &str) -> Result<String, Error> {
            self.calls.lock().unwrap().push((
                args.iter().map(|s| s.to_string()).collect(),
                input.to_string(),
            ));
            Ok(self.return_value.lock().unwrap().clone())
        }
    }

    #[test]
    fn test_mock_fzf_returns_value() {
        let mock = MockFzf::new("/home/user/project\n");
        let result = mock.execute(&["--layout", "reverse"], "item1\nitem2").unwrap();
        assert_eq!(result, "/home/user/project\n");
    }

    #[test]
    fn test_mock_fzf_records_calls() {
        let mock = MockFzf::new("selected");
        
        mock.execute(&["--header", "Pick:"], "a\nb\nc").unwrap();
        mock.execute(&["-m"], "x\ny").unwrap();
        
        let calls = mock.get_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].0, vec!["--header", "Pick:"]);
        assert_eq!(calls[0].1, "a\nb\nc");
        assert_eq!(calls[1].0, vec!["-m"]);
    }
}
