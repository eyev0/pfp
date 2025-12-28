# PFP (Project Fuzzy Picker) - API Documentation

**Version:** 0.2.0

A powerful command-line tool for managing tmux sessions and windows with fuzzy project selection. PFP scans your filesystem for project directories (identified by markers like `.git`, `Cargo.toml`, etc.) and provides an interactive fuzzy finder interface for quick navigation.

---

## Table of Contents

1. [Installation](#installation)
2. [Quick Start](#quick-start)
3. [CLI Commands](#cli-commands)
4. [Configuration](#configuration)
5. [Profiles](#profiles)
6. [Module Reference](#module-reference)
7. [Examples](#examples)
8. [Error Handling](#error-handling)

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
cd pfp
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

#### `new-window`

Pick a project directory and create a new window in the current tmux session.

```bash
pfp new-window
```

#### `sessions`

List all active tmux sessions and switch to the selected one.

```bash
pfp sessions
```

#### `kill-session`

Kill the current tmux session and switch to the last/previous session.

```bash
pfp kill-session
```

#### `start`

Start predefined tmux sessions from the configuration file.

```bash
pfp start           # Start sessions (stay detached)
pfp start --attach  # Start and attach to tmux
pfp start -a        # Short form
```

#### `print-config`

Print the parsed configuration to stdout for debugging.

```bash
pfp print-config
```

---

## Configuration

PFP uses a JSON configuration file with support for comments (JSONC).

### Default Location

```
${XDG_CONFIG_HOME}/pfp/config.json
```

If `XDG_CONFIG_HOME` is not set, this typically resolves to `~/.config/pfp/config.json`.

### Configuration Schema

```jsonc
{
  // Custom profiles (override or extend built-in profiles)
  "profiles": {
    "projects": {
      "markers": [".git", "Cargo.toml", "go.mod", "pom.xml"]  // Override markers only
    },
    "my-custom": {
      "mode": "file",
      "markers": ["*.sh", "*.py"],
      "depth": 3
    }
  },

  // Directories to scan
  "include": [
    // Simple string = path with "projects" profile
    "$HOME/dev",
    
    // Object for custom settings
    { "paths": ["$HOME/Downloads", "$HOME/Documents"], "profile": "browse" },
    { "paths": ["$HOME/.config"], "profile": "files", "depth": 2 }
  ],

  // Predefined tmux sessions
  "sessions": [
    {
      "name": "work",
      "windows": ["$HOME/projects/api", "$HOME/projects/web"]
    }
  ]
}
```

### Environment Variable Expansion

All paths support environment variable expansion:

```json
{
  "include": [
    "$HOME/projects",
    "${XDG_CONFIG_HOME}/nvim"
  ]
}
```

---

## Profiles

PFP uses **profiles** to define scanning behavior. Built-in profiles are embedded in the application; see [`defaults.json`](../defaults.json) for reference values.

### Built-in Profiles

| Profile | Mode | Markers | Ignore | Depth | stop_on_marker | intermediate_paths |
|---------|------|---------|--------|-------|-----------------|-------------------|
| `projects` | dir | `.git`, `Cargo.toml`, `go.mod`, `package.json` | `node_modules`, `target`, `venv` | 255 | true | true |
| `browse` | dir | `*` (all) | - | 2 | false | true |
| `files` | file | `*` (all) | - | 1 | - | false |

### Profile Fields

| Field | Type | Description |
|-------|------|-------------|
| `base` | `string` | Name of another profile to inherit from |
| `mode` | `"dir"` \| `"file"` | Scan for directories or files |
| `markers` | `string[]` | Patterns to match (glob supported: `*.sh`, `tree-*`) |
| `ignore` | `string[]` | Patterns to skip (glob supported) |
| `depth` | `number` | Maximum recursion depth |
| `stop_on_marker` | `boolean` | Stop descending when marker found |
| `intermediate_paths` | `boolean` | Include parent directories in results |

### Customizing Profiles

To customize a built-in profile, add it to your config with only the fields you want to change:

```jsonc
{
  "profiles": {
    // Partial override: only changes markers, keeps other defaults
    "projects": {
      "markers": [".git", "Cargo.toml", "pom.xml", "build.gradle"]
    }
  }
}
```

The final profile is merged: `defaults.json` ← `user profile` ← `include entry overrides`

### Profile Inheritance

Use the `base` field to create a profile that inherits from another profile:

```jsonc
{
  "profiles": {
    // Create a custom profile based on "projects"
    "my_projects": {
      "base": "projects",
      "depth": 3,
      "markers": [".git", "pom.xml"]  // Override markers
    },

    // Inheritance chains are supported
    "deep_projects": {
      "base": "my_projects",
      "depth": 10  // Override depth, keep markers from my_projects
    }
  }
}
```

All fields from the base profile are copied, then the child profile's explicit fields override them. This is useful for creating variations of existing profiles without duplicating all settings.

### Include Entry Format

Include entries can be:

1. **Simple string** - uses `projects` profile:
   ```json
   "$HOME/dev"
   ```

2. **Object** - with profile and optional overrides:
   ```json
   {
     "paths": ["$HOME/Downloads"],
     "profile": "browse",
     "depth": 3  // Override depth for this entry only
   }
   ```

---

## Module Reference

### `config` Module

#### Key Types

```rust
// Profile with optional fields for merging
pub struct Profile {
    pub base: Option<String>,  // Inherit from another profile
    pub mode: Option<Mode>,
    pub markers: Option<Vec<String>>,
    pub ignore: Option<Vec<String>>,
    pub depth: Option<u8>,
    pub stop_on_marker: Option<bool>,
    pub intermediate_paths: Option<bool>,
}

// Include entry - string or detailed object
pub enum IncludeEntry {
    Simple(String),
    Detailed(IncludeEntryDetailed),
}

// Main configuration
pub struct Config {
    pub profiles: HashMap<String, Profile>,
    pub include: Vec<IncludeEntry>,
    pub sessions: Vec<Session>,
}
```

### `fs` Module

#### `expand` Function

Expands environment variables in paths (`$VAR` or `${VAR}` format).

#### `scan_paths` Function

```rust
pub fn scan_paths(
    paths: &[&str],
    profile: &ResolvedProfile,
    output: &mut HashMap<String, ()>,
) -> Result<(), Error>
```

Scans directories according to profile settings.

#### Pattern Matching

Markers and ignore patterns support:
- **Exact match**: `".git"`, `"node_modules"`
- **Glob patterns**: `"*.sh"`, `"tree-sitter-*"`, `"*.py"`
- **Match all**: `"*"`

---

## Examples

### Example 1: Minimal Config

```json
{
  "include": ["$HOME/dev"]
}
```

### Example 2: Multiple Directories with Profiles

```json
{
  "include": [
    "$HOME/dev",
    { "paths": ["$HOME/Downloads", "$HOME/Documents"], "profile": "browse" },
    { "paths": ["$HOME/.config", "$HOME/.cache"], "profile": "files", "depth": 1 }
  ]
}
```

### Example 3: Custom Profile

```json
{
  "profiles": {
    "scripts": {
      "mode": "file",
      "markers": ["*.sh", "*.py", "*.rb"],
      "depth": 2
    }
  },
  "include": [
    "$HOME/dev",
    { "paths": ["$HOME/bin", "$HOME/scripts"], "profile": "scripts" }
  ]
}
```

### Example 4: Predefined Sessions

```json
{
  "sessions": [
    {
      "name": "work",
      "windows": ["$HOME/projects/api", "$HOME/projects/web"]
    },
    {
      "name": "dotfiles",
      "windows": ["$HOME/.config/nvim", "$HOME/.config/pfp"]
    }
  ],
  "include": ["$HOME/dev"]
}
```

### Example 5: tmux Keybindings

Add to `~/.tmux.conf`:

```tmux
bind-key C-n run-shell "pfp new-session"
bind-key C-w run-shell "pfp new-window"
bind-key C-s run-shell "pfp sessions"
bind-key C-k run-shell "pfp kill-session"
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `Config error: Read config` | Config file not found | Create config or use `-c` flag |
| `Env var error` | Environment variable not set | Ensure `$HOME` etc. exist |
| `Empty pick!` | User cancelled selection | Expected behavior |
| `IO error` | tmux/fzf not found | Install dependencies |

### Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `65` | Data error |

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `clap` | CLI argument parsing |
| `serde` | Serialization |
| `serde_jsonc` | JSON with comments |
| `glob` | Pattern matching |
| `anyhow` | Error handling |
| `thiserror` | Error type derivation |
| `log` | Logging |

---

## License

See LICENSE file in the repository.
