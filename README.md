# pfp - Project Fuzzy Picker

> **Fuzzy picker for your git repos and project directories**

A fast, ergonomic CLI tool for managing tmux sessions and windows with fuzzy project selection. PFP scans your filesystem for project directories (identified by markers like `.git`, `Cargo.toml`, `go.mod`) and provides an interactive fuzzy finder interface powered by [fzf](https://github.com/junegunn/fzf).

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)

## ✨ Features

- 🔍 **Smart Project Detection** - Automatically finds projects by markers (`.git`, `Cargo.toml`, `go.mod`, etc.)
- ⚡ **Fast Scanning** - Efficient filesystem traversal with configurable depth and ignore patterns
- 🎯 **Fuzzy Selection** - Interactive selection powered by fzf with live previews
- 📁 **Session Management** - Create, switch, and kill tmux sessions effortlessly
- 🪟 **Window Management** - Quickly spawn new windows in your current session
- 📋 **Predefined Sessions** - Configure and start your common work environments with one command
- 🔧 **Profile-Based Config** - Built-in profiles for common use cases + easy customization

## 📦 Installation

### Prerequisites

- [Rust](https://rustup.rs/) (1.70+)
- [tmux](https://github.com/tmux/tmux)
- [fzf](https://github.com/junegunn/fzf)
- [tree](https://mama.indstate.edu/users/ice/tree/) (optional, for directory previews)

### Build from Source

```bash
git clone https://github.com/your-username/pfp.git
cd pfp
cargo build --release

# Install to PATH
sudo cp target/release/pfp /usr/local/bin/
```

## 🚀 Quick Start

```bash
# Pick a project and create a new tmux session
pfp new-session

# Pick a project and create a new window in current session
pfp new-window

# List and switch between active sessions
pfp sessions

# Kill current session and switch to another
pfp kill-session

# Start predefined sessions from config
pfp start --attach
```

## ⚙️ Configuration

Create a config file at `~/.config/pfp/config.json`:

```jsonc
{
  // Directories to scan (simple format uses "projects" profile)
  "include": [
    "$HOME/dev",
    { "paths": ["$HOME/Downloads", "$HOME/Documents"], "profile": "browse" },
    { "paths": ["$HOME/.config"], "profile": "files", "depth": 2 }
  ],

  // Override built-in profiles or create custom ones
  "profiles": {
    "projects": {
      "markers": [".git", "Cargo.toml", "go.mod", "pom.xml"]
    }
  },

  // Predefined sessions for quick start
  "sessions": [
    {
      "name": "work",
      "windows": [
        "$HOME/projects/api",
        "$HOME/projects/frontend"
      ]
    }
  ]
}
```

### Built-in Profiles

| Profile | Description |
|---------|-------------|
| `projects` | Find project directories by markers (`.git`, `Cargo.toml`, etc.) |
| `browse` | Browse all directories (depth 2) |
| `files` | Find all files (depth 1) |

See [`defaults.json`](defaults.json) for full profile definitions and reference values.

### Customizing Profiles

Add only the fields you want to change — they merge with defaults:

```jsonc
{
  "profiles": {
    "projects": {
      "markers": [".git", "Cargo.toml", "pom.xml"]  // Only override markers
    }
  }
}
```

### Profile Inheritance

Use the `base` field to create profiles that inherit from existing ones:

```jsonc
{
  "profiles": {
    "my_projects": {
      "base": "projects",  // Inherit all fields from "projects"
      "depth": 3           // Override only depth
    }
  }
}
```

## 📖 Commands

| Command | Description |
|---------|-------------|
| `pfp new-session` | Pick a project → Create new tmux session |
| `pfp new-window` | Pick a project → Create new window in current session |
| `pfp sessions` | List active sessions → Switch to selected one |
| `pfp kill-session` | Kill current session → Switch to previous |
| `pfp start [-a]` | Start predefined sessions (optionally attach) |
| `pfp open` | Pick a project → Print path (for shell integration) |
| `pfp init <shell>` | Print shell function (`zsh`, `bash`, `fish`) |
| `pfp print-config` | Print parsed configuration |

### Global Options

| Option | Description |
|--------|-------------|
| `-c, --config <FILE>` | Custom config file path |

## 🐚 Shell Integration

Use `pfp init` to generate a shell function that lets you `cd` to projects directly:

```bash
# Add to ~/.zshrc or ~/.bashrc
eval "$(pfp init zsh)"   # or bash, fish

# Now use:
pf              # fuzzy pick → cd to directory or open file in $EDITOR
```

## ⌨️ Recommended tmux Keybindings

Add to your `~/.tmux.conf`:

```tmux
bind-key C-n run-shell "pfp new-session"
bind-key C-w run-shell "pfp new-window"
bind-key C-s run-shell "pfp sessions"
bind-key C-k run-shell "pfp kill-session"
```

## 📚 Documentation

- **[Full API Documentation](docs/API.md)** - Complete reference for all modules and configuration
- **[Default Profiles](defaults.json)** - Reference values for built-in profiles

## 🏗️ Project Structure

```
src/
├── main.rs      # Entry point & error types
├── cli.rs       # Command-line interface (clap)
├── config.rs    # Configuration parsing & profiles
├── fs.rs        # Filesystem scanning with glob patterns
├── fzf.rs       # FZF integration
├── selectors.rs # High-level selection functions
└── tmux.rs      # Tmux command execution
```

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
