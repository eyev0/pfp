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
  "include": [{ "paths": ["$HOME/projects"] }]
}
```

### Full Config

```jsonc
{
  "sessions": [
    { "name": "work", "windows": ["$HOME/api", "$HOME/web"] }
  ],
  "markers": {
    "exact": [".git", "Cargo.toml"],   // Project root indicators
    "pattern": [],                       // Regex patterns
    "traverse_hidden": true,             // Descend into .dirs
    "chain_root_markers": true           // Merge with entry markers
  },
  "ignore": {
    "exact": ["node_modules", "target"], // Skip these dirs
    "pattern": [],                        // Regex patterns
    "chain_root_ignore": true             // Merge with entry ignore
  },
  "include": [
    {
      "paths": ["$HOME/projects"],        // Scan roots
      "mode": "dir",                      // "dir" or "file"
      "depth": 3,                         // Max recursion
      "yield_on_marker": true,            // Stop at project root
      "include_intermediate_paths": true  // Include parent dirs
    }
  ]
}
```

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

## Default Markers

`.git`, `Cargo.toml`, `go.mod`

## Default Ignores

`node_modules`, `venv`, `bin`, `target`, `debug`, `src`, `test`, `tests`, `lib`, `docs`, `pkg`, `.DS_Store`
