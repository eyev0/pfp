//! End-to-end CLI tests.
//!
//! All tests use an isolated XDG_CONFIG_HOME to avoid reading user's config.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Create a command with isolated config environment.
/// Sets XDG_CONFIG_HOME to an empty temp dir so tests don't read user's config.
fn isolated_cmd(temp: &TempDir) -> assert_cmd::Command {
    let mut cmd = cargo_bin_cmd!("pfp");
    cmd.env("XDG_CONFIG_HOME", temp.path());
    cmd
}

/// Test that --help shows help text.
#[test]
fn test_help_flag() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("pfp"))
        .stdout(predicate::str::contains("new-session"))
        .stdout(predicate::str::contains("new-window"))
        .stdout(predicate::str::contains("sessions"))
        .stdout(predicate::str::contains("init"));
}

/// Test print-config with default config (no user config file).
#[test]
fn test_print_config_default() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("profiles"))
        .stdout(predicate::str::contains("projects"));
}

/// Test print-config with custom config file.
#[test]
fn test_print_config_custom() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{
        "include": ["/tmp"],
        "sessions": []
    }"#).unwrap();
    
    isolated_cmd(&temp)
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("include"))
        .stdout(predicate::str::contains("profiles"));
}

/// Test print-config with profiles override.
#[test]
fn test_print_config_with_profiles() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{
        "profiles": {
            "projects": {
                "markers": [".git", "pom.xml"]
            }
        },
        "include": ["/tmp"]
    }"#).unwrap();
    
    isolated_cmd(&temp)
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("pom.xml"));
}

/// Test init zsh command.
#[test]
fn test_init_zsh() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("init")
        .arg("zsh")
        .assert()
        .success()
        .stdout(predicate::str::contains("pf()"))
        .stdout(predicate::str::contains("pfp open"))
        .stdout(predicate::str::contains("cd \"$target\""));
}

/// Test init bash command.
#[test]
fn test_init_bash() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("init")
        .arg("bash")
        .assert()
        .success()
        .stdout(predicate::str::contains("pf()"))
        .stdout(predicate::str::contains("EDITOR"));
}

/// Test init fish command.
#[test]
fn test_init_fish() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("init")
        .arg("fish")
        .assert()
        .success()
        .stdout(predicate::str::contains("function pf"))
        .stdout(predicate::str::contains("$status"));
}

/// Test init with invalid shell.
#[test]
fn test_init_invalid_shell() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("init")
        .arg("powershell")
        .assert()
        .failure();
}

/// Test that missing config file returns error when explicitly specified.
#[test]
fn test_missing_config_error() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .arg("-c")
        .arg("/nonexistent/path/config.json")
        .arg("print-config")
        .assert()
        .failure();
}

/// Test config with sessions.
#[test]
fn test_print_config_with_sessions() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{
        "include": ["/tmp"],
        "sessions": [
            {
                "name": "work",
                "windows": ["/tmp"]
            }
        ]
    }"#).unwrap();
    
    isolated_cmd(&temp)
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("work"));
}

/// Test no subcommand shows help.
#[test]
fn test_no_subcommand_shows_help() {
    let temp = TempDir::new().unwrap();
    isolated_cmd(&temp)
        .assert()
        .success();
}

/// Test config with custom ignore patterns.
#[test]
fn test_config_with_ignore_override() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{
        "profiles": {
            "projects": {
                "ignore": ["my_custom_ignore", "build_output"]
            }
        },
        "include": ["/tmp"]
    }"#).unwrap();
    
    isolated_cmd(&temp)
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("my_custom_ignore"))
        .stdout(predicate::str::contains("build_output"));
}

/// Test config with inline ignore override.
#[test]
fn test_config_with_inline_ignore() {
    let temp = TempDir::new().unwrap();
    let config_path = temp.path().join("config.json");
    
    fs::write(&config_path, r#"{
        "include": [
            { "paths": ["/tmp"], "ignore": ["inline_ignored"] }
        ]
    }"#).unwrap();
    
    isolated_cmd(&temp)
        .arg("-c")
        .arg(config_path.to_str().unwrap())
        .arg("print-config")
        .assert()
        .success()
        .stdout(predicate::str::contains("inline_ignored"));
}

