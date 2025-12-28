//! Command-line interface implementation.
//!
//! This module defines the CLI using [clap](https://docs.rs/clap) and handles
//! all subcommand dispatching.
//!
//! # Subcommands
//!
//! | Command | Description |
//! |---------|-------------|
//! | `new-session` | Pick a project and create a new tmux session |
//! | `new-window` | Pick a project and create a new window |
//! | `sessions` | List and switch between active sessions |
//! | `kill-session` | Kill current session, switch to previous |
//! | `start` | Start predefined sessions from config |
//! | `open` | Pick a project and print path (for shell integration) |
//! | `init` | Print shell function for integration |
//! | `print-config` | Print parsed configuration |
//!
//! # Global Options
//!
//! - `-c, --config <FILE>` - Path to configuration file

use log::trace;
use std::process::Stdio;

use crate::config::{read_config, Config, Session};
use crate::context::AppContext;
use crate::fs::{expand, trim_session_name, trim_window_name};
use crate::selectors::{pick_project, select_from_list};
use crate::Error;

use clap::{Arg, ArgAction};

/// Application name used in CLI help text
static APP_NAME: &str = "pfp";

/// Default configuration file path (supports environment variable expansion)
static CONFIG_PATH_DEFAULT: &str = "${XDG_CONFIG_HOME}/pfp/config.json";

// Subcommand names
const KILL_SESSION_SUBC: &str = "kill-session";
const SESSIONS_SUBC: &str = "sessions";
const START_SUBC: &str = "start";
const PRINT_CONFIG_SUBC: &str = "print-config";
const NEW_SESSION_SUBC: &str = "new-session";
const NEW_WINDOW_SUBC: &str = "new-window";
const OPEN_SUBC: &str = "open";
const INIT_SUBC: &str = "init";

// Argument names
const CONFIG_ARG: &str = "config";
const START_INHERIT_STDIN_ARG: &str = "attach";
const SHELL_ARG: &str = "shell";

/// Main CLI entry point.
///
/// Parses command-line arguments, loads configuration, and dispatches
/// to the appropriate subcommand handler.
pub fn cli() -> Result<(), crate::Error> {
    cli_with_context(AppContext::default())
}

/// CLI entry point with injectable context (for testing).
pub fn cli_with_context(ctx: AppContext) -> Result<(), crate::Error> {
    // parse cli args
    let mut cmd = clap::Command::new(APP_NAME)
        .about("Pfp helps you manage your projects with tmux sessions and windows")
        .arg(
            Arg::new(CONFIG_ARG)
                .short('c')
                .long(CONFIG_ARG)
                .action(ArgAction::Set)
                .default_value(CONFIG_PATH_DEFAULT)
                .value_name("FILE")
                .help("config file full path"),
        )
        .subcommand(clap::Command::new(PRINT_CONFIG_SUBC).about("Print parsed config to stdout"))
        .subcommand(clap::Command::new(NEW_SESSION_SUBC).about("Pick a path and create new tmux session"))
        .subcommand(clap::Command::new(NEW_WINDOW_SUBC).about("Pick a path and create new tmux window"))
        .subcommand(
            clap::Command::new(KILL_SESSION_SUBC)
                .about("Kill current session and switch to last/previous session"),
        )
        .subcommand(
            clap::Command::new(SESSIONS_SUBC)
                .about("Show list of active sessions, select one to switch to it"),
        )
        .subcommand(
            clap::Command::new(START_SUBC)
                .about("Start tmux sessions from predefined list")
                .arg(
                    Arg::new(START_INHERIT_STDIN_ARG)
                        .short('a')
                        .long(START_INHERIT_STDIN_ARG)
                        .action(ArgAction::SetTrue)
                        .help("attach to tmux session after start"),
                ),
        )
        .subcommand(
            clap::Command::new(OPEN_SUBC)
                .about("Pick a project and print its path (for shell integration)"),
        )
        .subcommand(
            clap::Command::new(INIT_SUBC)
                .about("Print shell function for integration")
                .arg(
                    Arg::new(SHELL_ARG)
                        .required(true)
                        .value_parser(["zsh", "bash", "fish"])
                        .help("Shell type (zsh, bash, fish)"),
                ),
        );

    let help = cmd.render_help();
    let arg_matches = cmd.get_matches();

    let config_path_raw = arg_matches
        .get_one::<String>(CONFIG_ARG)
        .ok_or_else(|| Error::CmdArg(format!("error: wrong type used for {}", CONFIG_ARG)))?;
    let is_default_path = config_path_raw == CONFIG_PATH_DEFAULT;
    let path = expand(config_path_raw)?;

    let config = {
        let cfg = read_config(&path);
        if cfg.is_err() && is_default_path {
            cfg.map_err(|e| eprintln!("{}, config path={}, using default config", e, path))
                .unwrap_or_default()
        } else {
            cfg?
        }
    };
    trace!("config {:#?}", config);

    match arg_matches.subcommand() {
        Some((KILL_SESSION_SUBC, _)) => handle_kill_session(&ctx)?,
        Some((PRINT_CONFIG_SUBC, _)) => println!("{:#?}", config),
        Some((SESSIONS_SUBC, _)) => handle_sessions(&ctx)?,
        Some((START_SUBC, arg_matches)) => {
            let attach = *arg_matches.get_one(START_INHERIT_STDIN_ARG).unwrap_or(&false);
            handle_start(&ctx, &config, attach)?;
        }
        Some((NEW_WINDOW_SUBC, _)) => handle_new_window(&ctx, &config)?,
        Some((NEW_SESSION_SUBC, _)) => handle_new_session(&ctx, &config)?,
        Some((OPEN_SUBC, _)) => handle_open(&ctx, &config)?,
        Some((INIT_SUBC, arg_matches)) => {
            let shell = arg_matches
                .get_one::<String>(SHELL_ARG)
                .ok_or_else(|| Error::CmdArg("shell argument required".to_string()))?;
            print_shell_init(shell);
        }
        _ => println!("{}", help),
    }

    Ok(())
}

fn handle_kill_session(ctx: &AppContext) -> Result<(), Error> {
    let mut session_name =
        String::from_utf8(ctx.tmux_execute("tmux display-message -p '#S'")?.stdout)?;
    session_name.retain(|x| x != '\'' && x != '\n');
    let out = ctx.tmux_execute("tmux switch-client -l")?;
    if !out.status.success() {
        ctx.tmux_execute("tmux switch-client -p")?;
    }
    ctx.tmux_execute(&format!("tmux kill-session -t {}", session_name))?;
    Ok(())
}

fn handle_sessions(ctx: &AppContext) -> Result<(), Error> {
    let mut current_session =
        String::from_utf8(ctx.tmux_execute("tmux display-message -p '#S:#I'")?.stdout)?;
    current_session.retain(|x| x != '\'' && x != '\n');
    
    let mut sessions = String::from_utf8(
        ctx.tmux_execute("tmux list-sessions -F '#S:#I,#{session_id}'")?.stdout,
    )?
    .trim_end()
    .to_owned();
    sessions.retain(|x| x != '\'');
    
    let mut s = sessions
        .split('\n')
        .map(|x| x.split_once(',').expect("Wrong list-sessions format!"))
        .collect::<Vec<(&str, &str)>>();
    s.sort_by_key(|k| k.1);
    sessions = s.into_iter().map(|x| x.0).collect::<Vec<&str>>().join("\n");
    
    let idx = sessions
        .split('\n')
        .enumerate()
        .find(|x| x.1 == current_session)
        .map(|x| x.0)
        .unwrap_or(0);
    
    let mut pick = select_from_list(
        ctx,
        &sessions,
        "Active sessions:",
        &[
            "--layout",
            "reverse",
            "--preview",
            "tmux capture-pane -ept {}",
            "--preview-window",
            "right:nohidden",
            "--sync",
            "--bind",
            &format!("load:pos({})", idx + 1),
        ],
    )?;
    pick.retain(|x| x != '\'' && x != '\n');
    
    if !pick.is_empty() {
        ctx.tmux_execute(&format!("tmux switch-client -t {}", pick))?;
    }
    Ok(())
}

fn handle_start(ctx: &AppContext, config: &Config, attach: bool) -> Result<(), Error> {
    let stdin_opt = if attach { Stdio::inherit() } else { Stdio::piped() };
    
    if config.sessions.is_empty() {
        ctx.tmux_execute_with_stdin("tmux", stdin_opt)?;
        return Ok(());
    }
    
    let mut sessions = String::from_utf8(ctx.tmux_execute("tmux list-sessions -F '#S'")?.stdout)?;
    sessions.retain(|x| x != '\'');
    
    let pick = select_from_list(
        ctx,
        &config
            .sessions
            .iter()
            .map(|s| s.name.as_str())
            .collect::<Vec<&str>>()
            .join("\n"),
        "Start sessions:",
        &[
            "-m",
            "--layout",
            "reverse",
            "--preview",
            &format!(
                "echo '{}'",
                config
                    .sessions
                    .iter()
                    .map(Session::to_string)
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
            "--preview-window",
            "right:nohidden",
        ],
    )?;
    
    let picked_sessions = pick.split('\n').filter(|x| !x.is_empty()).collect::<Vec<&str>>();
    
    for session in &config.sessions {
        if picked_sessions.contains(&session.name.as_str()) {
            let session_exists = sessions
                .split('\n')
                .any(|x| x == session.name);
            
            if session_exists {
                println!("session {} exists", session.name);
                continue;
            }
            
            for (i, window) in session.windows.iter().enumerate() {
                let window_path = &expand(window.trim_end_matches('/'))?;
                let cmd = if i == 0 {
                    format!(
                        "tmux new-session -d -s {} -n {} -c {}",
                        session.name,
                        trim_window_name(window_path)?,
                        window_path,
                    )
                } else {
                    format!(
                        "tmux new-window -d -n {} -P -F '#S:#I' -c {}",
                        trim_window_name(window_path)?,
                        window_path,
                    )
                };
                
                let mut window_output = String::from_utf8(
                    ctx.tmux_window_command(&cmd, window_path)?.stdout
                )?;

                if i > 0 {
                    window_output.retain(|x| x != '\'' && x != '\n');
                    ctx.tmux_execute(&format!(
                        "tmux move-window -s {} -t {}:",
                        window_output, session.name
                    ))?;
                }
            }
            
            ctx.tmux_execute(&format!(
                "tmux movew -r -s {}:1 -t {}:1",
                session.name, session.name
            ))?;
        }
    }
    
    ctx.tmux_execute_with_stdin("tmux attach", stdin_opt)?;
    Ok(())
}

fn handle_new_window(ctx: &AppContext, config: &Config) -> Result<(), Error> {
    let pick = pick_project(ctx, config, "New window:")?;
    ctx.tmux_window_command(
        &format!("tmux new-window -n {} -c {}", &trim_window_name(&pick)?, &pick),
        &pick,
    )?;
    Ok(())
}

fn handle_new_session(ctx: &AppContext, config: &Config) -> Result<(), Error> {
    let pick = pick_project(ctx, config, "New session:")?;
    let mut window_name = trim_window_name(&pick)?;
    let session_name = trim_session_name(&window_name);
    
    ctx.tmux_window_command(
        &format!(
            "tmux new-session -d -s {} -n {} -c {}",
            session_name, window_name, &pick
        ),
        &pick,
    )?;
    
    window_name.retain(|x| x != '\'' && x != '\n');
    ctx.tmux_execute(&format!("tmux switch-client -t {}:1", session_name))?;
    Ok(())
}

fn handle_open(ctx: &AppContext, config: &Config) -> Result<(), Error> {
    let pick = pick_project(ctx, config, "Open:")?;
    println!("{}", pick);
    Ok(())
}

/// Print shell initialization function for the given shell type.
fn print_shell_init(shell: &str) {
    match shell {
        "zsh" | "bash" => {
            println!(r#"# PFP shell integration
# Add this to your ~/.{shell}rc:

pf() {{
  local target
  target=$(pfp open "$@")
  local exit_code=$?
  
  if [[ $exit_code -ne 0 ]]; then
    return $exit_code
  fi
  
  if [[ -n "$target" ]]; then
    if [[ -d "$target" ]]; then
      cd "$target"
    elif [[ -f "$target" ]]; then
      ${{EDITOR:-vim}} "$target"
    fi
  fi
}}

# Usage:
#   pf              - fuzzy pick project, cd to it or open in editor
#   pf -c config    - use custom config
"#, shell = shell);
        }
        "fish" => {
            println!(r#"# PFP shell integration
# Add this to your ~/.config/fish/config.fish:

function pf
  set -l target (pfp open $argv)
  set -l exit_code $status
  
  if test $exit_code -ne 0
    return $exit_code
  end
  
  if test -n "$target"
    if test -d "$target"
      cd "$target"
    else if test -f "$target"
      $EDITOR "$target"
    end
  end
end

# Usage:
#   pf              - fuzzy pick project, cd to it or open in editor
#   pf -c config    - use custom config
"#);
        }
        _ => {
            eprintln!("Unsupported shell: {}", shell);
        }
    }
}
