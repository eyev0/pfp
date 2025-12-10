# PFP Quick Reference

## Commands

```bash
pfp new-session      # Pick project → Create tmux session
pfp new-window       # Pick project → Create tmux window
pfp sessions         # List sessions → Switch to selected
pfp kill-session     # Kill current → Switch to previous
pfp start [-a]       # Start configured sessions
pfp print-config     # Debug: print parsed config
```

## Global Options

```bash
pfp -c <file> <command>   # Use custom config file
```

## Configuration

**Location:** `~/.config/pfp/config.json`

### Minimal Config

```json
{
  "include": ["$HOME/dev"]
}
```

### Full Config

```jsonc
{
  // Override built-in profiles or create custom ones
  "profiles": {
    "projects": {
      "markers": [".git", "Cargo.toml", "pom.xml"]  // Only override markers
    },
    "scripts": {
      "mode": "file",
      "markers": ["*.sh", "*.py"],
      "depth": 2
    }
  },

  // Directories to scan
  "include": [
    "$HOME/dev",                                        // String = projects profile
    { "paths": ["$HOME/Downloads"], "profile": "browse" },
    { "paths": ["$HOME/.config"], "profile": "files", "depth": 2 }
  ],

  // Predefined tmux sessions
  "sessions": [
    { "name": "work", "windows": ["$HOME/api", "$HOME/web"] }
  ]
}
```

## Built-in Profiles

| Profile | Mode | Markers | Depth | Description |
|---------|------|---------|-------|-------------|
| `projects` | dir | `.git`, `Cargo.toml`, `go.mod`, `package.json` | 255 | Find project roots |
| `browse` | dir | `*` | 2 | Browse all directories |
| `files` | file | `*` | 1 | Find all files |

See [`defaults.json`](../defaults.json) for full definitions.

## Profile Fields

| Field | Type | Description |
|-------|------|-------------|
| `mode` | `"dir"` / `"file"` | Scan for directories or files |
| `markers` | `string[]` | Patterns to match (glob: `*.sh`, `tree-*`) |
| `ignore` | `string[]` | Patterns to skip |
| `depth` | `number` | Max recursion depth |
| `stop_on_marker` | `boolean` | Stop at project root |
| `intermediate_paths` | `boolean` | Include parent dirs |

## tmux.conf Keybindings

```tmux
bind C-n run-shell "pfp new-session"
bind C-w run-shell "pfp new-window"
bind C-s run-shell "pfp sessions"
bind C-k run-shell "pfp kill-session"
```

## Shell Aliases

```bash
alias pp="pfp new-session"
alias pw="pfp new-window"
alias ps="pfp sessions"
```

## Environment Variables

| Variable | Usage |
|----------|-------|
| `$HOME` | Home directory expansion |
| `${XDG_CONFIG_HOME}` | Default config location |
| `$EDITOR` | Editor for file mode |

## Glob Patterns

- `".git"` — exact match
- `"*.sh"` — files ending with .sh
- `"tree-*"` — names starting with tree-
- `"*"` — match everything
