# PFP (Project Fuzzy Picker) - API Documentation

**Version:** 0.1.0

A powerful command-line tool for managing tmux sessions and windows with fuzzy project selection. PFP scans your filesystem for project directories (identified by markers like `.git`, `Cargo.toml`, etc.) and provides an interactive fuzzy finder interface for quick navigation.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [CLI Commands](#cli-commands)
4. [Configuration](#configuration)
5. [Module Reference](#module-reference)
6. [Examples](#examples)
7. [Error Handling](#error-handling)

---

## Installation

### Prerequisites

- **Rust** (1.70+) - Install via [rustup](https://rustup.rs/)
- **tmux** - Terminal multiplexer
- **fzf** - Fuzzy finder
- **tree** (optional) - For directory previews

### Build from Source

```bash
git clone <repository-url>
cd project-fuzzy-picker
cargo build --release

# The binary will be at ./target/release/pfp
# Optionally, copy it to your PATH:
sudo cp target/release/pfp /usr/local/bin/
```

---

## Quick Start

```bash
# Create a new tmux session from a fuzzy-picked project
pfp new-session

# Create a new window in the current session
pfp new-window

# Switch between active sessions
pfp sessions

# Start predefined sessions from config
pfp start

# Kill current session and switch to another
pfp kill-session
```

---

## CLI Commands

### Global Options

| Option | Short | Description | Default |
|--------|-------|-------------|---------|
| `--config <FILE>` | `-c` | Path to configuration file | `${XDG_CONFIG_HOME}/pfp/config.json` |

### Subcommands

#### `new-session`

Pick a project directory using fuzzy finder and create a new tmux session.

```bash
pfp new-session
pfp -c ~/custom-config.json new-session
```

**Behavior:**
1. Scans configured directories for projects (based on markers)
2. Opens fzf with directory preview (`tree -C`)
3. Creates a new tmux session named after the selected directory
4. Switches to the new session

---

#### `new-window`

Pick a project directory and create a new window in the current tmux session.

```bash
pfp new-window
```

**Behavior:**
1. Scans for project directories
2. Opens fzf for selection
3. Creates a new window in the current session with the selected path as working directory

---

#### `sessions`

List all active tmux sessions and switch to the selected one.

```bash
pfp sessions
```

**Features:**
- Shows all active tmux sessions sorted by session ID
- Live preview of the selected session's current pane content
- Pre-selects the current session in the list

---

#### `kill-session`

Kill the current tmux session and switch to the last/previous session.

```bash
pfp kill-session
```

**Behavior:**
1. Gets the current session name
2. Attempts to switch to the last session (`switch-client -l`)
3. Falls back to previous session (`switch-client -p`) if last is unavailable
4. Kills the original session

---

#### `start`

Start predefined tmux sessions from the configuration file.

```bash
pfp start           # Start sessions (stay detached)
pfp start --attach  # Start and attach to tmux
pfp start -a        # Short form
```

**Options:**

| Option | Short | Description |
|--------|-------|-------------|
| `--attach` | `-a` | Attach to tmux after starting sessions |

**Behavior:**
1. Shows configured sessions in fzf (multi-select enabled)
2. Creates selected sessions with their predefined windows
3. Skips sessions that already exist
4. Optionally attaches to tmux

---

#### `print-config`

Print the parsed configuration to stdout for debugging.

```bash
pfp print-config
pfp -c ~/custom-config.json print-config
```

---

## Configuration

PFP uses a JSON configuration file (with support for comments via JSONC).

### Default Location

```
${XDG_CONFIG_HOME}/pfp/config.json
```

If `XDG_CONFIG_HOME` is not set, this typically resolves to `~/.config/pfp/config.json`.

### Configuration Schema

```jsonc
{
  // Predefined tmux sessions
  "sessions": [
    {
      "name": "work",
      "windows": [
        "$HOME/projects/frontend",
        "$HOME/projects/backend",
        "$HOME/projects/docs"
      ]
    },
    {
      "name": "personal",
      "windows": [
        "$HOME/personal/blog",
        "$HOME/personal/dotfiles"
      ]
    }
  ],

  // Global project markers (files/dirs that indicate a project root)
  "markers": {
    "exact": [".git", "Cargo.toml", "go.mod", "package.json"],
    "pattern": ["^Makefile$", ".*\\.sln$"],
    "traverse_hidden": true,
    "chain_root_markers": true
  },

  // Global ignore patterns
  "ignore": {
    "exact": ["node_modules", "target", "venv", ".cache"],
    "pattern": ["^\\.", ".*_test$"],
    "chain_root_ignore": true
  },

  // Directory scanning configuration
  "include": [
    {
      "paths": ["$HOME/projects", "$HOME/work"],
      "mode": "dir",
      "depth": 3,
      "markers": {
        "exact": [".git"],
        "pattern": [],
        "traverse_hidden": false,
        "chain_root_markers": true
      },
      "ignore": {
        "exact": ["vendor"],
        "pattern": [],
        "chain_root_ignore": true
      },
      "include_intermediate_paths": true,
      "yield_on_marker": true
    },
    {
      "paths": ["$HOME/scripts"],
      "mode": "file",
      "depth": 2
    }
  ]
}
```

### Configuration Fields

#### Top-Level Fields

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `sessions` | `Session[]` | No | `[]` | Predefined tmux sessions |
| `markers` | `Markers` | No | See below | Global project markers |
| `ignore` | `Ignore` | No | See below | Global ignore patterns |
| `include` | `IncludeEntry[]` | **Yes** | N/A | Directories to scan for projects |

---

#### `Session` Object

Defines a predefined tmux session configuration.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | `string` | Yes | Session name (displayed in tmux) |
| `windows` | `string[]` | Yes | List of paths for windows in this session |

**Example:**
```json
{
  "name": "development",
  "windows": [
    "$HOME/projects/api",
    "$HOME/projects/frontend",
    "$HOME/projects/shared"
  ]
}
```

---

#### `Markers` Object

Defines patterns that identify project root directories.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `exact` | `string[]` | `[".git", "Cargo.toml", "go.mod"]` | Exact file/directory names |
| `pattern` | `string[]` | `[]` | Regex patterns for matching |
| `traverse_hidden` | `boolean` | `true` | Whether to descend into hidden directories |
| `chain_root_markers` | `boolean` | `true` | Merge with global markers |

---

#### `Ignore` Object

Defines patterns for directories/files to skip during scanning.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `exact` | `string[]` | See below* | Exact names to ignore |
| `pattern` | `string[]` | `[]` | Regex patterns for ignoring |
| `chain_root_ignore` | `boolean` | `true` | Merge with global ignore list |

*Default exact ignores: `.DS_Store`, `node_modules`, `venv`, `bin`, `target`, `debug`, `src`, `test`, `tests`, `lib`, `docs`, `pkg`

---

#### `IncludeEntry` Object

Defines a set of directories to scan for projects.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `paths` | `string[]` | **Required** | Root paths to scan |
| `mode` | `"dir"` \| `"file"` | `"dir"` | Scanning mode |
| `depth` | `number` | `255` | Maximum recursion depth |
| `markers` | `Markers` | `{}` | Entry-specific markers |
| `ignore` | `Ignore` | `{}` | Entry-specific ignore patterns |
| `include_intermediate_paths` | `boolean` | `true` | Include parent directories in results |
| `yield_on_marker` | `boolean` | `true` | Stop descending when marker found |

**Mode Descriptions:**
- `"dir"` - Scan for directories containing markers (project roots)
- `"file"` - Scan for individual files (useful for scripts)

---

### Environment Variable Expansion

All paths in the configuration support environment variable expansion:

```json
{
  "include": [
    {
      "paths": [
        "$HOME/projects",           // Simple format
        "${HOME}/projects",         // Braced format
        "${XDG_CONFIG_HOME}/nvim"   // Any env var
      ]
    }
  ]
}
```

---

### Minimal Configuration Example

```json
{
  "include": [
    {
      "paths": ["$HOME"]
    }
  ]
}
```

This scans from `$HOME`, looking for default markers (`.git`, `Cargo.toml`, `go.mod`).

---

## Module Reference

### `main` Module

Entry point and error type definitions.

#### Error Enum

```rust
enum Error {
    Config(ConfigError),      // Configuration parsing errors
    CmdArg(String),           // Command-line argument errors
    Descend(anyhow::Error),   // Directory traversal errors
    IO(std::io::Error),       // I/O errors
    UnwrapIOStream(&'static str), // Stream unwrapping errors
    Regex(regex::Error),      // Regex compilation errors
    EnvVar(VarError, String), // Environment variable errors
    ParseUTF8(FromUtf8Error), // UTF-8 parsing errors
    EmptyPick(),              // User cancelled selection
}
```

#### `measure` Function

```rust
pub fn measure<F>(name: &str, f: F)
where
    F: FnMut()
```

Utility function for measuring execution time of closures. Logs timing via the `log` crate at `info` level.

**Parameters:**
- `name` - Descriptive name for the operation
- `f` - Closure to execute and measure

**Example:**
```rust
measure("directory_scan", || {
    scan_directories(&config);
});
// Logs: "Time elapsed for directory_scan is: 150ms"
```

---

### `cli` Module

Command-line interface implementation using `clap`.

#### `cli` Function

```rust
pub(crate) fn cli() -> Result<(), Error>
```

Main CLI entry point. Parses arguments and dispatches to appropriate subcommand handlers.

**Returns:** `Ok(())` on success, `Err(Error)` on failure.

---

### `config` Module

Configuration parsing and data structures.

#### `ConfigError` Enum

```rust
pub enum ConfigError {
    Parse(serde_jsonc::Error), // JSON parsing error
    Read(std::io::Error),      // File reading error
}
```

#### `Config` Struct

```rust
pub struct Config<'a> {
    pub sessions: Vec<Session<'a>>,
    pub markers: Markers<'a>,
    pub ignore: Ignore<'a>,
    pub include: Vec<IncludeEntry<'a>>,
}
```

Main configuration structure containing all settings.

#### `Session` Struct

```rust
pub struct Session<'a> {
    pub name: &'a str,
    pub windows: Vec<&'a str>,
}
```

Represents a predefined tmux session.

**Methods:**
- `to_string(&self) -> String` - Formats session for display

#### `IncludeEntry` Struct

```rust
pub struct IncludeEntry<'a> {
    pub paths: Vec<&'a str>,
    pub mode: Mode,
    pub markers: Markers<'a>,
    pub ignore: Ignore<'a>,
    pub include_intermediate_paths: bool,
    pub yield_on_marker: bool,
    pub depth: u8,
}
```

Configuration for a set of paths to scan.

#### `Mode` Enum

```rust
pub enum Mode {
    Dir,  // Scan for directories (default)
    File, // Scan for files
}
```

#### `Markers` Struct

```rust
pub struct Markers<'a> {
    pub exact: Vec<&'a str>,
    pub pattern: Vec<&'a str>,
    pub traverse_hidden: bool,
    pub chain_root_markers: bool,
}
```

#### `Ignore` Struct

```rust
pub struct Ignore<'a> {
    pub exact: Vec<&'a str>,
    pub pattern: Vec<&'a str>,
    pub chain_root_ignore: bool,
}
```

#### `read_config` Function

```rust
pub fn read_config(path: &str) -> Result<Config, ConfigError>
```

Reads and parses a configuration file.

**Parameters:**
- `path` - Path to the configuration file

**Returns:** Parsed `Config` or `ConfigError`

---

### `fs` Module

Filesystem utilities for path manipulation and directory scanning.

#### `expand` Function

```rust
pub fn expand(path: &str) -> Result<String, Error>
```

Expands environment variables in a path string.

**Parameters:**
- `path` - Path string potentially containing env vars (e.g., `$HOME/projects`)

**Returns:** Expanded path string or error if variable not found

**Supported formats:**
- `$VAR` - Simple format
- `${VAR}` - Braced format

**Example:**
```rust
let path = expand("$HOME/projects")?;
// Returns: "/home/username/projects"
```

---

#### `trim_window_name` Function

```rust
pub fn trim_window_name(path: &str) -> Result<String, anyhow::Error>
```

Creates a short window name from a path by retaining the last two path components.

**Parameters:**
- `path` - Full directory path

**Returns:** Shortened name suitable for tmux window

**Example:**
```rust
let name = trim_window_name("/home/user/projects/myapp")?;
// Returns: "proj/myapp" (first 4 chars of parent + full last component)
```

---

#### `trim_session_name` Function

```rust
pub fn trim_session_name(name: &String) -> String
```

Removes dots from a session name (tmux displays dots as underscores).

**Parameters:**
- `name` - Original session name

**Returns:** Session name with dots removed

**Example:**
```rust
let name = trim_session_name(&"my.app".to_string());
// Returns: "myapp"
```

---

#### `get_included_paths_list` Function

```rust
pub fn get_included_paths_list(
    path: &str,
    depth: u8,
    output: &mut HashMap<String, ()>,
    include_entry: &IncludeEntry,
    config: &Config,
) -> Result<bool, Error>
```

Recursively scans directories and populates `output` with matching paths.

**Parameters:**
- `path` - Starting directory path
- `depth` - Current recursion depth
- `output` - HashMap to populate with results
- `include_entry` - Configuration for this scan
- `config` - Global configuration

**Returns:** `true` if this path or any children yielded matches

**Algorithm:**
1. Read directory contents
2. Check for marker files/directories
3. If marker found and `yield_on_marker` is true, add to output and return
4. Otherwise, recurse into non-ignored subdirectories
5. Include intermediate paths if configured

---

#### `is_dir` Function

```rust
pub fn is_dir(path: &str, ft: &FileType) -> Result<bool, std::io::Error>
```

Checks if a path is a directory, following symlinks.

---

#### `is_file` Function

```rust
pub fn is_file(path: &str, ft: &FileType) -> Result<bool, std::io::Error>
```

Checks if a path is a file, following symlinks.

---

#### `path_is_file` Function

```rust
pub fn path_is_file(path: &str) -> bool
```

Convenience function to check if a path is a file using metadata.

---

### `fzf` Module

FZF (fuzzy finder) integration.

#### `execute_fzf_command` Function

```rust
pub fn execute_fzf_command<'a>(
    args: impl Iterator<Item = &'a str>,
    input: &str,
) -> Result<String, Error>
```

Executes fzf with the given arguments and input.

**Parameters:**
- `args` - Command-line arguments to pass to fzf
- `input` - Newline-separated list of items to select from

**Returns:** Selected item(s) or error

**Example:**
```rust
let result = execute_fzf_command(
    ["--layout", "reverse", "--header", "Select:"].iter().cloned(),
    "option1\noption2\noption3"
)?;
```

---

### `selectors` Module

High-level selection functions.

#### `select_from_list` Function

```rust
pub fn select_from_list(
    list: &str,
    header: &'static str,
    args: &[&str],
) -> Result<String, Error>
```

Presents a list to the user for selection via fzf.

**Parameters:**
- `list` - Newline-separated items
- `header` - Header text displayed in fzf
- `args` - Additional fzf arguments

**Returns:** Selected item or `EmptyPick` error if cancelled

---

#### `pick_project` Function

```rust
pub fn pick_project(config: &Config, header: &'static str) -> Result<String, Error>
```

Scans for projects based on config and presents them for selection.

**Parameters:**
- `config` - Application configuration
- `header` - Header text for fzf

**Returns:** Selected project path

**Features:**
- Automatically expands environment variables in paths
- Shows tree preview of directories
- Returns trimmed path string

---

### `tmux` Module

Tmux command execution utilities.

#### `execute_tmux_command` Function

```rust
pub fn execute_tmux_command(cmd: &str) -> std::io::Result<process::Output>
```

Executes a tmux command with piped stdin.

**Parameters:**
- `cmd` - Full tmux command (e.g., `"tmux list-sessions"`)

**Returns:** Command output

---

#### `execute_tmux_command_with_stdin` Function

```rust
pub fn execute_tmux_command_with_stdin(
    cmd: &str,
    stdin: process::Stdio,
) -> std::io::Result<process::Output>
```

Executes a tmux command with custom stdin handling.

**Parameters:**
- `cmd` - Full tmux command
- `stdin` - Stdin handling mode (piped or inherit)

---

#### `execute_tmux_window_command` Function

```rust
pub fn execute_tmux_window_command(
    cmd: &str,
    target: &str,
) -> Result<process::Output, anyhow::Error>
```

Executes tmux new-window/new-session with special handling for files.

**Parameters:**
- `cmd` - Tmux command (with `-c` flag at end for working directory)
- `target` - Target path

**Behavior:**
- If target is a file, opens it in `$EDITOR` instead of creating a shell window
- Otherwise, creates a normal window with the specified working directory

---

## Examples

### Example 1: Basic Setup

Create a minimal configuration to scan your home directory:

```bash
mkdir -p ~/.config/pfp
cat > ~/.config/pfp/config.json << 'EOF'
{
  "include": [
    {
      "paths": ["$HOME/projects"],
      "depth": 3
    }
  ]
}
EOF
```

Now use pfp:

```bash
pfp new-session  # Pick a project, create new session
```

---

### Example 2: Multiple Project Directories

```json
{
  "markers": {
    "exact": [".git", "Cargo.toml", "go.mod", "package.json", "pyproject.toml"],
    "pattern": []
  },
  "ignore": {
    "exact": ["node_modules", "target", "venv", ".venv", "__pycache__"],
    "pattern": ["^\\."]
  },
  "include": [
    {
      "paths": ["$HOME/work"],
      "depth": 4,
      "yield_on_marker": true
    },
    {
      "paths": ["$HOME/personal"],
      "depth": 3
    },
    {
      "paths": ["$HOME/scripts"],
      "mode": "file",
      "depth": 2
    }
  ]
}
```

---

### Example 3: Predefined Sessions

Configure sessions that you use regularly:

```json
{
  "sessions": [
    {
      "name": "main",
      "windows": [
        "$HOME/projects/api",
        "$HOME/projects/web",
        "$HOME/projects/mobile"
      ]
    },
    {
      "name": "devops",
      "windows": [
        "$HOME/infrastructure/terraform",
        "$HOME/infrastructure/kubernetes",
        "$HOME/infrastructure/ansible"
      ]
    }
  ],
  "include": [
    {
      "paths": ["$HOME/projects", "$HOME/infrastructure"],
      "depth": 3
    }
  ]
}
```

Start your work sessions:

```bash
pfp start -a  # Select sessions to start and attach
```

---

### Example 4: Keybindings (tmux.conf)

Add these to your `~/.tmux.conf` for quick access:

```tmux
# Create new session with project picker
bind-key C-n run-shell "pfp new-session"

# Create new window with project picker
bind-key C-w run-shell "pfp new-window"

# Switch between sessions
bind-key C-s run-shell "pfp sessions"

# Kill current session
bind-key C-k run-shell "pfp kill-session"
```

---

### Example 5: Shell Alias

Add to your `.bashrc` or `.zshrc`:

```bash
# Quick project navigation
alias pp="pfp new-session"
alias pw="pfp new-window"
alias ps="pfp sessions"

# Start work sessions
alias work="pfp start -a"
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Config error: Read config` | Config file not found | Create config at `~/.config/pfp/config.json` or use `-c` flag |
| `Env var error` | Environment variable not set | Ensure referenced env vars exist (e.g., `$HOME`) |
| `Empty pick!` | User cancelled fzf selection | Expected when pressing Escape - not a bug |
| `IO error` | tmux or fzf not found | Install tmux and fzf |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `65` | Data error (config parse, invalid selection, etc.) |

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | Command-line argument parsing |
| `serde` | Serialization/deserialization |
| `serde_jsonc` | JSON with comments support |
| `regex` | Pattern matching |
| `anyhow` | Error handling |
| `thiserror` | Error type derivation |
| `log` | Logging facade |

---

## License

See LICENSE file in the repository.
