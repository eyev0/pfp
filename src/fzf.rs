//! FZF integration for fuzzy selection.
//!
//! This module provides the interface to the [fzf](https://github.com/junegunn/fzf)
//! fuzzy finder, enabling interactive selection from lists of items.

use std::{
    io::{Read, Write},
    process::{Command, Stdio},
};

use crate::Error;

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
/// # Example
///
/// ```ignore
/// let result = execute_fzf_command(
///     ["--layout", "reverse", "--header", "Select:"].iter().cloned(),
///     "option1\noption2\noption3"
/// )?;
/// println!("User selected: {}", result);
/// ```
///
/// # Note
///
/// This function blocks until the user makes a selection or cancels.
/// The fzf process inherits the current terminal for display.
pub(crate) fn execute_fzf_command<'a>(
    args: impl Iterator<Item = &'a str>,
    input: &str,
) -> Result<String, crate::Error> {
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
